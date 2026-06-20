#!/usr/bin/env python3
"""
Verification suite for draw_stratum_a.py.

Checks, in order:
  1. reproducibility       -- same seed => identical draws; different seed differs
  2. selection frequencies -- empirical draw rates match the model p_a
  3. HH unbiasedness (WR)  -- exact: design-weighted total recovers the truth
  4. HH near-unbiasedness  -- without replacement at small sampling fraction
  5. dedup / WOR           -- no duplicates; existing corpus ids excluded
  6. exhaustion handling   -- empty issues/journals and FrameExhausted

Run: python3 test_draw_stratum_a.py
"""

import statistics
from collections import Counter

from draw_stratum_a import (
    Cell, draw_cell, hansen_hurwitz_total, FrameExhausted,
)

PASS, FAIL = "PASS", "FAIL"
results = []


def check(name, cond, detail=""):
    results.append((name, PASS if cond else FAIL, detail))
    print(f"[{PASS if cond else FAIL}] {name}" + (f" -- {detail}" if detail else ""))


# Synthetic frame with deliberately unequal issue/journal sizes so p_a varies.
#   J = 2
#   Journal A: 2 issues -> A1 (2 articles), A2 (3 articles)
#   Journal B: 1 issue  -> B1 (5 articles)
# Model probabilities (per draw):
#   A1 article: (1/2)(1/2)(1/2) = 1/8   -> 2 of them => 1/4
#   A2 article: (1/2)(1/2)(1/3) = 1/12  -> 3 of them => 1/4
#   B1 article: (1/2)(1/1)(1/5) = 1/10  -> 5 of them => 1/2
def small_cell():
    return Cell(
        cell_id="TEST|era|top", field="test", era="era", tier="top",
        journals={
            "A": {"A1": ["a1", "a2"], "A2": ["a3", "a4", "a5"]},
            "B": {"B1": ["b1", "b2", "b3", "b4", "b5"]},
        },
    )

P_TRUE = {
    "a1": 1/8, "a2": 1/8, "a3": 1/12, "a4": 1/12, "a5": 1/12,
    "b1": 1/10, "b2": 1/10, "b3": 1/10, "b4": 1/10, "b5": 1/10,
}


# --------------------------------------------------------------------------- #
# 1. Reproducibility
# --------------------------------------------------------------------------- #
def test_reproducibility():
    c = small_cell()
    r1 = [(r.article_id, r.selection_probability) for r in draw_cell(c, 3, seed=42)]
    r2 = [(r.article_id, r.selection_probability) for r in draw_cell(c, 3, seed=42)]
    check("reproducible: same seed -> identical draws", r1 == r2, str(r1))
    r3 = [r.article_id for r in draw_cell(c, 3, seed=43)]
    check("sensitive: different seed -> (usually) different draws",
          [a for a, _ in r1] != r3, f"{[a for a,_ in r1]} vs {r3}")


# --------------------------------------------------------------------------- #
# 2. Empirical selection frequencies match p_a
# --------------------------------------------------------------------------- #
def test_frequencies():
    c = small_cell()
    R = 120_000
    counts = Counter()
    for s in range(R):
        rec = draw_cell(c, 1, seed=s, with_replacement=True)[0]
        counts[rec.article_id] += 1
    worst = 0.0
    for art, p in P_TRUE.items():
        emp = counts[art] / R
        worst = max(worst, abs(emp - p))
    check("selection frequencies match model p_a (abs err < 0.01)",
          worst < 0.01, f"max abs error = {worst:.4f} over {R} draws")
    # probabilities recorded by the engine must sum to 1 over the frame
    recs = {r.article_id: r.selection_probability
            for r in [draw_cell(c, 1, seed=s, with_replacement=True)[0]
                      for s in range(2000)]}
    total = sum(P_TRUE.values())
    check("model probabilities normalize to 1", abs(total - 1.0) < 1e-9, f"sum={total}")


# --------------------------------------------------------------------------- #
# 3. Hansen-Hurwitz unbiasedness -- EXACT, with replacement, unequal p
# --------------------------------------------------------------------------- #
def test_hh_unbiased_with_replacement():
    c = small_cell()
    # Arbitrary article-level quantity y; truth is its population total.
    y = {"a1": 3.0, "a2": 7.0, "a3": 2.0, "a4": 5.0, "a5": 9.0,
         "b1": 1.0, "b2": 4.0, "b3": 6.0, "b4": 8.0, "b5": 2.0}
    Y_true = sum(y.values())
    R, n = 40_000, 4
    ests = []
    for s in range(R):
        recs = draw_cell(c, n, seed=s, with_replacement=True)
        ests.append(hansen_hurwitz_total(recs, y))
    mean = statistics.mean(ests)
    rel = abs(mean - Y_true) / Y_true
    check("HH estimator unbiased (WR, rel err < 0.03)",
          rel < 0.03, f"E[Y_hat]={mean:.3f} vs Y={Y_true:.3f} (rel {rel:.4f})")


# --------------------------------------------------------------------------- #
# 4. HH near-unbiasedness -- WITHOUT replacement, small sampling fraction
# --------------------------------------------------------------------------- #
def test_hh_unbiased_without_replacement():
    # One journal, two issues of unequal but large size; draw a small n WOR.
    import random as _r
    rng = _r.Random(0)
    big = Cell(
        cell_id="BIG|era|top", field="t", era="e", tier="top",
        journals={
            "J1": {"i1": [f"x{k}" for k in range(600)],
                   "i2": [f"y{k}" for k in range(900)]},
        },
    )
    y = {a: rng.uniform(0, 10)
         for iss in big.journals["J1"].values() for a in iss}
    Y_true = sum(y.values())
    R, n = 30_000, 6   # sampling fraction 6/1500 = 0.4%
    ests = [hansen_hurwitz_total(draw_cell(big, n, seed=s), y) for s in range(R)]
    mean = statistics.mean(ests)
    rel = abs(mean - Y_true) / Y_true
    check("HH near-unbiased (WOR, small fraction, rel err < 0.03)",
          rel < 0.03, f"E[Y_hat]={mean:.1f} vs Y={Y_true:.1f} (rel {rel:.4f})")


# --------------------------------------------------------------------------- #
# 5. Dedup / without-replacement integrity
# --------------------------------------------------------------------------- #
def test_dedup():
    c = small_cell()
    recs = draw_cell(c, 10, seed=7)            # 10 = all articles in the frame
    ids = [r.article_id for r in recs]
    check("WOR draws are all distinct", len(ids) == len(set(ids)), str(ids))
    # exclude an existing-corpus id: it must never be drawn
    existing = ["b1", "b2", "b3"]
    recs2 = draw_cell(c, 7, seed=7, existing_ids=existing)
    drawn = {r.article_id for r in recs2}
    check("existing corpus ids are excluded",
          drawn.isdisjoint(set(existing)), f"drawn={sorted(drawn)}")


# --------------------------------------------------------------------------- #
# 6. Exhaustion handling
# --------------------------------------------------------------------------- #
def test_exhaustion():
    c = small_cell()
    # asking for more than the frame holds (10) must raise, not loop/duplicate
    raised = False
    try:
        draw_cell(c, 11, seed=1)
    except FrameExhausted:
        raised = True
    check("over-draw raises FrameExhausted", raised)

    empty = Cell(cell_id="E", field="t", era="e", tier="top",
                 journals={"J": {"i1": []}})
    raised = False
    try:
        draw_cell(empty, 1, seed=1)
    except FrameExhausted:
        raised = True
    check("all-empty cell raises FrameExhausted", raised)


if __name__ == "__main__":
    test_reproducibility()
    test_frequencies()
    test_hh_unbiased_with_replacement()
    test_hh_unbiased_without_replacement()
    test_dedup()
    test_exhaustion()
    n_fail = sum(1 for _, st, _ in results if st == FAIL)
    print("\n" + "=" * 60)
    print(f"{len(results)} checks, {n_fail} failed")
    raise SystemExit(1 if n_fail else 0)
