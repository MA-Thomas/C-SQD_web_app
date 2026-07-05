#!/usr/bin/env python3
"""Preflight cell builder (browser-route batches, 2026-06-21).

Given an OpenAlex frame JSON (fetched in-browser and downloaded to the
corpus, since the sandbox itself has no direct network), this:
  1. applies the eligibility filter (type=article, OA-accessible, era, has DOI
     + volume/issue archival unit),
  2. writes a frozen source inventory (archive_pages.csv, units.csv, items.csv,
     inventory_manifest.json with SHA-256 hashes),
  3. runs a seeded unit->paper random draw recording each paper's inclusion
     probability p_a=(1/U_v)(1/N_u) and design_weight=1/p_a, deduplicating
     against the existing corpus and within the draw,
  4. emits selected_pending.json (paper_ids assigned) for the browser
     acquisition step + a partial draw_audit.json.

PDF acquisition (direct->publisher-browser->OA), verification, text extraction,
and the final papers.csv / acquisition_log / draw_audit writes are done by the
finalize step after the browser has fetched the PDFs.
"""
from __future__ import annotations
import argparse, csv, hashlib, json, random, re
from datetime import date, datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAPERS_CSV = ROOT / "metadata" / "papers.csv"
INVENTORIES = ROOT / "source_inventories"
AUDIT_ROOT = ROOT / "sources" / "draw_audits"

NONRESEARCH_TITLE = re.compile(
    r"(?i)^(correction|erratum|corrigendum|addendum|retraction|editorial|"
    r"publisher correction|author correction|comment|reply|response|"
    r"in this issue|highlights?|research highlights?|meeting report|"
    r"book review|obituary|errata|withdrawn)\b")


def sha256(p: Path) -> str:
    h = hashlib.sha256()
    h.update(p.read_bytes())
    return h.hexdigest()


def existing_dois() -> set[str]:
    out = set()
    with PAPERS_CSV.open() as f:
        for row in csv.DictReader(f):
            d = (row.get("doi") or "").strip().lower()
            if d:
                out.add(d.replace("https://doi.org/", ""))
    return out


def next_paper_id() -> int:
    mx = 0
    with PAPERS_CSV.open() as f:
        for row in csv.DictReader(f):
            m = re.match(r"P(\d+)", row.get("paper_id", ""))
            if m:
                mx = max(mx, int(m.group(1)))
    return mx + 1


def norm_doi(d):
    if not d:
        return ""
    return d.strip().lower().replace("https://doi.org/", "")


def pmcid_of(rec):
    # PMCID may live in ids.pmcid, open_access.oa_url, or best_oa_location.
    cands = []
    ids = rec.get("ids") or {}
    cands.append(ids.get("pmcid") or "")
    oa = rec.get("open_access") or {}
    cands.append(oa.get("oa_url") or "")
    best = rec.get("best_oa_location") or {}
    cands.append((best.get("landing_page_url") or "") if isinstance(best, dict) else "")
    cands.append((best.get("pdf_url") or "") if isinstance(best, dict) else "")
    for loc in (rec.get("locations") or []):
        if isinstance(loc, dict):
            cands.append(loc.get("landing_page_url") or "")
            cands.append(loc.get("pdf_url") or "")
    for v in cands:
        m = re.search(r"PMC(\d+)", v)
        if m:
            return "PMC" + m.group(1)
        m = re.search(r"/pmc/articles/(\d+)", v)
        if m:
            return "PMC" + m.group(1)
    return ""


def pmid_of(rec):
    ids = rec.get("ids") or {}
    v = ids.get("pmid") or ""
    m = re.search(r"(\d+)", v)
    return m.group(1) if m else ""


def arxiv_of(rec):
    """Extract an arXiv id from locations or ids."""
    cands = []
    for loc in (rec.get("locations") or []):
        if isinstance(loc, dict):
            cands.append(loc.get("landing_page_url") or "")
            cands.append(loc.get("pdf_url") or "")
    ids = rec.get("ids") or {}
    cands.append(ids.get("arxiv") or "")
    for v in cands:
        m = re.search(r"arxiv\.org/(?:abs|pdf)/([^\s?]+?)(?:v\d+)?(?:\.pdf)?$", v)
        if m:
            return m.group(1)
        m = re.search(r"arxiv\.org/(?:abs|pdf)/([^\s?]+)", v)
        if m:
            return re.sub(r"(v\d+)?(\.pdf)?$", "", m.group(1))
    return ""


def primary_pdf_of(rec):
    pl = rec.get("primary_location") or {}
    return pl.get("pdf_url") or ""


def authors_str(rec):
    names = []
    for a in (rec.get("authorships") or []):
        n = (a.get("author") or {}).get("display_name")
        if n:
            names.append(n)
    if not names:
        return ""
    if len(names) > 6:
        return names[0] + " et al."
    return "; ".join(names)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--frame", required=True)
    ap.add_argument("--source", required=True)
    ap.add_argument("--source-slug", required=True)
    ap.add_argument("--field", required=True)
    ap.add_argument("--field-slug", required=True)
    ap.add_argument("--venue-tier", required=True)
    ap.add_argument("--era-band", required=True)
    ap.add_argument("--year-min", type=int, required=True)
    ap.add_argument("--year-max", type=int, required=True)
    ap.add_argument("--inventory-id", required=True)
    ap.add_argument("--round-id", required=True)
    ap.add_argument("--source-block-order", default="")
    ap.add_argument("--seed", type=int, required=True)
    ap.add_argument("--n", type=int, default=10)
    ap.add_argument("--pool", type=int, default=18,
                    help="ordered candidate pool size (over-draw for eligibility rejection)")
    ap.add_argument("--pdf-template", default="",
                    help="PDF URL template with {doi} (route=doi)")
    ap.add_argument("--route", default="doi", choices=["doi", "pmc", "arxiv", "primary_pdf"],
                    help="acquisition route: doi, pmc, arxiv, or primary_pdf (OpenAlex primary_location.pdf_url)")
    ap.add_argument("--require-primary-pdf", action="store_true",
                    help="OA-accessible eligibility requires a primary_location.pdf_url")
    ap.add_argument("--require-pmcid", action="store_true",
                    help="OA-accessible eligibility requires a PMC copy (PMCID)")
    ap.add_argument("--require-arxiv", action="store_true",
                    help="OA-accessible eligibility requires an arXiv green copy")
    ap.add_argument("--journal", required=True)
    args = ap.parse_args()

    frame = json.loads(Path(args.frame).read_text())
    records = frame["records"] if isinstance(frame, dict) and "records" in frame else frame

    # ---- eligibility filter ----
    def pdf_for(doi, pmcid, arxiv, primary=""):
        if args.route == "pmc":
            return f"https://pmc.ncbi.nlm.nih.gov/articles/{pmcid}/pdf/"
        if args.route == "arxiv":
            return f"https://arxiv.org/pdf/{arxiv}"
        if args.route == "primary_pdf":
            return primary
        return args.pdf_template.format(doi=doi)

    eligible = []
    drop = {"not_article": 0, "not_oa": 0, "era": 0, "no_doi": 0,
            "no_unit": 0, "nonresearch_title": 0, "no_pmcid": 0, "no_arxiv": 0,
            "no_primary_pdf": 0}
    for r in records:
        if r.get("type") != "article":
            drop["not_article"] += 1; continue
        oa = r.get("open_access") or {}
        # arXiv green route: OA-accessibility is via the arXiv copy, not the
        # publisher OA flag, so don't require is_oa there.
        if args.route != "arxiv" and not oa.get("is_oa"):
            drop["not_oa"] += 1; continue
        y = r.get("publication_year")
        if not (y and args.year_min <= y <= args.year_max):
            drop["era"] += 1; continue
        doi = norm_doi(r.get("doi"))
        if not doi:
            drop["no_doi"] += 1; continue
        bib = r.get("biblio") or {}
        vol, iss = bib.get("volume"), bib.get("issue")
        # Archival unit = (volume, issue). For continuous-publication journals
        # that lack issue numbers, the volume is the archival unit.
        if not vol:
            drop["no_unit"] += 1; continue
        title = (r.get("display_name") or "")
        if NONRESEARCH_TITLE.match(title.strip()):
            drop["nonresearch_title"] += 1; continue
        if args.require_pmcid and not pmcid_of(r):
            drop["no_pmcid"] += 1; continue
        if args.require_arxiv and not arxiv_of(r):
            drop["no_arxiv"] += 1; continue
        if args.require_primary_pdf and not primary_pdf_of(r):
            drop["no_primary_pdf"] += 1; continue
        eligible.append(r)

    # ---- units ----
    units: dict[tuple, list] = {}
    for r in eligible:
        bib = r["biblio"]
        key = (str(bib.get("volume")), str(bib.get("issue") or ""))
        units.setdefault(key, []).append(r)
    unit_keys = sorted(units.keys())
    U_v = len(unit_keys)

    # ---- write inventory ----
    inv_dir = INVENTORIES / args.source_slug / args.inventory_id
    inv_dir.mkdir(parents=True, exist_ok=True)

    with (inv_dir / "archive_pages.csv").open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["source", "archive_page_type", "year", "volume", "page_url"])
        years = sorted({r.get("publication_year") for r in eligible})
        for y in years:
            w.writerow([args.source, "openalex_year_slice", y, "",
                        f"https://api.openalex.org/works?filter=primary_location.source.id:{frame.get('source_id','')},publication_year:{y}"])

    with (inv_dir / "units.csv").open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["source", "unit_id", "unit_type", "year", "volume", "issue",
                    "n_eligible_items", "unit_url"])
        for (vol, iss) in unit_keys:
            items = units[(vol, iss)]
            yr = items[0].get("publication_year")
            uid = f"{args.source_slug}_v{vol}_i{iss}"
            w.writerow([args.source, uid, "journal_issue", yr, vol, iss,
                        len(items), ""])

    with (inv_dir / "items.csv").open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["source", "unit_id", "year", "volume", "issue", "doi",
                    "pmcid", "openalex_id", "title", "authors", "type", "oa_status",
                    "oa_url", "pdf_url", "landing_page_url"])
        for (vol, iss) in unit_keys:
            uid = f"{args.source_slug}_v{vol}_i{iss}"
            for r in units[(vol, iss)]:
                oa = r.get("open_access") or {}
                doi = norm_doi(r.get("doi"))
                pmcid = pmcid_of(r)
                arxiv = arxiv_of(r)
                pl = r.get("primary_location") or {}
                w.writerow([args.source, uid, r.get("publication_year"), vol, iss,
                            doi, pmcid, r.get("id"), r.get("display_name"),
                            authors_str(r), r.get("type"), oa.get("oa_status"),
                            oa.get("oa_url"), pdf_for(doi, pmcid, arxiv, primary_pdf_of(r)),
                            pl.get("landing_page_url")])

    hashes = {n: sha256(inv_dir / n) for n in
              ("archive_pages.csv", "units.csv", "items.csv")}
    manifest = {
        "source": args.source, "source_slug": args.source_slug,
        "inventory_id": args.inventory_id,
        "parser_id": "openalex_source_inventory",
        "parser_version": "1.0.0",
        "built_at": date.today().isoformat(),
        "field": args.field, "venue_tier": args.venue_tier,
        "era_band": args.era_band,
        "coverage_note": (
            "OpenAlex source records type=article, OA-accessible, "
            f"{args.year_min}-{args.year_max}, fetched in-browser (CORS) and "
            "downloaded to the corpus; sandbox has no direct network. "
            "Eligibility (research-article type, OA, volume/issue archival unit) "
            "applied at build."),
        "coverage_years": [args.year_min, args.year_max],
        "source_id_openalex": frame.get("source_id", ""),
        "user_agent": "TextDataMining-CorpusExpansion/0.1 (research corpus; in-browser fetch; <=1 request/sec)",
        "rate_limit": "OpenAlex polite pool, paged per-page=200",
        "n_raw_records": len(records),
        "n_eligible": len(eligible),
        "n_units": U_v,
        "eligibility_drops": drop,
        "archive_pages_csv": "archive_pages.csv",
        "units_csv": "units.csv",
        "items_csv": "items.csv",
        "archive_pages_sha256": hashes["archive_pages.csv"],
        "units_sha256": hashes["units.csv"],
        "items_sha256": hashes["items.csv"],
        "known_gaps": ("Records missing volume/issue (no resolvable archival "
                       "unit) were excluded from the draw frame; see "
                       "eligibility_drops.no_unit."),
    }
    (inv_dir / "inventory_manifest.json").write_text(json.dumps(manifest, indent=2))

    # ---- seeded ordered candidate-pool draw (no paper_ids yet) ----
    # Over-draw an ordered pool; final eligibility (research-article section
    # header) is applied at acquisition, where the PDF is available. Draw is
    # without replacement at the paper level, deduped vs the existing corpus.
    rng = random.Random(args.seed)
    have = existing_dois()
    pool, pool_dois = [], set()
    draws_log = []
    attempts = 0
    while len(pool) < args.pool and attempts < 50000:
        attempts += 1
        uk = unit_keys[rng.randrange(U_v)]
        items = units[uk]
        N_u = len(items)
        rec = items[rng.randrange(N_u)]
        doi = norm_doi(rec.get("doi"))
        if doi in have or doi in pool_dois:
            draws_log.append({"attempt": attempts, "doi": doi, "result": "dup_skip"})
            continue
        p_a = (1.0 / U_v) * (1.0 / N_u)
        weight = 1.0 / p_a
        pool_dois.add(doi)
        oa = rec.get("open_access") or {}
        bib = rec.get("biblio") or {}
        cand = {
            "rank": len(pool) + 1,
            "doi": doi,
            "title": rec.get("display_name"),
            "authors": authors_str(rec),
            "year": rec.get("publication_year"),
            "journal": args.journal,
            "field": args.field, "venue_tier": args.venue_tier,
            "openalex_id": rec.get("id"),
            "pmcid": pmcid_of(rec),
            "pmid": pmid_of(rec),
            "arxiv_id": arxiv_of(rec),
            "cited_by_count": rec.get("cited_by_count"),
            "oa_status": oa.get("oa_status"),
            "oa_url": oa.get("oa_url"),
            "pdf_url": pdf_for(doi, pmcid_of(rec), arxiv_of(rec), primary_pdf_of(rec)),
            "unit_id": f"{args.source_slug}_v{bib.get('volume')}_i{bib.get('issue')}",
            "U_v": U_v, "N_u": N_u,
            "selection_probability": p_a,
            "design_weight": weight,
        }
        pool.append(cand)
        draws_log.append({"attempt": attempts, "rank": cand["rank"], "doi": doi,
                          "result": "pool", "U_v": U_v, "N_u": N_u,
                          "p_a": p_a, "design_weight": weight})

    audit = {
        "round_id": args.round_id,
        "run_date": date.today().isoformat(),
        "run_ts_utc": datetime.now(timezone.utc).isoformat(),
        "field": args.field, "venue_tier": args.venue_tier,
        "era_band": args.era_band, "source": args.source,
        "source_block_order": args.source_block_order,
        "inventory_id": args.inventory_id,
        "inventory_manifest_sha256": sha256(inv_dir / "inventory_manifest.json"),
        "units_sha256": hashes["units.csv"],
        "items_sha256": hashes["items.csv"],
        "seed": args.seed, "target_n": args.n, "pool_size": args.pool,
        "U_v": U_v, "n_eligible": len(eligible),
        "n_pool": len(pool), "attempts": attempts,
        "ordered_pool_draws": draws_log,
        "acquisition_status": "pending_browser_acquisition",
    }
    rnd_dir = AUDIT_ROOT / args.round_id
    rnd_dir.mkdir(parents=True, exist_ok=True)
    (rnd_dir / "draw_audit.json").write_text(json.dumps(audit, indent=2))
    (rnd_dir / "candidates_pool.json").write_text(json.dumps({
        "round_id": args.round_id, "field": args.field,
        "field_slug": args.field_slug, "venue_tier": args.venue_tier,
        "source": args.source, "inventory_id": args.inventory_id,
        "target_n": args.n, "candidates": pool}, indent=2))

    print(json.dumps({
        "inventory_dir": str(inv_dir),
        "n_raw": len(records), "n_eligible": len(eligible), "U_v": U_v,
        "drops": drop, "n_pool": len(pool),
        "pool_dois": [d["doi"] for d in pool],
    }, indent=2))


if __name__ == "__main__":
    main()
