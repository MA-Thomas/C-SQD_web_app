# Candidate Venue and Source List (v0)

Status: working draft. This instantiates the "tiered, field-specific, OA-accessible
venue/source list" required by `selection_protocol.md` §4, for the eight-field list in §4.1. Every entry
here is a **candidate to be verified per `selection_protocol.md` §4** (tier assignment and
OA-accessibility confirmed per venue/source and per era before use). OA modes below are best
current understanding, not settled facts.

For the `v1` clinical medicine / epidemiology / evidence-based medicine revision, see
`journal_list_v1.md`. This `v0` file remains the eight-field baseline.

---

## 1. How a venue/source is described (three axes)

- **Scope** — *general* (eligible as a source block for every field) vs *field-specific*
  (eligible only for its field's cells).
- **Tier** — `top` / `mid` / `specialist`.
- **OA accessibility route** — how OA-accessible papers are obtained:
  - **Gold** — immediate free access to all articles (born-OA).
  - **Green / delayed** — free after an embargo and/or reliably mirrored in PMC/repository.
  - **Hybrid / archive** — subscription venue in which *some* papers are OA-accessible
    (author-paid gold-in-hybrid, funder-mandated green, or public archive / arXiv copies).

---

## 2. Eligibility and the OA-subset restriction

**Hybrid venues are eligible.** Eligibility does not require born-OA. Any venue/source that has
an OA-accessible subset of papers for the target era is in-frame.

**The draw is restricted to the OA-accessible subset.** Within a chosen venue/source and
archival unit, the §1.2 paper draw samples only from papers whose full text is OA-accessible.

### 2.1 Consequence: this is sampling on a non-random subset (record and weight it)

Article-level OA status inside a hybrid venue is **not random**. It is correlated with:

- **Funding** — gold-in-hybrid requires an APC, so it tracks well-funded labs/fields; green
  OA tracks funder mandates (NIH Public Access, Wellcome, Plan S).
- **Field and sub-field** — OA uptake differs sharply across disciplines.
- **Era** — OA availability rises steeply post-~2008 and is sparse before.
- Possibly **prominence** — higher-profile work is more often made open.

So within a hybrid venue the realized sample is the **OA stratum of the archival unit, not
the full unit**. This does not disqualify hybrids, but it means:

1. **Record the OA route per paper** (`gold` / `green` / `archive`) alongside
   `sample_source`. It is a neutral provenance fact, not a paradigm label.
2. **Fold OA-subset coverage into the inclusion probability** of §1.3, so the
   `design_weight` reflects "probability the paper was both drawn *and* OA-accessible."
   Otherwise OA-correlated funding/era leaks in as a confound.
3. **Watch the gold/green era interaction.** Because OA coverage is thin pre-2008, hybrid
   venues contribute mostly recent papers; early-era cells should lean on green/delayed and
   public-archive routes (as the pilot already did for 1950s-1990s papers).

---

## 3. General venues (eligible for any field)

| Journal | Tier | OA route | Note |
|---------|------|----------|------|
| Nature | top | hybrid/archive | Field of an article is set by the article, not the venue — apply the §4 field filter. |
| Science | top | hybrid/archive | Same. |
| PNAS | top | green/delayed (free after 6 mo; PMC) + gold option | Reliable OA-accessible coverage. |
| Nature Communications | top/mid | gold | Multidisciplinary born-OA. |
| Science Advances | mid | gold | Multidisciplinary born-OA. |
| PNAS Nexus | mid | gold | Multidisciplinary born-OA. |
| PLOS ONE | mid | gold | Multidisciplinary born-OA; high volume. |
| Royal Society Open Science | mid | gold | Multidisciplinary born-OA. |
| Scientific Reports | mid | gold | Multidisciplinary born-OA; high volume. |

### 3.1 General-venue field filter (resolved)

A random article from a general venue is not automatically in the target field. General
venues are therefore **field-filtered before the final paper draw**. The target field is
chosen as part of the `field × era × source × tier` cell (`selection_protocol.md` §1.1); after
a general-venue source block is active and an archival unit is drawn, filter that unit to
OA-accessible research papers matching the target field by pre-specified neutral article-level
facts. If the unit has zero matching papers, reject the
unit and redraw a unit from the same venue/source and era frame. This supersedes the earlier
opportunistic-cell-filling option.

---

## 4. Source/unit rules by field

The general draw inside an active source block is `archival unit -> paper`. The archival unit
depends on field practice. arXiv, bioRxiv, ChemRxiv, PsyArXiv, OSF and similar repositories
are normally **OA routes and version records**, not sampling frames, unless a separate
preprint stratum is explicitly declared.

1. **Computational biology / genomics-ML** — sample published journals and proceedings-like
   journals. Unit: journal issue or official article batch. PMC, bioRxiv and institutional
   repositories are OA routes and metadata aids.
2. **ML / AI methods proper** — sample accepted archival venues. Unit: conference-year for
   NeurIPS, ICML and ICLR main proceedings; journal issue/article batch for JMLR, TMLR, AIJ
   and TPAMI. Track subdivision is used only if pre-specified for a cell. arXiv `cs.LG` /
   `stat.ML` is an OA route and version record by default.
3. **Structure-driven experimental physics (HEP, neutrino, gravitational waves)** — sample
   published journal venues. Unit: journal issue for PRL, PRD, Physics Letters B, JHEP,
   EPJ C, etc. arXiv `hep-ex`, `hep-ph`, `gr-qc` and experiment publication lists may verify
   access and field eligibility, but they do not replace the venue frame unless a separate
   experiment-publication-list frame is declared before sampling.
4. **Statistically-oriented physics** — sample published journals for precision cosmology,
   exclusion-limit and ML-for-physics work. Unit: journal issue for ApJ/ApJS, A&A, MNRAS,
   JCAP, PRD, PRL, etc. arXiv categories such as `astro-ph.CO`, `hep-ph` and `hep-ex` are
   OA routes and field signals, not the default frame.
5. **Mechanistic molecular / cell / developmental biology** — sample journals. Unit:
   journal issue or official article batch. PMC and bioRxiv are OA routes and metadata aids.
6. **Structure-driven condensed matter / chemistry** — sample journals. Unit: journal issue
   or official article batch. arXiv and ChemRxiv are OA routes by default.
7. **Computational neuroscience** — sample journals, plus conference proceedings only if a
   computational-methods subcell pre-specifies them. Unit: journal issue or conference-year.
8. **Cognitive science / psychology** — sample journals. Unit: journal issue or official
   article batch. PsyArXiv and OSF are OA routes by default.

Treat each unit like an issue in the probability model: enumerate the active venue/source
inventory, draw a unit at random, draw an OA-accessible eligible paper at random, and record
the source-block inclusion probability.

---

## 5. Field-specific candidate venues/sources

All `top`-tier general journals above are *also* eligible for each field; the lists below add
field-specific venues. Verify tier/OA per entry. Repository services such as arXiv are listed
as OA routes in notes, not as default sampling venues.

### Field 1 — Computational biology / genomics-ML
| Journal | Tier | OA route |
|---------|------|----------|
| Nature Methods | top | hybrid/archive |
| Nature Genetics | top | hybrid/archive |
| Nature Biotechnology | top | hybrid/archive |
| Genome Biology | mid | gold |
| Genome Research | mid | green/delayed |
| Nucleic Acids Research | mid | gold |
| PLOS Computational Biology | specialist | gold |
| Bioinformatics | specialist | hybrid |
| BMC Bioinformatics | specialist | gold |
| Cell Systems | specialist | hybrid |

### Field 2 — ML / AI methods proper  (see §4 for units)
| Venue | Tier | OA route |
|-------|------|----------|
| NeurIPS / ICML / ICLR proceedings | top | gold (open proceedings) |
| JMLR | top | gold |
| Nature Machine Intelligence | top | hybrid/archive |
| Artificial Intelligence (AIJ) | top | hybrid/archive |
| TMLR | mid | gold |
| IEEE TPAMI | top | hybrid/archive (arXiv) |

### Field 3 — Structure-driven experimental physics (HEP, neutrino, GW)
| Journal | Tier | OA route |
|---------|------|----------|
| Physical Review Letters | top | hybrid/archive (arXiv) |
| Physical Review D | mid | hybrid/archive (arXiv) |
| Physics Letters B | mid | gold (SCOAP3) |
| JHEP | mid | gold (SCOAP3) |
| Eur. Phys. J. C | specialist | gold (SCOAP3) |

### Field 4 — Statistically-oriented physics (precision cosmology, exclusion-limit, ML-for-physics)
| Journal | Tier | OA route |
|---------|------|----------|
| Astrophysical Journal (ApJ) | top | gold/green (AAS OA move — verify) |
| Astronomy & Astrophysics | top | hybrid/archive |
| MNRAS | mid | hybrid/archive |
| JCAP | specialist | hybrid/gold |
| Physical Review D | mid | hybrid/archive (arXiv) |

### Field 5 — Mechanistic molecular / cell / developmental biology
| Journal | Tier | OA route |
|---------|------|----------|
| Cell | top | hybrid/archive (PMC) |
| Nature Cell Biology | top | hybrid/archive |
| eLife | top | gold (reviewed preprint) |
| PLOS Biology | mid | gold |
| EMBO Journal | mid | hybrid/green |
| Journal of Cell Biology | mid | green/gold |
| Development | mid | hybrid/green |
| Genes & Development | mid | green/delayed |

### Field 6 — Structure-driven condensed matter / chemistry
| Journal | Tier | OA route |
|---------|------|----------|
| Physical Review X | top | gold |
| Physical Review B | mid | hybrid/archive (arXiv) |
| Nature Materials | top | hybrid/archive |
| JACS | top | hybrid/archive |
| Chemical Science | mid | gold (RSC) |
| IUCrJ | specialist | gold |
| npj Computational Materials | specialist | gold |

### Field 7 — Computational neuroscience
| Journal | Tier | OA route |
|---------|------|----------|
| Nature Neuroscience | top | hybrid/archive |
| Neuron | top | hybrid/archive |
| PLOS Computational Biology | specialist | gold |
| Journal of Neuroscience | mid | green/delayed |
| eNeuro | specialist | gold (SfN) |
| Neural Computation | specialist | hybrid |
| Journal of Computational Neuroscience | specialist | hybrid |
| Frontiers in Computational Neuroscience | specialist | gold |

### Field 8 — Cognitive science / psychology
| Journal | Tier | OA route |
|---------|------|----------|
| Nature Human Behaviour | top | hybrid/archive |
| Psychological Science | top | hybrid/green |
| Cognitive Research: Principles and Implications | mid | gold (SpringerOpen) |
| Psychological Review | top | hybrid/archive (theory venue) |
| Cognitive Science | mid | hybrid |
| Collabra: Psychology | specialist | gold |
| Computational Brain & Behavior | specialist | hybrid |
| Behavior Research Methods | specialist | hybrid |

> **Design-change log (2026-06-22, §4):** *Cognition* (Elsevier; was listed `mid | hybrid/archive`)
> is **removed** as the cog-sci mid source and **replaced by *Cognitive Research: Principles and
> Implications*** (SpringerOpen, gold OA). Rationale: Cognition proved unacquirable through every
> §1.4.1 route in the browser-acquisition environment — ScienceDirect serves an anti-bot
> (`crasolve`) challenge, the green PMC author-manuscript copies return tokenized-redirect stubs,
> Europe PMC returned HTTP 500, PhilPapers/PhilArchive host no copies, and only ~10 articles have
> an arXiv copy (too thin for a valid draw). Chasing the scattered open copies would select on
> accessibility and bias the random-base frame, so per §4 the venue was substituted with a
> fully-gold-OA cog-sci venue (every article OA → no accessibility bias). Recorded in
> `round_20260622_p1_crpi_cogsci_mid_2015_present_SUBSTITUTE`; the prior Cognition attempt is
> retained as a documented `acquisition_limited` exception in
> `round_20260621_p1_cognition_cogsci_mid_2015_present`.

---

## 6. Resolved decisions (v0)

Working defaults for the five items previously open. Each is reversible (logged as a design
change), but these are the rules the list is built on.

### D1 — General-venue field filter → pre-filtered within the drawn unit

General venues are sampled only inside a chosen target cell and active source block. Draw the
archival unit, then filter that unit to OA-accessible research papers matching the target
field. If the unit has zero matching papers, reject and redraw a unit from the same
venue/source and era frame. Field is assigned from neutral article-level facts only: the
venue's own subject taxonomy where one exists (Nature subject terms, journal/conference
section, arXiv primary category as a field signal), title/abstract keywords, MeSH/OpenAlex
concepts, or documented two-coder topic coding. Assignment is **blind to paradigm/outcome**.
Rationale: preserves deliberate field coverage while keeping field assignment paper-based,
not venue-based.

### D1b — Source-block sequencing → stepwise through the eligible source list

The eligible venue/source list is now implemented as **source blocks** rather than as a
per-paper random source draw. Within each active source block, sampling remains randomized at
the archival-unit and paper levels. Source-block order must be predeclared or randomized once
and saved with the audit. A partially completed field therefore represents the completed
source blocks only; it must not be described as a full field-level sample over all eligible
sources until the declared source-block frame has been completed or a source-level weighting
rule is applied.

### D2 — Sampling frame for ML/physics → venue/source-based; arXiv is a route, not a frame

*(Supersedes the older arXiv-category option list in §4.)* The default sampling frame is
**accepted/published archival venues**, to stay commensurable across fields. ML unit =
**conference-year of accepted main proceedings** (NeurIPS / ICML / ICLR), plus JMLR/TMLR
issues or article batches; subdivide by track only if a cell pre-specifies it. Physics
fields = **journal issues** (PRL, PRD, JHEP, ApJ, JCAP, etc.). **arXiv is used only** to
(i) obtain the OA full text, (ii) identify the version, and (iii) provide a neutral field
signal; it is not the default sampling frame because it mixes unrefereed preprints,
never-published notes and later-published papers. **Dedup:** the unit of record is the
version of record / first archival venue; the arXiv ID is stored as the access route; the
§1.2 canonical-DOI/ID dedup prevents entering one contribution twice (e.g., a conference
paper later extended to a journal — keep the first archival venue unless a different
first-venue rule is pre-specified).

### D3 — Verification → per-(venue/source, era) eligibility, built in phase order

Eligibility is established **per era band**, because OA route, tier, and even a venue's
existence change over time. A venue/source enters an era's eligible set only if it (i)
published accepted/published papers in that era and (ii) has a **verified OA-accessible
route** for that era's content. Record, per (venue/source, era): OA route, verification
source (OA-policy page, PMC / DOAJ / SCOAP3 listing, proceedings index, repository policy),
tier, and check-date — mirroring `papers.csv`'s `oa_status` / `source_checked_date`
discipline. Sequencing: verify **Phase-1 fields first** (structure-driven physics); do not
block the whole list on full eight-field verification.

### D4 — OA-subset handling → weight modern eras, archive-enrich early eras, with a diagnostic

Not either/or; the two tactics cover different eras. (i) **Modern eras (~2008+):** record OA
route per paper and fold OA-accessibility into the §1.3 inclusion probability / `design_weight`.
(ii) **Early eras:** enrich via **green/archive routes** (PMC, public archives), as the pilot
already did for 1950s-1990s papers; acknowledge these cells are sampled from the
archive-available subset. (iii) Compute a per-(venue/source, era) **OA-coverage fraction**
(share of an archival unit that is OA-accessible); if coverage is very low (placeholder
**< 20%**), flag the
cell *accessibility-limited* and prefer a gold/green venue for it. Implies one recorded field:
`oa_route` ∈ {gold, green, archive} per paper.

### D5 — Tier-assignment → venue-level, within-field, pre-specified; never the paper's outcome

Tier proxies prestige using **venue-level indicators fixed in advance**, assigned **within
field** (a top condensed-matter journal and a top psychology journal are each `top` relative
to their own field — absolute cross-field metrics would conflate field size with prestige).

- `top` — the field's flagship / most selective venues (incl. the general elite Nature /
  Science / PNAS and the field's premier specialty venue or conference).
- `mid` — established, selective field journals below the flagship.
- `specialist` — narrower-scope but reputable subfield / methods venues.

Operationalize with a **named venue-level source recorded per entry**, allowing
**era-specific tier** where standing shifted materially.

**Named tier sources (by venue type, chosen for methodological alignment with the
causal-Popperian thesis):**

- **Journal venues → SJR (Scimago Journal Rank).** Preferred over the Journal Impact Factor
  because SJR replaces enumerative citation-counting with a *structural* prestige-weighted
  eigenvector model (PageRank-style), with field normalization and self-citation correction,
  computed by a published, reproducible algorithm. The Journal Impact Factor (Clarivate/JCR)
  is deliberately **not** used: a 2-year mean-citation tally is induction-by-enumeration —
  the very inductivist move the corpus exists to study — and its use as a quality proxy is
  explicitly cautioned against by DORA (2012) and the Leiden Manifesto (Hicks et al., *Nature*
  2015).
- **Conference venues (the ML/AI conferences in field 2) → CORE.** Its tiers are set against
  public criteria with a community appeals process — institutionalized conjecture-and-revision
  — which is the most Popperian instrument available for venues that lack issue/citation
  structure. Journal venues *within* field 2 (JMLR, AIJ, TMLR, IEEE TPAMI) still use SJR.

**Circularity guard:** all citation-derived tiering remains partly inductive (it rewards
accumulated attention); this is acceptable only because tier is a **design covariate we
deliberately represent**, and venue metrics are **never** used to score an individual paper's
uptake (`openalex_cited_by_count` stays an outcome, per `schema.md`).

---

## 7. Still to operationalize

- The exact low-coverage threshold for the D4 `accessibility-limited` flag (20% placeholder).
- The SJR quartile → tier cut points (e.g. Q1 = `top`/`mid` split rule) and the CORE
  rank → tier mapping (A*/A → `top`, B → `mid`, C → `specialist`?). Source choice itself is
  resolved in D5 (SJR for journals, CORE for ML conferences).
- Whether conference *track* subdivides the D2 unit for any field.
- Adding `oa_route` (and a field-assignment-source note) to `schema.md` when `random_base`
  ingestion begins.

### References (tier-source rationale)

- DORA — San Francisco Declaration on Research Assessment (2012): https://sfdora.org/read/
- Hicks, Wouters, Waltman, de Rijcke & Rafols, "Bibliometrics: The Leiden Manifesto for
  research metrics," *Nature* 520(7548):429-431 (2015): https://www.nature.com/articles/520429a
