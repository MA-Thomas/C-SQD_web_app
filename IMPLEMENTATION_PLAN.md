# Implementation Plan: FEN Alignment + Public Registry Frontend

Date: June 11, 2026

Source precedence: `C_SQD_NEW_GTM.pdf` -> `FEN_for_CSQD_GTM.pdf` / `FEN_Schema_for_CSQD_GTM.tex` -> `UI_UX_STRATEGY_MEMO.md` -> `interpretation.md` -> `NEXT_STEPS.md`.

The plan is two tracks. Backend phases B0–B6 and frontend phases F0–F7. Cross-track dependencies are listed per phase. F0 and F2 can start immediately, in parallel with B0.

Verification gate after every phase:

```sh
cargo fmt --all
cargo check --workspace
cargo test --workspace
npm run build:web
```

---

## Implementation Status (updated June 16, 2026)

The plan is substantially implemented. Backend phases are verified by a clean
`cargo check`/`cargo test --workspace` and live endpoint checks against seeded
data; frontend phases are verified by a passing `npm run build:web` plus the
components/routes existing in the tree (not every UI flow has been manually
exercised end-to-end).

| Phase | Status | Notes |
|-------|--------|-------|
| B0 Type fidelity | ✅ Done | newtype IDs, `Timestamp = DateTime<Utc>`, payload-label variants, `Temporal`/`Authored` traits all present. |
| B1 Eval tuple pure fn | ✅ Done | `compute_eval_tuple` in `crates/domain`; unit tests present. Live check confirmed the solicited-review multiplier (scrutiny_depth 3.5 = 1.5 + 1 + 1) and N/M finding counts. |
| B2 Identity/auth | ✅ Done | Magic-link chosen (decision 1). Sessions + `require_role(Role::Operator)` enforced on backstage routes. |
| B3 Public summary API | ✅ Done | Single `/summary` and **batch `summaries?ids=`** (B3.2) both live. Batch currently loops server-side; single aggregating SQL pass remains an optional optimization (see B3 note). |
| B4 Relations/challenges | ✅ Done | `episode_relations`, synthesis `contests` relations, and `submitter_response` contests all wired; challenge counts surface in summaries. |
| B5 Participation/petitions | ✅ Done | `EpisodeParticipation`, `FeaturePetition`, `CWEPetition`, `CurationDecision` variants + routes; exercised by seed subject C. `CurationDecision` chosen for featuring (decision 2). |
| B6 Crate split | ✅ Done | `crates/academic-adapter` separated from `crates/domain`. |
| F0 Route groups | ✅ Done | `(public)` / `(backstage)` groups in `apps/web/app`. |
| F1 Thin data layer | ✅ Done | Reads the summary API; batch endpoint now available to collapse fan-out. |
| F2 Design system | ✅ Done | `TupleBadge`, `StatusPill`, `CrweCoverageMatrix`, `FactTimeline`, `ReportReader`, `GatedAction` all present; `/dev/components` demo route exists. |
| F3 Subject page | ✅ Done | `scholarly-objects/[id]` report layout + actions rail present. |
| F4 Auth/submission | ✅ Done | `/sign-in`, session context, element-review form present. |
| F5 Challenges/petitions UI | ✅ Done | Contest/petition forms in `subject-actions.tsx`. |
| F6 Advanced/recompute | ✅ Done | `tuple-recompute.tsx` "Recompute as…" panel. |
| F7 Seed data + polish | ✅ Done | `db/seeds/000002_status_showcase.sql` makes every status label appear in Discover (Unaudited, Registered for audit, ElementReviews submitted, In synthesis, Audit report available, Challenged, Superseded) plus one fully-worked audit with petitions, participation, curation, and a challenge thread. `scripts/setup_db.sh` now applies all `db/seeds/*.sql`. |

---

## Track B: Backend Alignment (Rust)

### B0. Type fidelity foundations

No behavior change; pure type-level alignment with the FEN schema. Do this first because every later phase touches these types.

1. **Newtype IDs.** Add `FactId(Uuid)`, `AuditSubjectId`, `AuditEpisodeId`, `MembershipId`, `SynthesisReviewId`, `DomainInstantiationId`, `CWENodeId`, `UserId`, `OrganizationId` in `crates/domain`, all `#[serde(transparent)]` with `Display`/`FromStr`. Replace `String` IDs field-by-field; the compiler drives the refactor. Repositories convert at the SQL boundary (`id::text` stays in queries initially).
2. **Real timestamps.** Replace `pub type Timestamp = String` with `chrono::DateTime<Utc>` (RFC3339 serde). Update `row_to_*` mappers in `services/api/src/repositories/*`. This unblocks `T_eval` cutoffs (B1) and timeline sorting (B0.4).
3. **Restore payload-carrying variants.** `AuditSubjectType::Other(String)`, `DomainType::Custom(String)`, `OrganizationType::Other(String)`, `StakesDefinition::Custom(String)`, `UptakeDefinition::Custom(String)`, `CWESource::CommunityExtension { community_id }`. Use serde adjacently-tagged or `{ "other": "label" }` JSON forms; add a `*_label` column or reuse existing JSONB where a column is a bare enum string. Migration: `000002_variant_labels.sql`.
4. **`Temporal` and `Authored` traits.** Implement for `Fact`, `EpisodeMembership`, `EpisodeRelation`, `SynthesisReview`. Add one consumer immediately so the traits are exercised: `fn merge_timeline(items: Vec<Box<dyn Temporal>>)`-style helper (or a concrete `TimelineEntry` enum sorted via the trait) used by the episode facts endpoint.

Exit criteria: workspace compiles with zero `String` entity IDs in `crates/domain`; all existing tests pass; seeded demo data round-trips.

### B1. Evaluation tuple as a pure domain function

Depends on B0 (timestamps, IDs). This is the spec's most-argued property — implement it exactly.

1. Move computation out of `services/api/src/repositories/audit_episodes.rs` into `crates/domain/src/eval_tuple.rs`:

   ```rust
   pub fn compute_eval_tuple(
       inputs: &[(Fact, EpisodeMembership)],
       community: &ReviewerCommunityFilter,
       t_eval: DateTime<Utc>,
       config: &EvalTupleConfig,
   ) -> EvalTuple
   ```

2. Honor the spec: exclude retracted memberships and non-`Active` facts; exclude facts with `occurred_at > t_eval`; apply `ReviewerCommunityFilter` (tags require B2 — until then, an empty filter means "all reviewers", explicitly documented); weight `L` by `solicited_review_multiplier` and a pluggable `expertise_weight_fn` registry (string id -> fn, with a default identity weight); route `S` and `U` through `StakesDefinition` / `UptakeDefinition` operationalization hooks instead of the current funding heuristic and synthesis count.
3. Repository becomes fetch-only: load `(Fact, EpisodeMembership)` pairs for the episode and call the pure function.
4. API: `GET /api/audit-episodes/:id/eval-tuple?t_eval=...&tags=...&min_endorsements=...`. Defaults reproduce current behavior.
5. Unit tests in `crates/domain`: retracted membership exclusion, `t_eval` cutoff, solicited multiplier, per-domain S/U operationalization. This is the highest-value test surface in the codebase.

Exit criteria: `compute_eval_tuple` is deterministic and side-effect free; same inputs produce identical output in tests; existing frontend numbers unchanged under default parameters.

### B2. Identity: users, reviewer profiles, sessions, roles

Depends on B0. Unblocks F4 and every memo participation flow. DB tables already exist.

1. Domain types: `User`, `UserStatus`, `ReviewerProfile`, `ReviewerStatus`, `ReviewerTag`, `TagScope`, `ReviewerDomainExtension` per the FEN schema.
2. Repositories + routes: `POST /api/auth/sign-in`, `POST /api/auth/sign-out`, `GET /api/auth/session`, `GET /api/users/:id`, `GET /api/reviewer-profiles/:user_id`. Cookie sessions (e.g. `tower-sessions` + Postgres store). Password or magic-link — keep minimal; the MVP needs role state, not an identity product.
3. Roles as session claims: `public`, `member`, `sponsor(org_id)`, `reviewer`, `operator`. Route-layer extractors (`RequireRole`) for backstage endpoints.
4. Principal resolution helper: given a `Principal`, return display name + type for provenance rendering (used by F3 timeline and report bylines).

Exit criteria: `/sign-in` page authenticates against the API; session-aware `GET /api/auth/session` returns roles; one backstage route (e.g. solicitation creation) actually enforces a role.

### B3. Public subject summary API

**Status: ✅ Done.** Both the single `GET /api/public/audit-subjects/:id/summary`
and the batch `GET /api/public/audit-subjects/summaries?ids=...` (item 2) are
live and return server-side status labels. Open follow-up: the batch endpoint
assembles each summary with its own queries in a loop
(`summaries_for_audit_subjects` in `repositories/public_summary.rs`); converting
it to one aggregating SQL pass (item 2's "One SQL pass") is an optional
performance optimization, not a missing feature — the frontend already issues a
single HTTP call.

Depends on B0; better after B1. Kills the frontend N+1 (F1).

1. `GET /api/public/audit-subjects/:id/summary` returning: audit status label, evaluation tuple (default filter), latest public SynthesisReview (id, summary, authored_at, sections), CRWE coverage (reviewed/unreviewed node ids), ElementReview / SynthesisReview / challenge counts, episode list (id, label, status).
2. `GET /api/public/audit-subjects/summaries?ids=...` batch variant for Discover, home, and Public Audits. One SQL pass with aggregation, not per-subject loops.
3. Status label logic (`Unaudited`, `ElementReviews submitted`, `In synthesis`, `Audit report available`, `Challenged`, `Superseded`) moves server-side into one function — currently duplicated in `apps/web/app/lib/public-audit.ts`.

Exit criteria: home and Discover render from at most two API calls; `challengeCount` comes from the API (0 until B4, but no longer hardcoded in the frontend).

### B4. Relations, challenges, responses

Depends on B0, B2 (challenges need an authenticated principal). Brings the dead types to life.

1. Repositories + routes for `EpisodeRelation` (`POST /api/audit-episodes/:id/relations`, `GET` list) and `SynthesisReviewRelation` (`POST /api/synthesis-reviews/:id/relations`) including `Contests(ContestationInfo)`.
2. `SubmitterResponse` fact creation: `POST /api/audit-episodes/:id/facts/submitter-response` with `ResponseType::Contests` et al.
3. Challenge semantics per the memo: challenged artifacts are preserved; a contestation is a provenance-bearing record; superseding sets `FactStatus::Superseded` / `NarrativeStatus::Superseded` plus a relation, never deletion.
4. `expect_payload_kind(db, fact_id, FactPayloadKind)` verification helper; apply to all `FactId` cross-references (solicitation -> commission, solicitation_event -> solicitation, responses -> reviewed facts). Spec requires this at the application layer.
5. Challenge counts and open-challenge lists wired into B3 summaries.

Exit criteria: an ElementReview can be challenged end-to-end via API; the challenged review remains readable; counts surface in the public summary.

### B5. Participation and petitions (new FactPayload variants)

Depends on B2, B4. The FEN extension path is "add a variant; the compiler finds every match."

1. New `FactPayload` variants + migration for payload kind strings:
   - `EpisodeParticipation { participant: UserId, action: Start | Join }` — public episode start/join, prerequisite for unsolicited SynthesisReviews per the memo.
   - `FeaturePetition { element_review: FactId, petitioner: UserId, rationale: String }`.
   - `CWEPetition { kind: NewElement | Applicability, cwe_node: Option<CWENodeId>, proposed_label: Option<String>, rationale: String }`.
2. Routes: start/join public episode; submit petitions; list petitions for a subject/criterion.
3. Unsolicited ElementReview: open the existing `POST /api/audit-episodes/:id/facts/element-review` to authenticated members for public episodes; `solicitation: None` marks it unsolicited. Unsolicited SynthesisReview requires an `EpisodeParticipation` membership first (enforced server-side).
4. Featured-as-curation: stop treating `featured: bool` as mutable state on an immutable fact. Either (a) derive featured status from granted `FeaturePetition` facts, or (b) add a `CurationDecision` fact variant referencing the review. Decision needed — (a) is leaner; (b) gives operators direct control. Default recommendation: (b), with petitions as input to it.

Exit criteria: logged-in user can start a public episode, submit an unsolicited ElementReview, and petition; all acts are Facts with provenance and memberships.

### B6. Crate split: substrate vs adapter

Depends on B0–B5 settling (file moves are cheap; do them when types are stable).

1. `crates/domain` keeps the FEN core: domain instantiation, subjects, users/orgs, facts, episodes, syntheses, eval tuple, traits.
2. New `crates/academic-adapter`: `ScholarlyObject*`, article access/retrieval/version types, `LibraryItemSummary`, CRWE-browse view types. Depends on `csqd-domain`; never the reverse.
3. `services/api` route modules mirror the split (`/api/...` substrate vs `/api/peer-review/...` adapter). Verifies the "domains are lenses over one substrate" claim in the dependency graph and keeps a future clinical-trials adapter honest.

Exit criteria: `cargo check -p csqd-domain` succeeds with no scholarly/article code; no dependency cycle.

---

## Track F: Frontend Refactor (Next.js)

### F0. Two shells: route groups `(public)` and `(backstage)`

No backend dependency. The single biggest memo-alignment fix — public pages must stop rendering the admin sidebar.

1. Restructure `apps/web/app` into route groups:
   - `(public)`: `/`, `/discover`, `/public-audits`, `/domains`, `/method`, `/commission`, `/intake`, `/browse`, `/scholarly-objects/[id]`. Layout: slim top navbar (Discover, Public Audits, Domains, Method, Commission an Audit, Sign in), full-width reading column, footer with domain framing.
   - `(backstage)`: `/library`, `/sponsor-console`, `/reviewer-queue`, `/operations`, `/audit-episodes/[id]`, `/assignments`. Layout: keep the dense `AppSidebar`, but render Sponsor/Reviewer/Operations sections only for sessions holding those roles (hardcode-hidden until B2; role-driven after).
2. Domain switcher moves from sidebar to the public navbar as a compact control; planned domains marked `Planned`, no fake workflows.
3. Delete `AppSidebar` usage from all public pages.

Exit criteria: an anonymous visitor never sees sponsor/reviewer/operations chrome; backstage routes still work behind `AuthGate`.

### F1. Thin data layer over the summary API

Depends on B3.

1. Replace the fan-out in `apps/web/app/lib/public-audit.ts` (per-work episodes -> facts -> per-episode tuple + syntheses) with calls to the batch and single summary endpoints.
2. Delete client-side tuple aggregation, status-label duplication, and `challengeCount: 0`.
3. Keep `groupScholarlyObjects` version-grouping only if the summary API doesn't subsume it; otherwise remove.

Exit criteria: home, Discover, Public Audits, and subject pages each render from <= 2 API calls; `public-audit.ts` shrinks to formatting helpers.

### F2. Design system and core components

No backend dependency; parallel with F0/F1. Target feel per NEXT_STEPS: quiet, dense, operational — registry, not marketing.

1. Token layer (CSS variables): semantic status colors (unaudited / reviewed / problems / contested / superseded), surface scale, type scale, spacing. Replace ad hoc classes incrementally.
2. Components (single-purpose, reused everywhere):
   - `TupleBadge` — the five values as labeled marks (Problems, Ethical concerns, Stakes, Scrutiny depth, Uptake); identical rendering on cards, subject headers, report headers; hover/expand reveals `E(A | R, T_eval) -> (N, M, S, L, U)`. This is the platform's visual signature.
   - `StatusPill` — one component, semantic color from token layer.
   - `CrweCoverageMatrix` — one row per criterion, state-colored (unreviewed / no problems / problems / contested), click-through to reviews, per-row `Review this criterion` CTA.
   - `FactTimeline` — vertical interleaved timeline of commissions, reviews, solicitations, syntheses, challenges (consumes B0.4 Temporal-sorted API output; degrades to facts-only before B4).
   - `ReportReader` — SynthesisReview rendered as a document: typographic sections, inline citations linking to referenced ElementReview facts, permalink/cite affordance.
   - `GatedAction` — visible-but-auth-gated button; opens sign-in preserving `return_to`, subject id, and criterion id.

Exit criteria: Storybook-style demo page (dev-only route) showing all components against seed data; no public page defines bespoke tuple or status markup.

### F3. Subject page as flagship audit report

Depends on F1, F2; richer after B4.

1. Rebuild `/scholarly-objects/[id]` per the memo's structure: sticky header (title, `StatusPill`, `TupleBadge`), subject summary, latest report via `ReportReader`, `CrweCoverageMatrix`, ElementReviews grouped by criterion (collapsed), challenges section, audit trail via `FactTimeline` (collapsed, "Advanced").
2. Actions rail using `GatedAction`: Submit ElementReview, Review this criterion, Start/Join public episode, Submit SynthesisReview, Commission deeper audit, Challenge this review, Petition to feature, Petition CRWE change, Save to library.
3. Citable/shareable: anchor links per section and per review, copy-citation control, stable permalinks. "Reports are the destination."

Exit criteria: page is readable top-to-bottom as a report by an anonymous visitor; every memo action is present (gated where identity is required).

### F4. Auth flows and submission UX

Depends on B2; submission depends on B5 items 3.

1. Real sign-in page against `/api/auth/*`; session context provider; navbar reflects auth state; backstage nav sections appear per role.
2. Submit ElementReview flow: choose criterion (preselected when entered from a CRWE row), view existing reviews for that criterion, structured form (finding, severity, confidence, limitations, recommendations, content, optional citations), submit as unsolicited. Preserve selection through the login redirect.
3. Sponsor Console and Reviewer Queue: restore real content behind roles (commissioned audits with delivery state; assigned solicitations with briefs, due dates, compensation state, submission status).

Exit criteria: end-to-end: anonymous visitor clicks Review one criterion -> signs in -> lands back on the prefilled form -> submits -> review appears on the subject page.

### F5. Challenges and petitions UI

Depends on B4, B5.

1. Challenge thread component: challenged artifact preserved and rendered, contestation with scope/rationale, responses, superseded markers linking to replacements.
2. Petition surfaces: petition-to-feature on each ElementReview, CRWE petition entry points on `/browse` rows and the coverage matrix, petition lists on subject pages.
3. Challenge counts/badges on cards and Discover filters (`Challenged` status).

### F6. Progressive disclosure and client-side tuple recomputation

Depends on B1 (pure function), B2 (tags for community filters).

1. App-wide Advanced toggle (persisted in `localStorage`): swaps friendly tuple labels for symbolic notation, reveals provenance detail, raw fact payloads, and membership records.
2. "Recompute as..." panel on subject/episode pages: reviewer-community filter and `T_eval` slider, recomputing the tuple live. Implementation options: (a) call the parameterized B1 endpoint per change — ship this first; (b) mirror the pure function in TS or compile to WASM for instant client-side recomputation — later polish. Watching the tuple change with `T_eval` is the clearest possible demonstration that it is a derived view, not a score.

### F7. Seed data and polish

Last; everything teaches through artifacts, so artifacts must exist.

1. Enrich `db/seeds`: 2–3 fully-worked public audits (multi-criterion ElementReviews with varied findings/severities, a sectioned SynthesisReview, one challenge thread, one superseded review, one petition), plus several partially-audited and unaudited works so every status label appears in Discover.
2. Empty states reviewed everywhere; loading states; mobile pass on the public shell; accessibility pass on `TupleBadge` and `CrweCoverageMatrix` (color is never the only signal).

---

## Suggested Execution Order (interleaved)

| Step | Work | Why now |
|------|------|---------|
| 1 | B0 + F0 + F2 in parallel | Foundations; F0/F2 have no backend dependency |
| 2 | B1, then B3 | Pure tuple, then the summary API that exposes it |
| 3 | F1, then F3 | Thin data layer, then the flagship page |
| 4 | B2, then F4 | Identity unblocks all participation |
| 5 | B4, then B5 | Challenges, then participation/petitions |
| 6 | F5 | Challenge/petition UI on live APIs |
| 7 | F6 | Advanced mode + recomputation demo |
| 8 | B6 + F7 | Crate split when types are stable; seed/polish last |

Each step ends at a shippable state: after step 3 the public registry is fast, memo-shaped, and report-centric even with auth still stubbed; after step 4 participation works; after step 6 the full memo surface exists.

## Decisions To Confirm Before Starting

These are now resolved as implemented:

1. Auth mechanism for B2: **magic-link** (chosen). Password path not built.
2. Featured-review mechanism in B5: **`CurationDecision` fact** (chosen), with petitions as input. Implemented as the `CurationDecision` `FactPayload` variant.
3. Whether `EpisodeParticipation`, `FeaturePetition`, `CWEPetition` (and `CurationDecision`) belong in the FEN schema document itself — **resolved.** `FEN_Schema_for_CSQD_GTM.tex` already defines all four variants in the `FactPayload` enum, field-for-field with `crates/domain/src/fact.rs`, along with the supporting enums and the `Participation`/`Petition`/`Curation` membership roles. Schema source and code are in sync. (Open only: re-render `FEN_for_CSQD_GTM.pdf` from the `.tex` if it predates these variants, since the rendered PDF outranks the tex in source precedence.)
