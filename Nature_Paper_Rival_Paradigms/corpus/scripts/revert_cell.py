#!/usr/bin/env python3
"""Revert a finalized cell's writes so it can be re-finalized.

Removes the round's accepted paper rows from papers.csv and acquisition_log.csv,
drops the schedule row for the round's inventory_id, moves the filed PDFs/texts
to pdfs/_superseded_<round>/, and resets the draw_audit status. The frozen
inventory and candidates_pool.json are kept. Idempotent-ish.
"""
from __future__ import annotations
import argparse, csv, json, shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAPERS_CSV = ROOT / "metadata" / "papers.csv"
ACQ_LOG = ROOT / "sources" / "acquisition_log.csv"
SCHEDULE = ROOT / "sources" / "source_block_schedule_v0.csv"
AUDIT_ROOT = ROOT / "sources" / "draw_audits"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--round-id", required=True)
    ap.add_argument("--field-slug", required=True)
    args = ap.parse_args()

    rnd = AUDIT_ROOT / args.round_id
    audit = json.loads((rnd / "draw_audit.json").read_text())
    pool = json.loads((rnd / "candidates_pool.json").read_text())
    inv_id = pool["inventory_id"]
    ids = {a["paper_id"] for a in audit.get("accepted", [])}
    if not ids:
        print("no accepted ids recorded; nothing to revert"); return

    # papers.csv
    rows = list(csv.reader(open(PAPERS_CSV)))
    hdr = rows[0]
    kept = [hdr] + [r for r in rows[1:] if r and r[0] not in ids]
    csv.writer(open(PAPERS_CSV, "w", newline="")).writerows(kept)

    # acquisition_log
    rows = list(csv.reader(open(ACQ_LOG)))
    hdr = rows[0]
    kept2 = [hdr] + [r for r in rows[1:] if r and r[0] not in ids]
    csv.writer(open(ACQ_LOG, "w", newline="")).writerows(kept2)

    # schedule (drop row for this inventory)
    rows = list(csv.reader(open(SCHEDULE)))
    hdr = rows[0]; idx = hdr.index("source_inventory_id")
    kept3 = [hdr] + [r for r in rows[1:] if not (r and r[idx] == inv_id)]
    csv.writer(open(SCHEDULE, "w", newline="")).writerows(kept3)

    # move files aside
    sup = ROOT / "pdfs" / f"_superseded_{args.round_id}"
    (sup).mkdir(parents=True, exist_ok=True)
    moved = 0
    for pid in ids:
        for sub, ext in (("pdfs", "pdf"), ("text", "txt")):
            p = ROOT / sub / args.field_slug / f"{pid}.{ext}"
            if p.exists():
                shutil.move(str(p), str(sup / f"{pid}.{ext}")); moved += 1

    audit["acquisition_status"] = "reverted_pending_refinalize"
    audit.pop("accepted", None); audit.pop("rejected_candidates", None)
    audit.pop("n_acquired", None)
    (rnd / "draw_audit.json").write_text(json.dumps(audit, indent=2))

    print(json.dumps({"reverted_ids": sorted(ids), "files_moved": moved,
                      "papers_rows_now": len(kept) - 1}, indent=2))


if __name__ == "__main__":
    main()
