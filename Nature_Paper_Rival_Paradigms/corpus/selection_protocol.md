# Paper Selection Protocol (Two-Stratum Hybrid)

Status: protocol specification. This document defines *how* papers are chosen for the
corpus. It operationalizes the desiderata in `ingestion_plan_field_differential.md`, which
specifies *what* the additions must achieve (span the paradigm gradient, break the era and
venue-tier confounds) but does not specify the selection mechanism itself. This document
supplies that mechanism.

It is consistent with, and does not override, the design rules in
`ingestion_plan_field_differential.md` and `metadata/schema.md`. Where this document adds a
column or a rule, those files should be updated to match (see §7).

---

## 0. The one fact everything follows from

**"Landmark" status is an outcome, not a sampling fact.** A paper is a landmark *because*
it accrued durable uptake and recognition — which is the dependent variable the thesis is
about. It is therefore impossible to select a landmark set in an outcome-blind way, and a
landmark set must never be pooled into population estimates. The only safe way to include
recognizable papers is to collect them as a **separate, explicitly flagged stratum that is
walled off from base-rate inference.**

This yields a two-stratum hybrid:

- a **random base sample** that is outcome-blind and carries the inference, and
- a **landmark set** that supplies face validity and rubric calibration but never enters a
  base-rate estimate.

Both strata are coded identically and blindly. They differ only in how a paper entered the
corpus and in how its codes are *used* downstream.

---

## 1. Stratum A — Random base sample (the inferential backbone)

This is the probability sample. All field-differential tests and all base-rate paradigm
estimates run on this stratum, with design weights.

### 1.1 Sampling frame

The frame is the cross-classification **field x era x venue-tier**, all neutral facts
(per `schema.md`). Enumerate the cells before sampling:

- **field** — drawn from a predefined field list (see §4). A verifiable descriptor, not a
  paradigm label.
- **era** — derived from publication year. Suggested bands: `pre-1990`, `1990-2004`,
  `2005-2014`, `2015-present`. Bands exist primarily to give each field enough era coverage to
  estimate its orientation-drift time series (the primary temporal test). Bracketing a field's
  ML-penetration year is a *secondary* overlay, required only for the fields where the
  machine-learning-timing hypothesis (TS1) is in play; the penetration band is held in the
  separate field-level reference table, not on the paper.
- **venue-tier** — `top` / `mid` / `specialist`, from a predefined tiered journal list (§4).

Set a target N per occupied cell (suggested 4-6, per `ingestion_plan` §8). Cells are filled
*deliberately* — the strata are chosen, not left to chance — so priority gaps (e.g.
structure-driven physics) are guaranteed coverage rather than left to a random draw.

### 1.2 Within-cell randomization

Within a chosen (field, era, tier) cell, select each paper by:

1. **Journal** — draw at random from the predefined list of OA-accessible journals that
   belong to that field and tier. "OA-accessible" means a verified public full text can be
   obtained — a born-OA journal, **or** a public archive / PMC / repository copy. This is
   broader than "OA journal" on purpose: restricting to born-OA journals would truncate the
   early eras, exactly where the corpus needs depth (see §3).
2. **Issue** — draw at random from issues of that journal published in a year within the
   cell's era band.
3. **Article** — draw at random from research articles in that issue (exclude editorials,
   errata, front matter, and other non-article content via a pre-stated inclusion rule).
4. **Deduplicate** — if the drawn article is already in the corpus, discard and redraw at
   the article level (then issue, then journal) until a new article is found. This is simple
   sampling without replacement.

### 1.3 Record the selection probability

Because issue-then-article draws give articles unequal inclusion probabilities (a paper in a
thin journal is more likely to be drawn than one in a large journal), **store each paper's
inclusion probability and its inverse-probability design weight.** Without this, journal size
leaks in as a confound. The weight is used in all Stratum A estimates.

### 1.4 What Stratum A is for

Estimating each field's base-rate paradigm orientation and testing whether deceleration
tracks paradigm orientation rather than statistical intensity, era, or venue tier. This is
the only stratum that drives those estimates.

---

## 2. Stratum B — Landmark set (calibration and contrast only)

A purposive, **pre-specified** set of recognized papers, compiled *before and independently
of coding* from an explicit, written-down criterion (e.g. textbook-cited, prize-associated,
or a named "classic papers" list per field). The criterion is recorded; it is permitted to
be outcome-correlated *because this stratum is quarantined from base-rate inference.*

### 2.1 What Stratum B is for

1. **Rubric calibration / validation.** Do CACS and SIDS score recognized causal papers and
   recognized inductivist papers the way the theory predicts? If the instrument cannot
   separate clear cases, it is not ready. This is the primary justification for the stratum.
2. **Face validity.** The corpus contains recognizable papers, so it does not read as a pile
   of obscure articles.
3. **Explicit contrast.** Comparing Stratum B against Stratum A *within a field* estimates
   how unrepresentative the famous papers are of their field's base rate — itself a useful
   quantity, reported as a clearly labeled separate comparison.

### 2.2 Hard rules

- Stratum B carries **no design weights** and is **never pooled** into a Stratum A estimate
  or a population-level base rate.
- Any figure or table that mixes the strata must label them as separate series. There is no
  combined denominator.

### 2.3 The existing 16 papers are Stratum B

The current pilot (AlphaFold, Watson-Crick, Meselson-Stahl, the quasicrystal papers, etc.)
is already a convenience set of landmark papers. Re-tag it `sample_source=landmark` rather
than treating it as a random sample. This recovers the acquisition work, seeds Stratum B for
free, and leaves Stratum A to be built fresh.

---

## 3. Acquisition frame: OA-accessible, not born-OA

A uniform reach across eras is incompatible with a born-OA-journal-only list, because OA
journals are largely a post-2000 phenomenon. The pilot's own early papers were obtained from
public archives and PMC full-text XML, not OA journals. Therefore the journal list (§4) and
the within-cell draw (§1.2) operate over **OA-accessible** sources: born-OA journals plus
journals whose issues are reliably mirrored in a public archive / PMC / repository for the
era in question. Accessibility is verified at acquisition, as in the current pilot.

---

## 4. The predefined lists are part of the design

The randomization is conditional on two hand-built lists, and **those lists carry the design
decisions** — they are not neutral background:

- **Field list** — the fields to sample, chosen to span the paradigm x statistics-intensity
  gradient (`ingestion_plan` §2). Document the rationale per field.
- **Tiered, field-specific journal list** — for each (field, tier), the set of
  OA-accessible journals eligible for the journal draw. Document the inclusion rule
  (how a journal earns a tier; how OA-accessibility was verified).

Both lists are versioned and stored with the corpus. Changing a list is a design change and
is logged. Stating the lists explicitly is what keeps "random" from quietly meaning
"convenient."

### 4.1 Initial field list (v0)

A field earns its place by providing one or both of two distinct kinds of discriminating
power:

- **Between-field dissociation** — the field occupies a *corner* of the paradigm x
  statistics-intensity plane that is otherwise empty (e.g. high-statistics yet causal). This
  separates paradigm orientation from statistical intensity *across* fields.
- **Within-field paradigm variance** — the field contains, under one disciplinary roof, a
  strongly inductivist tradition *and* a strongly causal/mechanistic one. This lets paradigm
  vary *while field-level institutional factors are held roughly constant*, a dissociation
  that is largely independent of the between-field one.

A field earns its place primarily through paradigm variance of these two kinds. A clean,
datable machine-learning-adoption inflection (noted for some fields below) is a *secondary*
bonus relevant only to the machine-learning-timing hypothesis (TS1); it is not what justifies
including a field. Psychology illustrates the point: its salient inflection is a
significance-testing and replication inflection rather than a machine-learning one, which is
fine — it participates fully in the primary temporal tier and is simply marked
`ml_penetration_applicability = low` for TS1.

| # | Field | Primary role | Rationale (expectation, to be tested by coding) |
|---|-------|--------------|--------------------------------------------------|
| 1 | Computational biology / genomics-ML | Between-field | Focal high-statistics, likely-inductivist case. |
| 2 | ML / AI methods proper | Between-field | Expected most inductivist; native output is predictive performance. |
| 3 | Structure-driven experimental physics (HEP, neutrino, gravitational waves) | Between-field | High statistics but expected causal-Popperian — the key dissociation; **Phase 1 priority**. |
| 4 | Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics) | Between-field | Within-physics contrast to the structure-driven subfields. |
| 5 | Mechanistic molecular / cell / developmental biology | Between-field | Within-biology contrast to computational biology. |
| 6 | Structure-driven condensed matter / chemistry | Between-field | Fills the lower-statistics, structure-driven corner. |
| 7 | Computational neuroscience | Within-field (+ between) | Holds a high-statistics theory/mechanism tradition (dynamical-systems, normative/Bayesian models) next to a high-dimensional data-driven one (large-scale recordings, deep-net models). Independently occupies the high-statistics-yet-causal corner, so it serves as a second, non-physics replication of the central dissociation. Clean ML-adoption inflection (~2014) for the time series. |
| 8 | Cognitive science / psychology | Within-field | Canonical inductivist failure-mode exemplar (publishable significant effects that often fail to convert into durable causal abstractions) sitting alongside a causal/mechanistic tradition (mathematical psychology, process/computational models, psychophysics, Bayesian cognition). High within-field paradigm variance; well-documented inflection (~2011, replication crisis) for the time series. |

The list is versioned (`v0`); revisions are logged as design changes per §4.

### 4.2 Two cautions for the within-field-variance fields (7, 8)

1. **Budget more per-field N.** A field chosen for *within-field* variance is informative
   only if the corpus can estimate the *distribution* of paradigm orientation inside it, not
   just a field mean. Allocate more N to fields 7-8 than to a paradigm-homogeneous field;
   the field mean alone discards the very signal these fields contribute.
2. **Do not conflate weak measurement with inductivist paradigm.** In both fields,
   deceleration can be driven by soft or noisy measurement (low power, unreliable
   constructs) rather than by paradigm orientation as such. These must be kept separate —
   via the CACS/SIDS coding and as an explicit measurement-quality covariate — or the test
   will mistake a measurement problem for a paradigm effect.

These additions do **not** change the phasing in `ingestion_plan` §7: they lean toward the
inductivist / moderate-statistics region, so adding them before the structure-driven physics
anchor would worsen the existing imbalance. Physics remains Phase 1.

---

## 5. Blinding: `sample_source` must not reach judges

Coders must not know whether a paper is a landmark; otherwise they will score landmarks more
generously and contaminate the calibration the landmark stratum exists to provide.

The blind packets already exclude field labels, citation counts, and prior scores
(`schema.md` §Blind Packets). Extend the same exclusion to **`sample_source`**: it must
never appear in `paper_metadata.json`, `packet_manifest.json`, or any judge-visible file.
The packet builder's metadata whitelist already omits it; this rule makes the omission
intentional and permanent. Stratum membership is revealed only at the analysis stage.

---

## 6. Procedure summary

1. Define and version the field list and the tiered OA-accessible journal list (§4).
2. Enumerate field x era x tier cells; set a target N per cell (§1.1).
3. For each cell, run the journal -> issue -> article random draw, deduplicating and
   recording each paper's inclusion probability / design weight (§1.2-1.3). Tag
   `sample_source=random_base`.
4. Compile the pre-specified landmark list per field from a written criterion; acquire it;
   tag `sample_source=landmark`. Re-tag the existing 16 papers as `landmark` (§2.3).
5. Acquire over OA-accessible sources, verifying access as in the current pilot (§3).
6. Build blind packets that exclude `sample_source` and all other outcome/identity-of-design
   fields (§5). Code both strata identically and blindly.
7. Analyze: base-rate and field-differential estimates use **weighted Stratum A only**;
   Stratum B is used for rubric calibration and as a separately labeled contrast (§2).

---

## 7. Schema changes

Add to `papers.csv` (neutral provenance facts only):

- `sample_source` — `random_base` / `landmark`. Records *how the paper entered the corpus*,
  not what it is. It is provenance, not a paradigm or role label, and is excluded from blind
  packets.
- `selection_probability` — the article's inclusion probability under the §1.2 draw
  (`random_base` papers only; blank for `landmark`).
- `design_weight` — inverse-probability weight used in Stratum A estimates
  (`random_base` only; blank for `landmark`).

Add a small **field-level reference table** (`metadata/fields.csv`, not per paper), as
anticipated in `ingestion_plan` §6 and documented in `metadata/schema.md`:
`field`, `ml_penetration_onset_early`, `ml_penetration_onset_late`, `ml_penetration_source`,
`ml_penetration_applicability` (`high` / `low` / `not_applicable`), `notes`. The ML-penetration
fields feed the secondary timing hypothesis (TS1) only; the primary temporal tier is measured
from the era-stratified coded cohort and does not read this table.

Do **not** add: any paradigm/orientation prior, any `design_role`, or any stratum-as-outcome
column. `openalex_cited_by_count` remains an outcome, never a sampling weight
(`schema.md` §23). Update `schema.md` to document the three new columns and to add
`sample_source` to the blind-packet exclusion list.

---

## 8. Open decisions

- Final field list and subfield granularity (shared with `ingestion_plan` §8).
- Exact era bands and each field's ML-penetration year.
- Target N per cell, and whether mid/specialist tiers get the same N as top.
- The written landmark-inclusion criterion per field, and the target size of Stratum B.
- Whether `random_base` and `landmark` papers share an ID series or are namespaced.
- Handling of multi-field journals when assigning a journal to a field in the §4 list.
