# Scripts

This folder contains small, auditable helpers, such as:

- DOI metadata refresh from OpenAlex or Crossref.
- Open-access PDF acquisition.
- PDF-to-text extraction.
- Bibliometric outcome measurement.
- Blind packet generation for prior-evaluation-blind judging.
- Multi-judge aggregate rebuilding.

Current helpers:

- `build_blind_packets.py`: generates `corpus/blind_packets/<round_id>/` from `papers.csv`.
- `build_multi_judge_aggregates.py`: rebuilds aggregate tables from `corpus/coding/multi_judge/` raw ratings.
- `draw_stratum_a.py`: seeded Stratum A within-cell draw (journal -> issue -> article) per
  `selection_protocol.md` §1.2-1.3. Computes each article's per-draw selection probability
  `p_a = (1/J)(1/I_j)(1/N_i)` and Hansen-Hurwitz design weight `1/(n*p_a)`, deduplicates
  against the existing corpus, and writes a full audit log. It consumes an **already-enumerated**
  frame (`{cell -> journals -> {issue -> [eligible OA article ids]}}`) and does the random
  selection only; enumeration (PubMed / Crossref / arXiv / browsing a journal's table of
  contents) is a separate upstream step and must come from an authoritative source, never from
  model recall. Run: `python3 draw_stratum_a.py <frame.json> --n 5 --seed <int> --out log.json`.
- `test_draw_stratum_a.py`: verification suite for the engine (reproducibility, selection
  frequencies vs model, Hansen-Hurwitz unbiasedness with/without replacement, dedup, exhaustion).
  Run: `python3 test_draw_stratum_a.py` (10 checks, all passing).
