# Pilot Corpus Workspace

This folder is the working scaffold for the "Statistical Induction, Causal Criticism, and the Deceleration of Scientific Progress" corpus.

## Structure

- `pdfs/`: locally downloaded PDFs, organized by field.
- `text/`: extracted full text when available, organized by field.
- `metadata/`: stable bibliographic registry and schema notes.
- `coding/`: paper classifications, provisional scores, outcome scaffold, and coding guide.
- `sources/`: acquisition log and source audit trail.
- `scripts/`: small helper scripts can live here later.
- `blind_packets/`: generated prior-evaluation-blind judging packets.

Papers are stored under `pdfs/<field>/` and `text/<field>/`. The field is a neutral, factual descriptor (e.g., `computational_biology`, `molecular_cell_biology`, `condensed_matter_physics`). It is **not** a paradigm label: paradigm orientation (statistical-inductivist vs causal-Popperian) is an outcome of blinded coding, not of the folder a paper sits in. Placeholder folders such as `particle_gravitational_physics/` and `statistical_physics/` mark fields the corpus is being built to add; see `ingestion_plan_field_differential.md`.

The earlier outcome-defined strata (the A/B/C/D scheme, including the matched-control group) have been retired. Under the cohort design every paper is simply a member of a field; field, publication year, and venue tier are sampling-frame facts and analysis covariates, not comparison groups.

## Pilot Status

The current pilot contains 16 papers across three currently-populated fields:

- `computational_biology/` (8 papers, including statistical genetics).
- `molecular_cell_biology/` (6 papers).
- `condensed_matter_physics/` (2 papers).

As of the current acquisition pass, the corpus has 16 local PDF artifacts and 16 extracted text files. Eleven are downloaded or assembled from native/public archive PDF sources; two were added from user-provided PDFs; three are locally rendered review PDFs from public PMC full-text XML because the native PDF route did not return a verified PDF in this environment.

Coverage is uneven by design intent: it is concentrated in computational biology, lacks the structure-driven and statistically-oriented physics fields, and has no recent mechanistic biology. The gaps and the planned ingestion sequence are documented in `ingestion_plan_field_differential.md`.

Scores are provisional and are meant to support review of the coding rules. They should not be treated as final measurements.

## Important Design Choice

The registry identifies papers. The coding tables interpret papers. Outcome measures are deliberately separate so that later bibliometric or content-analysis workflows do not overwrite the original corpus identity.
