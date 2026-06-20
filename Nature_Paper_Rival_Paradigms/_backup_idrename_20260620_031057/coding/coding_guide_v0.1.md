# Pilot Coding Guide

This guide operationalizes the paper's proposed distinction between statistical-inductivist dependence and causal-Popperian abstraction. It is intentionally preliminary.

## Unit Of Analysis

Score the central contribution of the paper as it appeared at publication. Do not use later fame, later textbook status, or later mechanistic completion to inflate scores.

If a paper is a tool or infrastructure paper, score its direct causal-abstraction commitment normally, but flag it in notes. Tool papers may have high progress value even when direct causal-abstraction scores are modest.

## Paper Classification

Use the Section 10 categories:

- `Statistical regularity`: association, signature, cluster, prediction, enrichment, correlation, fitted model.
- `Descriptive ontology`: map, atlas, taxonomy, dataset, category system, structural description.
- `Causal mechanism`: mechanism or causal structure is the central contribution.
- `Intervention result`: manipulation of a variable changes the phenomenon.
- `Severe test of a theory`: risky implication of a causal abstraction is tested.
- `Tool or infrastructure`: method, instrument, assay, computational system, or resource enabling future inquiry.

Mixed classifications are expected. Choose one primary classification and list secondary classifications.

## Causal-Abstraction Commitment Score

Score each dimension 0-2.

- `0`: absent or not relevant to the main claim.
- `1`: present but weak, rhetorical, local, or not central.
- `2`: central to the paper's argument.

Dimensions:

- `entity_specification`: relevant entities, variables, structures, or categories are clearly named.
- `causal_relation`: dependencies or directional relations among entities are specified.
- `mechanism`: the paper explains how the phenomenon is generated.
- `intervention_relevance`: the paper identifies manipulations that should alter the phenomenon.
- `invariance`: the paper states or implies where the claim should hold or fail.
- `rival_explanations`: concrete alternatives are addressed or ruled out.
- `severe_test`: the paper exposes the claim to a realistic possible failure.
- `measurement_model`: the paper explains how observations are produced from the underlying system.
- `abstraction_discipline`: interpretation consistently follows the proposed abstraction rather than ad hoc story-making.

## Statistical-Inductivist Dependence Score

Score each dimension 0-2.

- `0`: absent or peripheral.
- `1`: present but not dominant.
- `2`: central to the main claim.

Dimensions:

- `significance_dependence`: main claim depends primarily on p-values, q-values, or significance thresholds.
- `prediction_dependence`: main claim depends primarily on held-out performance, accuracy, AUC, calibration, or benchmark ranking.
- `high_dimensional_search`: main result emerges from many possible relationships, features, genes, annotations, or model choices.
- `flexible_pipeline`: many preprocessing, normalization, modeling, or analytic choices could have produced different outputs.
- `weak_mechanism`: mechanistic interpretation is thin, post hoc, or not needed for the main claim.
- `local_validation`: validation is confined to similar regimes, datasets, platforms, cohorts, or laboratories.
- `limited_intervention`: direct perturbational or intervention tests are absent or weak.

This is not a "bad science" score. It measures dependence on statistical certification or data-centered regularity, not quality.

## Outcome Measures

Outcome dimensions should usually be scored only after a dedicated downstream uptake workflow is defined. The current `outcomes.csv` table is therefore mostly a scaffold.

Suggested 0-2 anchors once measured:

- `0`: little or no evidence of durable uptake.
- `1`: limited, local, or ambiguous uptake.
- `2`: broad, durable, independent uptake.

Outcome dimensions:

- `citation_durability`
- `independent_lab_uptake`
- `mechanistic_uptake`
- `intervention_uptake`
- `ontological_uptake`
- `review_integration`
- `clinical_or_engineering_consequence`
- `replication_or_transport`
- `disruptiveness`

## Current Pilot Caveats

The Stratum B controls are the weakest part of the pilot by design. Several are plausible matched papers, but two may be better treated as progress-like papers or downstream theoretical uptake rather than true controls. This is useful: it reveals that the control-matching rule needs refinement before scaling.

The first coding tables are seed judgments, not final adjudicated scores. For human and AI panels, use the normalized tables in `multi_judge/`. They support multiple human judges, multiple AI chatbot judges, and repeated blinded AI rounds.

Repeated AI rounds should usually be aggregated within chatbot/model before entering a hybrid human-AI panel aggregate. Otherwise one chatbot can dominate the results simply because it was sampled more times.

Initial matching criteria to refine:

- Same field or problem area.
- Same decade, preferably within five years.
- Similar venue prestige or visibility.
- High contemporaneous plausibility or influence.
- Not itself a primary paper for a later canonical discovery, unless intentionally coded as a borderline case.

## Recommended Next Pilot Step

Have one human reviewer recode 4 papers blindly:

- `A0002`
- `B0001`
- `C0002`
- `D0004`

These four stress the key distinctions: severe causal test, failed structural abstraction, descriptive ontology, and high-dimensional statistical method.
