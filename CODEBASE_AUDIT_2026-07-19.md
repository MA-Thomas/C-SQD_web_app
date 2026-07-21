# C-SQD Codebase Audit — July 19, 2026

Scope: full read of the Rust workspace (`crates/domain`, `crates/academic-adapter`, `services/api`), the Next.js app (`apps/web`), migrations/seeds, and the strategy documents (`C-SQD_NEW_GTM.tex`, `interpretation.md`, `NEXT_STEPS.md`, `UI_UX_STRATEGY_MEMO.md`, `PUBLIC_FRONTEND_REBUILD_PLAN.md`). TypeScript was verified clean (`tsc --noEmit` passes); the Rust workspace was reviewed by reading, not compiled in this session.

---

## 1. What this project is, and whether the code agrees

The philosophical thesis is clear and consistently stated across the documents: C-SQD is not a journal or a publishing platform. It is infrastructure for commissioned, decomposed epistemic audits, built on the mediating-assessments insight from the decision-quality literature — complex judgments become more reliable when evaluators assess intermediate dimensions independently before forming a verdict. The product's central asset is the audit graph (subjects, facts, episodes, memberships, synthesis reviews, provenance), with the evaluation tuple as a derived, recomputable view rather than stored opinion. Academic publishing is the first adapter, not the identity.

The unusual and genuinely good news: **the code embodies this philosophy faithfully.** This is rare. Specifically:

- Facts are immutable, timestamped, provenance-bearing rows; commissions, solicitations, petitions, and curation decisions are all facts, exactly as the ontology demands.
- The eval tuple is computed on demand from memberships (`compute_eval_tuple`), parameterized by reviewer community and reference time — a derived view, as `interpretation.md` insists.
- The substrate/adapter split is real in the code, not just the docs: `crates/domain` knows nothing of papers; `crates/academic-adapter` and `routes/peer_review.rs` carry the scholarly surfaces.
- Rights-aware article access (native display only with trustworthy license signals, external links otherwise) is implemented, matching the hosting policy.
- The two visual registers (editorial public record vs. quiet operational backstage) exist and are deliberate, scoped in separate stylesheets.

Doc-to-code coherence is the strongest thing about this repo. The tech-debt story below is mostly about the gap between "coherent local MVP" and "something a real funder touches."

### One philosophical inconsistency worth fixing

`interpretation.md` says revenue design "should support the integrity and usefulness of the audit graph, not distort it." But in `crates/domain/src/eval_tuple.rs`, the stakes signal `S` is computed directly from the commission's funding amount (`funding_amount / 10_000`, clamped). A sponsor can raise a claim's public "stakes" score by paying more. That is a small but real distortion channel: money flowing into the graph currently changes the graph's epistemic output. Before funders see this, either remove funding from `S`, cap its weight and disclose it, or reframe `S` explicitly as "commissioned attention" rather than intrinsic stakes. This is exactly the kind of thing a sophisticated pilot funder (or a critic) will notice.

A second, softer tension: the public commission form ships with a fabricated default sponsor ("Northstar Bio Diligence") and a pre-filled $7,500. For a product whose entire value proposition is provenance and honesty, demo fabrications leaking into the real submission path undercut the brand at the most credibility-sensitive moment.

---

## 2. Technical audit

### Strengths (worth preserving)

The backend is well-structured for its size: repositories use transactions consistently for multi-row writes, there is a typed `ApiError`/`RepositoryError` chain, migrations carry indexes and check constraints, and the batch public-summary endpoint already collapsed the front-page N+1 fan-out. Auth is honest work: magic-link tokens and session tokens are stored only as SHA-256 hashes, links are single-use with a 15-minute TTL, CORS is origin-pinned and credentialed, and the sign-in page guards against open redirects. The domain crate has real unit tests. The frontend has a disciplined data layer (`csqd-api.ts` as the single API boundary, `public-audit.ts` as the view-model layer), a documented design-token system in `public.css`, and good empty states throughout.

### P0 — must fix before any external user touches it

**Auth is dev-mode by design and must be flipped.** `request_magic_link` returns the sign-in URL in the HTTP response body (and the sign-in page renders it). As deployed today, anyone can sign in as anyone by typing their email. This is documented as intentional for local dev — fine — but the pilot gate is: integrate a transactional email provider (Postmark/Resend/SES), return only `{email, expires_at}` from the endpoint, and put the link-in-response behavior behind an explicit `CSQD_DEV_AUTH=1` flag.

**Several write endpoints are unauthenticated.** `POST /api/audit-subjects`, `POST /api/audit-subjects/:id/audit-episodes` (commission), and `POST /api/library-items` require no session. The article-retrieval endpoints are GETs with side effects — they create scholarly objects and audit subjects on fetch, unauthenticated. Anyone with the API URL can flood the registry and mint fake "commissioned" episodes with arbitrary funding numbers that then surface on the public homepage. Require a session on every write (the episode-fact endpoints already do this correctly), and rate-limit auth and registration endpoints (e.g. `tower_governor`).

**The library is not actually per-user.** `list_library_items` lists all items with no user scoping and `add_library_item` ignores the session identity. The `/library` page is gated in the UI, but the data underneath is global. This needs `user_id` scoping from the session before "logged-in account" flows mean anything.

**Session cookie lacks `Secure`.** Add it (behind a config flag so local HTTP still works), since you will be deploying behind HTTPS.

**No deployment story.** `infra/docker-compose.yml` is Postgres-only; there are no Dockerfiles, no CI, no hosted environment, no backup policy. A pilot needs: one small deployment (Fly.io/Render/a VPS), managed Postgres with automated backups, HTTPS, and a `cargo test && tsc && next build` CI gate on push. Half a day of work, and it converts the repo from "local artifact" to "product."

**Seed data will read as real.** The seeded audits, sponsors, and reports are indistinguishable from real activity on the public record. Before funders browse, either run clean with 2–3 real (even self-commissioned) audits, or visibly label demo content. A registry of invented audits is the one first impression this product cannot afford.

### P1 — structural debt worth paying soon

- **No API-layer tests.** The domain crate is tested; the repositories and handlers (where the transactional integrity lives — commission creates sponsor + episode + fact + membership atomically) have none. Add a small integration-test harness against a throwaway Postgres (sqlx supports this well) covering commission, element-review submission, and the eval-tuple endpoint.
- **`Domain` errors map to HTTP 500.** Validation failures ("scoped-claim subjects require a claim statement") surface as server errors. Split into `UnprocessableEntity` (422) vs. true internals; the frontend can then show real field errors.
- **Role administration is SQL-only.** There is no way to grant sponsor/reviewer/operator roles except editing the `users.roles` array by hand. A minimal operator-console panel (list users, toggle roles) unblocks pilots and stops direct DB edits, which are provenance-invisible — ironic for this product. Consider recording role grants as facts.
- **Homepage/discover load the whole registry.** `getScholarlyObjects()` fetches everything, then sorts/filters in the page. Fine at pilot scale; add server-side pagination and sorting before the registry grows past a few hundred works. Same for the batch-summary server-side loop you already flagged in `NEXT_STEPS.md` — agreed it can wait.
- **Repo hygiene.** `Nature_Paper_Rival_Paradigms/` (a research-paper project with tracked `.docx` backups and timestamped `_backup_*` directories) lives inside the product repo — move it to its own repository. Git history is nearly all "updates"; adopt conventional short messages so the history becomes an audit trail (again: on-brand). Remove tracked backup files.
- **Config duplication risk.** `web_base_url`/`NEXT_PUBLIC_API_BASE_URL` pairing works, but there is no staging/prod config layering yet; do this alongside deployment.

---

## 3. Product: money and the logged-in experience

### Financial reality check

Today, money exists only as *claims within facts*: `funding` on `AuditCommission`, `payment_scheme` on solicitations. Nothing moves money, nothing verifies it, nothing tracks whether a reviewer was paid. That is actually the right amount of payments code for this stage — do not build Stripe checkout yet. Organizational sponsors at $4k–$10k pay by invoice/ACH, not credit card, and premature billing infrastructure is a classic time sink.

What *is* worth building now is the **commission lifecycle and money ledger, as facts** — which your ontology already wants:

1. Give episodes an explicit commercial state: `inquiry → proposed → funded → in_progress → delivered → closed`. Today an episode springs into existence fully "commissioned" the moment a web form is submitted, which is false for any real engagement.
2. Record `InvoiceIssued`, `PaymentReceived` (operator-confirmed), and `ReviewerPayoutSent` as fact payload variants. Manual money movement, provenance-tracked in the graph. This is philosophically perfect for C-SQD and about a day of schema/enum work.
3. Only show funding amounts publicly once operator-confirmed as received; until then the public record shows "commissioned, funding pending." This closes the fake-funding-number hole and the stakes-distortion hole at once.
4. Stripe Invoicing (not Checkout) can automate step 2 later, when volume justifies it.

### The commission flow itself

The current form asks a stranger, unauthenticated, to fill in twelve fields including funding amount, and instantly creates the sponsor org, episode, and commission fact. Real funders will not self-serve a $7,500 commitment this way, and you don't want them to — early audits need scoping conversations.

Restructure as two stages. Stage one, public: a short inquiry — who you are, the claim or artifact, decision context, rough budget band — that creates an `inquiry`-state record and notifies you. Stage two, backstage: after the scoping call, the operator (or the sponsor, signed in) finalizes scope criteria, funding, deadline, and confidentiality, flipping the episode to `proposed`/`funded`. The current rich form becomes the stage-two surface almost unchanged. Kill the fabricated defaults either way.

### Logged-in account flows — current state and gaps

What exists: magic-link sign-in with role-aware header, gated `/library`, `/sponsor-console`, `/reviewer-queue`, `/operations`, `/audit-episodes/:id`, episode-scoped submission of element reviews, warrants, responses, petitions, and curation decisions. The gating pattern (`AuthGate`, `require_role`) is clean.

What's missing for a credible pilot account experience, in rough order: a post-sign-in onboarding step (capture display name — currently derived from email — and affiliation/expertise for reviewers); an account/settings page (email, sign-out-everywhere, role visibility); sponsor console lifecycle views tied to the commercial states above ("your audit is in review, 3 of 5 element reviews delivered") rather than raw episode lists; reviewer queue showing assignment → acceptance → submission → payment status per solicitation; and email notifications for the events people currently must poll for (solicitation received, review submitted, report delivered, challenge filed). Notifications matter more than any new surface — an audit platform where participation requires remembering to check the site will stall at pilot scale.

---

## 4. Visual design, UI, UX

### What's working

The editorial-registry register is distinctive and correct for the mission. The token system in `public.css` is disciplined (documented type scale, two microlabel styles, semantic color roles, teal reserved for audit semantics). The briefing homepage answers "what changed?" — the right question for a registry. Empty states are written with care. The Advanced toggle is a smart pressure valve for notation density. This does not look like a template, which for an infrastructure-credibility product is worth a lot.

### Where to improve

**Vocabulary load is the biggest UX risk with funders.** The public surfaces lean on CRWE, tuple notation, ElementReview, SynthesisReview, FEN-flavored phrasing. Your buyers are diligence leads and program officers, not ontologists. Every public page should lead with the plain-language layer — "Independent structured reviews across N criteria; here's the report and what remains contested" — and keep the formal vocabulary behind the Advanced toggle or one level down. The method page can carry the full formalism; the homepage and subject pages should sell comprehension in five seconds. A concrete test: a program officer should understand a subject page's verdict without learning any C-SQD noun.

**Give the tuple a verdict companion.** Five dimensions with dots is honest but cognitively expensive as a first read. Pair every tuple badge with a one-line natural-language status ("2 upheld problems across statistical methodology; scrutiny is deep; no ethical concerns") — you already generate status labels server-side, so this is mostly copywriting plumbing.

**Navigation is one tab too wide.** Home, Discover, Claims, Audit Reports, Criteria, Domains, Method, Commission. Claims vs. Discover vs. Audit Reports forces the visitor to understand the works/claims distinction before they can navigate. Consider folding Claims into Discover as a filter (you already absorbed CRWE browse this way — same move), and Criteria/Domains under Method as reference pages. Five tabs: Home, Discover, Reports, Method, Commission.

**Polish inventory (small, high-leverage):**

- Faint text (`--ink-400`, #8a857c) on white is ~3.6:1 contrast — below WCAG AA for the small sizes it's used at; darken a step. The 10px microlabels are at the floor of legibility; consider 11px.
- Only 2 media queries in `public.css` — do a real pass on tablet/phone; funders open links from email on phones.
- Verify favicon/OG metadata so shared links to subject pages unfurl properly; a registry's links get pasted into Slack and diligence memos constantly.
- Loading: the public pages are server-rendered with full-registry fetches; as data grows, add streaming/skeletons to keep first paint fast.
- Form validation on Commission/register is browser-native only; with the 422 error split (above), show inline field errors.

---

## 5. Moving forward without moving arbitrarily

The GTM document states the falsifiable question precisely: *will organizations fund structured, decomposed audits?* No line of code answers that; only a commissioned audit does. So the forward test for every proposed piece of work should be: **does this remove an obstacle between a real funder and a delivered audit report?**

That test produces this sequence:

1. **Week 1–2 — make it real (P0 list):** email-based auth, session-gated writes, rate limiting, deployment with HTTPS and backups, demo-data scrub. Nothing new; just hardening what exists.
2. **Week 2–3 — make money legible:** commercial lifecycle states + invoice/payment/payout facts + two-stage commission intake. Manual, operator-driven money handling is fine — the point is that the system of record is the audit graph itself.
3. **Week 3–4 — make it comprehensible:** plain-language layer, verdict lines, navigation consolidation, mobile pass, notifications for the pilot's core loop (solicitation → submission → report).
4. **Then stop building and run the experiment:** hand-recruit 2–3 sponsors from the comp-bio/biomedical-ML network the GTM already identifies, concierge every step, and let the platform be the ledger and the public record while you are the workflow. What the pilots demand next is the roadmap; what they don't touch is the code you correctly didn't write.

Deliberately deferred, and rightly so per your own notes: second-domain configuration, Stripe automation, the batch-summary SQL optimization, publishing workflows, and any reader sophistication beyond what review creation needs.

The codebase is in better condition than most projects at this stage — coherent ontology, disciplined implementation, honest documentation of its own gaps. The distance to a funder-ready pilot is not architectural. It is one auth flip, one deployment, one money-lifecycle, and one layer of plain language.

---

## Implementation status — July 19, 2026

The following audit items were implemented directly after this report was written.

**Backend (needs a local `cargo check/test` pass — not compiled in the session that wrote it):** dev-auth is now a flag (`CSQD_DEV_AUTH`, default on locally): with it off, the magic-link URL is no longer returned in the API response, only logged, with a marked TODO hook for an email provider. Magic-link issuance is throttled to 3 per address per 15-minute window (new 429 `RateLimited` error variant). Session cookies gain a `Secure` attribute behind `CSQD_SECURE_COOKIES`. `POST /api/audit-subjects`, the commission endpoint, the four article-retrieval GETs, and both library endpoints now require a session; the library is scoped to the session user (the hardcoded demo user is gone). `Domain` errors map to 422 instead of 500. The funding term was removed from the stakes signal in `eval_tuple.rs`, with a comment recording why. `CSQD_API_BIND` allows non-loopback binding for containers.

**Frontend (type-checks clean):** the commission form's fabricated defaults are gone; its server action forwards request cookies, checks the session, and redirects to sign-in with an explanation when absent. The register page keeps local search public but asks for sign-in before external retrieval. The sign-in page handles the emailed-link (non-dev) response. Navigation collapsed to five tabs (Home, Discover, Audit Reports, Method, Commission); Claims, Criteria, and Domains remain first-class via Discover, Method, and an expanded footer. `/method#vocabulary` now carries the full formal vocabulary — each FEN term with a plain-language gloss and a note on why the precision is load-bearing — linked from the footer; the plain glosses are framed explicitly as on-ramps to the exact terms, not replacements. Tuple badges on the lead story, work pages, and claim pages render a one-sentence plain-language verdict beside the exact values. Faint-text contrast raised to ~5:1, microlabels bumped 10px → 11px, favicon/OG metadata added.

**Scaffolding:** `.github/workflows/ci.yml` (fmt/check/test + tsc/lint/build), `infra/Dockerfile.api`, `infra/Dockerfile.web`, expanded `.env.example`, README section on auth modes and deployment flags.

**Round two — July 19, 2026 (all remaining audit items):**

*Money as facts.* Three new fact payload variants — `invoice_issued`, `payment_received`, `reviewer_payout` — with operator-only endpoints, validated references back to the commission/solicitation facts they settle, and membership on the episode's audit trail. "Funded" is a derived view (an active payment fact exists), surfaced as `funding_confirmed` on episode summaries and as a Funding column in the sponsor/operations consoles. The evaluation tuple explicitly ignores all commercial facts. Migration `000004` extends the payload-kind constraint. Operators record money through a Commercial panel on the episode workspace — the workspace's first real content.

*Two-stage commissioning.* `/commission` now shows a public inquiry form to signed-out visitors (stage one: contact, subject description, decision context, budget band — recorded in a new `commission_inquiries` table, pre-graph by design, throttled per address) with a plain-language explanation of the inquiry → scoping → commission → delivery path. Signed-in users get the full scoped form (stage two). Operators triage inquiries in Operations and link converted inquiries to their episodes.

*Email + notifications.* A provider-agnostic mailer (Resend/Postmark via HTTP; logs when unconfigured) now delivers magic links when dev-auth is off, and sends the pilot loop's notifications: new inquiry → operator inbox, solicitation issued → reviewer, element review submitted → operator inbox. Delivery is best-effort; state is durably recorded first.

*Role admin + accounts.* Operations gains an Accounts panel (grant/revoke sponsor/reviewer/operator, with a guard against removing your own operator role) — role grants no longer require SQL. A new `/account` page handles display-name updates; first sign-in with a derived name routes through `/account?welcome=1` before continuing, so authorship on the record is chosen, not guessed.

*Polish and verification.* `NEXT_PUBLIC_DEMO_MODE=1` renders a demonstration-data banner on all public pages. `/api/scholarly-objects` accepts `limit`/`offset` (default 50, cap 200) for server-side pagination. `scripts/smoke_test.sh` exercises the full pilot loop over HTTP (health → auth → registration → commission → inquiry → eval tuple → public summary → operator gating); a Rust integration-test harness still wants a lib/bin split in `services/api`, noted below. TypeScript checks clean; the Rust workspace needs a local `cargo check/test` pass since this environment cannot compile it.

**Remaining beyond the audit's scope (deliberate):** run `cargo fmt/check/test` and `scripts/smoke_test.sh` locally; split `services/api` into lib + bin to enable in-process integration tests; Stripe Invoicing automation when volume justifies it; second-domain configuration when a real pilot needs it. Then stop building and commission the first real audits.
