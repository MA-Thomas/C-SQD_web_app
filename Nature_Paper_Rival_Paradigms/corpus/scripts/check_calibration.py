#!/usr/bin/env python3
"""Check a scored v0.3 paradigm-marker calibration round against the expected key.

The calibration set (``coding/calibration/calibration_items.csv``) declares, for each
synthetic item C01-C10, the expected score on the dimension(s) the item is built to probe,
plus a decision boundary describing the trap. This script compares a judge's actual scores
in ``coding/multi_judge/paradigm_marker_ratings.csv`` (filtered to the calibration round)
against that key.

Pass criterion, per the coding guide:
  * actual score within +/-1 of the expected score, AND
  * actual score on the correct side of the trap boundary (a rhetoric_without_move item must
    land 0-1, never 3-4; a move_without_vocab item must land 3-4, never 0-1).

Only the dimensions the item explicitly targets (non-blank expected_* cells) are graded; an
item may legitimately be scored on other dimensions too.

Usage:
    python3 corpus/scripts/check_calibration.py
    python3 corpus/scripts/check_calibration.py --round round_calibration_v03_001
    python3 corpus/scripts/check_calibration.py --judge-round jr_v03_calibration_r1

Exit code 0 if all graded cells pass, 1 otherwise.
"""

from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CAL = ROOT / "coding" / "calibration" / "calibration_items.csv"
RATINGS = ROOT / "coding" / "multi_judge" / "paradigm_marker_ratings.csv"
JUDGE_ROUNDS = ROOT / "coding" / "multi_judge" / "judge_rounds.csv"

DIMENSIONS = [
    "cp_risky_prediction",
    "cp_rival_elimination",
    "cp_generative_structure",
    "cp_counterfactual_intervention",
    "cp_assumption_vulnerability",
    "si_terminal_certification",
    "si_association_framing",
    "si_accumulation_progress",
]

# Low-side traps must stay <= 1; high-side (move) traps must stay >= 3.
LOW_SIDE = {"rhetoric_without_move", "negation_attribution", "boilerplate_limitations"}
HIGH_SIDE = {"move_without_vocab"}


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def num(value: str | None):
    if value is None or value.strip() == "":
        return None
    return float(value)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--round", default="round_calibration_v03_001")
    ap.add_argument(
        "--judge-round",
        default=None,
        help="restrict to one judge_round_id (default: all rounds of --round)",
    )
    args = ap.parse_args()

    if not CAL.exists():
        print(f"Missing calibration key: {CAL}")
        return 1
    key = {row["item_id"]: row for row in read_csv(CAL)}

    # Which judge_round_ids belong to the calibration round.
    jr_in_round = {
        r["judge_round_id"]
        for r in read_csv(JUDGE_ROUNDS)
        if r["round_id"] == args.round
    }
    ratings = [
        r
        for r in read_csv(RATINGS)
        if (args.judge_round is None and r["judge_round_id"] in jr_in_round)
        or (args.judge_round is not None and r["judge_round_id"] == args.judge_round)
    ]

    if not ratings:
        print(
            "No calibration ratings found yet.\n"
            f"  Expecting rows in {RATINGS.relative_to(ROOT)} with paper_id in C01..C10\n"
            f"  and a judge_round_id tied to round '{args.round}'.\n"
            "  Score the 10 synthetic items first, then re-run this checker."
        )
        return 1

    graded = passed = 0
    failures: list[str] = []
    for r in ratings:
        item = key.get(r["paper_id"])
        if item is None:
            continue
        trap = item.get("trap_type", "")
        # The trap boundary applies only to the dimension(s) the item is built to probe,
        # not to every dimension it happens to assert an expected value for. (A
        # rhetoric_without_move item can still legitimately score high on an si_ dimension.)
        target_dims = {
            t.strip() for t in item.get("target_dimension", "").split(";") if t.strip()
        }
        for dim in DIMENSIONS:
            expected = num(item.get(f"expected_{dim}"))
            if expected is None:
                continue  # dimension not graded for this item
            actual = num(r.get(dim))
            graded += 1
            ok = actual is not None and abs(actual - expected) <= 1
            if ok and dim in target_dims and trap in LOW_SIDE and actual > 1:
                ok = False
            if ok and dim in target_dims and trap in HIGH_SIDE and actual < 3:
                ok = False
            if ok:
                passed += 1
            else:
                failures.append(
                    f"  FAIL {r['paper_id']} [{trap}] {dim}: "
                    f"expected ~{expected:g}, got {actual if actual is not None else 'blank'}"
                    f"  ({item.get('decision_boundary','')})"
                )

    print(f"Calibration round: {args.round}")
    print(f"Graded cells: {graded}   Passed: {passed}   Failed: {graded - passed}")
    if failures:
        print("\n".join(failures))
        print(
            "\nRevise the anchor(s) that failed in coding_guide_v0.3_paradigm_markers.md "
            "before scoring real papers."
        )
        return 1
    print("ALL CALIBRATION CELLS PASS — instrument is scoring role, not vocabulary.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
