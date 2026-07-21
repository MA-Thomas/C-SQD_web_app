use axum::http::{HeaderValue, Method};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{AppendHeaders, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use csqd_domain::{
    sort_timeline, ApiHealth, AuditEpisode, AuditEpisodeSummary, AuditSubject, CWENodeId,
    CWEPetitionKind, CommissionAuditEpisodeRequest, CommissionAuditEpisodeResult,
    CreateAuditSubjectRequest, CreateEpisodeElementReviewRequest, CreateEpisodeRelationRequest,
    CreateEpisodeSolicitationEventRequest, CreateEpisodeSolicitationRequest,
    CreateEpisodeWarrantRequest, CreateInvoiceIssuedRequest, CreatePaymentReceivedRequest,
    CreateReviewerPayoutRequest, CreateSynthesisReviewRelationRequest,
    CreateSynthesisReviewRequest, CurationOutcome, CurationTarget, DomainInstantiationDetail,
    DomainInstantiationSummary, EpisodeRelation, EvalTuple, Fact, FactId, FactResponseType,
    Principal, ReviewerProfile, Role, SessionUser, SynthesisReview, SynthesisReviewRelation,
    TimelineEntry, User,
};
use serde::Deserialize;
use serde_json::json;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    error::ApiError,
    repositories::{
        audit_episodes, audit_subjects, auth, commission_inquiries, domain_instantiations,
        public_summary, relations, users,
    },
    state::AppState,
};

/// Academic Publishing adapter routes (scholarly objects, article retrieval,
/// library, problem-area browsing) — mirrors the substrate/adapter crate
/// split. URL shapes are unchanged.
pub mod peer_review;

const SESSION_COOKIE: &str = "csqd_session";

#[derive(Debug, Deserialize)]
struct EvalTupleParams {
    t_eval: Option<DateTime<Utc>>,
    /// Comma-separated reviewer community tags.
    tags: Option<String>,
    min_endorsements: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct RequestMagicLinkBody {
    email: String,
}

#[derive(Debug, Deserialize)]
struct CompleteMagicLinkBody {
    token: String,
}

#[derive(Debug, Deserialize)]
struct StartPublicEpisodeBody {
    label: String,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JoinPublicEpisodeBody {
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SubmitterResponseBody {
    responding_to: Vec<FactId>,
    response_type: FactResponseType,
    content: String,
}

#[derive(Debug, Deserialize)]
struct FeaturePetitionBody {
    element_review: FactId,
    rationale: String,
}

#[derive(Debug, Deserialize)]
struct CWEPetitionBody {
    kind: CWEPetitionKind,
    cwe_node: Option<CWENodeId>,
    proposed_label: Option<String>,
    rationale: String,
}

#[derive(Debug, Deserialize)]
struct UpdateDisplayNameBody {
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct SetRolesBody {
    roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CurationDecisionBody {
    target: CurationTarget,
    decision: CurationOutcome,
    rationale: Option<String>,
    #[serde(default)]
    petitions: Vec<FactId>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/auth/request-link", post(request_magic_link))
        .route("/api/auth/complete", post(complete_magic_link))
        .route("/api/auth/session", get(get_session))
        .route("/api/auth/sign-out", post(sign_out))
        .route("/api/users/:id", get(get_user))
        .route("/api/account/display-name", post(update_own_display_name))
        .route("/api/admin/accounts", get(list_accounts))
        .route("/api/admin/accounts/:id/roles", post(set_account_roles))
        .route("/api/reviewer-profiles/:user_id", get(get_reviewer_profile))
        .route(
            "/api/audit-subjects",
            get(list_audit_subjects).post(create_audit_subject),
        )
        .route("/api/audit-subjects/:id", get(get_audit_subject))
        .route(
            "/api/audit-subjects/:id/audit-episodes",
            get(list_audit_episodes_for_subject).post(commission_audit_episode),
        )
        .route("/api/audit-subjects/:id/facts", get(list_facts_for_subject))
        .route(
            "/api/audit-subjects/:id/public-episodes",
            post(start_public_episode),
        )
        .route("/api/audit-episodes", get(list_audit_episodes))
        .route("/api/audit-episodes/:id", get(get_audit_episode))
        .route("/api/audit-episodes/:id/facts", get(list_facts_for_episode))
        .route(
            "/api/audit-episodes/:id/timeline",
            get(get_episode_timeline),
        )
        .route("/api/audit-episodes/:id/join", post(join_public_episode))
        .route(
            "/api/audit-episodes/:id/facts/element-review",
            post(create_episode_element_review),
        )
        .route(
            "/api/audit-episodes/:id/facts/warrant",
            post(create_episode_warrant),
        )
        .route(
            "/api/audit-episodes/:id/facts/solicitation",
            post(create_episode_solicitation),
        )
        .route(
            "/api/audit-episodes/:id/facts/solicitation-event",
            post(create_episode_solicitation_event),
        )
        .route(
            "/api/audit-episodes/:id/facts/submitter-response",
            post(create_submitter_response),
        )
        .route(
            "/api/audit-episodes/:id/facts/feature-petition",
            post(create_feature_petition),
        )
        .route(
            "/api/audit-episodes/:id/facts/cwe-petition",
            post(create_cwe_petition),
        )
        .route(
            "/api/audit-episodes/:id/facts/curation-decision",
            post(create_curation_decision),
        )
        .route(
            "/api/audit-episodes/:id/facts/invoice-issued",
            post(create_invoice_issued),
        )
        .route(
            "/api/audit-episodes/:id/facts/payment-received",
            post(create_payment_received),
        )
        .route(
            "/api/audit-episodes/:id/facts/reviewer-payout",
            post(create_reviewer_payout),
        )
        .route(
            "/api/audit-episodes/:id/relations",
            get(list_episode_relations).post(create_episode_relation),
        )
        .route(
            "/api/audit-episodes/:id/synthesis-reviews",
            get(list_synthesis_reviews).post(create_synthesis_review),
        )
        .route("/api/audit-episodes/:id/eval-tuple", get(get_eval_tuple))
        .route(
            "/api/synthesis-reviews/:id/relations",
            get(list_synthesis_relations).post(create_synthesis_relation),
        )
        .route(
            "/api/commission-inquiries",
            get(list_commission_inquiries).post(create_commission_inquiry),
        )
        .route(
            "/api/commission-inquiries/:id/status",
            post(update_commission_inquiry),
        )
        .route(
            "/api/public/audit-subjects/summaries",
            get(get_public_subject_summaries),
        )
        .route(
            "/api/public/audit-subjects/:id/summary",
            get(get_public_subject_summary),
        )
        .route(
            "/api/domain-instantiations",
            get(list_domain_instantiations),
        )
        .route(
            "/api/domain-instantiations/:id",
            get(get_domain_instantiation),
        )
        .merge(peer_review::router())
        .layer(cors_layer(&state))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Credentialed CORS for the web app origin (session cookies cannot flow
/// through a wildcard policy).
fn cors_layer(state: &AppState) -> CorsLayer {
    let origin = state
        .config
        .web_base_url
        .parse::<HeaderValue>()
        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3000"));

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(true)
}

// ── Session helpers ─────────────────────────────────────────────

fn session_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;

    cookies.split(';').find_map(|pair| {
        let (name, value) = pair.trim().split_once('=')?;

        (name == SESSION_COOKIE).then(|| value.to_string())
    })
}

async fn current_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<SessionUser>, ApiError> {
    match session_token_from_headers(headers) {
        Some(token) => Ok(auth::session_user_for_token(&state.db, &token).await?),
        None => Ok(None),
    }
}

async fn require_session(state: &AppState, headers: &HeaderMap) -> Result<SessionUser, ApiError> {
    current_session(state, headers)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("sign in to perform this action".to_string()))
}

fn require_role(session: &SessionUser, role: Role) -> Result<(), ApiError> {
    auth::require_role(session, role).map_err(ApiError::from)
}

fn session_cookie(token: &str, max_age_seconds: i64, secure: bool) -> String {
    let secure_attr = if secure { "; Secure" } else { "" };

    format!(
        "{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_seconds}{secure_attr}"
    )
}

// ── Auth handlers ───────────────────────────────────────────────

async fn request_magic_link(
    State(state): State<AppState>,
    Json(body): Json<RequestMagicLinkBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let link = auth::request_magic_link(&state.db, &body.email).await?;
    let sign_in_url = format!(
        "{}/sign-in/complete?token={}",
        state.config.web_base_url.trim_end_matches('/'),
        link.token
    );

    // The link is always logged so an operator can hand-deliver it until
    // email delivery is wired up. It is returned in the response ONLY in
    // dev-auth mode — in any shared deployment that would let anyone sign
    // in as any address.
    tracing::info!(email = %link.email, %sign_in_url, "magic sign-in link issued");

    if state.config.dev_auth {
        Ok(Json(json!({
            "email": link.email,
            "expires_at": link.expires_at,
            "sign_in_url": sign_in_url,
        })))
    } else {
        crate::mailer::send_magic_link(&state, &link.email, &sign_in_url).await;

        Ok(Json(json!({
            "email": link.email,
            "expires_at": link.expires_at,
        })))
    }
}

async fn complete_magic_link(
    State(state): State<AppState>,
    Json(body): Json<CompleteMagicLinkBody>,
) -> Result<impl IntoResponse, ApiError> {
    let session = auth::complete_magic_link(&state.db, &body.token).await?;
    let max_age = (session.expires_at - Utc::now()).num_seconds().max(0);
    let cookie = session_cookie(&session.token, max_age, state.config.secure_cookies);

    Ok((
        AppendHeaders([(header::SET_COOKIE, cookie)]),
        Json(json!({ "user": session.user })),
    ))
}

async fn get_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let session = current_session(&state, &headers).await?;

    Ok(Json(json!({ "user": session })))
}

async fn sign_out(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(token) = session_token_from_headers(&headers) {
        auth::revoke_session(&state.db, &token).await?;
    }

    Ok((
        AppendHeaders([(
            header::SET_COOKIE,
            session_cookie("", 0, state.config.secure_cookies),
        )]),
        Json(json!({ "user": null })),
    ))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<User>, ApiError> {
    let user = users::find(&state.db, &id).await?;

    Ok(Json(user))
}

/// Self-service display-name update (account page / onboarding).
async fn update_own_display_name(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateDisplayNameBody>,
) -> Result<Json<User>, ApiError> {
    let session = require_session(&state, &headers).await?;
    let user =
        users::update_display_name(&state.db, session.user_id.as_str(), &body.display_name).await?;

    Ok(Json(user))
}

// ── Account administration (operator-only) ──────────────────────

async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<users::AccountSummary>>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let accounts = users::list_accounts(&state.db).await?;

    Ok(Json(accounts))
}

async fn set_account_roles(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<SetRolesBody>,
) -> Result<Json<users::AccountSummary>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    // Operators cannot silently drop their own operator role — a soft
    // guard against locking the last operator out.
    if id == session.user_id.as_str() && !body.roles.iter().any(|role| role == "operator") {
        return Err(ApiError::Forbidden(
            "you cannot remove your own operator role".to_string(),
        ));
    }

    let account = users::set_roles(&state.db, &id, &body.roles).await?;

    Ok(Json(account))
}

async fn get_reviewer_profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<ReviewerProfile>, ApiError> {
    let profile = users::find_reviewer_profile(&state.db, &user_id).await?;

    Ok(Json(profile))
}

async fn health() -> Json<ApiHealth> {
    Json(ApiHealth {
        service: "csqd-api".to_string(),
        status: "ok".to_string(),
    })
}

// ── Domains ─────────────────────────────────────────────────────

async fn list_domain_instantiations(
    State(state): State<AppState>,
) -> Result<Json<Vec<DomainInstantiationSummary>>, ApiError> {
    let domains = domain_instantiations::list_summaries(&state.db).await?;

    Ok(Json(domains))
}

async fn get_domain_instantiation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DomainInstantiationDetail>, ApiError> {
    let domain = domain_instantiations::find_detail(&state.db, &id).await?;

    Ok(Json(domain))
}

// ── Audit subjects ──────────────────────────────────────────────

async fn list_audit_subjects(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditSubject>>, ApiError> {
    let subjects = audit_subjects::list(&state.db).await?;

    Ok(Json(subjects))
}

/// Registering an audit subject is a durable write to the public audit
/// graph; it requires an identified session.
async fn create_audit_subject(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateAuditSubjectRequest>,
) -> Result<Json<AuditSubject>, ApiError> {
    require_session(&state, &headers).await?;

    let subject = audit_subjects::create(&state.db, request).await?;

    Ok(Json(subject))
}

async fn get_audit_subject(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AuditSubject>, ApiError> {
    let subject = audit_subjects::find(&state.db, &id).await?;

    Ok(Json(subject))
}

// ── Episodes ────────────────────────────────────────────────────

async fn list_audit_episodes_for_subject(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<AuditEpisode>>, ApiError> {
    let episodes = audit_episodes::list_for_subject(&state.db, &id).await?;

    Ok(Json(episodes))
}

/// Commissioning creates a sponsor organization, an episode, a commission
/// fact, and a membership — the most consequential write in the system.
/// It requires an identified session; the commission form stays public up
/// to submission, but the submitter must be signed in.
async fn commission_audit_episode(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CommissionAuditEpisodeRequest>,
) -> Result<Json<CommissionAuditEpisodeResult>, ApiError> {
    require_session(&state, &headers).await?;

    let result = audit_episodes::commission_for_subject(&state.db, &id, request).await?;

    Ok(Json(result))
}

async fn start_public_episode(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<StartPublicEpisodeBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let session = require_session(&state, &headers).await?;
    let (episode, fact) = audit_episodes::start_public_episode(
        &state.db,
        &id,
        &session.user_id,
        &body.label,
        body.notes,
    )
    .await?;

    Ok(Json(
        json!({ "episode": episode, "participation_fact": fact }),
    ))
}

async fn join_public_episode(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<JoinPublicEpisodeBody>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    let fact =
        audit_episodes::join_public_episode(&state.db, &id, &session.user_id, body.note).await?;

    Ok(Json(fact))
}

async fn list_audit_episodes(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditEpisodeSummary>>, ApiError> {
    let episodes = audit_episodes::list_summaries(&state.db).await?;

    Ok(Json(episodes))
}

async fn get_audit_episode(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AuditEpisode>, ApiError> {
    let episode = audit_episodes::find(&state.db, &id).await?;

    Ok(Json(episode))
}

async fn list_facts_for_subject(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Fact>>, ApiError> {
    let facts = audit_episodes::list_facts_for_subject(&state.db, &id).await?;

    Ok(Json(facts))
}

async fn list_facts_for_episode(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Fact>>, ApiError> {
    let facts = audit_episodes::list_facts_for_episode(&state.db, &id).await?;

    Ok(Json(facts))
}

/// Interleaved audit timeline: facts, memberships, synthesis reviews, and
/// relations on one `Temporal`-sorted axis.
async fn get_episode_timeline(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<TimelineEntry>>, ApiError> {
    let memberships = audit_episodes::list_fact_memberships_for_episode(&state.db, &id).await?;
    let reviews = audit_episodes::list_synthesis_reviews(&state.db, &id).await?;
    let episode_relations = relations::list_episode_relations(&state.db, &id).await?;
    let mut entries: Vec<TimelineEntry> = Vec::new();

    for (fact, membership) in memberships {
        entries.push(TimelineEntry::Fact { fact });
        entries.push(TimelineEntry::Membership { membership });
    }

    for review in reviews {
        for relation in
            relations::list_synthesis_relations_for_review(&state.db, review.id.as_str()).await?
        {
            entries.push(TimelineEntry::SynthesisRelation { relation });
        }

        entries.push(TimelineEntry::SynthesisReview { review });
    }

    for relation in episode_relations {
        entries.push(TimelineEntry::EpisodeRelation { relation });
    }

    sort_timeline(&mut entries);

    Ok(Json(entries))
}

// ── Episode facts ───────────────────────────────────────────────

async fn create_episode_element_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(mut request): Json<CreateEpisodeElementReviewRequest>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;

    // The session identifies the reviewer; explicit overrides are an
    // operator affordance.
    match &request.submitted_by {
        Some(submitted_by) if submitted_by != &session.user_id => {
            require_role(&session, Role::Operator)?;
        }
        _ => {
            request.submitted_by = Some(session.user_id.clone());
        }
    }

    let fact = audit_episodes::create_element_review_fact(&state.db, &id, request).await?;

    crate::mailer::notify_review_submitted(&state, &id).await;

    Ok(Json(fact))
}

/// Warrant assertions record why an attached evidence artifact is supposed
/// to bear on the target claim (claim-scoped audits memo). Any signed-in
/// participant may assert one; overriding the author is an operator
/// affordance, mirroring element reviews.
async fn create_episode_warrant(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(mut request): Json<CreateEpisodeWarrantRequest>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;

    match &request.asserted_by {
        Some(asserted_by) if asserted_by != &session.user_id => {
            require_role(&session, Role::Operator)?;
        }
        _ => {
            request.asserted_by = Some(session.user_id.clone());
        }
    }

    let fact = audit_episodes::create_warrant_fact(&state.db, &id, request).await?;

    Ok(Json(fact))
}

async fn create_episode_solicitation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreateEpisodeSolicitationRequest>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let issued_to = request.issued_to.clone();
    let fact = audit_episodes::create_solicitation_fact(&state.db, &id, request).await?;

    // Notify the solicited reviewer by email when we can resolve one.
    if let Some(user_id) = issued_to {
        if let Ok(user) = users::find(&state.db, user_id.as_str()).await {
            crate::mailer::notify_solicitation(&state, &user.email, &id).await;
        }
    }

    Ok(Json(fact))
}

async fn create_episode_solicitation_event(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreateEpisodeSolicitationEventRequest>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let fact = audit_episodes::create_solicitation_event_fact(&state.db, &id, request).await?;

    Ok(Json(fact))
}

async fn create_submitter_response(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<SubmitterResponseBody>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    let fact = audit_episodes::create_submitter_response_fact(
        &state.db,
        &id,
        &session.user_id,
        body.responding_to,
        body.response_type,
        body.content,
    )
    .await?;

    Ok(Json(fact))
}

async fn create_feature_petition(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<FeaturePetitionBody>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    let fact = audit_episodes::create_feature_petition_fact(
        &state.db,
        &id,
        &session.user_id,
        body.element_review,
        body.rationale,
    )
    .await?;

    Ok(Json(fact))
}

async fn create_cwe_petition(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<CWEPetitionBody>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    let fact = audit_episodes::create_cwe_petition_fact(
        &state.db,
        &id,
        &session.user_id,
        body.kind,
        body.cwe_node,
        body.proposed_label,
        body.rationale,
    )
    .await?;

    Ok(Json(fact))
}

async fn create_curation_decision(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<CurationDecisionBody>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let fact = audit_episodes::create_curation_decision_fact(
        &state.db,
        &id,
        Principal::User {
            user_id: session.user_id.clone(),
        },
        body.target,
        body.decision,
        body.rationale,
        body.petitions,
    )
    .await?;

    Ok(Json(fact))
}

// ── Commercial lifecycle facts (operator-only) ──────────────────
//
// Money movement is recorded on the audit record as administrative facts.
// These never affect the evaluation tuple; an episode counts as funded
// once an active payment_received fact exists.

async fn create_invoice_issued(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreateInvoiceIssuedRequest>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let fact = audit_episodes::create_invoice_issued_fact(
        &state.db,
        &id,
        Principal::User {
            user_id: session.user_id.clone(),
        },
        request,
    )
    .await?;

    Ok(Json(fact))
}

async fn create_payment_received(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreatePaymentReceivedRequest>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let fact = audit_episodes::create_payment_received_fact(
        &state.db,
        &id,
        Principal::User {
            user_id: session.user_id.clone(),
        },
        request,
    )
    .await?;

    Ok(Json(fact))
}

async fn create_reviewer_payout(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreateReviewerPayoutRequest>,
) -> Result<Json<Fact>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let fact = audit_episodes::create_reviewer_payout_fact(
        &state.db,
        &id,
        Principal::User {
            user_id: session.user_id.clone(),
        },
        request,
    )
    .await?;

    Ok(Json(fact))
}

// ── Relations ───────────────────────────────────────────────────

async fn list_episode_relations(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<EpisodeRelation>>, ApiError> {
    let relations = relations::list_episode_relations(&state.db, &id).await?;

    Ok(Json(relations))
}

async fn create_episode_relation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(mut request): Json<CreateEpisodeRelationRequest>,
) -> Result<Json<EpisodeRelation>, ApiError> {
    let session = require_session(&state, &headers).await?;

    if request.asserted_by.is_none() {
        request.asserted_by = Some(Principal::User {
            user_id: session.user_id.clone(),
        });
    }

    let relation = relations::create_episode_relation(&state.db, &id, request).await?;

    Ok(Json(relation))
}

async fn list_synthesis_relations(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<SynthesisReviewRelation>>, ApiError> {
    let relations = relations::list_synthesis_relations_for_review(&state.db, &id).await?;

    Ok(Json(relations))
}

async fn create_synthesis_relation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(mut request): Json<CreateSynthesisReviewRelationRequest>,
) -> Result<Json<SynthesisReviewRelation>, ApiError> {
    let session = require_session(&state, &headers).await?;

    if request.asserted_by.is_none() {
        request.asserted_by = Some(Principal::User {
            user_id: session.user_id.clone(),
        });
    }

    let relation = relations::create_synthesis_relation(&state.db, &id, request).await?;

    Ok(Json(relation))
}

// ── Synthesis reviews ───────────────────────────────────────────

async fn list_synthesis_reviews(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<SynthesisReview>>, ApiError> {
    let reviews = audit_episodes::list_synthesis_reviews(&state.db, &id).await?;

    Ok(Json(reviews))
}

async fn create_synthesis_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(mut request): Json<CreateSynthesisReviewRequest>,
) -> Result<Json<SynthesisReview>, ApiError> {
    let session = require_session(&state, &headers).await?;

    match &request.submitted_by {
        Some(submitted_by) if submitted_by != &session.user_id => {
            require_role(&session, Role::Operator)?;
        }
        _ => {
            request.submitted_by = Some(session.user_id.clone());
        }
    }

    let review = audit_episodes::create_synthesis_review(&state.db, &id, request).await?;

    Ok(Json(review))
}

// ── Evaluation tuple ────────────────────────────────────────────

async fn get_eval_tuple(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<EvalTupleParams>,
) -> Result<Json<EvalTuple>, ApiError> {
    let query = audit_episodes::EvalTupleQuery {
        t_eval: params.t_eval,
        tags: params
            .tags
            .map(|tags| {
                tags.split(',')
                    .map(|tag| tag.trim().to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        min_endorsements: params.min_endorsements,
    };
    let eval_tuple = audit_episodes::compute_eval_tuple(&state.db, &id, query).await?;

    Ok(Json(eval_tuple))
}

// ── Commission inquiries (two-stage intake) ─────────────────────

/// Stage one, public: a short inquiry, not a commission. No session
/// required — the barrier to *asking* stays low; the barrier to entering
/// the audit graph stays at stage two.
async fn create_commission_inquiry(
    State(state): State<AppState>,
    Json(request): Json<commission_inquiries::CreateCommissionInquiryRequest>,
) -> Result<Json<commission_inquiries::CommissionInquiry>, ApiError> {
    let inquiry = commission_inquiries::create(&state.db, request).await?;

    // Surface new inquiries to the operator inbox via the mailer when one
    // is configured; the inquiry is durably recorded either way.
    crate::mailer::notify_new_inquiry(&state, &inquiry).await;

    Ok(Json(inquiry))
}

async fn list_commission_inquiries(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<commission_inquiries::CommissionInquiry>>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let inquiries = commission_inquiries::list(&state.db).await?;

    Ok(Json(inquiries))
}

async fn update_commission_inquiry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<commission_inquiries::UpdateCommissionInquiryRequest>,
) -> Result<Json<commission_inquiries::CommissionInquiry>, ApiError> {
    let session = require_session(&state, &headers).await?;
    require_role(&session, Role::Operator)?;

    let inquiry = commission_inquiries::update_status(&state.db, &id, request).await?;

    Ok(Json(inquiry))
}

// ── Public summaries ────────────────────────────────────────────

async fn get_public_subject_summary(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<public_summary::PublicSubjectSummary>, ApiError> {
    let summary = public_summary::summary_for_audit_subject(&state.db, &id).await?;

    Ok(Json(summary))
}

#[derive(Debug, Deserialize)]
struct SubjectSummariesParams {
    /// Comma-separated audit-subject ids.
    ids: Option<String>,
}

/// Batch variant of the public subject summary: one call returns the summaries
/// for several audit subjects, collapsing the Discover/home/Public Audits
/// fan-out into a single request.
async fn get_public_subject_summaries(
    State(state): State<AppState>,
    Query(params): Query<SubjectSummariesParams>,
) -> Result<Json<Vec<public_summary::PublicSubjectSummary>>, ApiError> {
    let ids: Vec<String> = params
        .ids
        .map(|ids| {
            ids.split(',')
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let summaries = public_summary::summaries_for_audit_subjects(&state.db, &ids).await?;

    Ok(Json(summaries))
}

// Unused status import guard: keep StatusCode referenced for future handlers.
#[allow(dead_code)]
fn _status() -> StatusCode {
    StatusCode::OK
}
