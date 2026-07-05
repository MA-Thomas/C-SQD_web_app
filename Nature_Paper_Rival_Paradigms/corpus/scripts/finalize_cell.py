#!/usr/bin/env python3
"""Finalize a preflight cell after browser acquisition of a candidate pool.

Reads candidates_pool.json (ordered, over-drawn) from build_cell.py. Candidate
PDFs must have been downloaded by the browser into pdfs/_incoming/ named
<round_id>__c<rank2>.pdf (e.g. ..._c01.pdf). For each candidate in rank order:
  - verify the PDF is a real, non-empty PDF (pypdf),
  - extract page-1 text and apply the research-article eligibility check
    (reject Research Highlights / Comments / Q&A / Editorials / Meeting reports
     / Corrections etc. that OpenAlex still types as 'article'),
  - accept research articles in order until target N reached.
Accepted papers get sequential paper_ids and are filed to corpus/pdfs|text/<slug>,
appended to papers.csv, acquisition_log.csv, source_block_schedule_v0.csv, and
recorded (with rejects + probabilities) in the round draw_audit.json.
Rejected candidate PDFs are moved to pdfs/_rejected/.
"""
from __future__ import annotations
import argparse, csv, json, re, shutil
from datetime import date
from pathlib import Path
from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[1]
PAPERS_CSV = ROOT / "metadata" / "papers.csv"
ACQ_LOG = ROOT / "sources" / "acquisition_log.csv"
SCHEDULE = ROOT / "sources" / "source_block_schedule_v0.csv"
INCOMING = ROOT / "pdfs" / "_incoming"
REJECTED = ROOT / "pdfs" / "_rejected"
AUDIT_ROOT = ROOT / "sources" / "draw_audits"

PAPERS_COLS = ["paper_id","field","venue_tier","paper_title","authors","year",
    "journal","doi","pmid","pmcid","url","pdf_url","pdf_path","text_path",
    "oa_status","download_status","openalex_cited_by_count","source_checked_date",
    "notes","sample_source","selection_probability","design_weight"]

# distinctive section-header phrases that mark non-research 'article' content
BLOCK_PHRASES = ["research highlight", "meeting report", "book review",
    "news and views", "news & views", "q&a", "question-and-answer",
    "correspondence", "editorial", "commentary", "comment", "opinion",
    "perspective", "obituary", "in this issue", "erratum", "corrigendum",
    "retraction", "author correction", "publisher correction", "news feature",
    "viewpoint", "world report", "personal view", "profile"]

CC_BOILER = re.compile(r"open access.*?creativecommons\.org/licenses/[^\s]+", re.I | re.S)

# Publisher pandemic/landing banners that some PMC/publisher PDFs prepend to
# page 1, which would otherwise hide the real section label from eligibility.
# 2026-06-24 fix (Elsevier COVID-19 / Monkeypox resource-centre banners).
PUB_BANNER = re.compile(
    r"(since january 2020 elsevier has created|elsevier has created a monkeypox).*?"
    r"(unless otherwise stated\.|these permissions\.|similar technologies\.)",
    re.I | re.S)


def existing_ids() -> set[str]:
    with PAPERS_CSV.open() as f:
        return {r["paper_id"] for r in csv.DictReader(f)}


def existing_dois() -> set[str]:
    out = set()
    with PAPERS_CSV.open() as f:
        for r in csv.DictReader(f):
            d = (r.get("doi") or "").strip().lower().replace("https://doi.org/", "")
            if d:
                out.add(d)
    return out


def next_paper_id() -> int:
    mx = 0
    with PAPERS_CSV.open() as f:
        for r in csv.DictReader(f):
            m = re.match(r"P(\d+)", r.get("paper_id", ""))
            if m:
                mx = max(mx, int(m.group(1)))
    return mx + 1


def doi_year(doi: str):
    """Best-effort publication year parsed from common DOI patterns
    (BMC: gb-YYYY-..., s<journal>-0YY-...). Returns int or None."""
    m = re.search(r"/gb-((?:19|20)\d{2})-", doi)
    if m:
        return int(m.group(1))
    m = re.search(r"/s\d+-(\d{3})-", doi)
    if m:
        return 2000 + int(m.group(1))
    return None


def num_pmid(v):
    if not v:
        return ""
    m = re.search(r"(\d+)", v)
    return m.group(1) if m else ""


def clean_title(t):
    import html as _html
    t = t or ""
    t = re.sub(r"<[^>]+>", "", t)          # strip XML/MathML/HTML tags
    t = _html.unescape(t)
    return " ".join(t.split())


def eligibility(page1_text: str):
    """Return (is_research, reason). Inspects the section-label region for
    non-research markers, handling both normal and letter-spaced headers
    (e.g. 'M E E T I N G  R E P O R T')."""
    t = CC_BOILER.sub(" ", page1_text or "", count=1)
    t = PUB_BANNER.sub(" ", t, count=1)
    head = " ".join(t.split())[:300].lower()
    # A research article's "correspondence to: <author>" byline must not be
    # misread as a non-research Correspondence/Letters section header
    # (BMJ research papers carry this byline on page 1). 2026-06-24 fix.
    head = head.replace("correspondence to", " ")
    # Lancet/other research Articles carry cross-references like "See Comment
    # page 198" / "See Correspondence" in the page-1 masthead; these are
    # pointers to related items, not the article's own section. 2026-06-24 fix.
    head = re.sub(r"see (comment|correspondence|editorial|perspective)s?\b", " ", head)
    despaced = re.sub(r"\s+", "", head)[:200]
    for ph in BLOCK_PHRASES:
        if ph in head or ph.replace(" ", "") in despaced:
            return False, f"section_header:{ph}"
    return True, "research"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--round-id", required=True)
    ap.add_argument("--field-slug", required=True)
    ap.add_argument("--meta", default="")
    ap.add_argument("--notes", required=True)
    ap.add_argument("--source-block-order", default="")
    ap.add_argument("--era-band", required=True)
    ap.add_argument("--target-n", type=int, default=10)
    ap.add_argument("--year-min", type=int, default=0)
    ap.add_argument("--year-max", type=int, default=9999)
    ap.add_argument("--route", default="publisher_pdf_browser",
                    help="acquisition source_type label for the acquisition log")
    args = ap.parse_args()

    rnd = AUDIT_ROOT / args.round_id
    pool = json.loads((rnd / "candidates_pool.json").read_text())
    audit = json.loads((rnd / "draw_audit.json").read_text())
    meta = json.loads(Path(args.meta).read_text()) if args.meta else {}

    pdf_dir = ROOT / "pdfs" / args.field_slug
    txt_dir = ROOT / "text" / args.field_slug
    pdf_dir.mkdir(parents=True, exist_ok=True)
    txt_dir.mkdir(parents=True, exist_ok=True)
    REJECTED.mkdir(parents=True, exist_ok=True)

    have_ids = existing_ids()
    have_dois = existing_dois()
    today = date.today().isoformat()
    pid = next_paper_id()

    new_rows, acq_rows, accepted, rejected = [], [], [], []

    for cand in pool["candidates"]:
        if len(accepted) >= args.target_n:
            break
        rank = cand["rank"]
        doi = cand["doi"]
        src_pdf = INCOMING / f"{args.round_id}__c{rank:02d}.pdf"
        if doi in have_dois:
            rejected.append({"rank": rank, "doi": doi, "reason": "already_in_corpus"})
            continue
        dy = doi_year(doi)
        if dy is not None and not (args.year_min <= dy <= args.year_max):
            rejected.append({"rank": rank, "doi": doi,
                             "reason": f"doi_year_out_of_era:{dy}"})
            continue
        if not src_pdf.exists():
            rejected.append({"rank": rank, "doi": doi, "reason": "acquisition_failed_no_pdf"})
            continue
        try:
            reader = PdfReader(str(src_pdf))
            npages = len(reader.pages)
            page1 = reader.pages[0].extract_text() or ""
        except Exception as e:
            rejected.append({"rank": rank, "doi": doi, "reason": f"pdf_invalid:{e}"})
            shutil.copyfile(src_pdf, REJECTED / src_pdf.name)
            continue
        if npages < 2 or src_pdf.stat().st_size < 8000:
            rejected.append({"rank": rank, "doi": doi, "reason": f"too_short:{npages}p"})
            shutil.copyfile(src_pdf, REJECTED / src_pdf.name)
            continue
        ok, reason = eligibility(page1)
        if not ok:
            rejected.append({"rank": rank, "doi": doi, "reason": reason})
            shutil.copyfile(src_pdf, REJECTED / src_pdf.name)
            continue

        paper_id = f"P{pid:04d}"
        if paper_id in have_ids:
            raise SystemExit(f"{paper_id} already present; abort")
        dst_pdf = pdf_dir / f"{paper_id}.pdf"
        shutil.copyfile(src_pdf, dst_pdf)
        full = []
        for pg in reader.pages:
            try:
                full.append(pg.extract_text() or "")
            except Exception:
                full.append("")
        (txt_dir / f"{paper_id}.txt").write_text("\n".join(full))

        m = meta.get(doi, {})
        authors = cand.get("authors") or m.get("authors", "")
        pmid = num_pmid(m.get("pmid")) or num_pmid(cand.get("pmid"))
        pmcid = (m.get("pmcid") or cand.get("pmcid") or "")
        cited = m.get("cited", cand.get("cited_by_count", ""))
        pdf_path = f"corpus/pdfs/{args.field_slug}/{paper_id}.pdf"
        text_path = f"corpus/text/{args.field_slug}/{paper_id}.txt"
        new_rows.append({
            "paper_id": paper_id, "field": cand["field"],
            "venue_tier": cand["venue_tier"], "paper_title": clean_title(cand["title"]),
            "authors": authors, "year": cand["year"],
            "journal": cand["journal"], "doi": doi,
            "pmid": pmid, "pmcid": pmcid,
            "url": f"https://doi.org/{doi}", "pdf_url": cand["pdf_url"],
            "pdf_path": pdf_path, "text_path": text_path,
            "oa_status": cand.get("oa_status") or "",
            "download_status": "downloaded_verified_pdf",
            "openalex_cited_by_count": cited if cited is not None else "",
            "source_checked_date": today, "notes": args.notes,
            "sample_source": "random_base",
            "selection_probability": f"{cand['selection_probability']:.10g}",
            "design_weight": f"{cand['design_weight']:.10g}",
        })
        acq_rows.append({
            "paper_id": paper_id, "source_type": args.route,
            "source_url": cand["pdf_url"], "access_status": "ok_200_pdf",
            "download_status": "downloaded_verified_pdf",
            "local_path": pdf_path, "checked_date": today,
            "notes": f"Browser-route fetch (%PDF verified, {npages} pages, "
                     f"{cand.get('oa_status','')} OA). Eligibility: research.",
        })
        accepted.append({"rank": rank, "paper_id": paper_id, "doi": doi,
                         "pages": npages, "bytes": src_pdf.stat().st_size,
                         "selection_probability": cand["selection_probability"],
                         "design_weight": cand["design_weight"],
                         "U_v": cand["U_v"], "N_u": cand["N_u"]})
        pid += 1
        have_dois.add(doi)

    if len(accepted) < args.target_n:
        print(json.dumps({"WARNING": "pool exhausted before target",
                          "accepted": len(accepted), "target": args.target_n,
                          "rejected": rejected}, indent=2))
        raise SystemExit(2)

    with PAPERS_CSV.open("a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=PAPERS_COLS)
        for r in new_rows:
            w.writerow(r)
    with ACQ_LOG.open("a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["paper_id","source_type","source_url",
            "access_status","download_status","local_path","checked_date","notes"])
        for r in acq_rows:
            w.writerow(r)
    ids = [r["paper_id"] for r in new_rows]
    sched_row = {
        "field": pool["field"], "era_band": args.era_band,
        "venue_tier": pool["venue_tier"],
        "source_block_order": args.source_block_order,
        "source": pool["source"], "source_inventory_id": pool["inventory_id"],
        "status": f"completed_target_{len(ids)}",
        "random_base_papers_in_block": len(ids), "paper_ids": ";".join(ids),
        "design_note": (f"Preflight; added by {args.round_id}; {len(ids)} random_base "
                        f"papers; OpenAlex frozen frame; direct publisher PDF via browser "
                        f"route (§1.4.1); {len(rejected)} candidate(s) rejected at "
                        "eligibility (non-research section / dup / access)."),
    }
    with SCHEDULE.open("a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["field","era_band","venue_tier",
            "source_block_order","source","source_inventory_id","status",
            "random_base_papers_in_block","paper_ids","design_note"])
        w.writerow(sched_row)

    audit["acquisition_status"] = "acquired_verified"
    audit["target_n"] = args.target_n
    audit["accepted"] = accepted
    audit["rejected_candidates"] = rejected
    audit["n_acquired"] = len(accepted)
    (rnd / "draw_audit.json").write_text(json.dumps(audit, indent=2))

    print(json.dumps({"round": args.round_id, "added": len(new_rows),
        "ids": ids, "pages": [a["pages"] for a in accepted],
        "n_rejected": len(rejected), "rejected": rejected}, indent=2))


if __name__ == "__main__":
    main()
