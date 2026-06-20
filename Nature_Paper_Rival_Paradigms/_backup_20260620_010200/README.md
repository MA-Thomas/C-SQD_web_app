# Pilot Corpus Workspace

This folder is the working scaffold for the "Statistical Induction, Causal Criticism, and the Deceleration of Scientific Progress" corpus.

## Structure

- `pdfs/`: locally downloaded PDFs, organized by corpus stratum.
- `text/`: extracted full text when available.
- `metadata/`: stable bibliographic registry and schema notes.
- `coding/`: paper classifications, provisional scores, outcome scaffold, and coding guide.
- `sources/`: acquisition log and source audit trail.
- `scripts/`: small helper scripts can live here later.
- `blind_packets/`: generated prior-evaluation-blind judging packets.

## Pilot Status

The current pilot contains 16 candidate papers across four strata:

- Stratum A: Nobel-related discovery papers.
- Stratum B: candidate matched high-impact contemporaneous controls.
- Stratum C: recent high-prestige frontier papers.
- Stratum D: computational biology papers.

As of the current acquisition pass, the corpus has 16 local PDF artifacts and 16 extracted text files. Eleven are downloaded or assembled from native/public archive PDF sources; two were added from user-provided PDFs; three are locally rendered review PDFs from public PMC full-text XML because the native PDF route did not return a verified PDF in this environment.

Scores are provisional and are meant to support review of the coding rules. They should not be treated as final measurements.

## Important Design Choice

The registry identifies papers. The coding tables interpret papers. Outcome measures are deliberately separate so that later bibliometric or content-analysis workflows do not overwrite the original corpus identity.
