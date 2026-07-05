# Public Frontend Rebuild Plan

Date: July 4, 2026
Status: implemented (July 4, 2026). `npm run build:web` passes; public routes verified rendering.
Design inspiration: Google News (briefing feeds, card hierarchy, "Full Coverage" clustering).

## 1. Scope

Rewritten: the `(public)` route group in `apps/web/app`, public components, and public CSS.

Untouched: `(backstage)` routes and styling, `lib/csqd-api.ts`, `lib/public-audit.ts`, `lib/session.tsx`, `lib/advanced-mode.tsx`, all Rust/API code, database. The batch public-summary endpoint (`/api/public/audit-subjects/summaries?ids=`) already supports feed-style pages; no backend changes are needed.

No backward compatibility: old public routes 404, no legacy shims, no "new version" comments in code.

## 2. Information Architecture

New public routes:

| Route | Purpose | Replaces |
|---|---|---|
| `/` | Briefing homepage | portal-style registry home |
| `/discover` | Search + filtered card grid; absorbs CRWE-based exploration as filter chips | `/discover` + most of `/browse` |
| `/audits` | Delivered public audit reports feed | `/public-audits` |
| `/works/[id]` | "Full coverage" public audit subject page | `/scholarly-objects/[id]` |
| `/works/[id]/review` | ElementReview submission | `/scholarly-objects/[id]/review` |
| `/works/[id]/view` | Article access/reading view | `/scholarly-objects/[id]/view` |
| `/register` | External search + register ramp | `/intake` + `/retrieve` |
| `/criteria` | Slim taxonomy reference for the active domain's CWE set | reference half of `/browse` |
| `/method` | Method explainer | kept, restyled |
| `/commission` | Commission flow, public until submit | kept, restyled |
| `/domains` | Domains-as-lenses overview | kept, restyled |
| `/sign-in`, `/sign-in/complete` | Magic-link auth | kept, restyled |

Net: the discover/browse/intake overlap collapses into one consumption surface (`/discover`), one contribution ramp (`/register`), one reference page (`/criteria`).

## 3. Shell (Google News pattern)

Two-row header replacing the current single-row `PublicNav`:

- Row 1: brand; persistent search box (submits to `/discover?q=`); advanced toggle, session/role links, sign-in.
- Row 2: section tabs — Home, Discover, Audit Reports, Criteria, Method, Commission.

White header, 1px bottom border, no shadow. Near-white canvas, white cards, max content width ~1440px. No sidebar on any public page. Quiet footer kept.

## 4. Page Designs

### Home — briefing, not portal

- Lead cluster: latest SynthesisReview as a large card (headline = paper title, deck = report summary excerpt) with its ElementReviews and challenges as sub-links — the "top story + related coverage" pattern.
- Three rails: Recently challenged, Gaining scrutiny, Awaiting review (dense `WorkListRow` lists).
- Signed in: a "For you" strip driven by library/watchlist.
- One quiet method strip at the bottom. No marketing hero.

### Works page (`/works/[id]`) — "Full Coverage"

Top to bottom:

1. Subject header: title, authors, identifiers, source links, versions, status pill, tuple chips.
2. Latest public audit report as the lead story (`ReportReader`).
3. CRWE coverage matrix.
4. ElementReviews clustered by criterion, collapsed by default (`CriterionCluster`).
5. Challenges as a visually distinct dissent block (`DissentBlock`).
6. Fact timeline, advanced-gated.

Sticky right action rail: Submit ElementReview, Start/Join public episode, Submit SynthesisReview, Challenge, Petition to feature, Petition CRWE change, Commission deeper audit, Watch/Save. All visible; write actions auth-gated via `GatedAction`.

### Discover

Search bar + filter chips (domain, audit status, criterion category, sort by scrutiny depth / report availability / recent activity) above a responsive 2–3 column card grid.

### Audits (`/audits`)

Feed of completed public outputs: reports, subjects with meaningful review depth, challenged audits, recently updated records.

### Register (`/register`)

External metadata search (DOI, arXiv, PubMed, title) → create/reuse `ScholarlyObject` → register `AuditSubject` → route to `/works/[id]`.

### Criteria (`/criteria`)

Taxonomy reference for the active domain's CWE nodes with counts and links into filtered `/discover`. Not a second discover surface.

## 5. Component System

New: `SiteHeader`, `SearchBox`, `SectionTabs`, `LeadStoryCard`, `WorkCard` (strict hierarchy: status kicker → title → authors/venue/year source line → tuple chips + review/challenge counts), `WorkListRow` (dense rail variant), `SectionRail`, `CriterionCluster`, `DissentBlock`, `ActionRail`.

Kept, restyled: `TupleBadge`, `StatusPill`, `CrweCoverageMatrix`, `ReportReader`, `FactTimeline`, `GatedAction`, `ElementReviewForm`, `SubjectActions` (logic reused, presentation rewritten), `EpisodeConsole` participation pieces as needed.

Deleted from public: `AppSidebar` (backstage keeps its own), `PublicNav`, `PublicWorkCard`.

## 6. CSS Strategy

Split the 2,575-line `globals.css`:

- `backstage.css`: extracted backstage styles, byte-for-byte behavior preserved.
- `public.css`: fresh token set — near-white canvas, white surfaces, blue links, existing teal reserved for audit-semantic accents (tuple, status), larger headline type scale, tight metadata type, minimal shadows.

Public and backstage intentionally stop sharing a visual vocabulary: feed = inviting, artifact/operations = rigorous.

## 7. Domain Scoping Rule

The CWE taxonomy belongs to each `DomainInstantiation` (`cwe_nodes` in `crates/domain/src/domain_instantiation.rs`). CRWE is the Academic Peer Review instantiation.

- `/criteria` and Discover filter chips fetch nodes from the active domain's config; nothing hard-coded.
- Say "CRWE" only inside Academic Peer Review surfaces; use generic "criteria" in domain-neutral chrome.
- Adding a second domain should be a config change, not a redesign.

## 8. Public/Auth Boundary

Rule: reading is public, writing requires identity, money and assignments require roles.

- Public: all subjects, tuples, reports, ElementReviews, challenge threads, Method, commission pitch + form.
- Login: submitting reviews/challenges/petitions, starting/joining episodes, library/watchlist, "For you".
- Roles: sponsor console, reviewer queue, operations, private episode data (all backstage, unchanged).
- Commission: form public up to submission, then sign-in with preserved return URL.
- `GatedAction` wraps every public write CTA with return-URL-preserving sign-in.

## 9. Order of Work

1. `public.css` tokens + `SiteHeader`/shell (everything depends on these).
2. Card system (`WorkCard`, `WorkListRow`, `LeadStoryCard`, `SectionRail`) + home page.
3. `/works/[id]` full-coverage page + action rail + review flow.
4. `/discover` and `/audits`.
5. `/register` and `/criteria`.
6. `/method`, `/commission`, `/domains`, `/sign-in` polish.
7. Verification: `npm run build:web`, then route-by-route check against seeded data (all status labels from `db/seeds/000002_status_showcase.sql` visible; gated actions behave correctly signed out/in).
