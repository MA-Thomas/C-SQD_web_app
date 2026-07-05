# Multi-Judge Coding Layer

This folder normalizes corpus coding for multiple human judges, multiple AI chatbot judges, and repeated blinded AI judging rounds.

The original files one level up remain convenient pilot summaries. The files here are the aggregation-ready source tables.

## Core Design

- `judges.csv`: one row per human judge or AI chatbot/model identity.
- `rounds.csv`: one row per judging design, including blinding conditions.
- `judge_rounds.csv`: one row per judge assignment or independent AI replicate. For AI, repeated blinded rounds should be separate `judge_round_id` values tied to the same `judge_id`.
- `classification_ratings.csv`: raw classification ratings, one row per paper per judge-round.
- `causal_abstraction_ratings.csv`: raw causal-abstraction scores, one row per paper per judge-round.
- `statistical_inductivist_dependence_ratings.csv`: raw statistical-inductivist scores, one row per paper per judge-round.
- `outcome_ratings.csv`: raw outcome ratings once a downstream outcome workflow exists.
- `aggregation_sets.csv`: explicit rules for how raw ratings should be pooled.
- `score_aggregates_long.csv`: long-format numeric aggregates by paper, instrument, and dimension.
- `classification_aggregates.csv`: aggregate classification modes by paper.

## Recommended AI Round Structure

For AI chatbot judging, use repeated blinded rounds, but avoid treating every repeat as a fully independent judge in the final panel.

Recommended hierarchy:

1. Within each chatbot/model, run several independent blinded replicates.
2. Average those replicates into a model-level score.
3. Aggregate model-level scores with human judge scores according to a declared `aggregation_set_id`.

This avoids pseudo-replication, where one chatbot with many repeated rounds overwhelms human judges or other AI models.

## Blinding Fields

Each raw rating records whether the judge-round was blinded to:

- field,
- outcome information,
- prior ratings,
- paper identity.

Paper identity may be hard to blind for famous papers, but the field should still record the intended condition.

## Current Seed Data

The current rows are seeded from Codex's initial pilot judgments:

- `judge_id`: `ai_codex_initial`
- `round_id`: `round_pilot_seed_001`
- `judge_round_id`: `jr_codex_initial_round_001`

These seed judgments are explicitly marked as non-blinded and not adjudication-eligible. They are useful for testing the rubric and the aggregation pipeline, not as final independent evidence.

## Rebuilding Aggregates

After adding raw ratings, run:

```bash
python3 corpus/scripts/build_multi_judge_aggregates.py
```

The script rebuilds:

- `score_aggregates_long.csv`
- `classification_aggregates.csv`
