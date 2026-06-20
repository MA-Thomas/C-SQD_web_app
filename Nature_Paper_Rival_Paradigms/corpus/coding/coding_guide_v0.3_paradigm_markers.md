# Coding Guide Supplement v0.3-pilot — Paradigm-Marker Language Instrument

This supplement is **additive**. It does not change any v0.2 dimension, anchor, or weight, and
it does not touch any completed round. v0.2 (`coding_guide.md`) remains the live guide for the
two existing instruments. v0.3 introduces a **third instrument** — the *paradigm-marker language*
scale — and the rules for scoring it. The earlier guides (`coding_guide_v0.1.md`,
`coding_guide.md`) are preserved verbatim; the `v0.1-pilot` and `v0.2-pilot` rows in
`multi_judge/rubrics.csv` are left untouched. Apply v0.3 only to new rounds.

## What This Instrument Is For

The two existing instruments score a paper's **substantive** commitment (did it actually build a
causal abstraction; does it actually depend on statistical certification). This instrument scores
something narrower and methodologically distinct: the **epistemic moves the paper's own language
performs** — coded by *role*, with a required verbatim evidence span for every nonzero score.

It exists because LLM judges are unusually good at semantic role detection, and because the
inclusion/exclusion of paradigm-specific moves is itself informative. But the same strength is a
trap if mis-specified, so the instrument is governed by one overriding principle.

## The Governing Principle: Role, Not Word

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

### The field-dialect warning

Fields have house dialects. Physics writes "exclude," "rule out," "constrain at 5σ"; biology
writes "consistent with," "suggests," "associated with." A dimension that secretly detects
dialect rather than move is not just noisy — it is **dangerous** here, because field correlates
with era, so a dialect detector would *spuriously confirm* the temporal thesis. When you score,
ask whether you are rewarding an epistemic move or a regional accent. The validation plan below
includes an explicit field-confound check for exactly this reason.

## Scale, Anchors, and Occasion Gating

All eight dimensions use the standard 0–4 anchors (`0` absent / `1` rhetorical, not load-bearing /
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

## Causal-Popperian Move Dimensions (`cp_`)

### `cp_risky_prediction`
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

### `cp_rival_elimination`
Named competing explanations are confronted and the evidence is used to **discriminate** among
them — not merely to be "consistent with" the favored one.
- *Occasion*: more than one explanation could plausibly account for the central signal.
- `0`: no alternatives named, or alternatives named but the evidence cannot discriminate
  ("consistent with our model").
- `1`: alternatives mentioned rhetorically, not engaged.
- `2`: one alternative seriously addressed.
- `3`: the principal rivals are named and the evidence discriminates against them.
- `4`: the rival set is comprehensive and the design's discriminating power is made explicit.

### `cp_generative_structure`
The text posits an underlying process or structure responsible for the observed signal and reasons
*from* that structure to consequences — going beyond redescribing the signal.
- *Occasion*: the paper reports any signal, regularity, or pattern (nearly always yes).
- `0`: the claim stops at the signal (association / signature / cluster) with no posited generator.
- `1`: a generator is named as a label ("driver", "regulator") but does no inferential work.
- `2`: a generator is posited and loosely linked to the signal.
- `3`: a structure is posited and consequences are derived from it.
- `4`: the structure is the engine of the paper; multiple consequences are derived and checked.

### `cp_counterfactual_intervention`
The text reasons about what *would* happen under manipulation or perturbation, not only what
co-occurs.
- *Occasion*: the phenomenon admits, in principle, a manipulation or counterfactual contrast.
- `0`: purely observational/associational language; no manipulation reasoned about.
- `1`: intervention mentioned only as future work.
- `2`: a manipulation is described but is not central to the claim.
- `3`: counterfactual or interventional reasoning is central.
- `4`: an intervention is enacted and its predicted differential is the crux of the result.

### `cp_assumption_vulnerability`
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

## Statistical-Inductivist Move Dimensions (`si_`)

These score the role certification / regularity language plays as the **terminal warrant**. High
scores here are **not** a quality penalty, and they are deliberately **compatible with high `cp_`
scores** — that independence is the design's whole point (see "Discriminant validity" below).

### `si_terminal_certification`
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
reading of that item is a misread of this dimension. *(Anchor clarified 2026-06-20 after the first
calibration pass surfaced this ambiguity; no calibration expected-score changed.)*

### `si_association_framing`
The central contribution is framed as an association / enrichment / cluster / signature /
predictive mapping rather than a generative account.
- `0`: framed as mechanism or structure.
- `1`: mechanism-framed but leaning on associational evidence.
- `2`: mixed framing.
- `3`: primarily associational framing.
- `4`: the deliverable is explicitly a signature, association, or predictor.

### `si_accumulation_progress`
The text frames the path to knowledge as **accumulation** (more data, more replication, a bigger
atlas, more features) rather than as sharper tests or deeper structure.
- `0`: progress framed as test or structure improvement.
- `1`: incidental accumulation language.
- `2`: accumulation is one stated path among others.
- `3`: accumulation is the primary forward-looking frame.
- `4`: scale/accumulation is presented as the central value proposition.

## Totals, and the Relationship to the Existing Scales

Report three summaries per paper per judge-round:

- **`cp_marker_mean`** — mean over the *applicable* `cp_` dimensions (NA cells dropped), range 0–4.
- **`si_marker_mean`** — mean over the three `si_` dimensions, range 0–4.
- **`cp_exclusion_rate`** — among `cp_` dimensions with `occasion = 1`, the fraction scored `0`.
  This is the instrument's distinctive exclusion signal.

A raw `total` column exists in the ratings table for convenience, but because the number of
applicable `cp_` dimensions varies across papers, **compare per-dimension means, not raw totals**
(same rule as v0.2).

**Do not collapse instruments into one grand total.** The `cp_` markers and the v0.2
`causal_abstraction` scale measure the *same construct two different ways* (textual move vs.
substantive commitment). Summing them double-counts and rebuilds the exact v0.1 error the project
already corrected. Keep instruments separate; their correlation is a *result*, not a score.

### Preregistered expected relationships

State these before scoring real papers, so confirmation is not retrofitted:

- **Convergent:** `cp_marker_mean` should correlate positively and moderately-to-strongly with the
  v0.2 `causal_abstraction` per-dimension mean; `si_marker_mean` with `statistical_inductivist_
  dependence`. Moderate is the target — near `1.0` means the instrument is redundant, near `0`
  means it is invalid.
- **Discriminant:** `cp_marker_mean` and `si_marker_mean` must **not** sit near `−1`. A paper can be
  high on both (particle physics) or low on both (a purely descriptive atlas with no certification
  emphasis). If they collapse toward a single axis, the v0.1 mistake has returned in new clothing.

## Validation Plan (run before trusting any real-paper round)

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

## How To Wire A v0.3 Round

1. Add a `v0.3-pilot` judge round in `judge_rounds.csv` tied to a `rounds.csv` row whose
   `rubric_version = v0.3-pilot`, with the intended blinding profile.
2. Record raw scores in `multi_judge/paradigm_marker_ratings.csv` (one row per paper per
   judge-round) and the supporting spans in `multi_judge/paradigm_marker_evidence.csv`.
3. Add a `v0.3-pilot` aggregation set in `aggregation_sets.csv` once eligible rounds exist (leave
   `eligible_round_ids = TBD` until then so the build script skips it).
4. Run `python3 corpus/scripts/build_multi_judge_aggregates.py`. The script is rubric-version
   aware and now knows the `paradigm_marker` instrument for `v0.3-pilot`.

## Status

Provisional. The eight dimensions, anchors, occasion-gating, and thresholds in the validation plan
are all subject to revision after the red-team pass and the first blinded round. Treat v0.3 as a
hypothesis about how to measure paradigm-marker language, not as a settled instrument.
