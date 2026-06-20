# Candidate Journal List (v0)

Status: working draft. This instantiates the "tiered, field-specific, OA-accessible journal
list" required by `selection_protocol.md` §4, for the eight-field list in §4.1. Every entry
here is a **candidate to be verified per `selection_protocol.md` §4** (tier assignment and
OA-accessibility confirmed per journal and per era before use). OA modes below are best
current understanding, not settled facts.

---

## 1. How a journal is described (three axes)

- **Scope** — *general* (eligible for every field's draw) vs *field-specific* (eligible only
  for its field's cells).
- **Tier** — `top` / `mid` / `specialist`.
- **OA accessibility route** — how OA-accessible articles are obtained:
  - **Gold** — immediate free access to all articles (born-OA).
  - **Green / delayed** — free after an embargo and/or reliably mirrored in PMC/repository.
  - **Hybrid / archive** — subscription journal in which *some* articles are OA-accessible
    (author-paid gold-in-hybrid, funder-mandated green, or public archive / arXiv copies).

---

## 2. Eligibility and the OA-subset restriction

**Hybrid journals are eligible.** Eligibility does not require born-OA. Any journal that has
an OA-accessible subset of articles for the target era is in-frame.

**The draw is restricted to the OA-accessible subset.** Within a chosen journal and issue,
the §1.2 article draw samples only from articles whose full text is OA-accessible.

### 2.1 Consequence: this is sampling on a non-random subset (record and weight it)

Article-level OA status inside a hybrid journal is **not random**. It is correlated with:

- **Funding** — gold-in-hybrid requires an APC, so it tracks well-funded labs/fields; green
  OA tracks funder mandates (NIH Public Access, Wellcome, Plan S).
- **Field and sub-field** — OA uptake differs sharply across disciplines.
- **Era** — OA availability rises steeply post-~2008 and is sparse before.
- Possibly **prominence** — higher-profile work is more often made open.

So within a hybrid venue the realized sample is the **OA stratum of the issue, not the
issue**. This does not disqualify hybrids, but it means:

1. **Record the OA route per paper** (`gold` / `green` / `archive`) alongside
   `sample_source`. It is a neutral provenance fact, not a paradigm label.
2. **Fold OA-subset coverage into the inclusion probability** of §1.3, so the
   `design_weight` reflects "probability the article was both drawn *and* OA-accessible."
   Otherwise OA-correlated funding/era leaks in as a confound.
3. **Watch the gold/green era interaction.** Because OA coverage is thin pre-2008, hybrid
   venues contribute mostly recent papers; early-era cells should lean on green/delayed and
   public-archive routes (as the pilot already did for 1950s-1990s papers).

---

## 3. General journals (eligible for any field)

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

### 3.1 General-journal field filter (decision needed)

A random article from a general journal is not in any particular field. Two options:

- **(a) Opportunistic cell-filling (recommended).** Draw a random OA-accessible article, then
  assign it post-hoc to whatever (field, era, tier) cell it belongs to. Less judgment at
  selection; the field is a recorded fact, not a pre-filter.
- **(b) Pre-filtered draw.** Restrict the draw to articles already tagged to the target
  field. Requires a field-tagging step that injects classifier judgment before sampling.

---

## 4. Non-journal venues (ML and frontier physics need an analog)

The journal -> issue -> article unit does not exist for the discovery venues of fields 2-4.
Define an analog sampling unit, all openly accessible:

- **ML / AI (field 2)** — conferences: **NeurIPS** (proceedings.neurips.cc), **ICML** (PMLR),
  **ICLR** (OpenReview); journals **JMLR** and **TMLR** (gold). Unit: *conference-year x
  track* (or arXiv `cs.LG`/`stat.ML` monthly listing). Inclusion still random within unit.
- **HEP / GW physics (field 3)** — **arXiv** `hep-ex`, `gr-qc` monthly listings; most HEP
  journals are gold OA via **SCOAP3** (Phys. Lett. B, JHEP, Eur. Phys. J. C). Unit: *arXiv
  category-month*, or journal issue where SCOAP3 applies.
- **Statistically-oriented physics (field 4)** — **arXiv** `astro-ph.CO`; AAS journals (ApJ,
  ApJS — moved to OA) and JCAP. Unit: arXiv category-month or journal issue.

Treat each analog unit exactly like an issue: enumerate, draw a unit at random, draw an
OA-accessible item at random, record inclusion probability. **See §6 D2 for the resolved
form of this rule** (venue-based frame; arXiv as access route, not sampling frame).

---

## 5. Field-specific candidate journals

All `top`-tier general journals above are *also* eligible for each field; the lists below add
field-specific venues. Verify tier/OA per entry.

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
| arXiv cs.LG / stat.ML | — | gold (preprint) |

### Field 3 — Structure-driven experimental physics (HEP, neutrino, GW)
| Journal | Tier | OA route |
|---------|------|----------|
| Physical Review Letters | top | hybrid/archive (arXiv) |
| Physical Review D | mid | hybrid/archive (arXiv) |
| Physics Letters B | mid | gold (SCOAP3) |
| JHEP | mid | gold (SCOAP3) |
| Eur. Phys. J. C | specialist | gold (SCOAP3) |
| arXiv hep-ex / gr-qc | — | gold (preprint) |

### Field 4 — Statistically-oriented physics (precision cosmology, exclusion-limit, ML-for-physics)
| Journal | Tier | OA route |
|---------|------|----------|
| Astrophysical Journal (ApJ) | top | gold/green (AAS OA move — verify) |
| Astronomy & Astrophysics | top | hybrid/archive |
| MNRAS | mid | hybrid/archive |
| JCAP | specialist | hybrid/gold |
| Physical Review D | mid | hybrid/archive (arXiv) |
| arXiv astro-ph.CO | — | gold (preprint) |

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
| Cognition | mid | hybrid/archive |
| Psychological Review | top | hybrid/archive (theory venue) |
| Cognitive Science | mid | hybrid |
| Collabra: Psychology | specialist | gold |
| Computational Brain & Behavior | specialist | hybrid |
| Behavior Research Methods | specialist | hybrid |

---

## 6. Resolved decisions (v0)

Working defaults for the five items previously open. Each is reversible (logged as a design
change), but these are the rules the list is built on.

### D1 — General-journal field filter → opportunistic, with a low-discretion field rule

General journals are sampled **opportunistically** (option (a)): draw a random OA-accessible
article, assign it post-hoc to whatever (field, era, tier) cell it belongs to, discard if it
falls outside the eight-field frame. They **supplement** field-specific journals, which
remain the primary cell-fillers; do **not** rely on a general venue to hit a *targeted* cell
(too many of its articles are out of frame). Field is assigned from the **venue's own subject
taxonomy** where one exists (Nature subject terms, journal section, arXiv primary category),
else by **documented two-coder topic coding**. Assignment is on neutral topic only and is
**blind to paradigm/outcome**. Rationale: keeps selection judgment-light, makes `field` a
recorded fact rather than a pre-filter, and avoids the inefficiency of field-tagging an
entire issue before drawing.

### D2 — Sampling frame for ML/physics → venue-based; arXiv is a route, not a frame

*(Supersedes the option list in §4.)* The sampling frame is **published venues**, to stay
commensurable with the journal-based fields. ML unit = **conference-year of accepted
proceedings** (NeurIPS / ICML / ICLR), plus JMLR/TMLR issues; subdivide by track only if a
cell needs it. Physics fields = **journal issues** (PRL, PRD, JHEP, ApJ, JCAP, …). **arXiv is
used only** to (i) obtain the OA full text and (ii) identify the version — it is never the
sampling frame, because it mixes unrefereed preprints and never-published notes, a different
population from "published contributions a venue rewarded." **Dedup:** the unit of record is
the **version of record / first archival venue**; the arXiv ID is stored as the access route;
the §1.2 canonical-DOI/ID dedup prevents entering one contribution twice (e.g., a conference
paper later extended to a journal — keep the first archival venue).

### D3 — Verification → per-(journal, era) eligibility, built in phase order

Eligibility is established **per era band**, because OA route, tier, and even a journal's
existence change over time. A journal enters an era's eligible set only if it (i) published in
that era and (ii) has a **verified OA-accessible route** for that era's content. Record, per
(journal, era): OA route, a **verification source** (OA-policy page, PMC / DOAJ / SCOAP3
listing), tier, and **check-date** — mirroring `papers.csv`'s `oa_status` / `source_checked_date`
discipline. Sequencing: verify **Phase-1 fields first** (structure-driven physics); do not
block the whole list on full eight-field verification.

### D4 — OA-subset handling → weight modern eras, archive-enrich early eras, with a diagnostic

Not either/or; the two tactics cover different eras. (i) **Modern eras (~2008+):** record OA
route per paper and fold OA-accessibility into the §1.3 inclusion probability / `design_weight`.
(ii) **Early eras:** enrich via **green/archive routes** (PMC, public archives), as the pilot
already did for 1950s-1990s papers; acknowledge these cells are sampled from the
archive-available subset. (iii) Compute a per-(journal, era) **OA-coverage fraction** (share
of an issue that is OA-accessible); if coverage is very low (placeholder **< 20%**), flag the
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
