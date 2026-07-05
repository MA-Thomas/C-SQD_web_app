#!/usr/bin/env python3
"""Protocol-compliant random-base batch expansion.

This helper completes the current scheduled source blocks to the working target
and records a source-block audit trail. It samples only from an active acquisition
frame: eligible inventory rows with a recorded or deterministic direct PDF route.
That frame choice is written into the audit because it is narrower than every OA
metadata row in an OpenAlex-derived inventory. Direct-route failures are not final
publisher/source failures; queue them for the publisher-page browser fallback in
selection_protocol.md before marking a source block acquisition-limited.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import random
import re
import signal
import time
import urllib.error
import urllib.request
from collections import defaultdict
from datetime import date
from pathlib import Path

from pypdf import PdfReader


ROOT = Path(__file__).resolve().parents[1]
PAPERS_CSV = ROOT / "metadata" / "papers.csv"
SCHEDULE_CSV = ROOT / "sources" / "source_block_schedule_v0.csv"
ACQUISITION_LOG_CSV = ROOT / "sources" / "acquisition_log.csv"
INVENTORIES = ROOT / "source_inventories"
AUDIT_ROOT = ROOT / "sources" / "draw_audits"

ROUND_ID = "round_20260620_random_base_to_100_total"
RUN_DATE = date.today().isoformat()
USER_AGENT = (
    "TextDataMining-CorpusExpansion/0.1 "
    "(research corpus; local run; <=1 request/sec)"
)

TARGET_PER_TUPLE = 10
START_ID = 57
SEED_BASE = 202606200300

FIELD_SLUG = {
    "Cognitive science / psychology": "cognitive_science_psychology",
    "Computational biology / genomics-ML": "computational_biology",
    "Computational neuroscience": "computational_neuroscience",
    "ML / AI methods proper": "ml_ai_methods",
    "Mechanistic molecular / cell / developmental biology": "molecular_cell_biology",
    "Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics)": "statistical_physics",
    "Structure-driven condensed matter / chemistry": "condensed_matter_physics",
    "Structure-driven experimental physics (HEP, neutrino, gravitational waves)": "particle_gravitational_physics",
}

EXTRA_TUPLES = [
    {
        "field": "Structure-driven experimental physics (HEP, neutrino, gravitational waves)",
        "era_band": "2005-2014_completed_years_2005-2014",
        "year_min": 2005,
        "year_max": 2014,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "The European Physical Journal C",
        "source_inventory_id": "epj_c_openalex_v1.0.1_through2025_20260620",
        "status": "planned_to_target_10",
        "random_base_papers_in_block": "0",
        "paper_ids": "",
        "design_note": (
            "New Phase-1 historical source block scheduled to cross the 100-paper "
            "total-corpus target without overfilling existing completed tuples."
        ),
    }
]

SKIPPED_TUPLES = {
    (
        "Cognitive science / psychology",
        "2015-present_completed_years_2015-2025",
        "specialist",
        "Collabra Psychology",
    ): (
        "direct-route-limited in this environment: direct PDF routes recorded in the "
        "OpenAlex-derived inventory failed before enough verified PDFs could be acquired. "
        "Per protocol, this tuple requires a publisher-page browser fallback pass before "
        "it can be treated as acquisition-limited."
    )
}


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def write_csv(path: Path, rows: list[dict[str, object]], fieldnames: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def find_inventory(inventory_id: str) -> Path:
    matches = list(INVENTORIES.glob(f"*/{inventory_id}"))
    if len(matches) != 1:
        raise SystemExit(f"Expected one inventory for {inventory_id}, found {len(matches)}")
    return matches[0]


def max_paper_number(rows: list[dict[str, str]]) -> int:
    nums = []
    for row in rows:
        m = re.fullmatch(r"P(\d{4})", row["paper_id"])
        if m:
            nums.append(int(m.group(1)))
    return max(nums) if nums else 0


def deterministic_pdf_url(row: dict[str, str], source: str) -> str:
    if row.get("pdf_url", "").strip():
        return row["pdf_url"].strip()

    doi = row.get("doi", "").strip()
    landing = row.get("landing_page_url", "").strip()

    if source == "NeurIPS" and landing.endswith("-Abstract.html"):
        return (
            landing.replace("https://papers.nips.cc/", "https://proceedings.neurips.cc/")
            .replace("/hash/", "/file/")
            .replace("-Abstract.html", "-Paper.pdf")
        )

    if source == "PLOS Computational Biology" and doi.startswith("10.1371/"):
        return f"https://journals.plos.org/ploscompbiol/article/file?id={doi}&type=printable"

    if source == "eLife" and doi.lower().startswith("10.7554/elife."):
        match = re.search(r"10\.7554/elife\.(\d+)", doi.lower())
        if not match:
            return ""
        article_id = match.group(1)
        return f"https://elifesciences.org/articles/{article_id}.pdf"

    if source == "The European Physical Journal C" and doi.startswith("10.1140/epjc/"):
        return f"https://link.springer.com/content/pdf/{doi}.pdf"

    return ""


def eligible(row: dict[str, str], source: str, year_min: int, year_max: int) -> bool:
    try:
        year = int(row.get("year", ""))
    except ValueError:
        return False
    if not (year_min <= year <= year_max):
        return False

    paper_type = row.get("paper_type", "").strip().lower()
    if paper_type in {
        "book",
        "editorial",
        "erratum",
        "correction",
        "retraction",
        "letter",
        "comment",
    }:
        return False

    if not deterministic_pdf_url(row, source):
        return False

    if (
        source == "Journal of Cosmology and Astroparticle Physics"
        and "iopscience.iop.org" in deterministic_pdf_url(row, source)
    ):
        return False

    if (
        source == "Physical Review X"
        and "aps.org" in deterministic_pdf_url(row, source)
    ):
        return False

    return True


def known_keys(papers: list[dict[str, str]]) -> set[tuple[str, str]]:
    keys: set[tuple[str, str]] = set()
    for row in papers:
        for key, col in [
            ("doi", "doi"),
            ("title", "paper_title"),
            ("url", "url"),
            ("pdf", "pdf_url"),
        ]:
            value = row.get(col, "").strip().lower()
            if value:
                keys.add((key, value))
    return keys


def row_keys(row: dict[str, str]) -> set[tuple[str, str]]:
    keys: set[tuple[str, str]] = set()
    for key in ["doi", "title", "landing_page_url", "pdf_url", "openalex_id", "source_item_id"]:
        value = row.get(key, "").strip().lower()
        if value:
            keys.add((key if key != "landing_page_url" else "url", value))
    return keys


def load_plan() -> tuple[list[dict[str, str]], list[dict[str, object]]]:
    schedule = read_csv(SCHEDULE_CSV)
    plan: list[dict[str, object]] = []

    for row in schedule:
        key = (row["field"], row["era_band"], row["venue_tier"], row["source"])
        if key in SKIPPED_TUPLES:
            continue
        current = int(row.get("random_base_papers_in_block", "0") or 0)
        needed = max(0, TARGET_PER_TUPLE - current)
        if needed:
            item = dict(row)
            item["year_min"] = 2015
            item["year_max"] = 2025
            item["needed"] = needed
            item["already_scheduled"] = True
            plan.append(item)

    for row in EXTRA_TUPLES:
        item = dict(row)
        item["needed"] = TARGET_PER_TUPLE
        item["already_scheduled"] = False
        plan.append(item)

    return schedule, plan


def draw_for_tuple(
    tuple_plan: dict[str, object],
    papers: list[dict[str, str]],
    taken: set[tuple[str, str]],
    next_id: int,
    tuple_index: int,
    dry_run: bool,
) -> tuple[list[dict[str, object]], list[dict[str, object]], int]:
    source = str(tuple_plan["source"])
    inventory_id = str(tuple_plan["source_inventory_id"])
    inv_dir = find_inventory(inventory_id)
    items_path = inv_dir / "items.csv"
    units_path = inv_dir / "units.csv"
    manifest_path = inv_dir / "inventory_manifest.json"

    inventory_rows = read_csv(items_path)
    year_min = int(tuple_plan["year_min"])
    year_max = int(tuple_plan["year_max"])
    frame = [r for r in inventory_rows if eligible(r, source, year_min, year_max)]
    units: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in frame:
        units[row["unit_id"]].append(row)
    unit_ids = sorted(uid for uid, rows in units.items() if rows)
    if not unit_ids:
        raise SystemExit(f"No eligible PDF-route units for {source} {year_min}-{year_max}")

    seed = SEED_BASE + tuple_index
    rng = random.Random(seed)
    selected: list[dict[str, object]] = []
    failures: list[dict[str, object]] = []
    local_taken: set[tuple[str, str]] = set()
    attempts = 0
    max_attempts = max(1000, int(tuple_plan["needed"]) * 200)
    max_failures = max(25, int(tuple_plan["needed"]) * 10)

    while len(selected) < int(tuple_plan["needed"]):
        attempts += 1
        if attempts > max_attempts:
            raise SystemExit(
                f"Exceeded {max_attempts} attempts for {source} {year_min}-{year_max}"
            )

        live_units = [
            uid
            for uid in unit_ids
            if any(not (row_keys(r) & (taken | local_taken)) for r in units[uid])
        ]
        if not live_units:
            raise SystemExit(f"Exhausted available papers for {source} {year_min}-{year_max}")

        unit_id = rng.choice(live_units)
        unit_rows = units[unit_id]
        available = [r for r in unit_rows if not (row_keys(r) & (taken | local_taken))]
        if not available:
            continue
        row = rng.choice(available)
        keys = row_keys(row)
        pdf_url = deterministic_pdf_url(row, source)

        paper_id = f"P{next_id:04d}"
        field = str(tuple_plan["field"])
        slug = FIELD_SLUG[field]
        pdf_path = ROOT / "pdfs" / slug / f"{paper_id}.pdf"
        text_path = ROOT / "text" / slug / f"{paper_id}.txt"

        p = 1.0 / len(unit_ids) / len(unit_rows)
        record = {
            "paper_id": paper_id,
            "field": field,
            "source": source,
            "venue_tier": tuple_plan["venue_tier"],
            "era_band": tuple_plan["era_band"],
            "source_block_order": tuple_plan["source_block_order"],
            "draw_idx": len(selected) + 1,
            "seed": seed,
            "inventory_id": inventory_id,
            "inventory_dir": str(inv_dir.relative_to(ROOT.parent)),
            "inventory_manifest_sha256": sha256_file(manifest_path),
            "items_sha256": sha256_file(items_path),
            "units_sha256": sha256_file(units_path),
            "unit_id": unit_id,
            "unit_year": row.get("year", ""),
            "unit_label": row.get("unit_label", ""),
            "title": row.get("title", ""),
            "authors": row.get("authors", ""),
            "doi": row.get("doi", ""),
            "openalex_id": row.get("openalex_id", ""),
            "source_item_id": row.get("source_item_id", ""),
            "landing_page_url": row.get("landing_page_url", ""),
            "pdf_url": pdf_url,
            "oa_marker": row.get("oa_marker", ""),
            "paper_type": row.get("paper_type", ""),
            "openalex_cited_by_count": row.get("cited_by_count", ""),
            "eligible_units": len(unit_ids),
            "eligible_papers_in_selected_unit": len(unit_rows),
            "selection_probability": p,
            "design_weight": 1.0 / p,
            "pdf_path": str(pdf_path.relative_to(ROOT.parent)),
            "text_path": str(text_path.relative_to(ROOT.parent)),
            "attempts_before_success": attempts,
            "_inventory_row": row,
        }

        if dry_run:
            record["pdf_sha256"] = ""
            record["text_chars"] = ""
            selected.append(record)
            local_taken |= keys
            next_id += 1
            continue

        try:
            print(
                f"  trying {paper_id}: {source} | {row.get('year', '')} | "
                f"{row.get('title', '')[:80]}",
                flush=True,
            )
            pdf_bytes, final_url = download_pdf(pdf_url)
            text = extract_text(pdf_bytes)
            if len(text.strip()) < 1000:
                raise ValueError(f"extracted text too short ({len(text.strip())} chars)")
        except Exception as exc:  # noqa: BLE001 - audit the acquisition failure
            failures.append(
                {
                    "field": field,
                    "source": source,
                    "inventory_id": inventory_id,
                    "unit_id": unit_id,
                    "title": row.get("title", ""),
                    "doi": row.get("doi", ""),
                    "pdf_url": pdf_url,
                    "error": repr(exc),
                }
            )
            print(f"    failed: {repr(exc)}", flush=True)
            local_taken |= keys
            if len(failures) >= max_failures:
                raise SystemExit(
                    f"{source} {year_min}-{year_max}: reached {max_failures} "
                    "acquisition failures before completing the tuple"
                )
            continue

        pdf_path.parent.mkdir(parents=True, exist_ok=True)
        text_path.parent.mkdir(parents=True, exist_ok=True)
        pdf_path.write_bytes(pdf_bytes)
        text_path.write_text(text.rstrip() + "\n", encoding="utf-8")

        record["pdf_url"] = final_url or pdf_url
        record["pdf_sha256"] = sha256_bytes(pdf_bytes)
        record["text_chars"] = len(text)
        record["pages"] = page_count(pdf_bytes)
        selected.append(record)
        print(
            f"    acquired {paper_id}: pages={record['pages']} text_chars={record['text_chars']}",
            flush=True,
        )
        local_taken |= keys
        next_id += 1

    taken |= local_taken
    return selected, failures, next_id


def download_pdf(url: str) -> tuple[bytes, str]:
    time.sleep(1.0)
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=60) as response:
        data = response.read()
        final_url = response.geturl()
    if not data.lstrip().startswith(b"%PDF"):
        raise ValueError("download did not return PDF bytes")
    return data, final_url


def page_count(pdf_bytes: bytes) -> int:
    reader = PdfReader(io.BytesIO(pdf_bytes))
    return len(reader.pages)


def extract_text(pdf_bytes: bytes) -> str:
    def timeout_handler(signum, frame):  # noqa: ARG001
        raise TimeoutError("PDF text extraction timed out")

    previous = signal.signal(signal.SIGALRM, timeout_handler)
    signal.alarm(45)
    try:
        reader = PdfReader(io.BytesIO(pdf_bytes))
        chunks: list[str] = []
        for idx, page in enumerate(reader.pages, start=1):
            try:
                page_text = page.extract_text() or ""
            except Exception as exc:  # noqa: BLE001 - preserve extraction progress
                page_text = f"[Text extraction failed on page {idx}: {exc}]"
            chunks.append(f"\n\n--- Page {idx} ---\n{page_text}")
        return "\n".join(chunks).strip()
    finally:
        signal.alarm(0)
        signal.signal(signal.SIGALRM, previous)


def append_outputs(
    schedule: list[dict[str, str]],
    selected: list[dict[str, object]],
    failures: list[dict[str, object]],
) -> None:
    papers = read_csv(PAPERS_CSV)
    paper_fieldnames = list(papers[0].keys())
    acquisition_rows = read_csv(ACQUISITION_LOG_CSV)
    acquisition_fieldnames = list(acquisition_rows[0].keys())

    for record in selected:
        papers.append(
            {
                "paper_id": record["paper_id"],
                "field": record["field"],
                "venue_tier": record["venue_tier"],
                "paper_title": record["title"],
                "authors": record["authors"],
                "year": record["unit_year"],
                "journal": record["source"],
                "doi": record["doi"],
                "pmid": "",
                "pmcid": "",
                "url": record["landing_page_url"],
                "pdf_url": record["pdf_url"],
                "pdf_path": record["pdf_path"],
                "text_path": record["text_path"],
                "oa_status": record["oa_marker"],
                "download_status": "downloaded_verified_pdf",
                "openalex_cited_by_count": record["openalex_cited_by_count"],
                "source_checked_date": RUN_DATE,
                "notes": (
                    f"{ROUND_ID}; source-blocked Stratum A draw; source={record['source']}; "
                    f"inventory={record['inventory_id']}; unit={record['unit_id']}; "
                    "active acquisition frame=eligible rows with recorded/deterministic direct PDF route; "
                    "direct-route failures require publisher-page browser fallback before final source-limit status"
                ),
                "sample_source": "random_base",
                "selection_probability": f"{record['selection_probability']:.15g}",
                "design_weight": f"{record['design_weight']:.15g}",
            }
        )
        acquisition_rows.append(
            {
                "paper_id": record["paper_id"],
                "source_type": "publisher_or_repository_pdf",
                "source_url": record["pdf_url"],
                "access_status": "oa_accessible_verified",
                "download_status": "downloaded_verified_pdf",
                "local_path": record["pdf_path"],
                "checked_date": RUN_DATE,
                "notes": (
                    f"{ROUND_ID}; source={record['source']}; inventory={record['inventory_id']}; "
                    f"text_path={record['text_path']}; pages={record.get('pages', '')}; "
                    f"text_chars={record.get('text_chars', '')}; source_blocked_design=true; "
                    f"source_block={record['source']}; source_block_order={record['source_block_order']}"
                ),
            }
        )

    write_csv(PAPERS_CSV, papers, paper_fieldnames)
    write_csv(ACQUISITION_LOG_CSV, acquisition_rows, acquisition_fieldnames)

    schedule_by_key = {
        (r["field"], r["era_band"], r["venue_tier"], r["source"]): dict(r)
        for r in schedule
    }
    for extra in EXTRA_TUPLES:
        key = (extra["field"], extra["era_band"], extra["venue_tier"], extra["source"])
        schedule_by_key.setdefault(key, dict(extra))

    grouped: dict[tuple[str, str, str, str], list[str]] = defaultdict(list)
    for record in selected:
        grouped[
            (
                str(record["field"]),
                str(record["era_band"]),
                str(record["venue_tier"]),
                str(record["source"]),
            )
        ].append(str(record["paper_id"]))

    for key, ids in grouped.items():
        row = schedule_by_key[key]
        existing_ids = [x for x in row.get("paper_ids", "").split(";") if x]
        merged = existing_ids + ids
        row["paper_ids"] = ";".join(merged)
        row["random_base_papers_in_block"] = str(len(merged))
        row["status"] = (
            "completed_target_10"
            if len(merged) >= TARGET_PER_TUPLE
            else "partially_completed"
        )
        row["design_note"] = (
            row.get("design_note", "")
            + f" Updated by {ROUND_ID}; active acquisition frame required a direct PDF route; "
            "direct-route failures require publisher-page browser fallback before final source-limit status."
        ).strip()
        schedule_by_key[key] = row

    schedule_fieldnames = list(schedule[0].keys())
    rows = sorted(
        (
            {field: row.get(field, "") for field in schedule_fieldnames}
            for row in schedule_by_key.values()
        ),
        key=lambda r: (
            r["field"],
            r["era_band"],
            r["venue_tier"],
            int(r.get("source_block_order", "999") or 999),
            r["source"],
        ),
    )
    write_csv(SCHEDULE_CSV, rows, schedule_fieldnames)

    audit_dir = AUDIT_ROOT / ROUND_ID
    audit_dir.mkdir(parents=True, exist_ok=True)
    public_records = [{k: v for k, v in r.items() if k != "_inventory_row"} for r in selected]
    audit = {
        "round_id": ROUND_ID,
        "run_date": RUN_DATE,
        "target_per_eligible_scheduled_tuple": TARGET_PER_TUPLE,
        "new_papers_added": len(selected),
        "user_agent": USER_AGENT,
        "rate_limit": "<=1 request/sec for acquisition requests",
        "protocol": (
            "source-blocked Stratum A: chosen cell -> scheduled source block -> random "
            "archival unit -> random paper; deduplicate and record source-block probability/weight."
        ),
        "active_acquisition_frame": (
            "Eligible inventory rows in the target years with a recorded or deterministic direct PDF route; "
            "direct-route failures are queued for publisher-page browser fallback before final source-limit status."
        ),
        "documented_exceptions": [
            {
                "field": key[0],
                "era_band": key[1],
                "venue_tier": key[2],
                "source": key[3],
                "reason": reason,
            }
            for key, reason in SKIPPED_TUPLES.items()
        ],
        "selected": public_records,
        "acquisition_failures_before_replacement": failures,
    }
    (audit_dir / "draw_audit.json").write_text(json.dumps(audit, indent=2) + "\n", encoding="utf-8")

    selection_fieldnames = [
        "paper_id",
        "field",
        "source",
        "inventory_id",
        "era_band",
        "draw_idx",
        "seed",
        "unit_id",
        "unit_year",
        "title",
        "doi",
        "eligible_units",
        "eligible_papers_in_selected_unit",
        "selection_probability",
        "design_weight",
        "pdf_url",
        "pdf_path",
        "text_path",
        "pdf_sha256",
        "text_chars",
    ]
    write_csv(
        audit_dir / "selection_table.csv",
        [
            {
                **{k: r.get(k, "") for k in selection_fieldnames},
                "selection_probability": f"{r['selection_probability']:.15g}",
                "design_weight": f"{r['design_weight']:.15g}",
            }
            for r in public_records
        ],
        selection_fieldnames,
    )

    if failures:
        write_csv(
            audit_dir / "acquisition_failures.csv",
            failures,
            ["field", "source", "inventory_id", "unit_id", "title", "doi", "pdf_url", "error"],
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    papers = read_csv(PAPERS_CSV)
    schedule, plan = load_plan()
    next_id = max(max_paper_number(papers) + 1, START_ID)
    taken = known_keys(papers)

    selected_all: list[dict[str, object]] = []
    failures_all: list[dict[str, object]] = []
    for idx, tuple_plan in enumerate(plan, start=1):
        print(
            f"\n== {tuple_plan['field']} | {tuple_plan['era_band']} | "
            f"{tuple_plan['venue_tier']} | {tuple_plan['source']} ==",
            flush=True,
        )
        selected, failures, next_id = draw_for_tuple(
            tuple_plan, papers, taken, next_id, idx, args.dry_run
        )
        selected_all.extend(selected)
        failures_all.extend(failures)
        print(
            f"{tuple_plan['field']} | {tuple_plan['source']} | "
            f"selected {len(selected)} | failures {len(failures)}"
        )

    print(f"total_selected={len(selected_all)}")
    print(f"total_failures_before_replacement={len(failures_all)}")
    if not args.dry_run:
        append_outputs(schedule, selected_all, failures_all)


if __name__ == "__main__":
    main()
