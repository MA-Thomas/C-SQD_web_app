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

### 1.1 Sampling frame and the unit of sampling: the cell

The frame is built from four neutral facts (per `schema.md`): **field**, **era**,
**source**, and **venue-tier**. The atomic unit of sampling — the thing that is targeted,
drawn from, and weighted — is the **cell**, defined as:

> **cell = [field × era × source × tier]**

Each cell is one source/venue, in one field, in one era band, at the tier that source holds
for that field and era. Enumerate cells before sampling.

- **field** — drawn from a predefined field list (see §4). A verifiable descriptor, not a
  paradigm label. For broad/general sources, field is assigned at the article level by the §1.2
  filter, so one source can appear in several field-cells.
- **era** — derived from publication year. Suggested bands: `pre-1990`, `1990-2004`,
  `2005-2014`, `2015-present`. Bands exist primarily to give each field enough era coverage to
  estimate its orientation-drift time series (the primary temporal test). Bracketing a field's
  ML-penetration year is a *secondary* overlay, required only for the fields where the
  machine-learning-timing hypothesis (TS1) is in play; the penetration band is held in the
  separate field-level reference table, not on the paper.
- **source** — one eligible OA-accessible venue/source (a journal, conference proceedings, or
  other declared archival publication source) from the §4 list. The source is what the
  archival-unit→paper draw (§1.2) runs inside, and it is what the inclusion probability and
  design weight (§1.3) are computed over. A cell is therefore a single-source probability
  sample.
- **tier** — `top` / `mid` / `specialist`, a **pinned attribute** of the source, not an
  independent axis. Per §4 (D5) tier is assigned at the venue level, *within field*, and may be
  *era-specific*; so the tier of a `[field × era × source]` triple is already determined. Tier
  is carried in the cell key for visibility and provenance — it forces tier to be recorded and
  confronted at enumeration, and it freezes which within-field, era-specific tier judgment was
  in force. **Consistency rule:** `cell.tier` must always equal the source's pre-specified tier
  for that field-era; if a tier is ever reassessed, both are updated in lockstep.

**Coverage target (documented).** Each eligible, scheduled cell has a documented coverage
target of **N ≥ 10** random-base papers. This is a breadth-first coverage target, **not a
cap**: design weights (§1.3) absorb unequal cell sizes in base-rate estimation, so over-target
cells (e.g. extended historical draws) remain valid. Priority goes to bringing underfilled
eligible cells up to 10 before deepening any cell further. Treat ineligible, historically
unavailable, exhausted, or accessibility-limited cells as documented exceptions, not targets to
force. Cells are filled *deliberately* — the strata are chosen, not left to chance — so priority
gaps (e.g. structure-driven physics) are guaranteed coverage.

**Hard requirement — tier coverage.** Including tier in the cell key makes tier *recorded*,
but it does not by itself make tier *spanned*: a field can be filled entirely from cells of one
tier and still be internally consistent. Therefore, as a hard requirement, **each field must
include cells spanning at least two venue tiers** (ideally all three: top / mid / specialist).
A field covered by a single tier (e.g. specialist-only) does not satisfy the frame, regardless
of how many papers it contains, because the reward-step hypothesis is defined *across* tiers.
Tier coverage is verified at the field level, separately from the per-cell N target.

### 1.2 Within-cell randomization

A cell *is* a single source within a field × era at its pinned tier (§1.1), so "cell" and
"source block" denote the same unit; "source block" is retained where the emphasis is on
walking through sources sequentially. Within a field × era, the protocol proceeds through the
predeclared sources stepwise — opening one cell at a time — rather than drawing a new
venue/source for every paper. The randomization happens *inside* the cell, at the archival-unit
and paper levels:

1. **Source-block schedule** — list the predefined OA-accessible venues or sources that
   belong to that field and tier. "OA-accessible" means a verified public full text can be
   obtained — a born-OA venue, **or** a public archive / PMC / repository / arXiv copy. This
   is broader than "OA journal" on purpose: restricting to born-OA journals would truncate
   the early eras, exactly where the corpus needs depth (see §3). arXiv is normally an
   access route and version record, not the sampling frame, unless a separate preprint
   stratum is explicitly declared. The source-block order must be predeclared or randomized
   once and saved before sampling from those blocks. A partially completed field × era group is
   therefore interpreted as coverage of the completed cells (sources), not as a full field-level
   draw over all eligible sources.
2. **Source inventory cache** — when a source block is opened, check whether a versioned
   local inventory already exists for it. If this is the first time the venue/source has
   been selected, build the inventory before drawing an archival unit. The inventory should
   cover all years exposed by the official archive source, or a documented complete subset
   if the source imposes bounds. It records neutral metadata only: archive pages, archival
   units, article/paper landing-page URLs, venue section/type labels, titles, summaries,
   authors when exposed, OA markers when exposed, source URLs, request dates, parser version,
   user agent, rate limit, and any known incompleteness. Do not download PDFs during
   inventory construction. Use source-appropriate text/data-mining access rules (for
   example, a `TextDataMining` user agent and <=1 request/sec for Nature). Future draws from
   the same venue/source reuse this inventory, filtered to the target era, paper type, field,
   and OA-accessibility rules, rather than reparsing the archive from scratch.
3. **Archival unit** — draw at random from units of the active source block published in a year
   within the cell's era band. Examples: a journal volume/issue, a conference-year of
   accepted proceedings, a proceedings volume, or an official collaboration publication-list
   year if such a source has been declared in advance.
4. **Field filter for broad venues** — if the active source block is multidisciplinary
   (`Science`, `Nature`, `PNAS`, broad conferences, etc.), venue identity does not establish
   field membership. Filter the drawn unit to OA-accessible papers that match the target
   field by pre-specified neutral article-level facts (title/abstract, keywords, MeSH terms,
   OpenAlex concepts, journal/conference section labels, arXiv primary category, or manual
   field adjudication). Do **not** use paradigm-orientation judgments for this filter. If the
   unit contains zero matching papers, reject that unit and redraw a unit from the same
   venue/source and era frame until a unit with at least one field-matching eligible paper is
   selected.
5. **Paper** — draw at random from the eligible papers in the accepted unit (exclude
   editorials, errata, front matter, retractions, posters-only abstracts, and other
   non-paper content via a pre-stated inclusion rule). For field-specific venues this is
   simply the eligible paper list; for broad venues it is the field-filtered eligible paper
   list.
6. **Deduplicate** — if the drawn paper is already in the corpus, discard and redraw at the
   paper level, then unit level if needed, within the active source block until a new paper
   is found. This is simple sampling without replacement inside each source block.

### 1.3 Record the selection probability

Because unit-then-paper draws give papers unequal inclusion probabilities (a paper in a thin
unit is more likely to be drawn than one in a large unit), **store each paper's
inclusion probability and its inverse-probability design weight.** Without this, journal size
leaks in as a confound. The weight is used in all Stratum A estimates.

For a field-specific source block, the per-draw paper probability is:

`p_a = (1 / U_v) * (1 / N_u)`

where `U_v` is the number of non-empty eligible archival units in the active venue/source and
era band, and `N_u` is the number of eligible papers in the accepted unit. These counts are
computed from the versioned source inventory plus the pre-specified eligibility filters used
for the draw.

For a broad source block, compute the same quantity over the **field-filtered** effective
frame:

`p_a = (1 / U_v,field) * (1 / N_u,field)`

where `U_v,field` is the number of units in venue/source `v` and the era band that contain at
least one OA-accessible paper matching the target field, and `N_u,field` is the number of
eligible papers in the accepted unit matching the target field. The unit rejection step in
§1.2 is therefore equivalent to drawing uniformly from the non-empty field-eligible unit
frame; rejected empty units are not counted in `U_v,field`.

Each cell is a clean single-source probability sample, with its own per-paper inclusion
probabilities and inverse-probability design weights. Estimates above the cell level — a
field × era × tier estimate, a field × era estimate, or a field base rate — are therefore
**declared weighted aggregations over cells**, not a single pooled draw. The default
interpretation for a partially completed field × era group is **completed-cells-only**:
estimates describe the cells actually completed.

The across-cell combination is where the unit of analysis is chosen, and it is a modeling
decision, not a design-based one: sources were *purposively scheduled* (§4), so there is no
source-selection probability to recover — a weighting rule must be imposed, and that rule
fixes the estimand. The predeclared rule is:

1. **Always apply the within-cell design weight.** The unit→paper inclusion probability and its
   inverse-probability weight (the formulas above) are design-based and are applied in every
   estimate. This is non-negotiable; it removes archival-unit-size bias inside a source.

2. **Keep tier explicit; never collapse it by raw paper counts.** Tier enters every field-level
   number through the declared weights below, not by naive pooling. The reward-step hypothesis is
   tier-comparative and is reported **tier-stratified** (field × era × tier). The primary
   *temporal* analysis (orientation drift) groups by **field × era** per `metadata/schema.md`;
   there tier is collapsed, but only via the pre-declared tier weight in step 4 and reported beside
   its tier-stratified breakdown — not by pooling raw paper counts across tiers.

3. **Within a (field, era, tier) group, combine cells by equal-source weighting** — each source
   counts once, regardless of how many eligible papers it has — on top of the within-cell design
   weights. Rationale: a source's eligible-paper count is driven by journal volume and by its
   OA-accessible fraction (funding/era-correlated, per `journal_list_v0.md` D4), neither of which
   is the paradigm quantity under study. Paper-count ("source-size") weighting would let a
   high-volume or OA-heavy venue dominate the group and would stack a second OA-correlated factor
   on top of the within-cell weight, reimporting exactly the confounds the tiered, deliberately
   scheduled frame was built to control.

4. **Collapse across tier only with a pre-declared tier weight.** A field × era number (the
   primary temporal tier and any field base rate) is formed by combining the tier-stratified
   estimates with declared tier weights, and is always reported beside its tier-stratified
   breakdown — never as a raw pooled average over papers.

5. **Source-size weighting is the sensitivity arm, not the default.** It answers a legitimate but
   different question (the average eligible *paper* rather than the average *source*). Report the
   equal-source-vs-source-size gap as a published diagnostic so the choice is visible, not buried.

Do not silently treat a single cell as representative of its whole field × era group, and do not
treat a single-tier field as representative across tiers (§1.1 tier-coverage requirement). The
source/cell and inventory id must remain visible in analysis metadata.

### 1.4 Source inventory persistence

Source inventories are reusable sampling frames, not paper acquisitions. Store them under:

`corpus/source_inventories/<source_slug>/<inventory_id>/`

where `<inventory_id>` includes the parser id, parser version, coverage range, and build date.
Each inventory directory contains:

- `inventory_manifest.json` — source name, identifiers (ISSN/eISSN, conference acronym,
  proceedings series, or collaboration source when available), source URLs, parser id/version,
  build date, coverage years, user agent, rate limit, request log summary, file hashes, known
  gaps, and completeness notes.
- `archive_pages.csv` — one row per source archive page exposed by the official source
  (journal volume pages, conference-year pages, proceedings indexes, etc.).
- `units.csv` — one row per archival unit: unit type, year, date, volume/issue or
  conference-year/track/proceedings identifier, and unit URL.
- `items.csv` — one row per TOC/proceedings/publication-list item: unit id, paper landing-page
  URL, title, venue section/type, publication date, summary/abstract snippet, authors when
  exposed, OA marker when exposed, source URL, DOI/arXiv id when exposed.
- `field_assignments.csv` — optional derived table linking paper ids/URLs to neutral
  field labels and evidence. Keep this separate from the raw TOC inventory. It must not
  contain paradigm-orientation judgments.

Completed historical years are treated as frozen once successfully inventoried and are not
refreshed automatically. The current or otherwise incomplete year is refreshed when used.
Rebuild an inventory only when the parser logic changes materially, the source archive
structure changes, or a documented gap is discovered. Parser versions should be advanced
whenever changes affect which archive pages, units, or item rows enter the frame.

Every random draw must record the exact inventory used: `inventory_id`, manifest path,
manifest SHA-256, hashes of the unit and item tables used for the frame, parser id/version,
and any field-assignment table/version used for broad-venue filtering. This ties selection
probabilities to a frozen frame rather than to the source website as it may appear later.

### 1.4.1 Acquisition fallback after a draw

Inventory records are not acquisition attempts. After a paper is drawn, acquire it through
a tiered, auditable process:

1. Try the recorded or deterministic PDF route from the frozen inventory.
2. If that fails, open the article landing page on the publisher site in a browser and use
   the visible PDF, download, or full-text controls the way a reader would. Record the
   landing-page URL, final PDF/download URL if exposed, result, timestamp, and any ordinary
   cookie/session state needed for access. A stale or blocked direct PDF URL is not enough
   to mark the source acquisition-limited.
3. If the publisher-page browser path fails, try documented OA alternates already exposed
   by the inventory or an auditable resolver pass (for example repository, PMC, arXiv, or
   other legal open copy), preserving the article identity and version notes.
4. Mark a source block `acquisition_limited` only after these routes fail, or mark it
   `direct_route_limited_browser_recovery_pending` when only the scripted/direct route has
   failed and a browser recovery pass remains.

For analysis, this fallback affects acquisition status only. It does not change the sampled
unit, paper, inclusion probability, or design weight.

### 1.5 What Stratum A is for

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

### 2.4 Second Stratum B criterion: `curated_interest`

Stratum B may admit papers under more than one written criterion (§2). Alongside the
`recognized_landmark` criterion (textbook-cited, prize-associated, or named "classic papers"
lists per field), a second criterion is declared:

- **`curated_interest`** — papers the project lead selects as of interest and worth including,
  with an optional per-paper rationale. This criterion is explicitly subjective and
  outcome-correlated. That is permissible *only* because Stratum B is quarantined: like all of
  Stratum B, these papers carry **no design weights** and are **never pooled** into a Stratum A
  estimate or a population-level base rate (§2.2). They are not a field and never enter the
  Stratum A cell frame (§1.1).

Handling rules:

1. **Record the inclusion basis per paper.** Every Stratum B paper records which criterion
   admitted it (`recognized_landmark` / `curated_interest`; a paper may carry both) in the
   Stratum B manifest `metadata/stratum_b_manifest.csv`, with an optional `criterion_detail`
   rationale and an `added_date`. This keeps the basis auditable and lets analyses subset B by
   criterion.
2. **Protect the calibration subset.** Rubric calibration (§2.1, item 1) depends on *clear,
   recognized* exemplars, so calibration analyses use the `recognized_landmark` subset.
   `curated_interest` papers contribute to face validity and the labeled Stratum A contrast
   (§2.1, items 2-3), not to the calibration claim, unless a paper independently also meets the
   recognized-landmark criterion.
3. **Fix the list before coding.** The curated list and its criterion are recorded before the
   papers are coded (the `added_date` is the audit trail), so selection cannot drift into
   post-hoc cherry-picking that flatters the rubric.
4. **Blinding is inherited.** These papers are `sample_source=landmark`, which is already on the
   blind-packet exclusion list (§5); judges therefore never learn a paper was hand-picked.

The existing Stratum B papers (§2.3) are back-filled into the manifest with basis
`recognized_landmark` (or `curated_interest` where that is the truer description).

---

## 3. Acquisition frame: OA-accessible, not born-OA

A uniform reach across eras is incompatible with a born-OA-journal-only list, because OA
journals are largely a post-2000 phenomenon and several target fields use conferences or
repository-backed journal publication as their normal access route. The pilot's own early
papers were obtained from public archives and PMC full-text XML, not OA journals. Therefore
the venue/source list (§4) and the within-cell draw (§1.2) operate over **OA-accessible**
sources: born-OA journals/proceedings plus venues whose papers are reliably mirrored in a
public archive / PMC / repository / arXiv route for the era in question. Accessibility is
verified at acquisition, as in the current pilot.

---

## 4. The predefined lists are part of the design

The randomization is conditional on two hand-built lists, and **those lists carry the design
decisions** — they are not neutral background:

- **Field list** — the fields to sample, chosen to span the paradigm x statistics-intensity
  gradient (`ingestion_plan` §2). Document the rationale per field.
- **Tiered, field-specific venue/source list** — for each (field, tier), the set of
  OA-accessible journals, proceedings venues, or other declared archival publication sources
  eligible as source blocks. Document the inclusion rule (how a source earns a tier; how
  OA-accessibility was verified), the planned source-block order, and whether that order was
  randomized once or predeclared.

Both lists are versioned and stored with the corpus. Changing a list or source-block order is
a design change and is logged. Stating the lists explicitly is what keeps "random" from
quietly meaning "convenient."

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

### 4.1.1 Field-list revision (v1): add clinical medicine

`v1` adds one field:

| # | Field | Primary role | Rationale (expectation, to be tested by coding) |
|---|-------|--------------|--------------------------------------------------|
| 9 | Clinical medicine / epidemiology / evidence-based medicine | Within-field (+ between) | Places randomized trials, causal inference, treatment-effect estimation, guideline/evidence culture, diagnostic/prognostic modeling, observational epidemiology, clinical ML, biomarker studies, and health-services outcomes inside one institutional field. This gives a high-stakes human-outcomes field with strong within-field paradigm variance: intervention/causal traditions sit beside predictive, associative, and risk-stratification traditions. It is not a second mechanistic-biology field; it earns inclusion by testing whether clinical evidence culture has a distinct paradigm distribution from basic biomedical mechanism papers. |

Clinical medicine is scoped to **human clinical evidence**. Include clinical trials, clinical
observational studies, diagnostic/prognostic studies, epidemiology, public/global health
studies, health-services research, clinical decision studies, and clinical-translational
papers where human data or direct clinical consequences are central. Exclude, by default,
animal-only, in-vitro-only, basic biomedical, and purely molecular-mechanism papers; those
belong to Field 5 unless a predeclared translational-clinical subcell requires both
preclinical and human clinical evidence. Protocols, guidelines, editorials, narrative
reviews, and meta-analyses are excluded from the random-base draw unless a separate
article-type stratum is declared.

The corresponding venue/source revision is stored in `journal_list_v1.md`.

### 4.2 Cautions for the within-field-variance fields (7, 8, 9)

1. **Budget more per-field N.** A field chosen for *within-field* variance is informative
   only if the corpus can estimate the *distribution* of paradigm orientation inside it, not
   just a field mean. Allocate more N to fields 7-9 than to a paradigm-homogeneous field;
   the field mean alone discards the very signal these fields contribute.
2. **Do not conflate weak measurement with inductivist paradigm.** In these fields,
   deceleration can be driven by soft or noisy measurement (low power, unreliable
   constructs) rather than by paradigm orientation as such. These must be kept separate —
   via the CACS/SIDS coding and as an explicit measurement-quality covariate — or the test
   will mistake a measurement problem for a paradigm effect.
3. **Do not conflate clinical consequence with causal abstraction.** In Field 9, a paper can
   be clinically important while remaining predictive, correlational, or operational rather
   than mechanistic. Conversely, a trial can test a sharply causal claim without explaining
   the underlying molecular mechanism. Code the paradigm features from the paper's argument
   structure, not from its medical importance.

These additions do **not** change the phasing in `ingestion_plan` §7: Field 9 should be added
only through a predeclared source-block schedule and balanced against the existing fields, not
used as a convenience source while lower-coverage fields remain underfilled. Physics remains
Phase 1.

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

1. Define and version the field list and the tiered OA-accessible venue/source list (§4).
2. Enumerate cells, where **cell = [field × era × source × tier]** (§1.1). Apply the documented
   coverage target of N ≥ 10 per eligible cell, and check the hard tier-coverage requirement at
   the field level.
3. Walk the predeclared sources within each field × era; each source (at its pinned tier) is a
   cell. Within each active cell, run the archival unit -> paper random draw, deduplicating and
   recording each paper's within-cell inclusion probability / design weight (§1.2-1.3). Tag
   `sample_source=random_base`.
4. Compile the pre-specified landmark list per field from a written criterion; acquire it;
   tag `sample_source=landmark`. Record each paper's `inclusion_basis`
   (`recognized_landmark` / `curated_interest`, §2.4) in the Stratum B manifest
   (`metadata/stratum_b_manifest.csv`). Re-tag the existing 16 papers as `landmark` (§2.3) and
   back-fill them into the manifest.
5. Acquire over OA-accessible sources using the tiered direct-route -> publisher-browser ->
   documented-OA-alternate process in §1.4.1, verifying access as in the current pilot (§3).
6. Build blind packets that exclude `sample_source` and all other outcome/identity-of-design
   fields (§5). Code both strata identically and blindly.
7. Analyze: base-rate and field-differential estimates use **weighted Stratum A only**;
   Stratum B is used for rubric calibration and as a separately labeled contrast (§2).

### 6.1 Operator prompt for adding random-base papers

Use this prompt when asking an assistant or analyst to add the next batch of Stratum A
papers:

> Add the next protocol-compliant random-base batch. Before adding anything, produce a
> preflight table of target cells, where **cell = [field × era × source × tier]**. Prioritize
> roughly even coverage across eligible cells, and verify the hard tier-coverage requirement
> (each field must span at least two tiers, §1.1) at the field level. Do not overfill one cell
> while comparable cells remain underfilled. If even coverage conflicts with eligibility,
> inventory availability, or acquisition feasibility, first try the publisher-page browser
> fallback for direct-route failures; if that still fails, stop and explain the constraint
> before proceeding.

Working target: the documented coverage target is **N ≥ 10** random-base papers per eligible,
scheduled cell `[field × era × source × tier]` (§1.1). It is breadth-first, not a cap: bring
underfilled eligible cells up to 10 before deepening any cell, and let design weights absorb the
residual size differences in estimation. Treat ineligible, historically unavailable, exhausted,
or accessibility-limited cells as documented exceptions rather than targets to force. Separately
from the N target, enforce the field-level tier-coverage requirement as a hard rule.

---

## 7. Schema changes

Add to `papers.csv` (neutral provenance facts only):

- `sample_source` — `random_base` / `landmark`. Records *how the paper entered the corpus*,
  not what it is. It is provenance, not a paradigm or role label, and is excluded from blind
  packets.
- `selection_probability` — the article's inclusion probability under the §1.2 source-block
  draw (`random_base` papers only; blank for `landmark`).
- `design_weight` — inverse-probability weight used in Stratum A estimates
  (`random_base` only; blank for `landmark`).

Add a small **field-level reference table** (`metadata/fields.csv`, not per paper), as
anticipated in `ingestion_plan` §6 and documented in `metadata/schema.md`:
`field`, `ml_penetration_onset_early`, `ml_penetration_onset_late`, `ml_penetration_source`,
`ml_penetration_applicability` (`high` / `low` / `not_applicable`), `notes`. The ML-penetration
fields feed the secondary timing hypothesis (TS1) only; the primary temporal tier is measured
from the era-stratified coded cohort and does not read this table.

Add a **Stratum B manifest** (`metadata/stratum_b_manifest.csv`, not per paper in `papers.csv`)
recording each landmark paper's written `inclusion_basis` (`recognized_landmark` /
`curated_interest`, §2.4) and `added_date`. It is provenance for the quarantined landmark
stratum and carries no design weights; columns are documented in `metadata/schema.md`.

Do **not** add: any paradigm/orientation prior, any `design_role`, or any stratum-as-outcome
column. `openalex_cited_by_count` remains an outcome, never a sampling weight
(`schema.md` §23). Update `schema.md` to document the three new columns and to add
`sample_source` to the blind-packet exclusion list.

---

## 8. Open decisions

- Further field additions and subfield granularity, including Field 9 clinical subcells
  (shared with `ingestion_plan` §8).
- Exact era bands and each field's ML-penetration year.
- The written landmark-inclusion criterion per field, and the target size of Stratum B.
- Whether `random_base` and `landmark` papers share an ID series or are namespaced.
- Pre-specified paper-level field-match rules for broad venues in the §4 list.
- The exact pre-declared tier weight for the secondary cross-tier collapse (§1.3, step 4).

Resolved:
- The sampling unit is the **cell = [field × era × source × tier]** (§1.1), with a documented
  coverage target of N ≥ 10 per eligible cell (breadth-first, not a cap) and a hard field-level
  tier-coverage requirement. All tiers share the same N ≥ 10 cell target.
- The across-cell weighting rule (§1.3): within-cell design weights always applied; tier kept
  explicit (reward-step reported tier-stratified, temporal tier grouped by field × era and
  collapsed only via a declared tier weight); **equal-source weighting** within a
  (field, era, tier) group; source-size weighting demoted to a reported sensitivity arm.
