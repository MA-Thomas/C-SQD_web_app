# Blind Packet Instructions

Blind packets are generated judging packets for prior-evaluation-blind paper scoring.

The goal is not to hide paper identity. Judges may see the paper title, authors, year, journal, DOI, PDF, text, and rubric. The goal is to prevent judges from seeing how the same paper was previously evaluated on the rubric.

## Generate Or Refresh Packets

From the repository workspace, run:

```bash
python3 corpus/scripts/build_blind_packets.py --round-id round_001 --clean
```

The generator reads `corpus/metadata/papers.csv`. If new papers are added to that registry with valid `pdf_path` and `text_path` values, rerunning the command creates matching packet folders automatically.

Use a new `round_id` when packets have already been distributed and you want a frozen audit trail:

```bash
python3 corpus/scripts/build_blind_packets.py --round-id round_002 --clean
```

## Packet Contents

Each paper packet contains:

- `paper.pdf`
- `paper.txt`
- `paper_metadata.json`
- `coding_guide.md`
- `judge_instructions.md`
- `blank_rating_form.json`
- `packet_manifest.json`

The round folder also contains `_round_manifest.json`.

## Excluded By Design

Packets do not include:

- previous paper classifications,
- causal-abstraction scores,
- statistical-inductivist scores,
- outcome scores,
- multi-judge raw ratings,
- aggregate scores,
- previous rationales,
- field labels,
- citation counts,
- outcome information.

## Human Judge Workflow

Send a judge only the packet folder for the assigned paper or papers. Do not send the master corpus folder.

The judge should complete `blank_rating_form.json` or use it as the response schema. If the judge is accidentally shown prior evaluations for the assigned paper, they should stop and report the exposure.

## AI Chatbot Workflow

For each AI chatbot round, start a fresh session with no prior conversation history. Provide only:

- the packet contents,
- the instruction to score independently,
- the requirement to return the completed JSON form.

For repeated AI rounds, create one `judge_round_id` per independent replicate in `corpus/coding/multi_judge/judge_rounds.csv`.

When aggregating later, average repeated rounds within chatbot/model before combining them with other AI models or human judges. This avoids pseudo-replication.

## Importing Completed Ratings

The coordinator should import completed forms into:

- `corpus/coding/multi_judge/classification_ratings.csv`
- `corpus/coding/multi_judge/causal_abstraction_ratings.csv`
- `corpus/coding/multi_judge/statistical_inductivist_dependence_ratings.csv`
- `corpus/coding/multi_judge/outcome_ratings.csv`, once outcome coding is active

Judges should not write directly into the master coding files.
