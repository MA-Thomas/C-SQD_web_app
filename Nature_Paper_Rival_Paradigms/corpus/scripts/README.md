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
- `draw_stratum_a.py`: legacy seeded Stratum A draw helper for an already-enumerated frame.
  The current protocol uses source blocks: the active source is chosen by the predeclared
  source-block schedule, then the helper's unit -> paper randomization logic applies inside
  that source. For source-block draws, the per-paper probability is
  `p_a = (1/U_v)(1/N_u)`; the older helper also supports a venue/source factor for historical
  frames. Its current JSON keys use `journals` and `issues` for historical reasons;
  conceptually those are venues/sources and archival units. Enumeration (publisher archives,
  proceedings indexes, PubMed / Crossref / OpenAlex, arXiv as access metadata, etc.) is a
  separate upstream step and must come from an authoritative source, never from model recall.
  Run:
  `python3 draw_stratum_a.py <frame.json> --n 5 --seed <int> --out log.json`.
- `expand_random_base_batch.py`: direct-route batch acquisition helper for already scheduled
  source blocks. It verifies downloaded bytes, records source-block probabilities, and
  audits failures. A direct-route failure should now be queued for the publisher-page browser
  fallback described in `selection_protocol.md` §1.4.1 before a source block is treated as
  truly acquisition-limited.
- `test_draw_stratum_a.py`: verification suite for the engine (reproducibility, selection
  frequencies vs model, Hansen-Hurwitz unbiasedness with/without replacement, dedup, exhaustion).
  Run: `python3 test_draw_stratum_a.py` (10 checks, all passing).
