# Preflight — next random-base batch (2026-06-21)

Status: **plan for review. No papers have been acquired.** This is the §6.1 preflight gate that
must be produced and approved before any acquisition. Companion to
`preflight_next_batch_20260621.csv`.

> **P1 COMPLETE (2026-06-22).** All 14 P1 tier-coverage cells are filled: **140 random-base
> papers added (P0519–P0658)**, all via the §1.4.1 browser route over frozen OpenAlex/OpenReview
> inventories with recorded inclusion probabilities and design weights. **All five originally
> single-tier fields now span ≥2 tiers** (comp bio, comp neuro, structure physics, ML, cog-sci).
> One substitution: the cog-sci mid source **Cognition was acquisition_limited** (ScienceDirect
> anti-bot wall; PMC author-manuscript redirect stubs) and was **replaced by *Cognitive Research:
> Principles and Implications*** (SpringerOpen, gold OA) — logged in `journal_list_v0.md` §4 and
> round `round_20260622_p1_crpi_cogsci_mid_2015_present_SUBSTITUTE`. Per-row outcomes are in the
> `status_20260622` column of the companion CSV; per-cell audits in
> `sources/draw_audits/round_2026062*_p1_*`. **P2–P4 remain pending.**

> **P2 COMPLETE (2026-06-24).** All 7 P2 recovery cells executed; **52 papers added (P0659–P0710,
> P0711–P0730)** via the §1.4.1 browser route over frozen OpenAlex/PMC frames (downloads landed in
> `pdfs/_incoming/` via the browser; frames frozen and drawn by `build_cell.py`; verified +
> filed by `finalize_cell.py`). Per-cell:
> **Collabra +5** (P0659–P0663, cell→10), **npj Comp Materials +5** (P0664–P0668, cell→10),
> **eLife** (P0669–P0678, 10) — *rebuilt the contaminated 2005–2014 slice*: era re-keyed on
> **volume (1–3 = 2012–2014)** to correct OpenAlex `publication_year` mis-dating, spurious issue
> normalized; **NEJM** (P0711–P0720, 10), **Lancet** (P0721–P0730, 10), **BMJ** (P0699–P0708, 10)
> all top-tier clinical via OA routes (NEJM/Lancet PMC; BMJ bmj.com). **JAMA is
> `acquisition_limited` (P0709–P0710, 2 of 10)**: PMC deposits ~90% XML-only, the PMC-PDF subset is
> dominated by corrections/comments whose PMC link resolves to a *different* article, and publisher
> PDFs are HTML-gated via Silverchair (documented exception per §1.4.1 step 4 and the JAMA
> exception_note). The four hybrid clinical journals sample the **OA-accessible subset**, which
> skews NIH-funded/COVID-era — recorded as a documented limitation in each inventory. Clinical
> cells were full-text vetted to research-only (publisher non-research banners/section labels
> defeated the automated eligibility heuristic; NEJM `round_..._nejm` and Lancet `round_..._lancet`
> were reverted and re-finalized after the fix; `finalize_cell.py` eligibility hardened for
> "correspondence to"/"see Comment" bylines and Elsevier banners). Per-cell audits in
> `sources/draw_audits/round_20260623_p2_*`. **P3–P4 remain pending.**
>
> **Before starting P3/P4, read `sources/acquisition_lessons_p2_for_p3_p4.md`** — failure→fix
> notes from the P2 run (Chrome download allow-listing and CDN origins, PMC XML-only vs PDF and
> OpenAlex PMC mislinkage, full-text research-vetting vs the finalizer heuristic, revert-before-
> rebuild, and OpenAlex `publication_year` mis-dating / volume-based era keys).

> **P3 COMPLETE (2026-06-24).** Of 9 early-era cells: **6 finalized, 60 papers added (P0731–P0790)**,
> all via the §1.4.1 browser route over frozen OpenAlex frames; **2 documented `accessibility_limited`**;
> **1 deferred**. Per-cell:
> **Genome Research 1999–2004 +10** (P0731–P0740) — route corrected planned-PMC → **CSHL
> publisher-bronze** (`genome.cshlp.org`; OpenAlex has 0 PMCIDs pre-2005);
> **J Cell Biology 1990–2004 +10** (P0741–P0750, `rupress.org`, navigate-to-PDF Silverchair token mint);
> **Phys Rev B 1992–2004 +10** (P0751–P0760, arXiv cond-mat green);
> **A&A 1990–2004 +10** (P0761–P0770, Cosmology-concept subset, **effective era 2001–2004** — no
> cosmology OA pre-2001, cf. eLife 2012 floor);
> **PRL pre-1990 +10** (P0771–P0780) — **viable vs the preflight's expected `accessibility_limited`**:
> APS free-to-read (bronze) particle-physics subset on `journals.aps.org`; ~22% full-text yield (rest
> first-page previews), pool extended to 40 to reach N=10; OA subset is **not** landmark-biased (bronze
> over-represents low-citation papers);
> **BMJ 1996–2004 +10** (P0781–P0790, `bmj.com`) — clinical **full-text research-vetted** (pool restricted
> to first-10 confirmed-research ranks; a 7-page guideline and 1-page editorials/news excluded; see
> `eligibility_refinement` + `candidates_pool_FULL.json`).
> **`accessibility_limited` (frozen inventory retained, not finalized):** **Cognition 1990–2004** (OA subset
> 85/843; ~20 of 21 direct-PDFs ScienceDirect-walled per the P1 wall; 1/85 PMCID) and **Neural Computation
> 1990–2004** (OA subset 110/1344; 6 with PDF+archival-unit; scattered ~19 institutional repos). Both early
> anchors remain field gaps. **Deferred: NeurIPS 1990–2004** — the existing NeurIPS inventory used a bespoke
> `neurips_proceedings_index` crawler (not in `scripts/`) and conference papers lack the volume/issue
> archival unit `build_cell.py` requires. Per-cell audits in `sources/draw_audits/round_20260624_p3_*`.
> **P4 remains pending.**

> **P4 COMPLETE (2026-06-24).** All 3 optional second-source cells finalized; **30 papers added
> (P0791–P0820)** via the §1.4.1 browser route over frozen OpenAlex frames:
> **Phys Rev D 2015–present +10** (P0791–P0800) — 2nd specialist besides JCAP; Cosmology-concept
> arXiv green (both modern PRD source ids merged);
> **Nature Materials 2015–present +10** (P0801–P0810) — 2nd top besides PRX; arXiv green subset
> (255/2609 ≈10%; hybrid OA-subset skews to arXiv-posting/physics-leaning materials — documented
> representativeness limitation);
> **EMBO Journal 2015–present +10** (P0811–P0820) — 2nd mid besides PLOS Biology; 97% OA. All EMBO
> content (incl. the 2017–2023 ex-Wiley issues) now resolves on `link.springer.com/content/pdf/{doi}.pdf`
> after EMBO's 2024 move to Springer Nature; OpenAlex's Wiley URLs are stale 404s.
> Per-cell audits in `sources/draw_audits/round_20260624_p4_*`. **This batch (P1–P4) is complete; only the
> P3 `accessibility_limited` anchors (Cognition, Neural Computation) and the deferred NeurIPS cell remain
> as documented open items.**

## Unit and target

Per `selection_protocol.md` §1.1, the sampling unit is the **cell = [field × era × source ×
tier]**, with a documented coverage target of **N ≥ 10** random-base papers per eligible cell
(breadth-first, not a cap). Each row below is one target cell.

## Headline finding (drives priority)

Under the new **hard tier-coverage requirement** (each field must span ≥2 venue tiers), **5 of
9 fields currently fail** — they sit entirely in one tier:

| Field | Current tiers | Failure |
|-------|---------------|---------|
| Computational biology / genomics-ML | specialist only (94 papers, PLOS Comp Bio) | single-tier |
| Computational neuroscience | specialist only (94, Frontiers) | single-tier |
| Structure-driven experimental physics | specialist only (97, EPJ C) | single-tier |
| ML / AI methods | top only (45) | single-tier |
| Cognitive science / psychology | specialist only (25) | single-tier |

So the top-priority action is **adding new tiers** to each failing field — which, for the three
deep fields, simultaneously de-single-sources them. Per direction, the three deep fields
(comp bio, comp neuro, structure physics) get **both a mid and a top tier** added in each modern
era, so each spans specialist + mid + top; the two single-tier fields with no depth (ML, cog-sci)
get one new tier. This is also why we **add, not remove**: the deep cells are valid samples and
equal-source weighting (§1.3) already stops them dominating; the real defect is missing tiers,
which only acquisition fixes.

## Priority scheme

- **P1 — tier-coverage hard fixes (14 cells).** Add new tiers to the 5 failing fields (modern
  eras): the three deep fields get **both mid and top** (full specialist+mid+top span); ML and
  cog-sci get one new tier. Protocol-mandatory for validity.
- **P2 — recovery (7 cells).** Finish already-scheduled blocks that stalled: clinical top-tier
  big-4 (NEJM, Lancet, JAMA, BMJ) via browser recovery; Collabra (+5) and npj Comp Materials
  (+5); rebuild the contaminated eLife 2005-2014 slice.
- **P3 — early-era expansion (9 cells).** One pre-2005 anchor per field (8 fields lack any early
  coverage; structure physics lacks pre-1990). Restores the orientation-drift time series, which
  is currently a 2005→2015 contrast for most fields.
- **P4 — remaining single-source cells (3 cells, deferrable).** Optional second sources in
  tier-OK fields so equal-source weighting has >1 source per tier group. Documented target, not
  hard; can be dropped or deferred at review.

Total if all rows execute: **~320 papers across 33 cells.** P1+P2 alone (the validity + finish
work) is ~200 papers across 21 cells.

## Column glossary

- `inventory_status` — `exists` (frozen inventory ready) / `must_build` / `extend_existing`
  (add years to an existing inventory) / `rebuild` (replace contaminated slice) / `blocked_frame`
  / `probe`. New cells cannot be drawn until a frozen inventory exists.
- `current_n`, `target_n` (10), `gap` — fill state.
- `frame_capacity` — eligible OA paper count in the frozen frame; `verify@inventory` until built.
  Caps the achievable gap (can't draw 10 if fewer eligible exist).
- `oa_route` / `oa_route_verified` — D3/D4 per-(venue,era) OA verification; `pending` until checked.
- `acquisition_route` — tiered §1.4.1 plan: direct → publisher-page browser → documented OA alternate.
- `fixes_tier_coverage` — whether the row satisfies the hard requirement for its field.
- `design_change_rationale` — one-line justification; this row doubles as the schedule-edit log
  for the new blocks (`source_block_schedule_v0.csv` is updated only when a cell is executed).
- `exception_note` — documented exceptions (thin frames, era-availability bounds).

## Candidate venues are proposals, not verified

Every new source is a candidate from `journal_list_v0.md`/`v1.md`, marked `oa_route_verified =
pending`. Per D3, OA route / tier / capacity are verified per (venue, era) **before** any draw.
If a frame comes back too thin or inaccessible, it becomes a documented exception, and a
substitute from the same field-tier list is used.

## Known exceptions (not targets to force)

- **Structure physics pre-1990 (P3):** arXiv begins 1992 and SCOAP3 is recent, so a verified
  pre-1990 OA frame is unlikely — expected `accessibility_limited`.
- **Early clinical (P3, BMJ ~1996-2004):** early clinical OA is thin; probe before committing.
- **ML mid tier (P1, TMLR):** TMLR only exists 2022+; substitute if the frame is too thin.
- **eLife (P2):** began 2012, so its 2005-2014 cell only covers 2012-2014.

## Not in scope this batch

`fields.csv` is still absent and the corpus `README.md` still reports the stale "16 papers"
pilot status; both are documentation fixes tracked separately, not acquisition.
