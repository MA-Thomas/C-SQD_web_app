# Paradigm-Marker Red-Team Calibration Set (v0.3-pilot)

## Purpose

Before any real paper is scored on the v0.3 paradigm-marker instrument, a judge (human or AI)
must score these constructed items and reproduce the expected scores. The set exists to test the
one thing that can break the instrument: **does it score the epistemic move, or just the
vocabulary?**

Every item is **synthetic** — authored for this test, not drawn from any corpus paper. Each is
built to spring a specific trap. A keyword counter fails most of them. The rubric passes only if it
scores *role*, not *words*.

Trap types:

- `rhetoric_without_move` — Popperian vocabulary present, the move absent. Must score the `cp_`
  target **low**.
- `move_without_vocab` — the move enacted, canonical vocabulary absent. Must score the `cp_` target
  **high**.
- `negation_attribution` — the marker word appears inside a negation or is attributed to others
  (the paper disclaims the move). Must not be credited.
- `field_dialect` — physics-style "exclude/rule out" certification language. Tests whether the
  judge distinguishes certification that is *instrumental to a test* (low `si_terminal`) from
  certification that is *terminal* (high `si_terminal`), without being fooled by accent.
- `boilerplate_limitations` — generic limitations text that is not a load-bearing failure
  condition. Must score `cp_assumption_vulnerability` **low**.

Pass criterion: the judge's score is within ±1 of every expected score AND on the correct side of
the trap's decision boundary (e.g. a `rhetoric_without_move` item must land at `0`–`1`, never `3`–`4`).
Structured expected values are in `calibration_items.csv`.

---

### C01 — `rhetoric_without_move` — target `cp_risky_prediction`
> "Our analysis provides a rigorous test of the hypothesis and decisively refutes the null,
> confirming a highly significant association between marker expression and patient outcome
> (p < 0.001, FDR < 0.01)."

**Expected:** `cp_risky_prediction = 1`, `si_terminal_certification = 4`, `si_association_framing = 3`.
**Why:** "test" and "refute" are present, but the only thing refuted is a statistical null; no
structural claim is staked on an outcome that could have failed. The warrant terminates in p/FDR.
A keyword counter would score this Popperian — the trap.

### C02 — `move_without_vocab` — target `cp_risky_prediction`, `cp_rival_elimination`
> "If replication were conservative, both strands of every daughter molecule would band at the
> parental density. After one generation we instead found a single band at exactly the intermediate
> density, and after two generations equal intermediate and light bands appeared."
**Expected:** `cp_risky_prediction = 4`, `cp_rival_elimination = 4`, `si_terminal_certification = 0`.
**Why:** No "falsify," "test," or "Popper" anywhere, yet this stakes the central claim on a result
that could have come out the rival way and discriminates conservative vs. semiconservative vs.
dispersive replication. A keyword counter scores it 0 — the inverse trap.

### C03 — `negation_attribution` — target `cp_risky_prediction`, `cp_generative_structure`
> "Unlike falsificationist accounts that seek to refute a single mechanism, we make no mechanistic
> conjecture; we instead characterize the full distribution of expression states across 200,000
> cells."
**Expected:** `cp_risky_prediction = 0`, `cp_generative_structure = 0`, `si_association_framing = 3`,
`si_accumulation_progress = 2`.
**Why:** "falsificationist," "refute," and "mechanism" all appear, but inside an explicit
disclaimer. The paper announces it is *not* making the move. Credit nothing to `cp_`.

### C04 — `field_dialect` (instrumental certification) — target `si_terminal_certification`, `cp_rival_elimination`
> "The excess over background reaches 5.1σ, allowing us to exclude the no-resonance hypothesis and
> establish a new state at 750 GeV consistent with the predicted spin-2 mediator."
**Expected:** `si_terminal_certification = 2`, `cp_rival_elimination = 3`, `cp_risky_prediction = 3`.
**Why:** Heavy certification vocabulary ("5.1σ," "exclude"), but the statistics are *instrumental*
to ruling out a named rival hypothesis and testing a predicted structure — so `si_terminal` is
**not** maxed, and the `cp_` moves score high. This is the particle-physics case from §3.3; getting
it right is the core discriminant test.

### C05 — `rhetoric_without_move` — target `cp_generative_structure`
> "We identify a 12-gene transcriptional signature that drives disease progression and could serve
> as a therapeutic target."
**Expected:** `cp_generative_structure = 1`, `si_association_framing = 4`,
`si_terminal_certification = 1`.
**Why:** "drives" and "target" imply mechanism, but the deliverable is a fitted signature; no
generative process is posited or reasoned from. The mechanism word is a label doing no inferential
work. *(Note: the snippet states no explicit certification statistic, so `si_terminal_certification`
is genuinely ambiguous here — anything 0–2 is defensible; the graded purpose of C05 is the
`cp_generative_structure` rhetoric-without-move trap. Softened from 2→1 after the first independent
blind replicate, 2026-06-20.)*

### C06 — `boilerplate_limitations` — target `cp_assumption_vulnerability`
> "Limitations of our study include a modest sample size and possible residual batch effects.
> Future work with larger, multi-site cohorts will be needed to confirm these findings."
**Expected:** `cp_assumption_vulnerability = 1` (occasion = 1).
**Why:** Generic limitations boilerplate, not a load-bearing assumption with a stated condition under
which the central claim would be *false*. Occasion present, move absent → a conspicuous exclusion.

### C07 — `move_without_vocab` — target `cp_assumption_vulnerability`
> "This partition assumes the functional annotation captures the causal variants. If causal variants
> lie disproportionately in unannotated regions, the heritability assigned to each category is
> misallocated and the central enrichment estimate does not hold."
**Expected:** `cp_assumption_vulnerability = 4`, `cp_generative_structure = 2`.
**Why:** No "falsifiable" or "limitation," but it names a load-bearing assumption and the exact
condition under which the headline estimate becomes invalid. This is the real move C06 only
gestures at.

### C08 — `rhetoric_without_move` — target `cp_counterfactual_intervention`
> "These observational correlations suggest that pathway activity may influence the phenotype;
> interventional experiments to manipulate the pathway are an important direction for future work."
**Expected:** `cp_counterfactual_intervention = 1` (occasion = 1), `si_association_framing = 3`.
**Why:** "interventional" and "manipulate" appear, but only as deferred future work; the paper
itself reasons only over correlations. Occasion present, move absent → conspicuous exclusion.

### C09 — `move_without_vocab` — target `cp_counterfactual_intervention`
> "Injecting double-stranded RNA matching the gene abolished the corresponding phenotype, while
> dsRNA against an unrelated gene left it intact; sense or antisense strands alone had little
> effect."
**Expected:** `cp_counterfactual_intervention = 4`, `cp_rival_elimination = 3`,
`si_association_framing = 0`.
**Why:** No "counterfactual" or "causal," but the predicted differential under manipulation is the
crux, with built-in specificity controls acting as the contrast.

### C10 — `field_dialect` (terminal certification) — target `si_terminal_certification`
> "Our classifier achieves AUC 0.94 on held-out data, outperforming all prior benchmarks; this
> establishes the panel as a robust predictor of response."
**Expected:** `si_terminal_certification = 4`, `si_association_framing = 4`,
`cp_generative_structure = 0`.
**Why:** The warrant terminates in held-out performance; clearing the benchmark *is* the result,
with no structural conjecture served. Contrast with C04, where similar certification language is
instrumental — same accent, opposite role.

---

## Scoring the calibration round

Treat this as judge-round `round_calibration_v03_001` (rubric `v0.3-pilot`). Record scores in
`paradigm_marker_ratings.csv` using synthetic `paper_id`s `C01`–`C10`, and spans in
`paradigm_marker_evidence.csv`. Compare against `calibration_items.csv`. If any item lands on the
wrong side of its trap boundary, revise the anchor that failed **before** scoring real papers — the
calibration set is the gate, not a formality.
