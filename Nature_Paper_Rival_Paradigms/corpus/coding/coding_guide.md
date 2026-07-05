# Pilot Coding Guide (v0.1-pilot)

This guide operationalizes the paper's proposed distinction between statistical-inductivist
dependence and causal-Popperian abstraction. It is preliminary.

It defines three instruments:

1. the **Causal-Abstraction Commitment** score — a paper's substantive causal-Popperian
   commitment;
2. the **Statistical-Inductivist Dependence** score — a paper's substantive dependence on
   statistical certification or data-centered regularity; and
3. the **Paradigm-Marker Language** instrument — the epistemic moves the paper's own language
   performs, coded by role.

Keep the three instruments separate. They are scored independently and are not collapsed into a
single grand total; their correlations are a result, not a score.

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
  named. A `4` requires entities defined at the right level of abstraction for the claim, with
  operational criteria for identifying them; a `2` is naming without operational precision;
  reserve `3`-`4` for cases where the entity choice itself does explanatory work.
- `causal_relation`: dependencies or directional relations among entities are specified.
- `mechanism`: the paper explains how the phenomenon is generated.
- `intervention_relevance`: the paper identifies manipulations that should alter the phenomenon.
- `invariance`: the paper states or implies where the claim should hold or fail.
- `rival_explanations`: concrete alternatives are addressed or ruled out.
- `severe_test`: the paper exposes the claim to a realistic possible failure.
- `measurement_model`: measurement-to-mechanism linkage. Score how well the paper connects how
  observations are produced to the proposed generative structure. A `4` ties the measurement
  model directly to the mechanism (e.g., the measurement would read differently if the proposed
  structure were false). Careful measurement that is *not* linked to a generative claim scores
  `1`-`2`, not `4`.
- `abstraction_discipline`: interpretation consistently follows the proposed abstraction rather
  than ad hoc story-making.

## Statistical-Inductivist Dependence Score

Score each dimension 0-4 using the general anchors above.

This is not a "bad science" score. It measures dependence on statistical certification or
data-centered regularity, not quality.

Dimensions:

- `significance_dependence`: main claim depends primarily on p-values, q-values, or
  significance thresholds.
- `terminal_model_warrant`: the central claim counts as discovery because a trained model
  predicts, classifies, embeds, clusters, ranks, or benchmarks well, rather than because the
  model tests a specified causal or generative structure. Do not score this dimension merely
  because a model was trained; score the role model performance plays in warranting the central
  claim.
  - `0`: model performance absent, or fully subordinated to a causal/generative severe test.
  - `1`: model performance present but auxiliary.
  - `2`: model performance is a major co-warrant alongside structural claims.
  - `3`: model performance is the primary warrant, with only thin structural interpretation.
  - `4`: the trained model's performance or learned representation is effectively the discovery.
- `covariate_control_warrant`: the central claim is warranted by adding, selecting, matching on,
  stratifying by, adjusting for, or otherwise controlling for observed variables, without a clear
  causal account of which variables are confounders, mediators, colliders, instruments, or
  post-treatment variables. Do not score this as a methodological-error detector; score the role
  covariate control plays in warranting the central claim.
  - `0`: no covariate-control warrant, or adjustment is explicitly justified by a causal design,
    DAG, intervention logic, or equivalent structural rationale.
  - `1`: routine controls are present but auxiliary.
  - `2`: adjustment is a meaningful co-warrant, with partial causal rationale.
  - `3`: robustness to many controls is a primary warrant, but the causal roles of controls are
    weakly specified.
  - `4`: "control for everything" logic; dense adjustment, matching, stratification, or variable
    selection is treated as causal purification despite unclear or risky variable roles.
- `high_dimensional_search`: main result emerges from many possible relationships, features,
  genes, annotations, or model choices.
- `flexible_pipeline`: many preprocessing, normalization, modeling, or analytic choices could
  have produced different outputs.

This scale contains only dimensions that are distinct from the causal scale. Mechanistic
thinness, weak intervention, and narrow validation are not scored here; they are inferred from
low causal-abstraction scores instead. The covariate-control dimension scores
adjustment-as-warrant, not whether the paper in fact contains confounder or collider bias.

## Scoring And Totals

**Primary (unweighted) totals.** Sum the dimensions on each scale.

- Causal-abstraction total: 9 dimensions x 0-4 = 0-36.
- Statistical-inductivist total: 5 dimensions x 0-4 = 0-20.

Because the two scales have different numbers of dimensions, compare them as per-dimension
means (total / number of dimensions, range 0-4), not as raw totals.

**Derived aggregation features.** The mean remains the headline comparable score, but analyses
should also report secondary features that preserve the shape of each scale. These are computed
from the scored dimensions; coders do not score them directly.

- `scale_mean`: mean over nonblank dimensions.
- `scale_max`: highest dimension score on the scale.
- `scale_count_high_ge3`: number of dimensions scored `3` or `4`.
- `scale_any_high_ge3`: binary flag for at least one dimension scored `3` or `4`.
- `scale_count_extreme_eq4`: number of dimensions scored `4`.
- `scale_any_extreme_eq4`: binary flag for at least one dimension scored `4`.

For statistical-inductivist dependence, `scale_max` and `scale_any_high_ge3` are substantively
important because one strong warrant mode can dominate the paper's epistemic posture. For
causal-abstraction commitment, these features are diagnostic but not sufficient: one high causal
dimension does not make a paper strongly causal-Popperian. Interpret causal strength mainly through
`scale_mean`, `scale_count_high_ge3`, and the pattern across core dimensions. The high and extreme
thresholds apply to the live 0-4 rubric; the frozen 0-2 seed scores remain diagnostic only.

**Optional weighted causal total.** For analyses that want to emphasize the dimensions most
central to the thesis, apply weight `1.5` to `severe_test`, `intervention_relevance`,
`rival_explanations`, and `abstraction_discipline`, and weight `1.0` to the remaining five
causal dimensions. Report the weighted total in a separate column; never let it overwrite the
unweighted total. The weighting scheme is itself provisional and should be treated as a
sensitivity analysis, not the headline measure.

## Outcome Measures

Outcome dimensions should usually be scored only after a dedicated downstream uptake workflow is
defined. The current `outcomes.csv` table is therefore mostly a scaffold. Outcome anchors remain
on the 0-2 scale for now; widen them to 0-4 only when the outcome workflow is activated.

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

## Paradigm-Marker Language Instrument

The two substantive instruments above score a paper's **substantive** commitment (did it
actually build a causal abstraction; does it actually depend on statistical certification). This
instrument scores something narrower and methodologically distinct: the **epistemic moves the
paper's own language performs** — coded by *role*, with a required verbatim evidence span for
every nonzero score.

It exists because LLM judges are unusually good at semantic role detection, and because the
inclusion/exclusion of paradigm-specific moves is itself informative. But the same strength is a
trap if mis-specified, so the instrument is governed by one overriding principle.

### The Governing Principle: Role, Not Word

**Score the function the language performs in the argument, not the presence of vocabulary.**

- Vocabulary used as rhetorical garnish (e.g. "mechanism", "drives", "we test whether…") with no
  load-bearing move behind it scores `0`–`1`.
- A move genuinely enacted *without* the canonical vocabulary scores high. Meselson–Stahl never
  says "falsify" or "Popper," yet it is the paradigm severe test.

A keyword counter would systematically misclassify the corpus's clearest cases in **both**
directions. The whole reason to use an LLM here is to read role; the rubric must force that.
Every nonzero score therefore requires (a) a quoted span and (b) a one-line role judgment,
recorded in `multi_judge/paradigm_marker_evidence.csv`. **A score with no span defaults to 0.**
This is the anti-hallucination gate.

#### The field-dialect warning

Fields have house dialects. Physics writes "exclude," "rule out," "constrain at 5σ"; biology
writes "consistent with," "suggests," "associated with." A dimension that secretly detects
dialect rather than move is not just noisy — it is **dangerous** here, because field correlates
with era, so a dialect detector would *spuriously confirm* the temporal thesis. When you score,
ask whether you are rewarding an epistemic move or a regional accent. The validation plan below
includes an explicit field-confound check for exactly this reason.

### Scale, Anchors, and Occasion Gating

All nine dimensions use the standard 0–4 anchors (`0` absent / `1` rhetorical, not load-bearing /
`2` present but secondary / `3` substantial and central / `4` central and rigorously developed),
specialized per dimension below. Half steps are not used; when between two anchors, take the
lower and explain.

**Occasion gating (causal-Popperian dimensions only).** Each `cp_` dimension carries a companion
`*_occasion` flag (`1` = the paper's own central claims created the occasion for this move;
`0` = the move was not called for). This is what lets the instrument measure *exclusion* honestly:

- `occasion = 1` and `score = 0` → a **conspicuous exclusion**: the paper had the occasion to make
  a Popperian move its own argument invited, and did not. This is diagnostic and is the signal the
  exclusion analysis is built on.
- `occasion = 0` → the move was not relevant. **Leave the score blank (NA), not 0.** A structural
  zero would drag down the per-dimension mean for reasons unrelated to the construct. NA cells are
  dropped from numeric aggregation; the aggregation script already skips blank cells.

The three `si_` dimensions have **no** occasion flag. Absence of inductivist language is not
"conspicuous" — it just means the paper isn't leaning on certification — so there is nothing to
gate. Score them `0`–`4` by role directly.

### Causal-Popperian Move Dimensions (`cp_`)

#### `cp_risky_prediction`

The text commits the central claim to an outcome that could have come out otherwise, so the actual
result is evidentially valuable *because* it survived a test it could have failed.

- *Occasion*: the paper advances a claim with testable consequences (nearly always yes for
  empirical work).
- `0`: no risk staked; result framed purely as confirmation or description.
- `1`: risk gestured at rhetorically ("we tested whether…") but nothing about the central claim
  could have failed as framed. **Refuting only a statistical null sits here**, not higher, unless a
  structural claim is what is put at risk.
- `2`: a real but local or secondary prediction is put at risk.
- `3`: the central claim is staked on a prediction that could have failed, and survival is treated
  as the warrant.
- `4`: as `3`, and the failing outcome is explicitly specified and the design is built to make
  failure realistically possible.

#### `cp_rival_elimination`

Named competing explanations are confronted and the evidence is used to **discriminate** among
them — not merely to be "consistent with" the favored one.

- *Occasion*: more than one explanation could plausibly account for the central signal.
- `0`: no alternatives named, or alternatives named but the evidence cannot discriminate
  ("consistent with our model").
- `1`: alternatives mentioned rhetorically, not engaged.
- `2`: one alternative seriously addressed.
- `3`: the principal rivals are named and the evidence discriminates against them.
- `4`: the rival set is comprehensive and the design's discriminating power is made explicit.

#### `cp_generative_structure`

The text posits an underlying process or structure responsible for the observed signal and reasons
*from* that structure to consequences — going beyond redescribing the signal.

- *Occasion*: the paper reports any signal, regularity, or pattern (nearly always yes).
- `0`: the claim stops at the signal (association / signature / cluster) with no posited generator.
- `1`: a generator is named as a label ("driver", "regulator") but does no inferential work.
- `2`: a generator is posited and loosely linked to the signal.
- `3`: a structure is posited and consequences are derived from it.
- `4`: the structure is the engine of the paper; multiple consequences are derived and checked.

#### `cp_counterfactual_intervention`

The text reasons about what *would* happen under manipulation or perturbation, not only what
co-occurs.

- *Occasion*: the phenomenon admits, in principle, a manipulation or counterfactual contrast.
- `0`: purely observational/associational language; no manipulation reasoned about.
- `1`: intervention mentioned only as future work.
- `2`: a manipulation is described but is not central to the claim.
- `3`: counterfactual or interventional reasoning is central.
- `4`: an intervention is enacted and its predicted differential is the crux of the result.

#### `cp_assumption_vulnerability`

The text identifies a **load-bearing** assumption and states a condition under which the central
claim would fail or require revision — the falsifiable cousin of a boilerplate "Limitations"
paragraph.

- *Occasion*: the central claim rests on at least one identifiable assumption or scope condition
  (always yes).
- `0`: no assumptions surfaced, or only generic limitations boilerplate ("small sample, future work
  needed") unconnected to whether the claim is true.
- `1`: assumptions listed but none load-bearing, or no failure condition given.
- `2`: a load-bearing assumption is named but the failure condition is vague.
- `3`: a load-bearing assumption is named with a concrete condition under which the claim breaks.
- `4`: failure conditions are operationalized and used to bound the claim.

#### `cp_theory_relative_contrast`

The text makes explicit that the central result is *unlikely under the live rival frame(s)* yet
*expected under the paper's proposed frame* — foregrounding the differential in expectedness
across frames, not merely that the result is risky or that a generator exists.

- *Occasion*: the paper's own claims invoke, or could be set against, at least one incumbent
  theory, model, or standing expectation. In a young field with no dominant incumbent, occasion
  is still met as long as ≥1 live rival expectation exists. Occasion fails only for purely
  descriptive papers with no rival target — leave NA.
- `0`: no rival frame named, or the result is reported with no claim about what would have been
  expected under any prior framework.
- `1`: rhetorical novelty only ("surprisingly," "contrary to expectation") not tied to a
  specified rival frame, or not load-bearing. Also `1`–`2` if the result is improbable only under
  a strawman while a serious untreated rival makes it roughly equally expected.
- `2`: a one-sided contrast — a rival expectation is named and the result strains it, but the
  paper does not articulate why the result is *expected* under its own proposed frame (or vice
  versa).
- `3`: both sides stated against the dominant incumbent — unlikely under it, expected under the
  proposed frame — but other live rivals are unaddressed, or one remains under which the result is
  roughly equally expected (the reversal is real but not yet discriminating).
- `4`: the result is expected under the proposed frame and improbable under the *comprehensive set
  of live rivals the paper engages*; no untreated rival makes it equally expected, so the reversal
  discriminates the proposed frame specifically, and that reversal is the engine of the argument.

**Scope note.** This dimension scores the *contrast move*, not the risk (`cp_risky_prediction`)
or the generator (`cp_generative_structure`) on their own. A paper can stake a falsifiable risky
prediction, or posit a generative structure, without ever articulating the rival-vs-proposed
expectedness reversal; reserve `3`–`4` only when both sides of that reversal are made explicit.
Whether the bold claim turned out approximately correct is **not** scored here — that is an
Outcome Measure (`disruptiveness`, `mechanistic_uptake`), correlated after the fact, never used
to inflate the publication-time score.

**Multiple rivals.** "Incumbent frame" may be a set of standing expectations, not one theory.
Judge the reversal against the *strongest* live rival, not the weakest: a result improbable only
under a strawman but expected under a serious untreated alternative scores `1`–`2`. Breadth of the
rival set under which the reversal holds is what separates `3` from `4`. Record the rival set in
the evidence span.

**Boundary with `cp_rival_elimination`.** Keep the two on different questions:
`cp_rival_elimination` scores whether the *evidence discriminates* (rules rivals out);
`cp_theory_relative_contrast` scores whether the paper *articulates the expectedness reversal*
across the rival set. Evidence can rule out a rival without the paper ever framing it as
improbable-then-expected, and the reversal can be narrated without decisive discriminating
evidence. If the two correlate ~1.0 once multiple rivals are in play, they have collapsed into one
construct and only one should survive.

### Statistical-Inductivist Move Dimensions (`si_`)

These score the role certification / regularity language plays as the **terminal warrant**. High
scores here are **not** a quality penalty, and they are deliberately **compatible with high `cp_`
scores** — that independence is the design's whole point (see "Discriminant validity" below).

#### `si_terminal_certification`

The central claim's warrant **terminates** in a certification statistic (significance, FDR/q-value,
cross-validation accuracy, AUC, benchmark rank): the result "counts" because it cleared a
threshold, with no further structural conjecture the certification serves.

- `0`: certification absent, **or** present but *backgrounded and fully subordinated* to a
  structural test — the threshold is not foregrounded as part of the headline warrant. *The pure
  particle-physics case, where the test, not the number, is the point.*
- `1`: certification is instrumental to a test but is *foregrounded* as a stated co-warrant — the
  threshold is reported as a headline number even though a structural conjecture is what is really
  at stake.
- `2`: certification is a major co-warrant — either instrumental-but-prominent, or a terminal
  warrant that shares the stage with a structural claim.
- `3`: certification is the primary, near-terminal warrant for the central claim.
- `4`: the contribution essentially *is* the certified regularity; clearing the threshold is the
  result.

**Distinguishing question for instrumental certification.** When the statistics serve a test rather
than ending the inquiry, ask whether the threshold is *backgrounded* (→ `0`) or *foregrounded as a
stated co-warrant* (→ `1`–`2`). Reserve `3`–`4` only for certification that is the **terminal**
warrant with no structural test served. Calibration item C04 (a foregrounded 5σ exclusion in
service of a severe test) anchors the foregrounded-instrumental `2`; a strict "instrumental → 0"
reading of that item is a misread of this dimension.

#### `si_association_framing`

The central contribution is framed as an association / enrichment / cluster / signature /
predictive mapping rather than a generative account.

- `0`: framed as mechanism or structure.
- `1`: mechanism-framed but leaning on associational evidence.
- `2`: mixed framing.
- `3`: primarily associational framing.
- `4`: the deliverable is explicitly a signature, association, or predictor.

#### `si_accumulation_progress`

The text frames the path to knowledge as **accumulation** (more data, more replication, a bigger
atlas, more features) rather than as sharper tests or deeper structure.

- `0`: progress framed as test or structure improvement.
- `1`: incidental accumulation language.
- `2`: accumulation is one stated path among others.
- `3`: accumulation is the primary forward-looking frame.
- `4`: scale/accumulation is presented as the central value proposition.

### Totals, and the Relationship to the Substantive Scales

Report four summaries per paper per judge-round:

- **`cp_marker_mean`** — mean over the *applicable* `cp_` dimensions (NA cells dropped), range 0–4.
  This is the intensity summary: how strongly the paper makes the Popperian moves it had occasion
  to make.
- **`cp_applicable_count`** — the number of `cp_` dimensions with `occasion = 1` (range 0–6). This
  is the breadth summary: how many Popperian openings the paper's own claims created. It is itself
  an informative feature of paradigm orientation, so report it as its own column rather than
  folding it into the mean.
- **`si_marker_mean`** — mean over the three `si_` dimensions, range 0–4.
- **`cp_exclusion_rate`** — among `cp_` dimensions with `occasion = 1`, the fraction scored `0`.
  This is the instrument's distinctive exclusion signal.

Derived high-threshold marker features should follow the same gating rule. For `cp_` moves,
compute `cp_marker_max_applicable`, `cp_marker_count_high_ge3`, `cp_marker_any_high_ge3`,
`cp_marker_count_extreme_eq4`, and `cp_marker_any_extreme_eq4` only over dimensions with
`occasion = 1`. If `cp_applicable_count = 0`, leave these high-threshold `cp_` features blank
(NA), not `0`. For `si_` moves, which are not occasion-gated, compute `si_marker_max`,
`si_marker_count_high_ge3`, `si_marker_any_high_ge3`, `si_marker_count_extreme_eq4`, and
`si_marker_any_extreme_eq4` over all three `si_` dimensions.

**Use the mean, not the sum, as the comparable `cp_` summary.** Because the `cp_` dimensions are
occasion-gated, the number of applicable dimensions varies across papers. A raw sum would bundle
move intensity together with occasion count, so a paper would score higher merely for having more
openings — reintroducing the denominator-dependence the per-dimension mean exists to remove, and
spuriously widening the paradigm gap (intensity and breadth would be confounded). Keep them
separate: `cp_marker_mean` for intensity, `cp_applicable_count` for breadth. Since the raw sum is
just `cp_marker_mean × cp_applicable_count`, it can always be reconstructed if needed. A raw
`total` column exists in the ratings table for convenience, but **compare per-dimension means, not
raw totals** (same rule as the substantive scales).

**Do not collapse instruments into one grand total.** The `cp_` markers and the
`causal_abstraction` scale measure the *same construct two different ways* (textual move vs.
substantive commitment). Summing them double-counts and rebuilds the same double-counting error
the project already corrected. Keep instruments separate; their correlation is a *result*, not a
score.

#### Preregistered expected relationships

State these before scoring real papers, so confirmation is not retrofitted:

- **Convergent:** `cp_marker_mean` should correlate positively and moderately-to-strongly with the
  `causal_abstraction` per-dimension mean; `si_marker_mean` with `statistical_inductivist_
  dependence`. Moderate is the target — near `1.0` means the instrument is redundant, near `0`
  means it is invalid.
- **Discriminant:** `cp_marker_mean` and `si_marker_mean` must **not** sit near `−1`. A paper can be
  high on both (particle physics) or low on both (a purely descriptive atlas with no certification
  emphasis). If they collapse toward a single axis, the double-counting mistake has returned in new
  clothing.

### Validation Plan (run before trusting any real-paper round)

1. **Red-team calibration first.** Score the constructed items in
   `calibration/paradigm_marker_calibration.md`. The rubric must score the
   *rhetoric-without-move* and *move-without-vocabulary* traps correctly before any real round is
   trusted. If it cannot separate these, it is still a keyword counter — fix the anchors, do not
   proceed.
2. **Discriminant check.** Compute `corr(cp_marker_mean, si_marker_mean)`. Flag if `< −0.8`.
3. **Field-confound check.** Regress each marker dimension on `field`. Flag any dimension whose
   field `R²` exceeds a preregistered threshold (suggested `0.5`) as a probable dialect detector;
   re-anchor or drop it.
4. **Convergent check.** Compute the convergent correlations above; confirm they are moderate, not
   `~1.0` (redundant) and not `~0` (invalid).
5. **Reliability.** Use ≥3 blinded AI replicates per model plus ≥1 human judge. Aggregate AI
   replicates within model first (per `multi_judge/README.md`) to avoid pseudo-replication. Report
   per-dimension agreement (ICC or exact/adjacent agreement); route low-agreement items to
   adjudication.

### How To Wire A Paradigm-Marker Round

1. Add a `v0.1-pilot` judge round in `judge_rounds.csv` tied to a `rounds.csv` row whose
   `rubric_version = v0.1-pilot`, with the intended blinding profile.
2. Record raw scores in `multi_judge/paradigm_marker_ratings.csv` (one row per paper per
   judge-round) and the supporting spans in `multi_judge/paradigm_marker_evidence.csv`.
3. Add a `v0.1-pilot` aggregation set in `aggregation_sets.csv` once eligible rounds exist (leave
   `eligible_round_ids = TBD` until then so the build script skips it).
4. Run `python3 corpus/scripts/build_multi_judge_aggregates.py`. The script is rubric-version
   aware and knows the `paradigm_marker` instrument.

## Current Pilot Caveats

Corpus coverage is uneven: it is concentrated in computational biology and lacks a
structure-driven, statistics-intensive physics field, a statistically-oriented physics field,
and recent mechanistic biology. Field-level paradigm dominance must be measured from blinded
coding, not assumed from a paper's field. See `ingestion_plan_field_differential.md` for the
gaps and planned ingestion.

The Stratum B controls are the weakest part of the pilot by design. Several are plausible matched
papers, but two may be better treated as progress-like papers or downstream theoretical uptake
rather than true controls. This is useful: it reveals that the control-matching rule needs
refinement before scaling.

The seed scores are non-blinded single-coder judgments retained as a workbench artifact, not
adjudicated evidence. Before deleting any dimension on empirical grounds (for example, if a
dimension shows little variance), confirm the pattern across at least two blinded judges;
single-coder variance is weak evidence. For human and AI panels, use the normalized tables in
`multi_judge/`. They support multiple human judges, multiple AI chatbot judges, and repeated
blinded AI rounds.

Repeated AI rounds should usually be aggregated within chatbot/model before entering a hybrid
human-AI panel aggregate. Otherwise one chatbot can dominate the results simply because it was
sampled more times.

Initial matching criteria to refine:

- Same field or problem area.
- Same decade, preferably within five years.
- Similar venue prestige or visibility.
- High contemporaneous plausibility or influence.
- Not itself a primary paper for a later canonical discovery, unless intentionally coded as a
  borderline case.

## Recommended Next Pilot Step

Run a round with at least one human reviewer and one fresh blinded AI replicate, recoding four
papers blindly:

- `P0013`
- `P0015`
- `P0010`
- `P0014`

These four stress the key distinctions: severe causal test, failed structural abstraction,
descriptive ontology, and high-dimensional statistical method. Use this round to check whether
the 0-4 scale and the `entity_specification` and `measurement_model` dimensions produce usable
variance, and whether the statistical scale separates papers cleanly.

## Status

Provisional. The dimensions, anchors, occasion-gating, weights, and thresholds in the validation
plan are all subject to revision after the red-team pass and the first blinded round. Treat the
paradigm-marker instrument in particular as a hypothesis about how to measure paradigm-marker
language, not as a settled instrument.
