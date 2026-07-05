#!/usr/bin/env python3
"""Build long-format aggregate tables from multi-judge corpus ratings.

Aggregation is driven by ``aggregation_sets.csv`` and is rubric-version aware. Each
aggregation set declares the rounds it covers (``eligible_round_ids``), the rubric
version it uses, and the judge types it includes. Ratings are filtered to that set
before pooling, so ratings produced under different rubric versions (for example the
v0.0-seed 0-2 scores and the v0.1-pilot 0-4 scores) are never averaged together.

Sets whose ``eligible_round_ids`` is empty or still ``TBD`` are skipped as not yet
runnable.
"""

from __future__ import annotations

import csv
import statistics
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MULTI = ROOT / "coding" / "multi_judge"

# Dimension sets are rubric-version specific and must match build_blind_packets.py and
# coding_guide.md. Two versions exist after the consolidation rename:
#   * v0.0-seed   - the original 0-2 seed rubric (7-dim statistical scale, no markers);
#                   owns the frozen Codex seed round only.
#   * v0.1-pilot  - the consolidated live rubric (0-4, de-duplicated 5-dim statistical
#                   scale, plus the paradigm-marker language instrument).
_CAUSAL_DIMS_COMMON = [
    "entity_specification",
    "causal_relation",
    "mechanism",
    "intervention_relevance",
    "invariance",
    "rival_explanations",
    "severe_test",
    "measurement_model",
    "abstraction_discipline",
]

RUBRIC_DIMENSIONS = {
    "v0.0-seed": {
        "causal": list(_CAUSAL_DIMS_COMMON),
        "statistical": [
            "significance_dependence",
            "terminal_model_warrant",
            "covariate_control_warrant",
            "high_dimensional_search",
            "flexible_pipeline",
            "weak_mechanism",
            "local_validation",
            "limited_intervention",
        ],
    },
    # Consolidated live rubric: de-duplicated statistical scale plus the paradigm-marker
    # language instrument (move-coded, 0-4). cp_ dimensions are occasion-gated; NA (blank)
    # cells are dropped from the mean by `numeric`.
    "v0.1-pilot": {
        "causal": list(_CAUSAL_DIMS_COMMON),
        "statistical": [
            "significance_dependence",
            "terminal_model_warrant",
            "covariate_control_warrant",
            "high_dimensional_search",
            "flexible_pipeline",
        ],
        "paradigm_marker": [
            "cp_risky_prediction",
            "cp_rival_elimination",
            "cp_generative_structure",
            "cp_counterfactual_intervention",
            "cp_assumption_vulnerability",
            "cp_theory_relative_contrast",
            "si_terminal_certification",
            "si_association_framing",
            "si_accumulation_progress",
        ],
    },
}

RUBRIC_SCORE_MAX = {
    "v0.0-seed": 2,
    "v0.1-pilot": 4,
}

HIGH_SCORE_THRESHOLD = 3.0
EXTREME_SCORE_THRESHOLD = 4.0


def dims_for(rubric_version: str) -> tuple[list[str], list[str], list[str]]:
    if rubric_version not in RUBRIC_DIMENSIONS:
        known = ", ".join(sorted(RUBRIC_DIMENSIONS))
        raise SystemExit(
            f"Unknown rubric_version '{rubric_version}' in aggregation_sets.csv. "
            f"Known versions: {known}."
        )
    spec = RUBRIC_DIMENSIONS[rubric_version]
    return spec["causal"], spec["statistical"], spec.get("paradigm_marker", [])


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def write_csv(path: Path, rows: list[dict[str, object]], fieldnames: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def split_list(value: str) -> list[str]:
    """Split a semicolon- or comma-delimited cell into stripped tokens."""
    if not value:
        return []
    out: list[str] = []
    for chunk in value.replace(";", ",").split(","):
        token = chunk.strip()
        if token:
            out.append(token)
    return out


def is_runnable(eligible_round_ids: list[str]) -> bool:
    return bool(eligible_round_ids) and not any(
        token.upper() == "TBD" for token in eligible_round_ids
    )


def truthy(value: str) -> bool:
    return value.strip().lower() in {"true", "1", "yes"}


def judge_round_lookup() -> dict[str, dict[str, str]]:
    judges = {row["judge_id"]: row for row in read_csv(MULTI / "judges.csv")}
    rounds = {}
    for row in read_csv(MULTI / "judge_rounds.csv"):
        enriched = dict(row)
        judge = judges.get(row["judge_id"], {})
        enriched["judge_type"] = judge.get("judge_type", "")
        enriched["model_name"] = judge.get("model_name", "")
        rounds[row["judge_round_id"]] = enriched
    return rounds


def numeric(value: str | None) -> float | None:
    if value is None or value == "":
        return None
    return float(value)


def high_thresholds_active(rubric_version: str) -> bool:
    return RUBRIC_SCORE_MAX.get(rubric_version, 4) >= 4


def score_scale_features(
    row: dict[str, str], dims: list[str], rubric_version: str
) -> dict[str, float]:
    values = [numeric(row.get(dim, "")) for dim in dims]
    values = [value for value in values if value is not None]
    if not values:
        return {}

    features = {
        "scale_mean": statistics.mean(values),
        "scale_max": max(values),
    }
    if high_thresholds_active(rubric_version):
        high_count = sum(value >= HIGH_SCORE_THRESHOLD for value in values)
        extreme_count = sum(value >= EXTREME_SCORE_THRESHOLD for value in values)
        features.update(
            {
                "scale_count_high_ge3": float(high_count),
                "scale_any_high_ge3": float(high_count > 0),
                "scale_count_extreme_eq4": float(extreme_count),
                "scale_any_extreme_eq4": float(extreme_count > 0),
            }
        )
    return features


def occasion_is_active(row: dict[str, str], dim: str) -> bool:
    occasion = numeric(row.get(f"{dim}_occasion", ""))
    return occasion == 1


def should_aggregate_dimension(
    instrument: str, dim: str, row: dict[str, str]
) -> bool:
    if (
        instrument == "paradigm_marker"
        and dim.startswith("cp_")
        and f"{dim}_occasion" in row
    ):
        return occasion_is_active(row, dim)
    return True


def marker_derived_features(
    row: dict[str, str], marker_dims: list[str], rubric_version: str
) -> dict[str, float]:
    if not high_thresholds_active(rubric_version):
        return {}

    cp_values = [
        numeric(row.get(dim, ""))
        for dim in marker_dims
        if dim.startswith("cp_") and occasion_is_active(row, dim)
    ]
    cp_values = [value for value in cp_values if value is not None]

    si_values = [
        numeric(row.get(dim, ""))
        for dim in marker_dims
        if dim.startswith("si_")
    ]
    si_values = [value for value in si_values if value is not None]

    features: dict[str, float] = {}
    if cp_values:
        cp_high_count = sum(value >= HIGH_SCORE_THRESHOLD for value in cp_values)
        cp_extreme_count = sum(value >= EXTREME_SCORE_THRESHOLD for value in cp_values)
        features.update(
            {
                "cp_marker_max_applicable": max(cp_values),
                "cp_marker_count_high_ge3": float(cp_high_count),
                "cp_marker_any_high_ge3": float(cp_high_count > 0),
                "cp_marker_count_extreme_eq4": float(cp_extreme_count),
                "cp_marker_any_extreme_eq4": float(cp_extreme_count > 0),
            }
        )
    if si_values:
        si_high_count = sum(value >= HIGH_SCORE_THRESHOLD for value in si_values)
        si_extreme_count = sum(value >= EXTREME_SCORE_THRESHOLD for value in si_values)
        features.update(
            {
                "si_marker_max": max(si_values),
                "si_marker_count_high_ge3": float(si_high_count),
                "si_marker_any_high_ge3": float(si_high_count > 0),
                "si_marker_count_extreme_eq4": float(si_extreme_count),
                "si_marker_any_extreme_eq4": float(si_extreme_count > 0),
            }
        )
    return features


def eligibility_filter(agg_set: dict[str, str], round_meta: dict[str, dict[str, str]]):
    eligible_rounds = set(split_list(agg_set.get("eligible_round_ids", "")))
    rubric_version = agg_set["rubric_version"]
    type_filter = set(split_list(agg_set.get("judge_type_filter", "")))

    def keep(row: dict[str, str]) -> bool:
        meta = round_meta.get(row["judge_round_id"])
        if meta is None:
            return False
        if row.get("rubric_version") != rubric_version:
            return False
        if meta.get("round_id") not in eligible_rounds:
            return False
        if type_filter and meta.get("judge_type") not in type_filter:
            return False
        return True

    return keep


def unit_values(
    ratings: list[dict[str, object]],
    round_meta: dict[str, dict[str, str]],
    replicate_first: bool,
) -> list[float]:
    """Return the values to aggregate.

    When ``replicate_first`` is true, AI ratings are averaged within model first (so a
    chatbot sampled many times contributes a single model-level value), while each human
    judge contributes individually. Otherwise every rating is pooled directly.
    """
    raw = [float(r["value"]) for r in ratings]
    if not replicate_first:
        return raw

    buckets: dict[tuple[str, str], list[float]] = defaultdict(list)
    for r in ratings:
        meta = round_meta[r["judge_round_id"]]
        if meta.get("judge_type") == "ai_chatbot":
            unit = ("ai_model", meta.get("model_name") or meta["judge_id"])
        elif meta.get("judge_type") == "human":
            unit = ("human", meta["judge_id"])
        else:
            unit = ("other", r["judge_round_id"])
        buckets[unit].append(float(r["value"]))
    return [statistics.mean(vals) for vals in buckets.values()]


def aggregate_scores(
    agg_sets: list[dict[str, str]], round_meta: dict[str, dict[str, str]]
) -> None:
    rows = []
    for agg_set in agg_sets:
        eligible = split_list(agg_set.get("eligible_round_ids", ""))
        if not is_runnable(eligible):
            continue
        set_id = agg_set["aggregation_set_id"]
        rubric_version = agg_set["rubric_version"]
        replicate_first = truthy(agg_set.get("aggregate_ai_replicates_first", ""))
        causal_dims, stat_dims, marker_dims = dims_for(rubric_version)
        keep = eligibility_filter(agg_set, round_meta)
        policy = (
            "within_model_mean_then_unit_mean" if replicate_first else "pooled_mean"
        )
        set_notes = agg_set.get("notes") or agg_set.get("description", "")

        instruments = [
            (
                "causal_abstraction",
                MULTI / "causal_abstraction_ratings.csv",
                causal_dims + ["total"],
            ),
            (
                "statistical_inductivist_dependence",
                MULTI / "statistical_inductivist_dependence_ratings.csv",
                stat_dims + ["total"],
            ),
        ]
        # Paradigm-marker instrument exists only for rubric versions that declare it
        # (v0.3+) and only if its ratings file is present. cp_ summary means and the
        # exclusion rate are aggregated alongside the raw move dimensions.
        marker_path = MULTI / "paradigm_marker_ratings.csv"
        if marker_dims and marker_path.exists():
            instruments.append(
                (
                    "paradigm_marker",
                    marker_path,
                    marker_dims
                    + [
                        "cp_marker_mean",
                        "cp_applicable_count",
                        "si_marker_mean",
                        "cp_exclusion_rate",
                    ],
                )
            )

        grouped: dict[tuple[str, str, str], list[dict[str, object]]] = defaultdict(list)
        for instrument, path, dims in instruments:
            for row in read_csv(path):
                if not keep(row):
                    continue
                for dim in dims:
                    if not should_aggregate_dimension(instrument, dim, row):
                        continue
                    value = numeric(row.get(dim, ""))
                    if value is None:
                        continue
                    item = dict(row)
                    item["instrument"] = instrument
                    item["dimension"] = dim
                    item["value"] = value
                    grouped[(row["paper_id"], instrument, dim)].append(item)
                if instrument == "causal_abstraction":
                    derived = score_scale_features(row, causal_dims, rubric_version)
                elif instrument == "statistical_inductivist_dependence":
                    derived = score_scale_features(row, stat_dims, rubric_version)
                elif instrument == "paradigm_marker":
                    derived = marker_derived_features(row, marker_dims, rubric_version)
                else:
                    derived = {}
                for dim, value in derived.items():
                    item = dict(row)
                    item["instrument"] = instrument
                    item["dimension"] = dim
                    item["value"] = value
                    grouped[(row["paper_id"], instrument, dim)].append(item)

        for (paper_id, instrument, dim), ratings in sorted(grouped.items()):
            agg_values = unit_values(ratings, round_meta, replicate_first)
            judge_rounds = {r["judge_round_id"] for r in ratings}
            judge_ids = {round_meta[jr]["judge_id"] for jr in judge_rounds}
            human_judges = {
                round_meta[jr]["judge_id"]
                for jr in judge_rounds
                if round_meta[jr]["judge_type"] == "human"
            }
            ai_judges = {
                round_meta[jr]["judge_id"]
                for jr in judge_rounds
                if round_meta[jr]["judge_type"] == "ai_chatbot"
            }
            ai_models = {
                round_meta[jr]["model_name"]
                for jr in judge_rounds
                if round_meta[jr]["judge_type"] == "ai_chatbot"
                and round_meta[jr].get("model_name")
            }
            rows.append(
                {
                    "aggregate_id": f"{set_id}:{paper_id}:{instrument}:{dim}",
                    "paper_id": paper_id,
                    "instrument": instrument,
                    "dimension": dim,
                    "aggregation_set_id": set_id,
                    "rubric_version": rubric_version,
                    "n_ratings": len(ratings),
                    "n_distinct_judges": len(judge_ids),
                    "n_human_judges": len(human_judges),
                    "n_ai_judges": len(ai_judges),
                    "n_ai_models": len(ai_models),
                    "n_ai_replicates": len(judge_rounds),
                    "mean": f"{statistics.mean(agg_values):.4g}",
                    "median": f"{statistics.median(agg_values):.4g}",
                    "sd": f"{statistics.stdev(agg_values):.4g}"
                    if len(agg_values) > 1
                    else "",
                    "min": f"{min(agg_values):.4g}",
                    "max": f"{max(agg_values):.4g}",
                    "aggregation_policy": policy,
                    "adjudicated_value": "",
                    "adjudication_status": "not_adjudicated",
                    "notes": set_notes,
                }
            )

    rows.sort(key=lambda r: r["aggregate_id"])
    write_csv(
        MULTI / "score_aggregates_long.csv",
        rows,
        [
            "aggregate_id",
            "paper_id",
            "instrument",
            "dimension",
            "aggregation_set_id",
            "rubric_version",
            "n_ratings",
            "n_distinct_judges",
            "n_human_judges",
            "n_ai_judges",
            "n_ai_models",
            "n_ai_replicates",
            "mean",
            "median",
            "sd",
            "min",
            "max",
            "aggregation_policy",
            "adjudicated_value",
            "adjudication_status",
            "notes",
        ],
    )


def aggregate_classifications(
    agg_sets: list[dict[str, str]], round_meta: dict[str, dict[str, str]]
) -> None:
    rows = []
    for agg_set in agg_sets:
        eligible = split_list(agg_set.get("eligible_round_ids", ""))
        if not is_runnable(eligible):
            continue
        set_id = agg_set["aggregation_set_id"]
        rubric_version = agg_set["rubric_version"]
        keep = eligibility_filter(agg_set, round_meta)
        set_notes = agg_set.get("notes") or agg_set.get("description", "")

        grouped: dict[str, list[dict[str, str]]] = defaultdict(list)
        for row in read_csv(MULTI / "classification_ratings.csv"):
            if not keep(row):
                continue
            grouped[row["paper_id"]].append(row)

        for paper_id, ratings in sorted(grouped.items()):
            primaries = [
                r["primary_classification"] for r in ratings if r["primary_classification"]
            ]
            counts = Counter(primaries)
            mode, count = counts.most_common(1)[0] if counts else ("", 0)
            judge_rounds = {r["judge_round_id"] for r in ratings}
            judge_ids = {round_meta[jr]["judge_id"] for jr in judge_rounds}
            secondaries = sorted(
                {
                    item.strip()
                    for r in ratings
                    for item in r.get("secondary_classifications", "")
                    .replace(";", "|")
                    .split("|")
                    if item.strip()
                }
            )
            rows.append(
                {
                    "aggregate_id": f"{set_id}:{paper_id}:classification",
                    "paper_id": paper_id,
                    "aggregation_set_id": set_id,
                    "rubric_version": rubric_version,
                    "n_ratings": len(ratings),
                    "n_distinct_judges": len(judge_ids),
                    "primary_mode": mode,
                    "primary_mode_count": count,
                    "primary_mode_share": f"{count / len(ratings):.4g}" if ratings else "",
                    "secondary_labels_observed": "; ".join(secondaries),
                    "adjudicated_primary": "",
                    "adjudication_status": "not_adjudicated",
                    "notes": set_notes,
                }
            )

    rows.sort(key=lambda r: r["aggregate_id"])
    write_csv(
        MULTI / "classification_aggregates.csv",
        rows,
        [
            "aggregate_id",
            "paper_id",
            "aggregation_set_id",
            "rubric_version",
            "n_ratings",
            "n_distinct_judges",
            "primary_mode",
            "primary_mode_count",
            "primary_mode_share",
            "secondary_labels_observed",
            "adjudicated_primary",
            "adjudication_status",
            "notes",
        ],
    )


def main() -> None:
    agg_sets = read_csv(MULTI / "aggregation_sets.csv")
    round_meta = judge_round_lookup()
    aggregate_scores(agg_sets, round_meta)
    aggregate_classifications(agg_sets, round_meta)


if __name__ == "__main__":
    main()
