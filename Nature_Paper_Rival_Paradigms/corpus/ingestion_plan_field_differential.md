# Corpus Redesign and Ingestion Plan (Field-Differential)

Status: planning sketch. This plan reorganizes the corpus to serve the revised theory,
in which the sharpest test is **field-differential**: whether scientific deceleration
tracks a field's *paradigm orientation* (statistical-inductivist vs causal-Popperian)
rather than its statistical intensity, its age, or accumulated knowledge.

It supersedes the original four-stratum, matched-control design.

---

## 0. No stratum labels; paradigm is measured, not assigned

Two things must be kept apart:

- **Discarded: outcome-defined strata.** The old A/B/C/D scheme — especially Stratum B,
  "matched controls that did not become recognized progress" — is gone. Membership there
  depended on the outcome under study, which made it circular.
- **Kept: a neutral sampling frame.** The cohort still samples across **field, publication
  year, and venue tier**. These are verifiable facts about a paper, not judgments about its
  epistemic content, and they serve as the sampling frame and as analysis covariates.

Critically, **paradigm orientation is an outcome of blinded CACS/SIDS coding, not a label
stored on papers.** The corpus tables therefore carry only neutral descriptors. Stamping a
paper or a field as "inductivist" or "causal" at ingestion would (a) prejudge the very
quantity the field-differential test measures, (b) make field-level paradigm dominance no
longer independent of the design, and (c) leak information to blinded coders.

The "paradigm gradient" below is used **only as acquisition guidance** — a rationale for
which fields to go collect so the corpus spans the gradient. Those expectations are to be
confirmed or refuted by coding; they do not become corpus labels, and there are no
"control" or "focal" roles attached to papers.

---

## 1. What changed, and why the old corpus no longer fits

The original corpus was built for an A-vs-B *case-control discovery study*: famous
discovery papers vs hand-matched contemporaneous controls. The revised design needs three
things the current corpus does not supply:

1. **Fields spanning a paradigm gradient**, including at least one field expected to be
   statistically intensive but causal-Popperian (e.g., structure-driven particle physics),
   so that paradigm orientation can be dissociated from statistical intensity *once coded*.
   The current corpus has no such field.
2. **Within-field time series** with enough era coverage to estimate each field's
   orientation-drift trajectory — the *primary*, mechanism-agnostic temporal measurement.
   Coverage that also brackets machine-learning (ML) adoption additionally supports the
   *secondary* staggered-timing test (drift onset should *follow* adoption where ML applies).
3. **Era and venue-tier coverage**, so paradigm is not confounded with time period or with
   the prestige tail. The current corpus is almost entirely recent-or-canonical and
   entirely top-tier.

The current corpus is also concentrated: 8 of 12 active papers are top-tier computational
biology, while structure-driven physics and recent mechanistic biology are absent.

---

## 2. Target sampling frame (neutral facts only)

Frame = **field x publication year x venue tier**, defined independently of any outcome.

- **field** — the paper's actual field/subfield (e.g., "computational biology",
  "experimental particle physics", "molecular/cell biology"). A verifiable fact.
- **publication year** — already in `papers.csv`. Era groupings are *derived* in analysis,
  not stored as a label on the paper.
- **venue tier** — top / mid / specialist. Needed because the reward-step hypothesis is
  specifically about what prestige venues reward, so tiers must be represented, not
  collapsed.

ML-penetration timing is a property of a *field*, not of a paper; it belongs in a small
separate field-level reference table, not as a per-paper tag.

### Acquisition guidance (NOT corpus labels)

To make the corpus span the gradient, deliberately acquire fields at different expected
corners of paradigm x statistics-intensity. These expectations guide collection only and
will be tested by blinded coding:

| Field to acquire | Why we want it (expectation to be tested) |
|------------------|-------------------------------------------|
| Computational biology / genomics-ML | The focal high-statistics, likely-inductivist case |
| ML / AI methods proper | Likely the most inductivist; native output is predictive performance |
| Structure-driven experimental physics (HEP, neutrino, gravitational waves) | High statistics but expected causal-Popperian — the key dissociation |
| Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics) | Within-physics contrast to the structure-driven subfields |
| Mechanistic molecular / cell / developmental biology | Within-biology contrast to computational biology |
| Structure-driven condensed matter / chemistry | Fills the lower-statistics, structure-driven part of the gradient |

---

## 3. Mapping current papers onto the frame

Stored descriptors only: field, year, venue tier. (Italic field names are descriptive, not
labels with analytic weight.)

**Active papers (12):**

| ID | Short title | Year | Venue (tier) | Field |
|----|-------------|------|--------------|-------|
| P0009 | AlphaFold | 2021 | Nature (top) | Computational biology / ML |
| P0010 | Tabula Sapiens | 2022 | Science (top) | Computational biology |
| P0012 | Tabula Muris | 2018 | Nature (top) | Computational biology |
| P0011 | COVID immune atlas | 2020 | Nat Med (top) | Computational biology |
| P0007 | GSEA | 2005 | PNAS (top) | Computational biology |
| P0006 | Connectivity Map | 2006 | Science (top) | Computational biology |
| P0003 | Drop-seq | 2015 | Cell (top) | Computational biology |
| P0014 | Partitioning heritability (LDSC) | 2015 | Nat Genet (top) | Statistical genetics |
| P0004 | Watson-Crick DNA structure | 1953 | Nature (top) | Molecular biology |
| P0013 | Meselson-Stahl | 1958 | PNAS (top) | Molecular biology |
| P0002 | Fire-Mello RNAi | 1998 | Nature (top) | Molecular/developmental biology |
| P0008 | Shechtman quasicrystals | 1984 | PRL (top) | Condensed-matter physics |

**Former Stratum B papers — re-absorb as ordinary cohort members (no special role):**

| ID | Short title | Year | Venue (tier) | Field |
|----|-------------|------|--------------|-------|
| P0015 | Pauling-Corey nucleic-acid structure | 1953 | PNAS (top) | Molecular biology |
| P0016 | Taylor autoradiography | 1957 | PNAS (top) | Molecular/cell biology |
| P0001 | Guo-Kemphues par-1 | 1995 | Cell (top) | Developmental biology |
| P0005 | Levine-Steinhardt quasicrystal theory | 1984 | PRL (top) | Condensed-matter physics |

Re-absorbing them recovers real acquisition work. They are no longer "controls"; they are
simply papers in their fields. Drop the matched-control semantics from their rows.

**Resulting coverage (by field x era), for gap-spotting only:**

| Field | pre-2005 | 2005-2014 | 2015-present | Total |
|-------|----------|-----------|--------------|-------|
| Computational biology / genomics | 0 | 2 | 6 | 8 |
| Molecular / cell / developmental biology | 6 | 0 | 0 | 6 |
| Physics (condensed matter) | 2 | 0 | 0 | 2 |
| Experimental particle / gravitational physics | 0 | 0 | 0 | **0** |
| Statistically-oriented physics | 0 | 0 | 0 | **0** |

---

## 4. Gap analysis (priority-ordered)

1. **Structure-driven, statistics-intensive physics — absent. Highest priority.** Without a
   field expected to be high-statistics yet causal, the central dissociation (paradigm vs
   statistics-intensity) has nothing to rest on, and the theory's strongest discriminating
   prediction cannot be tested.
2. **Recent mechanistic biology — absent.** All mechanistic biology is pre-2000, so
   "computational vs mechanistic biology" is confounded with era. Need contemporary
   causal/intervention biology.
3. **Pre-2005 computational-biology baseline — absent.** Earliest such paper is 2005; the
   ML-adoption time series for the focal field needs ~1999-2004 anchors.
4. **Statistically-oriented physics — absent.** Needed for the within-physics contrast.
5. **Venue-tier diversity — none.** Everything is top-tier; representative field-level mix
   and conversion-rate estimates need mid/specialist venues.
6. **Within-field time series — thin.** Only computational biology has multiple time points.
   The primary orientation-drift test needs each field covered across several eras of its
   history; the secondary timing test additionally needs the relevant fields bracketed before
   and after their ML adoption.

---

## 5. Candidate ingestion targets

Illustrative; **metadata- and OA-verify at acquisition**. A starting shortlist, not a final
selection.

**Structure-driven physics (high statistics):**
- Higgs boson discovery (ATLAS; CMS), 2012.
- Atmospheric neutrino oscillation (Super-Kamiokande), 1998.
- Gravitational-wave detection GW150914 (LIGO), 2016.
- W/Z boson discovery (UA1/UA2), 1983 — earlier-era anchor.

**Statistically-oriented physics:**
- Precision cosmology parameter estimation (e.g., Planck final results), 2018/2020.
- Dark-matter direct-detection exclusion-limit papers.
- Deep-learning-for-physics method papers.

**Recent mechanistic biology (lower-statistics, causal):**
- CRISPR-Cas9 mechanism (Jinek et al.), 2012.
- Optogenetics (Boyden et al.), 2005.
- A recent cryo-EM structure-function or signaling-mechanism paper.

**Pre-2005 computational-biology baseline:**
- Molecular classification of cancer by expression monitoring (Golub et al.), 1999.
- Lymphoma expression profiling (Alizadeh et al.), 2000.
- Significance Analysis of Microarrays (Tusher et al.), 2001.

**Mid/specialist venue anchors:** for each field, add representative non-flagship papers
from the same years to break the top-tier-only bias.

---

## 6. Schema changes to `papers.csv`

Add (neutral facts only):
- `field` — the paper's field/subfield.
- `venue_tier` — top / mid / specialist.

Keep:
- `year` — already present; era groupings are derived in analysis, not stored as labels.

Add a separate small field-level reference table (`metadata/fields.csv`, not per paper; see
`metadata/schema.md`):
- `field`, `ml_penetration_onset_early`, `ml_penetration_onset_late`, `ml_penetration_source`,
  `ml_penetration_applicability`, `notes`. The ML-penetration fields feed the *secondary*
  timing test (TS1) only; the primary orientation-drift test is computed from the
  era-stratified coded cohort and does not read this table.

Do NOT store paradigm/role labels. Paradigm orientation and statistics dependence come from
blinded CACS/SIDS coding in `coding/`, never from a pre-assigned column. There is no
`design_role`, no `paradigm_orientation_prior`, and no stratum column.

Retire (legacy of the matched-control design):
- `matched_to_paper_id`, `matching_notes` — remove.
- `corpus_stratum`, `corpus_stratum_label` — retire; replaced by `field` + `year` +
  `venue_tier`. Keep briefly only for traceability, then drop.

---

## 7. Phased sequence

- **Phase 1 (enables the key test):** acquire the structure-driven physics field — a few
  papers across eras. This alone makes the paradigm-vs-statistics-intensity dissociation
  possible once coded.
- **Phase 2 (removes the era confound):** recent mechanistic biology + a pre-2005
  computational-biology baseline, building that field's time series.
- **Phase 3 (within-discipline contrast + tiers):** statistically-oriented physics; add
  mid/specialist-tier papers to each field.
- **Phase 4 (scale-up):** move from convenience anchors to **stratified probability
  sampling** within field x era x venue-tier cells, with design weights for any enriched
  cell. No citation-weighting (citation is an outcome).

---

## 8. Open decisions

- Final field list and how finely to split subfields.
- Pilot target N per field x era cell (suggest ~4-6 for occupied cells).
- Whether to re-ID the re-absorbed former-B papers or keep IDs with retired columns.
- Per-field ML-penetration years (for the field-level reference table).
- How far to pursue representative sampling now vs after the fields are populated.
