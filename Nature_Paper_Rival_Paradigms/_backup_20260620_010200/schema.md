# Corpus Schema

## `papers.csv`

One row per paper. This table should remain stable once a paper enters the corpus.

- `paper_id`: stable local identifier.
- `corpus_stratum`: short stratum code.
- `corpus_stratum_label`: readable stratum label.
- `pilot_role`: current role in the pilot; candidate roles may change after review.
- `paper_title`: title as recorded in the source metadata.
- `authors`: compact author string.
- `year`: publication year.
- `journal`: journal or venue.
- `doi`: DOI where available.
- `pmid`: PubMed identifier if already known.
- `pmcid`: PubMed Central identifier if already known.
- `url`: DOI, publisher page, repository page, or open landing page.
- `pdf_url`: direct PDF URL when found.
- `pdf_path`: local PDF path if the PDF was successfully downloaded and verified as a PDF.
- `text_path`: extracted local text path when available.
- `oa_status`: open-access status from source metadata or manual inspection.
- `download_status`: local acquisition status.
- `matched_to_paper_id`: for Stratum B controls, the paper they are matched against.
- `matching_notes`: notes on match quality or concerns.
- `openalex_cited_by_count`: OpenAlex citation count observed during pilot setup.
- `source_checked_date`: date metadata/access status was checked.
- `notes`: free-text audit notes.

Observed `download_status` values in the pilot include:

- `downloaded_verified_pdf`: a source PDF was downloaded and verified as a PDF.
- `assembled_from_public_archive_images`: a PDF was assembled from public archive page images.
- `generated_pdf_from_pmc_xml`: a review PDF was generated locally from public PMC full-text XML because the native PDF route did not return a verified PDF.
- `manual_access_required`: no verified public PDF copy was acquired in the automated pass.

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

## Blind Packets

`corpus/blind_packets/` is a generated view for prior-evaluation-blind judging. It is built from `papers.csv` by `corpus/scripts/build_blind_packets.py`.

Each packet includes paper identity, paper PDF/text, metadata, the coding guide, judge instructions, and a blank rating form. Packets intentionally exclude prior scores, rationales, aggregate tables, stratum labels, matching notes, citation counts, and outcome information.
