#!/usr/bin/env python3
"""
Stratum A random-base selection engine.

Implements the within-cell random draw of `selection_protocol.md` §1.2-1.3:
for a chosen (field x era x tier) cell, draw

    journal  ->  issue  ->  article

each uniformly at the stage indicated, deduplicate against the existing corpus,
and record every article's per-draw selection probability and its design weight.

WHAT THIS MODULE DOES AND DOES NOT DO
-------------------------------------
It does NOT enumerate the frame. Enumeration -- discovering which journals,
issues, and OA-accessible research articles exist -- is an upstream step that
must come from an authoritative bibliographic source (PubMed / Crossref / arXiv
/ PMC), never from model recall. This module consumes an already-enumerated
frame and performs ONLY the seeded random selection and the probability/weight
bookkeeping. That separation is deliberate: the part that must be auditable and
unbiased (the draw) is kept independent of the part that needs live data
(enumeration), so the draw can be verified in isolation.

The frame handed in is assumed to already be filtered to *eligible* units:
research articles only (no editorials/errata/front matter) and, for hybrid
venues, the OA-accessible subset only (journal_list_v0.md §2.1). The engine
draws over whatever lists it is given; eligibility/OA filtering is the
enumerator's job.

PROBABILITY MODEL
-----------------
Within a cell, let
    J     = number of eligible journals (journals with >=1 non-empty issue),
    I_j   = number of non-empty eligible issues of the drawn journal j,
    N_i   = number of eligible articles in the drawn issue i.

The draw is uniform at each stage, so the per-draw selection probability of a
specific article a (living in journal j, issue i) is

    p_a = (1 / J) * (1 / I_j) * (1 / N_i).

This is exactly the unequal-probability structure flagged in §1.3: an article in
a "thin" journal (small I_j) or a small issue (small N_i) is more likely to be
drawn. Left uncorrected, journal/issue size leaks in as a confound. The design
weight removes it.

DESIGN WEIGHTS
--------------
We record, per selected article:
  * selection_probability = p_a            (the per-draw probability above)
  * design_weight         = 1 / (n * p_a)  (Hansen-Hurwitz per-unit weight)

where n is the cell's target draw count. With these weights the Hansen-Hurwitz
estimator

    Y_hat = sum_over_sample( y_a * design_weight_a ) = (1/n) sum( y_a / p_a )

is UNBIASED for the population total of any article-level quantity y under
with-replacement sampling. The production draw is *without* replacement (dedup;
§1.2 step 4); for the tiny sampling fractions in this study (a handful of
articles out of thousands) the without-replacement first-order inclusion
probability is n*p_a to O(sampling fraction), so 1/(n*p_a) is the design weight
with negligible bias. test_draw_stratum_a.py checks both the exact
with-replacement property and the without-replacement approximation.

This module has no third-party dependencies (standard library only).
"""

from __future__ import annotations

import json
import random
from dataclasses import dataclass, asdict, field
from typing import Iterable


# --------------------------------------------------------------------------- #
# Frame data model
# --------------------------------------------------------------------------- #

@dataclass
class Cell:
    """An enumerated (field x era x tier) sampling cell.

    `journals` maps journal name -> {issue_id -> [eligible article ids]}.
    Article ids are stable identifiers (DOI / PMID / arXiv id). Lists hold only
    eligible, OA-accessible research articles (filtered upstream).
    """
    cell_id: str
    field: str
    era: str
    tier: str
    journals: dict[str, dict[str, list[str]]]

    def eligible_journals(self) -> list[str]:
        """Journals having at least one non-empty issue, sorted for determinism."""
        return sorted(
            j for j, issues in self.journals.items()
            if any(len(arts) > 0 for arts in issues.values())
        )

    def nonempty_issues(self, journal: str) -> list[str]:
        return sorted(
            i for i, arts in self.journals[journal].items() if len(arts) > 0
        )


@dataclass
class DrawRecord:
    """One selected article plus the full provenance of how it was drawn."""
    cell_id: str
    field: str
    era: str
    tier: str
    journal: str
    issue: str
    article_id: str
    n_journals_J: int
    n_issues_Ij: int
    n_articles_Ni: int
    selection_probability: float       # p_a (per-draw)
    design_weight: float               # 1 / (n_target * p_a)
    draw_index: int                    # 0-based order within the cell
    n_target: int
    seed: int
    article_retries: int               # collisions skipped before this hit


# --------------------------------------------------------------------------- #
# Core draw
# --------------------------------------------------------------------------- #

class FrameExhausted(Exception):
    """Raised when the cell cannot yield another distinct eligible article."""


def _draw_one(
    cell: Cell,
    rng: random.Random,
    taken: set[str],
    max_attempts: int = 100_000,
) -> tuple[str, str, str, int, int, int, int]:
    """Draw a single not-yet-taken article.

    Returns (journal, issue, article_id, J, I_j, N_i, article_retries).

    Strategy mirrors §1.2 step 4: redraw at the article level first, then issue,
    then journal. Implemented as: pick journal -> issue -> article uniformly; if
    the article is already taken, retry. To avoid pathological loops when an
    issue/journal is fully consumed, exhausted issues and journals are pruned
    from consideration for the remainder of this call.
    """
    J_full = cell.eligible_journals()
    if not J_full:
        raise FrameExhausted(f"{cell.cell_id}: no eligible journals")
    # J is fixed for the cell (the frame's journal count), independent of what
    # has been consumed -- it is a property of the design, not of the run.
    J = len(J_full)

    # Working copies we may prune as we discover exhaustion.
    live_journals = list(J_full)
    article_retries = 0

    for _ in range(max_attempts):
        if not live_journals:
            raise FrameExhausted(f"{cell.cell_id}: journals exhausted")
        journal = rng.choice(live_journals)

        issues = [i for i in cell.nonempty_issues(journal)
                  if any(a not in taken for a in cell.journals[journal][i])]
        if not issues:
            live_journals.remove(journal)
            continue
        # I_j is the design quantity: number of non-empty eligible issues of the
        # journal in this era (NOT reduced by what has been taken).
        I_j = len(cell.nonempty_issues(journal))
        issue = rng.choice(issues)

        arts = cell.journals[journal][issue]
        # N_i is the design quantity: eligible articles in the issue.
        N_i = len(arts)
        available = [a for a in arts if a not in taken]
        if not available:
            continue  # will reselect; this issue is now effectively pruned
        article = rng.choice(available)

        if article in taken:           # defensive; should not happen
            article_retries += 1
            continue
        return journal, issue, article, J, I_j, N_i, article_retries

    raise FrameExhausted(f"{cell.cell_id}: exceeded {max_attempts} attempts")


def draw_cell(
    cell: Cell,
    n_target: int,
    seed: int,
    existing_ids: Iterable[str] = (),
    with_replacement: bool = False,
) -> list[DrawRecord]:
    """Draw `n_target` articles from `cell` using a seeded RNG.

    `existing_ids` are article ids already in the corpus (cross-cell dedup).
    `with_replacement=True` disables dedup and is used only by the test suite to
    check the exact Hansen-Hurwitz unbiasedness property.
    """
    rng = random.Random(seed)
    existing = set(existing_ids)
    taken: set[str] = set(existing) if not with_replacement else set()
    records: list[DrawRecord] = []

    for k in range(n_target):
        journal, issue, article, J, I_j, N_i, retries = _draw_one(
            cell, rng, taken if not with_replacement else set(existing)
        )
        p_a = (1.0 / J) * (1.0 / I_j) * (1.0 / N_i)
        weight = 1.0 / (n_target * p_a)
        records.append(DrawRecord(
            cell_id=cell.cell_id, field=cell.field, era=cell.era, tier=cell.tier,
            journal=journal, issue=issue, article_id=article,
            n_journals_J=J, n_issues_Ij=I_j, n_articles_Ni=N_i,
            selection_probability=p_a, design_weight=weight,
            draw_index=k, n_target=n_target, seed=seed,
            article_retries=retries,
        ))
        if not with_replacement:
            taken.add(article)

    return records


# --------------------------------------------------------------------------- #
# Estimators (used by tests and by downstream analysis)
# --------------------------------------------------------------------------- #

def hansen_hurwitz_total(records: list[DrawRecord], y: dict[str, float]) -> float:
    """Design-weighted estimate of the population total of y over the cell."""
    return sum(y[r.article_id] * r.design_weight for r in records)


# --------------------------------------------------------------------------- #
# Audit log I/O
# --------------------------------------------------------------------------- #

def write_audit_log(records: list[DrawRecord], path: str) -> None:
    """Write the full draw provenance as JSON (one object per selected article)."""
    with open(path, "w") as fh:
        json.dump([asdict(r) for r in records], fh, indent=2)


def records_to_papers_rows(records: list[DrawRecord]) -> list[dict]:
    """Project draw records onto the papers.csv columns the protocol adds:
    sample_source / selection_probability / design_weight (schema.md §7)."""
    rows = []
    for r in records:
        rows.append({
            "article_id": r.article_id,
            "field": r.field,
            "sample_source": "random_base",
            "selection_probability": r.selection_probability,
            "design_weight": r.design_weight,
            "_provenance": {
                "cell_id": r.cell_id, "journal": r.journal, "issue": r.issue,
                "J": r.n_journals_J, "I_j": r.n_issues_Ij, "N_i": r.n_articles_Ni,
                "seed": r.seed, "draw_index": r.draw_index,
            },
        })
    return rows


def load_cell(obj: dict) -> Cell:
    """Build a Cell from a plain dict (e.g. parsed JSON enumeration output)."""
    return Cell(
        cell_id=obj["cell_id"], field=obj["field"], era=obj["era"],
        tier=obj["tier"], journals=obj["journals"],
    )


if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser(description="Run a Stratum A cell draw.")
    ap.add_argument("frame_json", help="path to an enumerated cell (JSON)")
    ap.add_argument("--n", type=int, required=True, help="target draws for the cell")
    ap.add_argument("--seed", type=int, required=True, help="RNG seed (record this)")
    ap.add_argument("--existing", default=None,
                    help="optional JSON list of article ids already in the corpus")
    ap.add_argument("--out", default=None, help="audit-log output path (JSON)")
    args = ap.parse_args()

    with open(args.frame_json) as fh:
        cell = load_cell(json.load(fh))
    existing = []
    if args.existing:
        with open(args.existing) as fh:
            existing = json.load(fh)

    recs = draw_cell(cell, n_target=args.n, seed=args.seed, existing_ids=existing)
    for r in recs:
        print(f"{r.article_id}\t{r.journal}\t{r.issue}\t"
              f"p={r.selection_probability:.3e}\tw={r.design_weight:.3f}")
    if args.out:
        write_audit_log(recs, args.out)
        print(f"\naudit log -> {args.out}")
