# Acquisition lessons from P2 — read before P3/P4

Notes from the 2026-06-24 P2 run (52 papers, 7 cells). These are failure→fix loops that
cost real time to find. Most apply directly to P3 (early-era anchors) and P4 (second sources).

## Environment / network
- **`web_fetch` does not work from the sandbox** (times out on every host, incl. example.com).
  The corpus was designed for the browser to do all fetching: browser fetches OpenAlex JSON +
  PDFs into `pdfs/_incoming/`, then `build_cell.py` / `finalize_cell.py` process them locally.
  Don't waste time on `web_fetch`; go straight to Claude-in-Chrome.
- **`_incoming` is a write/append/overwrite-only mount.** You can create files and `cp` over
  existing ones, but you **cannot `rm` or `mv`** ("Operation not permitted"). Plan filenames up
  front. To replace a wrongly-named staging file, download to a temp name then `cp temp dest`.

## Chrome download mechanics (the big time sinks)
- **Downloads must land in `pdfs/_incoming/`** (Chrome's download dir is pre-pointed there).
- **Chrome blocks the 2nd+ automatic download per site.** The user must allow-list each
  publisher *download origin* under `chrome://settings/content/automaticDownloads`.
  - Use the **wildcard form `[*.]domain.com`** — plain `domain.com` does NOT match subdomains.
  - The origin is the **final CDN after redirects**, not the journal domain. Confirmed origins:
    `[*.]silverchair.com` (UCPress/Collabra, JAMA), `[*.]nih.gov` (PMC), `[*.]nature.com`,
    `[*.]elifesciences.org`, `[*.]bmj.com`. Check `location.origin` after navigating to confirm.
- **OpenAlex JSON**: navigate to a normal allow-listed page (`example.com`), then in-page
  `fetch()` OpenAlex (it sends `Access-Control-Allow-Origin: *`), assemble all cursor pages,
  and `Blob`-download the result. Do NOT run this from the `api.openalex.org` page — its
  JSON-viewer context blocks blob downloads.
- **Publisher PDFs are CORS-blocked cross-origin.** You must navigate TO the PDF's origin, then
  same-origin `fetch(location.href)` → check first bytes are `%PDF-` → blob-download with the
  `round_<round_id>__cNN.pdf` name `finalize_cell.py` expects. The Cloudflare "Just a moment"
  JS check auto-clears in the real browser (this is normal reader access, not CAPTCHA-solving).
- **Acquisition poll pattern that works**: navigate → poll up to ~20s for
  `document.contentType==='application/pdf'` (or `location.href` contains the article id) →
  fetch → verify `%PDF-` → download.

## browser_batch / JS gotchas
- **Renderer freezes (CDP 45s timeout)** when heavy PDFs load in the Chrome viewer in rapid
  batches. Mitigations: (a) for journals that render PDFs inline, prefer navigating to the
  *HTML article page* (light) and same-origin-fetching the `.full.pdf` rather than opening the
  viewer (this is how BMJ finally worked); (b) keep batches ≤ ~12; (c) reset to a light page.
- **`SyntaxError: Identifier 's' already declared`** happens when a navigation hasn't finished
  and the next JS runs in the prior page's context. Wrap every snippet in an async IIFE
  `await (async()=>{ ... })()` so there are no persisting top-level `const`s.
- **`Execution context was destroyed`** = the page redirected mid-script. Navigate to the final
  publisher URL directly; avoid `doi.org` redirectors inside a step that then fetches.
- **Privacy filter** silently blocks tool output containing cookies/query strings (PDF token
  URLs). Return only `new URL(u).pathname` / strip query strings.

## PMC (the route for hybrid clinical journals)
- **PDF availability varies enormously by journal**: NEJM/Lancet ≈ 85%+ have a PMC PDF;
  **JAMA and BMJ deposits are ~90% XML-only** (`/articles/PMCxxxx/pdf/` returns a 404 HTML page).
  Probe content-type before bulk downloading. Run probe loops from a *light* PMC page
  (a deliberate 404 URL works) and chunk to ~30 with ~100ms delays, or the renderer freezes.
- **PMC mislinkage — verify every PMC PDF against the drawn DOI/title.** OpenAlex records for
  corrections/comments frequently carry the PMCID of the article they *reference*, so the PMC
  PDF is a *different* paper than the drawn DOI. This badly contaminated JAMA. Always title-match
  the downloaded page-1 text to the candidate title; discard mismatches (do not substitute).

## Eligibility — the finalizer heuristic is not enough for clinical
`finalize_cell.py`'s page-1 `BLOCK_PHRASES` check has real blind spots. It was hardened this run
(strip Elsevier/Monkeypox banners; ignore "correspondence to" bylines and "see Comment" cross-
refs) but still cannot reliably separate research from non-research. **Vet clinical cells by
full text**, not the heuristic:
- Research = has a Methods section AND Results/Findings AND ≥ ~5 pages.
- Non-research = Comment / Correspondence / Viewpoint / Perspective / Editorial / Feature /
  Careers / Obituary / "Analysis"/call-to-action (often ≤4 pages, no Methods+Results).
- Publisher banners (Elsevier COVID/Monkeypox) prepend page 1 and hide the section label —
  strip them before judging.
- The clean way to apply this: download a generous pool, classify by full text, then **restrict
  `candidates_pool.json` to the first-10 confirmed-research ranks in rank order** and record the
  excluded ranks in `draw_audit.eligibility_refinement`. Then finalize.

## Pipeline ordering traps
- **Always `revert_cell.py` BEFORE rebuilding/redrawing a pool.** `build_cell.py` dedups against
  existing corpus DOIs; if the cell's papers are still in `papers.csv`, the dedup shifts the
  draw ranks out of sync with already-downloaded files. (This happened to Lancet and required a
  clean rebuild.) Also: `revert_cell.py` reads the audit's `accepted` list — if you already
  overwrote the audit with a rebuild, revert can't find it and you must remove rows manually.
- Reverting + re-finalizing **renumbers** the cell's paper_ids (new IDs appended at the end),
  leaving gaps. That's fine but note it (P2 left gaps P0679–P0698; live IDs P0711–P0730).

## Frame / era correctness
- **Don't trust OpenAlex `publication_year` for era.** eLife's 2012–2014 frame was contaminated
  with later articles mis-dated to 2012–2014 (placeholder `YYYY-01-01` dates). The reliable era
  key was the **volume** (eLife vol 1=2012, 2=2013, 3=2014). For P3's pre-2005 anchors this
  matters even more — verify era via volume/issue or real `publication_date`, and watch for
  continuous-publication journals where `issue` is null except for stray artifacts (normalize it
  so the archival unit is the volume).

## What to expect for P3/P4
- **P3 (pre-2005 / early-era):** OA availability drops off fast going back in time; PMC coverage
  pre-2005 is sparse; arXiv starts 1992; structure-physics pre-1990 is already flagged unlikely.
  Expect several cells to end `acquisition_limited` — document them per §1.4.1 step 4 rather than
  forcing substitutes. Budget for low PMC-PDF hit rates and verify-title on everything.
- **P4 (optional second sources in tier-OK fields):** deferrable; low priority per the preflight.
- For any hybrid journal, you're sampling the **OA-accessible subset** (skews NIH-funded / topical)
  — record that limitation in the inventory `coverage_note`, as P2 did.
