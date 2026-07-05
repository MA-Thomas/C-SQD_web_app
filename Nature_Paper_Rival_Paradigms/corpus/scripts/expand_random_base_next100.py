#!/usr/bin/env python3
"""Add the next 100 protocol-compliant random-base papers.

This round extends already scheduled source blocks with frozen inventories. It
uses the direct-route acquisition helper from expand_random_base_batch.py, but
does not treat a direct-route failure as a final acquisition failure: if any
direct route fails, the script discards partial file outputs and stops so the
failed draws can receive the publisher-page browser fallback required by
selection_protocol.md.
"""

from __future__ import annotations

import argparse
import csv
import importlib.util
import json
from collections import Counter, defaultdict
from datetime import date
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "expand_random_base_batch", SCRIPT_DIR / "expand_random_base_batch.py"
)
base = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(base)

ROUND_ID = "round_20260620_random_base_next_100_round2"
RUN_DATE = date.today().isoformat()
REQUESTED_NEW_PAPERS = 100
SEED_BASE = 202606200700

base.SEED_BASE = SEED_BASE

PLAN = [
    {
        "field": "Computational biology / genomics-ML",
        "era_band": "2005-2014_completed_years_2005-2014",
        "year_min": 2005,
        "year_max": 2014,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "PLOS Computational Biology",
        "source_inventory_id": "plos_computational_biology_openalex_v1.0.1_through2025_20260620",
        "needed": 0,
        "round_reason": "Omitted from this round because prior expansion exposed possible older-inventory contamination in this historical slice.",
    },
    {
        "field": "Computational biology / genomics-ML",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "PLOS Computational Biology",
        "source_inventory_id": "plos_computational_biology_openalex_v1.0.1_through2025_20260620",
        "needed": 35,
        "round_reason": "Extend this direct-stable current block in the final clean round2 retry.",
    },
    {
        "field": "Computational neuroscience",
        "era_band": "2005-2014_completed_years_2007-2014",
        "year_min": 2007,
        "year_max": 2014,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "Frontiers in Computational Neuroscience",
        "source_inventory_id": "frontiers_computational_neuroscience_openalex_v1.0.1_through2025_20260620",
        "needed": 0,
        "round_reason": "Omitted from the final round2 run after a direct 404 in this historical slice; queued for browser recovery.",
    },
    {
        "field": "Computational neuroscience",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "Frontiers in Computational Neuroscience",
        "source_inventory_id": "frontiers_computational_neuroscience_openalex_v1.0.1_through2025_20260620",
        "needed": 30,
        "round_reason": "Extend this direct-stable current block in the final clean round2 retry.",
    },
    {
        "field": "ML / AI methods proper",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "top",
        "source_block_order": "1",
        "source": "NeurIPS",
        "source_inventory_id": "neurips_proceedings_v1.0.1_2015-2025_20260620",
        "needed": 0,
        "round_reason": "Omitted from the final round2 retry after one selected proceedings PDF produced low extracted text and was queued for recovery/replacement.",
    },
    {
        "field": "Mechanistic molecular / cell / developmental biology",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "top",
        "source_block_order": "1",
        "source": "eLife",
        "source_inventory_id": "elife_openalex_v1.0.1_through2025_20260620",
        "needed": 0,
        "round_reason": "Omitted from the final clean run after the second attempt exposed remaining reviewed-preprint direct-route failures requiring browser fallback.",
    },
    {
        "field": "Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics)",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "Journal of Cosmology and Astroparticle Physics",
        "source_inventory_id": "jcap_openalex_v1.0.1_through2025_20260620",
        "needed": 0,
        "round_reason": "Omitted from the clean retry after the first attempt exposed mixed 403/503/certificate direct-route failures requiring browser fallback.",
    },
    {
        "field": "Structure-driven experimental physics (HEP, neutrino, gravitational waves)",
        "era_band": "1990-2004_completed_years_1990-2004",
        "year_min": 1990,
        "year_max": 2004,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "The European Physical Journal C",
        "source_inventory_id": "epj_c_openalex_v1.0.1_through2025_20260620",
        "needed": 0,
        "round_reason": "Omitted from the clean retry after the first attempt exposed many non-PDF and sparse-scan direct-route failures.",
    },
    {
        "field": "Structure-driven experimental physics (HEP, neutrino, gravitational waves)",
        "era_band": "2005-2014_completed_years_2005-2014",
        "year_min": 2005,
        "year_max": 2014,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "The European Physical Journal C",
        "source_inventory_id": "epj_c_openalex_v1.0.1_through2025_20260620",
        "needed": 0,
        "round_reason": "Omitted from the clean retry after the first attempt exposed many non-PDF direct-route failures.",
    },
    {
        "field": "Structure-driven experimental physics (HEP, neutrino, gravitational waves)",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "The European Physical Journal C",
        "source_inventory_id": "epj_c_openalex_v1.0.1_through2025_20260620",
        "needed": 35,
        "round_reason": "Extend this direct-stable current block in the final clean round2 retry.",
    },
]


def write_csv(path: Path, rows: list[dict[str, object]], fieldnames: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def cleanup_file_outputs(selected: list[dict[str, object]]) -> None:
    workspace = base.ROOT.parent
    for record in selected:
        for key in ("pdf_path", "text_path"):
            value = str(record.get(key, ""))
            if value:
                path = workspace / value
                if path.exists():
                    path.unlink()


def append_outputs(
    schedule: list[dict[str, str]],
    selected: list[dict[str, object]],
    failures: list[dict[str, object]],
) -> None:
    papers = base.read_csv(base.PAPERS_CSV)
    paper_fieldnames = list(papers[0].keys())
    acquisition_rows = base.read_csv(base.ACQUISITION_LOG_CSV)
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

    write_csv(base.PAPERS_CSV, papers, paper_fieldnames)
    write_csv(base.ACQUISITION_LOG_CSV, acquisition_rows, acquisition_fieldnames)

    schedule_by_key = {
        (r["field"], r["era_band"], r["venue_tier"], r["source"]): dict(r)
        for r in schedule
    }
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
        row["status"] = "extended_next100_completed"
        row["design_note"] = (
            row.get("design_note", "")
            + f" Updated by {ROUND_ID}; added {len(ids)} papers in the next-100 expansion; "
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
    write_csv(base.SCHEDULE_CSV, rows, schedule_fieldnames)

    audit_dir = base.AUDIT_ROOT / ROUND_ID
    audit_dir.mkdir(parents=True, exist_ok=True)
    public_records = [{k: v for k, v in r.items() if k != "_inventory_row"} for r in selected]
    block_counts = Counter((r["field"], r["era_band"], r["source"]) for r in selected)
    audit = {
        "round_id": ROUND_ID,
        "run_date": RUN_DATE,
        "requested_new_papers": REQUESTED_NEW_PAPERS,
        "new_papers_added": len(selected),
        "seed_base": SEED_BASE,
        "user_agent": base.USER_AGENT,
        "rate_limit": "<=1 request/sec for acquisition requests",
        "selection_rationale": (
            "Extended already scheduled source blocks with frozen inventories. This round uses "
            "three current source slices that behaved as direct-stable across prior retry "
            "attempts. Historical PLOS is not extended further because prior expansion exposed "
            "possible older-inventory contamination; historical Frontiers had a direct 404 in "
            "this round and is queued for browser recovery; NeurIPS had one low-text PDF in "
            "this round and is queued for recovery/replacement; JCAP current, eLife "
            "reviewed-preprint routes, PRX, Collabra, and older EPJ C slices remain "
            "publisher-page browser-recovery pending."
        ),
        "active_acquisition_frame": (
            "Eligible inventory rows in target years with recorded/deterministic direct PDF routes. "
            "If any direct route fails, this script stops before metadata append so the drawn paper "
            "can receive the publisher-page browser fallback required by selection_protocol.md."
        ),
        "plan": PLAN,
        "block_counts_added": [
            {"field": k[0], "era_band": k[1], "source": k[2], "added": v}
            for k, v in sorted(block_counts.items())
        ],
        "selected": public_records,
        "acquisition_failures_before_replacement": failures,
    }
    (audit_dir / "draw_audit.json").write_text(
        json.dumps(audit, indent=2) + "\n", encoding="utf-8"
    )

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

    requested = sum(int(p["needed"]) for p in PLAN)
    if requested != REQUESTED_NEW_PAPERS:
        raise SystemExit(f"Plan requests {requested}, expected {REQUESTED_NEW_PAPERS}")

    papers = base.read_csv(base.PAPERS_CSV)
    schedule = base.read_csv(base.SCHEDULE_CSV)
    taken = base.known_keys(papers)
    next_id = base.max_paper_number(papers) + 1

    selected_all: list[dict[str, object]] = []
    failures_all: list[dict[str, object]] = []
    for idx, tuple_plan in enumerate(PLAN, start=1):
        if int(tuple_plan["needed"]) == 0:
            print(
                f"\n== skipped {tuple_plan['field']} | {tuple_plan['era_band']} | "
                f"{tuple_plan['source']} ==",
                flush=True,
            )
            continue
        print(
            f"\n== {tuple_plan['field']} | {tuple_plan['era_band']} | "
            f"{tuple_plan['source']} | needed={tuple_plan['needed']} ==",
            flush=True,
        )
        selected, failures, next_id = base.draw_for_tuple(
            tuple_plan, papers, taken, next_id, idx, args.dry_run
        )
        selected_all.extend(selected)
        failures_all.extend(failures)
        print(
            f"{tuple_plan['source']} selected={len(selected)} failures={len(failures)}",
            flush=True,
        )

    print(f"total_selected={len(selected_all)}")
    print(f"total_failures_before_replacement={len(failures_all)}")

    if args.dry_run:
        return

    if failures_all:
        cleanup_file_outputs(selected_all)
        audit_dir = base.AUDIT_ROOT / ROUND_ID
        audit_dir.mkdir(parents=True, exist_ok=True)
        write_csv(
            audit_dir / "direct_route_failures_requiring_browser_fallback.csv",
            failures_all,
            ["field", "source", "inventory_id", "unit_id", "title", "doi", "pdf_url", "error"],
        )
        raise SystemExit(
            "Direct-route failures occurred. Partial PDF/text outputs were removed; "
            "run publisher-page browser fallback for the failed draws before appending metadata."
        )

    append_outputs(schedule, selected_all, failures_all)


if __name__ == "__main__":
    main()
