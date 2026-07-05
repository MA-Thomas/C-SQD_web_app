# Candidate Venue and Source List (v1)

Status: working draft. This is a versioned revision to `journal_list_v0.md`. It inherits all
v0 source rules and venue lists unless explicitly superseded here, and adds Field 9 from
`selection_protocol.md` §4.1.1.

Revision date: 2026-06-21.

---

## 1. Change from v0

`v1` adds:

- **Field 9 — Clinical medicine / epidemiology / evidence-based medicine**
- A field-specific top-tier clinical/translational venue block including `NEJM`,
  `The Lancet`, `JAMA`, `The BMJ`, and `Nature Medicine`
- OA-heavy broad clinical source blocks for robust acquisition
- A medicine-specific field boundary separating clinical evidence from basic biomedical
  mechanism papers

All v0 rules still apply: source blocks are journals or official article batches; PMC,
PubMed, Crossref, DOI resolvers, institutional repositories, and publisher search pages are
metadata/access routes, not replacement sampling frames unless explicitly declared as a
separate source stratum.

---

## 2. Field 9 source/unit rule

**Clinical medicine / epidemiology / evidence-based medicine** samples published clinical
research venues. Unit: journal issue or official article batch, depending on the venue's
archive structure. Continuous-publication journals may use an official month/year article
batch when no issue unit exists.

Eligible article types by default:

- Original clinical research articles
- Randomized and non-randomized intervention studies
- Observational epidemiology and cohort/case-control studies
- Diagnostic, prognostic, and risk-prediction studies
- Biomarker studies with central human clinical evidence
- Health-services, public-health, implementation, and clinical-decision studies
- Clinical-translational studies where human data or direct clinical outcomes are central

Excluded by default:

- Animal-only, in-vitro-only, or purely molecular-mechanism papers
- Basic biomedical papers without central human clinical evidence
- Protocols, trial designs without outcome data, guidelines, editorials, narrative reviews,
  correspondence, news, and case reports
- Meta-analyses and systematic reviews, unless a separate evidence-synthesis stratum is
  declared before sampling

Boundary rule: when a paper mixes preclinical and human evidence, assign it to Field 9 only
if the human clinical evidence is central to the paper's claim. Otherwise it belongs to
Field 5 or another better-matching field.

---

## 3. Medicine acquisition rule

Clinical journals often expose several legal access routes for the same article. Record the
route actually used per paper:

- `gold` — publisher-labeled open access article, usually with a Creative Commons license
- `green` — author accepted manuscript or funder-mandated public-access copy, e.g. PMC
- `delayed_free` — publisher version made free after an embargo
- `hybrid` — article-level OA inside a subscription venue
- `archive` — legal institutional, society, or public archive copy

For a drawn paper, acquisition must follow the protocol in `selection_protocol.md` §1.4.1:

1. Try the recorded or deterministic direct route from the frozen inventory.
2. If that fails, open the article landing page on the publisher site in a browser and use
   the visible PDF, download, or full-text controls as a reader would.
3. If the publisher-browser route fails, try documented OA alternates such as PMC,
   institutional repository copies, or funder-mandated manuscripts.
4. Mark a source block acquisition-limited only after these routes fail, and record which
   route failed.

PMC is especially useful in medicine, but it is not automatically a sampling frame. The
sampled identity remains the journal article from the drawn venue/source unit.

---

## 4. Field-specific candidate venues/sources

### Field 9 — Clinical medicine / epidemiology / evidence-based medicine

| Venue | Tier | OA route | Note |
|-------|------|----------|------|
| New England Journal of Medicine (NEJM) | top | delayed_free / green / hybrid | Flagship general clinical venue; use publisher-browser route first, then documented public-access routes. |
| The Lancet | top | hybrid / green / archive | Flagship general clinical venue; article-level OA/free status varies by article and era. |
| JAMA | top | delayed_free / green / hybrid | Flagship general clinical venue; article-type and access status must be verified per article. |
| The BMJ | top | gold/research-free / archive | Flagship evidence-based medicine venue; restrict to original research unless a separate article-type stratum is declared. |
| Nature Medicine | top | hybrid / gold | High-prestige translational and clinical medicine venue; apply the clinical-evidence boundary because it also publishes biomedical mechanism work. |
| JAMA Network Open | top/mid | gold | Broad open-access clinical and health research venue; strong acquisition route for modern-era medicine. |
| PLOS Medicine | top/mid | gold | Broad OA clinical/public-health venue. |
| BMC Medicine | top/mid | gold | Broad OA medicine venue. |
| BMJ Open | mid | gold | Broad OA medical research; exclude protocols by default. |
| eClinicalMedicine | mid | gold | Lancet-family OA clinical medicine; useful for global clinical/outcomes work. |
| The Lancet Global Health | mid | gold | OA global-health clinical/public-health source; treat as a global-health subcell if heavily used. |
| The Lancet Digital Health | mid | gold | OA digital-health source; useful for clinical AI/prediction but should not dominate Field 9. |
| Communications Medicine | mid | gold | Nature Portfolio OA medicine venue. |
| Trials | specialist | gold | Specialist trial-methods and intervention venue; exclude protocols unless a protocol stratum is declared. |
| Diagnostic and Prognostic Research | specialist | gold | Specialist diagnostic/prognostic/prediction venue. |
| Implementation Science | specialist | gold | Specialist implementation and health-services intervention venue. |
| Journal of Clinical Epidemiology | specialist | hybrid/green | Methods and evidence-evaluation venue; verify OA route per era. |
| Medical Decision Making | specialist | hybrid/green | Decision-analysis and clinical decision science; verify OA route per era. |
| Clinical Epidemiology | specialist | gold | Epidemiology and outcomes venue; verify scope and standing per era. |

---

## 5. Source-block sequencing guidance for Field 9

Recommended initial sequence:

1. Start with OA-heavy broad clinical venues (`JAMA Network Open`, `PLOS Medicine`,
   `BMC Medicine`, `BMJ Open`, `Communications Medicine`) to establish reliable modern-era
   coverage without avoidable access failures.
2. Add specialist contrast blocks (`Trials`, `Diagnostic and Prognostic Research`,
   `Implementation Science`, `Journal of Clinical Epidemiology`, `Medical Decision Making`)
   to force within-field paradigm variance rather than letting general medicine dominate.
3. Add top clinical/translational venues (`NEJM`, `The Lancet`, `JAMA`, `The BMJ`,
   `Nature Medicine`) through the publisher-browser-first acquisition rule above. These
   venues are high-value but have article-level access variation, so each draw must record
   `oa_route`, browser outcome, and any legal alternate used.

Do not let clinical medicine become a convenience source for filling corpus size. Field 9
enters the design only through a predeclared source-block schedule with balanced targets
against the existing fields.

