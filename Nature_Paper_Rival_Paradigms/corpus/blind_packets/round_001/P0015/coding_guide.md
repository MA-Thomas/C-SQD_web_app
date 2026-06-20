# Pilot Coding Guide (v0.2-pilot)

This guide operationalizes the paper's proposed distinction between statistical-inductivist
dependence and causal-Popperian abstraction. It is still preliminary.

## What Changed From v0.1

v0.2 is a versioned revision. The earlier rubric is preserved verbatim in
`coding_guide_v0.1.md`, and the `v0.1-pilot` row in `multi_judge/rubrics.csv` is left
untouched. The Codex seed round (`round_pilot_seed_001`) was scored under v0.1 and must
**not** be rescored or overwritten; it remains valid as a v0.1 artifact. Apply v0.2 only to
new rounds.

Substantive changes:

1. **Score range widened to 0-4.** v0.1 used 0-2, which produced coarse scores, many ties,
   and an implicit equal-interval assumption. The wider anchored scale reduces ties and lets
   judges register degrees of commitment.
2. **Statistical-inductivist scale de-duplicated (7 -> 4 dimensions).** Three v0.1 dimensions
   (`weak_mechanism`, `limited_intervention`, `local_validation`) were sign-flipped copies of
   causal dimensions (`mechanism`, `intervention_relevance`, `invariance`). They double-counted
   the same constructs and collapsed the two scales toward one axis. They are removed. Those
   absences are now read directly off low causal-abstraction scores. The statistical scale now
   measures only dependence on the certification machinery itself.
3. **`entity_specification` re-anchored.** In v0.1 every pilot paper scored the maximum, so the
   dimension carried no discriminating information. The anchors now reserve the top score for
   entities defined at the right level of abstraction with operational criteria.
4. **`measurement_model` reframed as measurement-to-mechanism linkage.** In v0.1 it rewarded
   general measurement rigor, so descriptive/statistical papers scored high on a dimension meant
   to be causal. It now scores whether the measurement model connects observations to the
   proposed generative structure, not whether measurement is merely careful.
5. **Optional dimension weights added.** An unweighted total remains the primary score. An
   optional weighted total upweights the four dimensions that most directly instantiate the
   thesis. See "Scoring And Totals."

## Unit Of Analysis

Score the central contribution of the paper as it appeared at publication. Do not use later
fame, later textbook status, or later mechanistic completion to inflate scores.

If a paper is a tool or infrastructure paper, score its direct causal-abstraction commitment
normally, but flag it in notes. Tool papers may have high progress value even when direct
causal-abstraction scores are modest.

## General 0-4 Anchors

Unless a dimension specifies otherwise, interpret the scale as:

- `0`: absent, or not relevant to the main claim.
- `1`: minimal, rhetorical, or merely gestured at; not load-bearing.
- `2`: clearly present but secondary, local, or partial.
- `3`: substantial and central to the paper's argument.
- `4`: central and rigorously developed; an exemplary instance of the dimension.

Half steps are not used. When uncertain between two adjacent anchors, choose the lower and
explain in notes.

## Paper Classification

Use the Section 10 categories:

- `Statistical regularity`: association, signature, cluster, prediction, enrichment,
  correlation, fitted model.
- `Descriptive ontology`: map, atlas, taxonomy, dataset, category system, structural
  description.
- `Causal mechanism`: mechanism or causal structure is the central contribution.
- `Intervention result`: manipulation of a variable changes the phenomenon.
- `Severe test of a theory`: risky implication of a causal abstraction is tested.
- `Tool or infrastructure`: method, instrument, assay, computational system, or resource
  enabling future inquiry.

Mixed classifications are expected. Choose one primary classification and list secondary
classifications.

## Causal-Abstraction Commitment Score

Score each dimension 0-4 using the general anchors above, with the dimension-specific
clarifications noted.

- `entity_specification`: relevant entities, variables, structures, or categories are clearly
  named. **Re-anchored:** a `4` requires entities defined at the right level of abstraction for
  the claim, with operational criteria for identifying them; a `2` is naming without operational
  precision; reserve `3`-`4` for cases where the entity choice itself does explanatory work.
- `causal_relation`: dependencies or directional relations among entities are specified.
- `mechanism`: the paper explains how the phenomenon is generated.
- `intervention_relevance`: the paper identifies manipulations that should alter the phenomenon.
- `invariance`: the paper states or implies where the claim should hold or fail.
- `rival_explanations`: concrete alternatives are addressed or ruled out.
- `severe_test`: the paper exposes the claim to a realistic possible failure.
- `measurement_model`: **reframed** as measurement-to-mechanism linkage. Score how well the
  paper connects how observations are produced to the proposed generative structure. A `4` ties
  the measurement model directly to the mechanism (e.g., the measurement would read differently
  if the proposed structure were false). Careful measurement that is *not* linked to a
  generative claim scores `1`-`2`, not `4`.
- `abstraction_discipline`: interpretation consistently follows the proposed abstraction rather
  than ad hoc story-making.

## Statistical-Inductivist Dependence Score

Score each dimension 0-4 using the general anchors above. This scale now contains only
dimensions that are distinct from the causal scale. Mechanistic thinness, weak intervention,
and narrow validation are no longer scored here; they are inferred from low causal-abstraction
scores instead.

Dimensions:

- `significance_dependence`: main claim depends primarily on p-values, q-values, or
  significance thresholds.
- `prediction_dependence`: main claim depends primarily on held-out performance, accuracy, AUC,
  calibration, or benchmark ranking.
- `high_dimensional_search`: main result emerges from many possible relationships, features,
  genes, annotations, or model choices.
- `flexible_pipeline`: many preprocessing, normalization, modeling, or analytic choices could
  have produced different outputs.

This is not a "bad science" score. It measures dependence on statistical certification or
data-centered regularity, not quality.

## Scoring And Totals

**Primary (unweighted) totals.** Sum the dimensions on each scale.

- Causal-abstraction total: 9 dimensions x 0-4 = 0-36.
- Statistical-inductivist total: 4 dimensions x 0-4 = 0-16.

Because the two scales now have different numbers of dimensions, compare them as per-dimension
means (total / number of dimensions, range 0-4), not as raw totals.

**Optional weighted causal total.** For analyses that want to emphasize the dimensions most
central to the thesis, apply weight `1.5` to `severe_test`, `intervention_relevance`,
`rival_explanations`, and `abstraction_discipline`, and weight `1.0` to the remaining five
causal dimensions. Report the weighted total in a separate column; never let it overwrite the
unweighted total. The weighting scheme is itself provisional and should be treated as a
sensitivity analysis, not the headline measure.

## Outcome Measures

Outcome dimensions should usually be scored only after a dedicated downstream uptake workflow is
defined. The current `outcomes.csv` table is therefore mostly a scaffold. Outcome anchors remain
on the 0-2 scale for now; widen them to 0-4 only when the outcome workflow is activated, and
record that change as a further rubric version.

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

Corpus coverage is uneven: it is concentrated in computational biology and lacks a
structure-driven, statistics-intensive physics field, a statistically-oriented physics field,
and recent mechanistic biology. Field-level paradigm dominance must be measured from blinded
coding, not assumed from a paper's field. See `ingestion_plan_field_differential.md` for the
gaps and planned ingestion.

The v0.1 seed scores are non-blinded single-coder judgments retained as a workbench artifact,
not adjudicated evidence. Before deleting any v0.2 dimension on empirical grounds (for example,
if a dimension again shows little variance), confirm the pattern across at least two blinded
judges; single-coder variance is weak evidence. For human and AI panels, use the normalized
tables in `multi_judge/`. They support multiple human judges, multiple AI chatbot judges, and
repeated blinded AI rounds.

Repeated AI rounds should usually be aggregated within chatbot/model before entering a hybrid
human-AI panel aggregate. Otherwise one chatbot can dominate the results simply because it was
sampled more times.

## Recommended Next Pilot Step

Run a v0.2 round with at least one human reviewer and one fresh blinded AI replicate, recoding
four papers blindly:

- `P0013`
- `P0015`
- `P0010`
- `P0014`

These four stress the key distinctions: severe causal test, failed structural abstraction,
descriptive ontology, and high-dimensional statistical method. Use this round to check whether
the 0-4 scale and the re-anchored `entity_specification` and `measurement_model` dimensions now
produce usable variance, and whether the de-duplicated statistical scale separates papers as
cleanly as the combined v0.1 scale did.
