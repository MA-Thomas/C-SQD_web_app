#!/usr/bin/env python3
"""Generate prior-evaluation-blind judging packets from the corpus registry."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import shutil
from datetime import date
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT_ROOT = ROOT / "blind_packets"
PAPERS_CSV = ROOT / "metadata" / "papers.csv"
CODING_GUIDE = ROOT / "coding" / "coding_guide.md"

# Dimension sets are rubric-version specific. The causal-abstraction dimensions are
# unchanged across pilot versions to date; the statistical-inductivist scale was
# de-duplicated in v0.2 (the v0.1 dimensions weak_mechanism, local_validation, and
# limited_intervention were sign-flipped copies of causal dimensions and were removed).
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
    "v0.1-pilot": {
        "causal": list(_CAUSAL_DIMS_COMMON),
        "statistical": [
            "significance_dependence",
            "prediction_dependence",
            "high_dimensional_search",
            "flexible_pipeline",
            "weak_mechanism",
            "local_validation",
            "limited_intervention",
        ],
    },
    "v0.2-pilot": {
        "causal": list(_CAUSAL_DIMS_COMMON),
        "statistical": [
            "significance_dependence",
            "prediction_dependence",
            "high_dimensional_search",
            "flexible_pipeline",
        ],
    },
}

# Outcome dimensions are unchanged across pilot rubric versions to date.
OUTCOME_DIMS = [
    "citation_durability",
    "independent_lab_uptake",
    "mechanistic_uptake",
    "intervention_uptake",
    "ontological_uptake",
    "review_integration",
    "clinical_or_engineering_consequence",
    "replication_or_transport",
    "disruptiveness",
]


def rubric_dimensions(rubric_version: str) -> dict[str, object]:
    if rubric_version not in RUBRIC_DIMENSIONS:
        known = ", ".join(sorted(RUBRIC_DIMENSIONS))
        raise SystemExit(
            f"Unknown rubric_version '{rubric_version}'. Known versions: {known}."
        )
    return RUBRIC_DIMENSIONS[rubric_version]

PACKET_INSTRUCTION_TEXT = """# Judge Instructions

Score this paper independently using the included coding guide and blank rating form.

You may use the paper identity, bibliographic metadata, PDF, text file, and rubric. Do not consult prior evaluations, aggregate scores, coding notes, previous judge rationales, or any files outside this packet while scoring.

If prior evaluations for this paper are accidentally shown to you, stop and report the exposure in your response instead of completing the rating.

Return your completed scores using `blank_rating_form.json` as the schema.
"""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build prior-evaluation-blind packets for all papers in papers.csv."
    )
    parser.add_argument("--round-id", default="round_001", help="Output round folder name.")
    parser.add_argument(
        "--rubric-version",
        default="v0.1-pilot",
        help="Rubric version to record in packet manifests and blank forms.",
    )
    parser.add_argument(
        "--output-root",
        default=str(DEFAULT_OUTPUT_ROOT),
        help="Directory under which round folders are generated.",
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Remove stale paper packet folders in the target round before rebuilding.",
    )
    return parser.parse_args()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_papers() -> list[dict[str, str]]:
    with PAPERS_CSV.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def safe_copy(src: Path, dst: Path) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)


def public_metadata(row: dict[str, str]) -> dict[str, str]:
    return {
        "paper_id": row["paper_id"],
        "paper_title": row["paper_title"],
        "authors": row["authors"],
        "year": row["year"],
        "journal": row["journal"],
        "doi": row["doi"],
        "pmid": row["pmid"],
        "pmcid": row["pmcid"],
        "url": row["url"],
    }


def blank_rating_form(paper_id: str, round_id: str, rubric_version: str) -> dict[str, object]:
    dims = rubric_dimensions(rubric_version)
    causal_dims = dims["causal"]
    stat_dims = dims["statistical"]
    return {
        "paper_id": paper_id,
        "judge_id": "",
        "judge_round_id": "",
        "round_id": round_id,
        "rubric_version": rubric_version,
        "blinding_attestation": {
            "blinded_to_prior_ratings": True,
            "saw_prior_evaluations_for_this_paper": False,
            "attestation_notes": "",
        },
        "classification_rating": {
            "primary_classification": "",
            "secondary_classifications": [],
            "confidence": "",
            "rationale": "",
        },
        "causal_abstraction_rating": {
            **{dim: None for dim in causal_dims},
            "total": None,
            "confidence": "",
            "rationale": "",
        },
        "statistical_inductivist_dependence_rating": {
            **{dim: None for dim in stat_dims},
            "total": None,
            "confidence": "",
            "rationale": "",
        },
        "outcome_rating_optional": {
            **{dim: None for dim in OUTCOME_DIMS},
            "outcome_measurement_status": "not_measured",
            "confidence": "",
            "rationale": "",
        },
    }


def packet_manifest(
    row: dict[str, str],
    round_id: str,
    rubric_version: str,
    packet_dir: Path,
    copied_files: list[str],
) -> dict[str, object]:
    file_info = {}
    for filename in copied_files:
        path = packet_dir / filename
        file_info[filename] = {
            "bytes": path.stat().st_size,
            "sha256": sha256(path),
        }
    return {
        "packet_version": "1.0",
        "generated_date": date.today().isoformat(),
        "round_id": round_id,
        "rubric_version": rubric_version,
        "paper_id": row["paper_id"],
        "paper_identity_visible": True,
        "prior_evaluation_blinded": True,
        "metadata": public_metadata(row),
        "included_files": copied_files,
        "manifest_file": "packet_manifest.json",
        "file_checksums": file_info,
        "excluded_by_design": [
            "corpus/coding/causal_abstraction_scores.csv",
            "corpus/coding/statistical_inductivist_dependence_scores.csv",
            "corpus/coding/paper_classifications.csv",
            "corpus/coding/outcomes.csv",
            "corpus/coding/multi_judge/*ratings.csv",
            "corpus/coding/multi_judge/*aggregates.csv",
            "corpus/metadata/papers.csv fields for corpus stratum, pilot role, matching notes, citation counts, and prior notes",
        ],
    }


def build_packet(row: dict[str, str], round_dir: Path, round_id: str, rubric_version: str) -> dict[str, object]:
    paper_id = row["paper_id"]
    packet_dir = round_dir / paper_id
    packet_dir.mkdir(parents=True, exist_ok=True)

    copied_files = []
    pdf_path = ROOT.parent / row["pdf_path"] if row["pdf_path"].startswith("corpus/") else Path(row["pdf_path"])
    text_path = ROOT.parent / row["text_path"] if row["text_path"].startswith("corpus/") else Path(row["text_path"])

    safe_copy(pdf_path, packet_dir / "paper.pdf")
    copied_files.append("paper.pdf")
    safe_copy(text_path, packet_dir / "paper.txt")
    copied_files.append("paper.txt")
    safe_copy(CODING_GUIDE, packet_dir / "coding_guide.md")
    copied_files.append("coding_guide.md")

    (packet_dir / "judge_instructions.md").write_text(PACKET_INSTRUCTION_TEXT, encoding="utf-8")
    copied_files.append("judge_instructions.md")

    (packet_dir / "paper_metadata.json").write_text(
        json.dumps(public_metadata(row), indent=2) + "\n",
        encoding="utf-8",
    )
    copied_files.append("paper_metadata.json")

    (packet_dir / "blank_rating_form.json").write_text(
        json.dumps(blank_rating_form(paper_id, round_id, rubric_version), indent=2) + "\n",
        encoding="utf-8",
    )
    copied_files.append("blank_rating_form.json")

    manifest = packet_manifest(row, round_id, rubric_version, packet_dir, copied_files)
    (packet_dir / "packet_manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n",
        encoding="utf-8",
    )
    copied_files.append("packet_manifest.json")

    return {
        "paper_id": paper_id,
        "packet_path": str(packet_dir.relative_to(ROOT)),
        "status": "generated",
        "included_files": copied_files,
    }


def main() -> None:
    args = parse_args()
    output_root = Path(args.output_root)
    if not output_root.is_absolute():
        output_root = ROOT / output_root
    round_dir = output_root / args.round_id
    round_dir.mkdir(parents=True, exist_ok=True)

    papers = read_papers()
    ready = [row for row in papers if row.get("pdf_path") and row.get("text_path")]
    skipped = [
        {
            "paper_id": row["paper_id"],
            "reason": "missing pdf_path or text_path",
        }
        for row in papers
        if not row.get("pdf_path") or not row.get("text_path")
    ]

    if args.clean:
        current_ids = {row["paper_id"] for row in ready}
        for child in round_dir.iterdir():
            if child.is_dir() and child.name not in current_ids:
                shutil.rmtree(child)

    packet_rows = [
        build_packet(row, round_dir, args.round_id, args.rubric_version)
        for row in sorted(ready, key=lambda item: item["paper_id"])
    ]

    round_manifest = {
        "round_id": args.round_id,
        "rubric_version": args.rubric_version,
        "generated_date": date.today().isoformat(),
        "prior_evaluation_blinded": True,
        "paper_identity_visible": True,
        "source_registry": str(PAPERS_CSV.relative_to(ROOT)),
        "packet_count": len(packet_rows),
        "skipped_count": len(skipped),
        "packets": packet_rows,
        "skipped": skipped,
        "exclusion_note": "Packets do not include prior scoring files, aggregate files, coding notes, stratum labels, matching notes, citation counts, or outcome information.",
    }
    (round_dir / "_round_manifest.json").write_text(
        json.dumps(round_manifest, indent=2) + "\n",
        encoding="utf-8",
    )

    print(
        json.dumps(
            {
                "round_dir": str(round_dir.relative_to(ROOT)),
                "packet_count": len(packet_rows),
                "skipped_count": len(skipped),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
