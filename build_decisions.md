# C-SQD Build Decisions

## Purpose

This document records implementation decisions for the current C-SQD web app.

For ontology and product precedence, read `C_SQD_NEW_GTM.pdf`, `FEN_for_CSQD_GTM.pdf`, `FEN_Schema_for_CSQD_GTM.tex`, `interpretation.md`, and `NEXT_STEPS.md` first. Older MVP documents in `old_mvp_docs/` remain historical context only.

## Product Posture

C-SQD is web-native epistemic audit infrastructure.

The first GTM slice should prove the commissioned audit loop:

1. C-SQD identifies or ingests an `AuditSubject`.
2. An organization commissions an `AuditEpisode`.
3. The commission defines scoped criteria, funding, confidentiality, and delivery context.
4. Reviewers submit structured `ElementReview` facts.
5. A synthesis author submits a `SynthesisReview` over the episode.
6. Evaluation tuples are computed from facts and episode memberships.

The platform should not be designed as a paper repository or document reader. Native article display may be added when rights permit and when it directly improves evidence work.

## Recommended Stack

### Backend

Use Rust with `axum`.

Rationale:

- strong typing for audit facts, episodes, provenance, synthesis, and payment-adjacent workflows
- good fit for auditability and permission control
- stable path toward shared validation logic and future WebAssembly use
- explicit service boundaries without introducing a high-level framework too early

Rust modules should use the modern file layout without `mod.rs` files. Prefer `src/foo.rs` for top-level modules and `src/foo/bar.rs` for child modules, declared from the parent with `pub mod bar;`.

### Database

Use PostgreSQL as the primary source of truth.

PostgreSQL should hold users, organizations, reviewer profiles, domain instantiations, CWE taxonomies, audit subjects, facts, audit episodes, episode memberships, synthesis reviews, scholarly intake records, search projections, payment/ledger records when introduced, tags, relationships, and future graph records.

### Database Access

Use `sqlx` rather than a high-level ORM.

Rationale:

- C-SQD's data model is central to the product and should remain explicit.
- SQL will be important for audit queries, search projections, reporting, and graph-adjacent joins.
- The schema is likely to evolve in ways that are easier to reason about with direct queries.

### Frontend

Use Next.js with TypeScript and React.

Rationale:

- productive for dense web application workflows
- good fit for operational consoles, episode workspaces, dashboards, search, and forms
- large ecosystem for tables, charts, auth integrations, and design systems
- fast path to browser-tested workflows

### Browser Extension Companion

Treat a cross-browser browser extension as a possible future reviewer companion, not as a prerequisite.

The core C-SQD web app must remain browser-agnostic and fully usable without an extension. A future extension should improve work with externally hosted journal and publisher content by connecting the publisher page to the C-SQD audit graph.

The extension must not bypass publisher controls, scrape paywalled article text, store unauthorized publisher content, or make C-SQD responsible for hosting externally controlled works.

### Search

Use PostgreSQL full-text search for the initial implementation.

Add a dedicated search service, such as Meilisearch or OpenSearch, only when product needs exceed PostgreSQL search. Future needs may include higher-quality ranking, faceted discovery at scale, typo tolerance, semantic search, and community-aware ranking.

### Object Storage

Use S3-compatible object storage for user-generated attachments only.

Examples include evidence files, permitted author uploads, administrative attachments, and supporting materials for audits. C-SQD should not store article PDFs unless it has the right to do so.

### Background Jobs

Use a Rust worker with PostgreSQL-backed job and outbox tables when background work becomes necessary.

Initial jobs may include metadata ingestion, metadata refresh, notification dispatch, audit rollups, and search projection updates.

Avoid heavyweight queue infrastructure until the platform's load or reliability needs justify it.

### Payments

Use an internal payment ledger and a manual provider adapter for early payment workflows.

Do not begin with live payment processing or payouts. The early system should track obligations, approval states, funding sources, payout states, and provider references in a processor-agnostic way.

Stripe Connect or another payment provider can be added later through an adapter.

## Local Development

The application should be testable locally in a MacBook browser.

Expected development shape:

- frontend served at `http://localhost:3000`
- backend API served at `http://localhost:8080`
- PostgreSQL available locally or through Docker
- seeded demo users, reviewer profiles, scholarly intake records, audit subjects, facts, episodes, memberships, and synthesis scaffolding

The local app should support browser testing of core flows without requiring live ingestion or live payment processing.

## F-E-N / C-SQD Audit Architecture

C-SQD adapts the Fact-Episode-Narrative structure into the broader audit model described by the current GTM/FEN documents.

The implementation should center:

- `AuditSubject`
- `Fact`
- `AuditEpisode`
- `EpisodeMembership`
- `SynthesisReview`
- provenance
- derived evaluation tuples

Academic Publishing records are adapter data. They are useful for intake, access, rights handling, and search, but they should not define the universal platform model.

## F-E-N-Native Search

Search semantics should be built around the F-E-N schema, while search execution should use database and search infrastructure.

Recommended pattern:

1. Store normalized F-E-N records.
2. Build searchable projection documents from those records.
3. Index projections in PostgreSQL full-text search for the initial implementation.
4. Add external search or vector indexes later if needed.
5. Expose search through C-SQD domain APIs rather than raw database queries.

Initial search projections should include:

- `scholarly_object_search`
- future `fact_search`
- future `episode_search`
- future `synthesis_review_search`
- future `reviewer_profile_search`

Search projections may denormalize titles, abstracts where permitted, metadata, fact text, fact types, severities, criteria, tags, communities, provenance, and visibility rules.

## Payment Provider Independence

C-SQD's payment model should be independent from any payment processor's model.

Provider-specific details should live at the edge of the system. The C-SQD domain should talk to a payment provider adapter rather than directly to Stripe, PayPal, ACH, Wise, institutional invoicing, or any other processor.

Use provider-agnostic internal records:

- `PaymentObligation`: a C-SQD-domain obligation to collect or pay money
- `FundingSource`: the source of funds, such as an organizational commission, institutional contract, conference organizer, or manual admin allocation
- `PaymentAttempt`: a concrete attempt to collect, reserve, transfer, refund, or pay out money
- `PaymentProviderEvent`: a normalized provider event or manual administrative event
- `LedgerEntry`: an immutable accounting record
- `ProviderReference`: external provider identifiers stored as metadata, not as core payment identity

F-E-N should support payment intelligibility and auditability, but it should not replace the ledger.

## Initial Role Model

Begin with a small role model:

- `reader`: can discover visible audit subjects, episodes, facts, and synthesis outputs
- `reviewer`: can maintain a reviewer profile and complete solicited fact work
- `admin`: can manage subjects, episodes, facts, synthesis, solicitations, payment administration, and visibility
- `funder`: can be attached to organizations, funding sources, and commissioned audit episodes

Author-specific workflows can be added after the commissioned episode loop is reliable, unless needed for a specific early pilot.

## Initial Workflow Defaults

Use conservative workflow defaults:

- commissions are created by admins or sponsors
- submitted facts route to admin quality control before broad visibility
- payment records are tracked internally and approved manually
- ingestion starts with curated seeded data before automated ingestion
- subscriptions and complex reputation algorithms are deferred

These defaults preserve the future platform shape without pretending the early system can automate scientific judgment, reviewer quality, or payment operations.

## Expansion Commitments

The architecture should remain compatible with:

- reviewer communities and tags
- verified tags
- endorsements and reviewer status
- paid and unpaid fact work
- author or submitter responses
- subscriptions and gated insights
- institutional reporting
- social feeds and reviewer networks
- cross-browser publisher-side browser extension workflows
- AI-assisted review and research tools
- graph visualization

These features should be treated as expected future expansions, not as architectural surprises.
