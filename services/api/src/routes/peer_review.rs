//! Academic Publishing adapter routes.
//!
//! Everything scholarly — article retrieval, scholarly objects, the user
//! library, CRWE problem-area browsing, and scholarly-object-keyed public
//! summaries — lives here, mirroring the `csqd-academic-adapter` crate.
//! The substrate routes in `super` never depend on this module's types.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use csqd_academic_adapter::{
    ArticleAccessSummary, ArticleRetrievalResult, ArticleRetrievalSet, ClaimAuditIndexEntry,
    EvidenceArtifactSummary, LibraryItemSummary, ProblemAreaWorkSummary, ScholarlyObjectDetail,
    ScholarlyObjectSummary, WorkAuditInvolvement,
};
use csqd_domain::{AttachEvidenceArtifactRequest, Principal};
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::ApiError,
    repositories::{
        article_access, article_retrieval, claim_audits, doi_retrieval, evidence_artifacts,
        public_summary, pubmed_retrieval, scholarly_objects, title_retrieval, user_library,
    },
    state::AppState,
};

use super::require_session;

#[derive(Debug, Deserialize)]
struct ArticleRetrievalQuery {
    query: String,
    include_preprints: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct WorkSearchQuery {
    query: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProblemAreaBrowseQuery {
    query: Option<String>,
    cwe_node_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AddLibraryItemRequest {
    scholarly_object_id: String,
}

#[derive(Debug, Deserialize)]
struct SummaryBatchQuery {
    /// Comma-separated scholarly object ids.
    ids: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/article-retrieval/arxiv", get(retrieve_arxiv_article))
        .route("/api/article-retrieval/doi", get(retrieve_doi_article))
        .route(
            "/api/article-retrieval/pubmed",
            get(retrieve_pubmed_article),
        )
        .route("/api/article-retrieval/title", get(retrieve_title_article))
        .route(
            "/api/library-items",
            get(list_library_items).post(add_library_item),
        )
        .route(
            "/api/peer-review/problem-area-works",
            get(browse_problem_area_works),
        )
        .route("/api/claim-audits", get(list_claim_audits))
        .route("/api/scholarly-objects", get(list_scholarly_objects))
        .route("/api/work-search", get(search_work_summaries))
        .route(
            "/api/scholarly-objects/:id/article-access",
            get(get_article_access),
        )
        .route("/api/scholarly-objects/:id", get(get_scholarly_object))
        .route(
            "/api/scholarly-objects/:id/audit-involvements",
            get(list_work_audit_involvements),
        )
        .route(
            "/api/audit-episodes/:id/evidence-artifacts",
            get(list_episode_evidence_artifacts).post(attach_episode_evidence_artifact),
        )
        .route(
            "/api/audit-episodes/:id/evidence-artifacts/:artifact_id/retract",
            post(retract_episode_evidence_artifact),
        )
        .route(
            "/api/public/scholarly-objects/:id/summary",
            get(get_public_scholarly_object_summary),
        )
        .route(
            "/api/public/scholarly-objects/summaries",
            get(get_public_scholarly_object_summaries),
        )
}

// ── Article retrieval ───────────────────────────────────────────

async fn retrieve_arxiv_article(
    State(state): State<AppState>,
    Query(query): Query<ArticleRetrievalQuery>,
) -> Result<Json<ArticleRetrievalResult>, ApiError> {
    let result = article_retrieval::retrieve_arxiv(&state.db, &query.query).await?;

    Ok(Json(result))
}

async fn retrieve_doi_article(
    State(state): State<AppState>,
    Query(query): Query<ArticleRetrievalQuery>,
) -> Result<Json<ArticleRetrievalResult>, ApiError> {
    let result = doi_retrieval::retrieve_doi(&state.db, &query.query).await?;

    Ok(Json(result))
}

async fn retrieve_pubmed_article(
    State(state): State<AppState>,
    Query(query): Query<ArticleRetrievalQuery>,
) -> Result<Json<ArticleRetrievalResult>, ApiError> {
    let result = pubmed_retrieval::retrieve_pubmed(&state.db, &query.query).await?;

    Ok(Json(result))
}

async fn retrieve_title_article(
    State(state): State<AppState>,
    Query(query): Query<ArticleRetrievalQuery>,
) -> Result<Json<ArticleRetrievalSet>, ApiError> {
    let result = title_retrieval::retrieve_title(
        &state.db,
        &query.query,
        query.include_preprints.unwrap_or(false),
    )
    .await?;

    Ok(Json(result))
}

// ── Library ─────────────────────────────────────────────────────

async fn list_library_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<LibraryItemSummary>>, ApiError> {
    let items = user_library::list_items(&state.db).await?;

    Ok(Json(items))
}

async fn add_library_item(
    State(state): State<AppState>,
    Json(request): Json<AddLibraryItemRequest>,
) -> Result<Json<LibraryItemSummary>, ApiError> {
    let item = user_library::add_scholarly_object(&state.db, &request.scholarly_object_id).await?;

    Ok(Json(item))
}

// ── Scholarly objects ───────────────────────────────────────────

async fn list_scholarly_objects(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScholarlyObjectSummary>>, ApiError> {
    let objects = scholarly_objects::list_summaries(&state.db).await?;

    Ok(Json(objects))
}

async fn search_work_summaries(
    State(state): State<AppState>,
    Query(query): Query<WorkSearchQuery>,
) -> Result<Json<Vec<ScholarlyObjectSummary>>, ApiError> {
    let objects =
        scholarly_objects::search_summaries(&state.db, query.query.as_deref().unwrap_or_default())
            .await?;

    Ok(Json(objects))
}

async fn browse_problem_area_works(
    State(state): State<AppState>,
    Query(query): Query<ProblemAreaBrowseQuery>,
) -> Result<Json<Vec<ProblemAreaWorkSummary>>, ApiError> {
    let works = scholarly_objects::browse_problem_area_works(
        &state.db,
        query.query.as_deref().unwrap_or_default(),
        query.cwe_node_id.as_deref(),
    )
    .await?;

    Ok(Json(works))
}

async fn get_scholarly_object(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ScholarlyObjectDetail>, ApiError> {
    let object = scholarly_objects::find_detail(&state.db, &id).await?;

    Ok(Json(object))
}

async fn get_article_access(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ArticleAccessSummary>, ApiError> {
    let access = article_access::find_for_scholarly_object(&state.db, &id).await?;

    Ok(Json(access))
}

// ── Evidence artifacts (claim-scoped audits memo) ───────────────

/// Every audit that involves this work — as subject or as attached evidence.
/// Keeps the paper page a first-class discovery surface without making the
/// paper the audit's epistemic target.
async fn list_work_audit_involvements(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<WorkAuditInvolvement>>, ApiError> {
    let involvements =
        evidence_artifacts::list_involvements_for_scholarly_object(&state.db, &id).await?;

    Ok(Json(involvements))
}

async fn list_claim_audits(
    State(state): State<AppState>,
) -> Result<Json<Vec<ClaimAuditIndexEntry>>, ApiError> {
    let entries = claim_audits::list_index(&state.db).await?;

    Ok(Json(entries))
}

async fn list_episode_evidence_artifacts(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<EvidenceArtifactSummary>>, ApiError> {
    let artifacts = evidence_artifacts::list_summaries_for_episode(&state.db, &id).await?;

    Ok(Json(artifacts))
}

async fn attach_episode_evidence_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(mut request): Json<AttachEvidenceArtifactRequest>,
) -> Result<Json<EvidenceArtifactSummary>, ApiError> {
    let session = require_session(&state, &headers).await?;
    request.attached_by = Some(Principal::User {
        user_id: session.user_id.clone(),
    });

    let artifact = evidence_artifacts::attach(&state.db, &id, request).await?;

    Ok(Json(artifact))
}

async fn retract_episode_evidence_artifact(
    State(state): State<AppState>,
    Path((id, artifact_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let session = require_session(&state, &headers).await?;

    evidence_artifacts::retract(
        &state.db,
        &id,
        &artifact_id,
        Principal::User {
            user_id: session.user_id.clone(),
        },
    )
    .await?;

    Ok(Json(json!({ "retracted": true })))
}

// ── Public summaries keyed by scholarly object ──────────────────

async fn get_public_scholarly_object_summary(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<public_summary::PublicSubjectSummary>, ApiError> {
    let summary = public_summary::summary_for_scholarly_object(&state.db, &id).await?;

    Ok(Json(summary))
}

async fn get_public_scholarly_object_summaries(
    State(state): State<AppState>,
    Query(query): Query<SummaryBatchQuery>,
) -> Result<Json<Vec<public_summary::PublicSubjectSummary>>, ApiError> {
    let ids: Vec<String> = query
        .ids
        .split(',')
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect();

    if ids.len() > 100 {
        return Err(ApiError::Domain(
            "at most 100 summaries per request".to_string(),
        ));
    }

    let summaries = public_summary::summaries_for_scholarly_objects(&state.db, &ids).await?;

    Ok(Json(summaries))
}
