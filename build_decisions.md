# C-SQD Build Decisions

## Purpose

This document records implementation decisions for beginning the C-SQD codebase. It should be read alongside `interpretation.md`, `C_SQD_web_app_build_plan.pdf`, `C_SQD_web_app_product_memo.pdf`, and `C_SQD.pdf`.

The immediate goal is an MVP, but the architecture should remain compatible with the eventual full C-SQD platform, including social network features, bug bounties, review challenges, subscriptions, AI-assisted workflows, institutional reporting, and richer community-based evaluation.

## Product Posture

C-SQD will be built as a web-native review graph and scholarly evaluation marketplace.

The MVP should prove the core loop:

1. C-SQD identifies or ingests a reviewable scholarly object.
2. A reviewer receives or accepts a review assignment.
3. The reviewer submits structured evaluation.
4. The system records the review as part of the evaluation graph.
5. Any associated payment or bounty state is tracked.

The platform should not be designed as a paper repository or document reader. Native article display may be added when rights permit and when it directly improves review workflows.

## Recommended Stack

### Backend

Use Rust with `axum`.

Rationale:

- explicit state machines for reviews, assignments, bounties, payments, and publication
- strong typing for scientific and payment-adjacent workflows
- good fit for auditability and permission control
- stable path toward shared validation logic and future WebAssembly use

Rust modules should use the modern file layout without `mod.rs` files. Prefer `src/foo.rs` for top-level modules and `src/foo/bar.rs` for child modules, declared from the parent with `pub mod bar;`.

### Database

Use PostgreSQL as the primary source of truth.

PostgreSQL should hold users, reviewer profiles, scholarly objects, locations, assignments, EvaluationFacts, ReviewEpisodes, SynthesisReviews, ErrorClaims, bounties, payment records, audit events, tags, relationships, and future social graph records.

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
- good fit for admin screens, reviewer workspaces, dashboards, search, and social features
- large ecosystem for forms, tables, data visualization, auth integrations, and design systems
- faster path to a polished browser-tested MVP than a full Rust frontend

### Browser Extension Companion

Include an optional cross-browser browser extension in the MVP as a first-class reviewer companion.

The core C-SQD web app must remain browser-agnostic and fully usable without an extension. The extension should improve the workflow for externally hosted journal and publisher content by connecting the publisher page to the C-SQD review graph.

The MVP extension should support:

- detecting a DOI, canonical URL, title, and basic article metadata from the current publisher or repository page
- matching the current page to an existing C-SQD scholarly object, or starting a create/import flow when no match exists
- opening a C-SQD companion panel or sidebar for drafting ElementReview facts while the reviewer reads the externally hosted work
- opening a separate-window or side-by-side C-SQD workspace when a browser-native side panel is unavailable

Implementation should use the cross-browser WebExtensions model where practical. Chrome and Edge may use the Side Panel API, Firefox may use sidebar support, and Safari support may require Safari Web Extension packaging or conversion.

The extension must not be required for ordinary platform participation. It must not bypass publisher controls, scrape paywalled article text, store unauthorized publisher content, or make C-SQD responsible for hosting externally controlled works.

### Search

Use PostgreSQL full-text search for the MVP.

Add a dedicated search service, such as Meilisearch or OpenSearch, only when product needs exceed PostgreSQL search. Future needs may include higher-quality ranking, faceted discovery at scale, typo tolerance, semantic search, and community-aware ranking.

### Object Storage

Use S3-compatible object storage for user-generated attachments only.

Examples include review evidence files, bounty evidence, permitted author uploads, and administrative attachments. C-SQD should not store article PDFs unless it has the right to do so.

### Background Jobs

Use a Rust worker with PostgreSQL-backed job and outbox tables.

Initial jobs may include metadata ingestion, metadata refresh, notification dispatch, assignment deadlines, audit rollups, and search projection updates.

Avoid heavyweight queue infrastructure until the platform's load or reliability needs justify it.

### Payments

Use an internal payment ledger and a manual provider adapter for the MVP.

Do not begin with live payment processing or payouts. The MVP should track obligations, approval states, funding sources, payout states, and provider references in a processor-agnostic way.

Stripe Connect or another payment provider can be added later through an adapter.

## Local Development

The application should be testable locally in a MacBook browser.

Expected development shape:

- frontend served at `http://localhost:3000`
- backend API served at `http://localhost:8080`
- PostgreSQL available locally or through Docker
- seeded demo users, reviewer profiles, scholarly objects, assignments, reviews, bounties, and payment records

The local app should support browser testing of the core MVP flows without requiring live ingestion or live payment processing.

## F-E-N Architecture

C-SQD should adapt the Fact-Episode-Narrative structure to scholarly evaluation.

### EvaluationFact

An EvaluationFact is an atomic evaluative assertion.

Examples:

- a claim is unsupported by cited evidence
- a statistical method is inappropriate for the data structure
- data are unavailable
- code is unavailable
- an interpretation overstates the result
- an error claim has been validated

EvaluationFacts should be typed, linked to a scholarly object, optionally linked to article locations or external evidence, and severity-scored where appropriate.

### ReviewEpisode

A ReviewEpisode groups related facts into a coherent evaluative event.

Examples:

- ElementReview
- bug bounty submission
- author response
- bounty adjudication
- review challenge
- administrative quality-control decision

ReviewEpisodes should carry workflow state, authorship, visibility, assignment, and audit context.

### SynthesisReview

A SynthesisReview provides narrative interpretation across facts and episodes.

It should summarize contribution, strengths, weaknesses, reliability concerns, and overall interpretation while linking back to the structured evaluation graph.

## F-E-N-Native Search

Search semantics should be built around the F-E-N schema, while search execution should use database and search infrastructure.

The domain model should define what search means. The infrastructure should define how search is executed.

Recommended pattern:

1. Store normalized F-E-N records.
2. Build searchable projection documents from those records.
3. Index projections in PostgreSQL full-text search for the MVP.
4. Add external search or vector indexes later if needed.
5. Expose search through C-SQD domain APIs rather than raw database queries.

Initial search projections should include:

- `scholarly_object_search`
- `review_episode_search`
- `synthesis_review_search`
- `error_claim_search`
- `reviewer_profile_search`

Search projections may denormalize titles, abstracts where permitted, metadata, review text, fact types, severities, review elements, tags, communities, publication states, reviewer expertise, and visibility rules.

This preserves a path to future search features such as community-filtered evaluation, problem-based scientific search, semantic review discovery, institution-specific views, and social graph-aware ranking.

## Payment Provider Independence

C-SQD's payment model should be independent from any payment processor's model.

Provider-specific details should live at the edge of the system. The C-SQD domain should talk to a payment provider adapter rather than directly to Stripe, PayPal, ACH, Wise, institutional invoicing, or any other processor.

### Core Payment Concepts

Use provider-agnostic internal records:

- `PaymentObligation`: a C-SQD-domain obligation to collect or pay money
- `FundingSource`: the source of funds, such as an author fee, institutional contract, bounty sponsor, conference organizer, or manual admin allocation
- `PaymentAttempt`: a concrete attempt to collect, reserve, transfer, refund, or pay out money
- `PaymentProviderEvent`: a normalized provider event or manual administrative event
- `LedgerEntry`: an immutable accounting record
- `ProviderReference`: external provider identifiers stored as metadata, not as core payment identity

### Ledger and F-E-N

F-E-N should support payment intelligibility and auditability, but it should not replace the ledger.

Use the ledger for accounting-grade records, idempotency, reconciliation, and immutable financial history.

Use F-E-N-style payment facts and episodes to explain payment workflows to admins, reviewers, funders, and auditors.

Examples:

- `PaymentFact`: reviewer became eligible for payout
- `PaymentFact`: funds were collected
- `PaymentFact`: payout failed
- `PaymentEpisode`: paid ElementReview completion
- `PaymentEpisode`: bug bounty validation and payout
- `PaymentEpisode`: conference review pool funding

Recommended architecture:

`C-SQD payment state machine -> internal ledger + F-E-N audit trail -> payment provider adapter -> manual provider / Stripe / future processor`

## MVP Role Model

Begin with a small role model:

- `reader`: can discover public scholarly objects and visible review records
- `reviewer`: can maintain a reviewer profile and complete assigned reviews
- `admin`: can manage scholarly objects, assignments, reviews, bounties, payments, and publication states
- `funder`: can be attached to funding sources or bounties, initially through admin-managed records

Author-specific workflows can be added after the review marketplace loop is working, unless needed for a specific early pilot.

## Initial Workflow Defaults

Use conservative workflow defaults for the MVP:

- assignments are created by admins
- submitted reviews route to admin quality control before publication
- bug bounty claims route through manual triage
- payment records are tracked internally and approved manually
- ingestion starts with curated seeded data before automated ingestion
- subscriptions and complex reputation algorithms are deferred

These defaults preserve the future platform shape without pretending the early system can automate scientific judgment, reviewer quality, or payment operations.

## Expansion Commitments

The MVP architecture should remain compatible with:

- reviewer communities and tags
- verified tags
- endorsements and reviewer status
- challenge workflows
- bug bounties
- paid and unpaid review paths
- author responses
- subscriptions and gated insights
- institutional reporting
- social feeds and reviewer networks
- cross-browser publisher-side browser extension workflows
- AI-assisted review and research tools
- Observatory-style graph visualization

These features should be treated as expected future expansions, not as architectural surprises.
