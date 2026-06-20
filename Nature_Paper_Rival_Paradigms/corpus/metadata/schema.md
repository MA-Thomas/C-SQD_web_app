# Corpus Schema

## `papers.csv`

One row per paper. This table should remain stable once a paper enters the corpus.

- `paper_id`: stable local identifier.
- `field`: neutral field/subfield descriptor (e.g., `computational biology`, `molecular biology`, `condensed-matter physics`). A factual descriptor, **not** a paradigm label; paradigm orientation comes from blinded coding. Also determines the storage subfolder under `pdfs/` and `text/`.
- `venue_tier`: `top` / `mid` / `specialist`. Recorded because the reward-step hypothesis concerns what prestige venues reward.
- `paper_title`: title as recorded in the source metadata.
- `authors`: compact author string.
- `year`: publication year. Era groupings are derived in analysis, not stored as a label.
- `journal`: journal or venue.
- `doi`: DOI where available.
- `pmid`: PubMed identifier if already known.
- `pmcid`: PubMed Central identifier if already known.
- `url`: DOI, publisher page, repository page, or open landing page.
- `pdf_url`: direct PDF URL when found.
- `pdf_path`: local PDF path (`corpus/pdfs/<field>/<paper_id>.pdf`) if the PDF was successfully downloaded and verified as a PDF.
- `text_path`: extracted local text path (`corpus/text/<field>/<paper_id>.txt`) when available.
- `oa_status`: open-access status from source metadata or manual inspection.
- `download_status`: local acquisition status.
- `openalex_cited_by_count`: OpenAlex citation count observed during pilot setup. Treated as an outcome, not a sampling weight.
- `source_checked_date`: date metadata/access status was checked.
- `notes`: free-text audit notes.
- `sample_source`: `random_base` / `landmark`. Provenance only — records *how the paper entered the corpus*, not what it is. Not a paradigm or role label. `random_base` = drawn by the stratified within-cell random procedure in `selection_protocol.md`; `landmark` = pre-specified recognized paper used for rubric calibration and contrast, never pooled into base-rate estimates. Excluded from blind packets.
- `selection_probability`: the article's inclusion probability under the `selection_protocol.md` §1.2 draw. Populated for `random_base` papers only; blank for `landmark`.
- `design_weight`: inverse-probability design weight used in Stratum A (random-base) estimates. Populated for `random_base` papers only; blank for `landmark`.

Retired columns (legacy of the matched-control design, removed): `corpus_stratum`, `corpus_stratum_label`, `pilot_role`, `matched_to_paper_id`, `matching_notes`. The outcome-defined A/B/C/D strata no longer exist; field, year, and venue tier are the sampling-frame descriptors.

Observed `download_status` values in the pilot include:

- `downloaded_verified_pdf`: a source PDF was downloaded and verified as a PDF.
- `assembled_from_public_archive_images`: a PDF was assembled from public archive page images.
- `generated_pdf_from_pmc_xml`: a review PDF was generated locally from public PMC full-text XML because the native PDF route did not return a verified PDF.
- `manual_access_required`: no verified public PDF copy was acquired in the automated pass.

## Field-Level Reference Table

`metadata/fields.csv` — one row per field, holding facts that belong to a *field*, not to any single paper, and kept out of `papers.csv` for that reason.

- `field`: field descriptor, matching `papers.csv`.
- `ml_penetration_onset_early` / `ml_penetration_onset_late`: onset of machine-learning penetration expressed as a band, not a point. "ML adoption" has at least two defensible dates — method onset versus dominance — often a decade apart, so the onset is dated with its uncertainty carried explicitly.
- `ml_penetration_source`: the documented basis for the dates.
- `ml_penetration_applicability`: `high` / `low` / `not_applicable`. Flags fields (e.g. structure-driven experimental physics, mechanistic biology) where ML penetration is minimal. These are negative controls for the secondary timing hypothesis (TS1) but full participants in the primary temporal tier.
- `notes`: free text.

This table feeds the **secondary** machine-learning-timing hypothesis (TS1) only. The primary temporal tier — orientation drift and conversion-rate decline (TP1–TP5) — is measured directly from the era-stratified coded cohort and never reads this table, so the primary analysis can proceed before any penetration date is settled.

## Coding Tables

The coding tables use `paper_id` as the join key. This lets scores change without editing the bibliographic registry.

- `paper_classifications.csv`: Section 10 epistemic classification.
- `causal_abstraction_scores.csv`: Causal-Abstraction Commitment Score dimensions.
- `statistical_inductivist_dependence_scores.csv`: Statistical-Inductivist Dependence Score dimensions.
- `outcomes.csv`: downstream uptake dimensions, mostly awaiting a dedicated measurement workflow.

## Multi-Judge Coding Tables

`corpus/coding/multi_judge/` contains the normalized, aggregation-ready coding layer.

- `judges.csv`: human and AI judge identities, with AI model/provider fields.
- `rounds.csv`: judging-round design, including blinding conditions.
- `judge_rounds.csv`: one assignment or replicate per judge per round; repeated AI chatbot rounds get separate rows here.
- `classification_ratings.csv`: raw classifications by paper and judge-round.
- `causal_abstraction_ratings.csv`: raw causal-abstraction ratings by paper and judge-round.
- `statistical_inductivist_dependence_ratings.csv`: raw statistical-inductivist ratings by paper and judge-round.
- `outcome_ratings.csv`: raw outcome ratings once outcome coding is defined.
- `aggregation_sets.csv`: declared pooling rules, including how AI replicates should be handled.
- `score_aggregates_long.csv`: recomputed numeric aggregates in long format.
- `classification_aggregates.csv`: recomputed classification aggregates.

For final analyses, prefer the multi-judge raw tables plus a declared aggregation set over the one-row-per-paper pilot summary files.

The primary temporal analysis aggregates the dependence and commitment scores and the classification mix by `field × era`, where `era` is derived from `papers.csv` `year`. Use `era` as a standard grouping key over `score_aggregates_long.csv` with a declared aggregation set, so the primary temporal tier is a reproducible query rather than an ad hoc cut. No per-paper paradigm-orientation column is added: orientation stays an outcome of blinded coding.

## Blind Packets

`corpus/blind_packets/` is a generated view for prior-evaluation-blind judging. It is built from `papers.csv` by `corpus/scripts/build_blind_packets.py`.

Each packet includes paper identity, paper PDF/text, metadata, the coding guide, judge instructions, and a blank rating form. Packets intentionally exclude prior scores, rationales, aggregate tables, field labels, citation counts, `sample_source` (random-base vs landmark provenance), and outcome information. Excluding `sample_source` is required: a judge who knows a paper is a landmark may score it more generously, contaminating the calibration the landmark stratum exists to provide.
