use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use csqd_domain::{
    ApiHealth, ArticleAccessSummary, ArticleRetrievalResult, ArticleRetrievalSet,
    AuditObjectDetail, AuditObjectSummary, DomainInstantiationDetail, DomainInstantiationSummary,
    LibraryItemSummary, ReviewAssignmentSummary, ScholarlyObjectDetail, ScholarlyObjectSummary,
};
use serde::Deserialize;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    error::ApiError,
    repositories::{
        article_access, article_retrieval, audit_objects, doi_retrieval, domain_instantiations,
        pubmed_retrieval, review_assignments, scholarly_objects, title_retrieval, user_library,
    },
    state::AppState,
};

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
struct AddLibraryItemRequest {
    scholarly_object_id: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/article-retrieval/arxiv", get(retrieve_arxiv_article))
        .route("/api/article-retrieval/doi", get(retrieve_doi_article))
        .route(
            "/api/article-retrieval/pubmed",
            get(retrieve_pubmed_article),
        )
        .route("/api/article-retrieval/title", get(retrieve_title_article))
        .route("/api/audit-objects", get(list_audit_objects))
        .route("/api/audit-objects/:id", get(get_audit_object))
        .route(
            "/api/domain-instantiations",
            get(list_domain_instantiations),
        )
        .route(
            "/api/domain-instantiations/:id",
            get(get_domain_instantiation),
        )
        .route(
            "/api/library-items",
            get(list_library_items).post(add_library_item),
        )
        .route("/api/review-assignments", get(list_review_assignments))
        .route("/api/scholarly-objects", get(list_scholarly_objects))
        .route("/api/work-search", get(search_work_summaries))
        .route(
            "/api/scholarly-objects/:id/article-access",
            get(get_article_access),
        )
        .route("/api/scholarly-objects/:id", get(get_scholarly_object))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

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

async fn health() -> Json<ApiHealth> {
    Json(ApiHealth {
        service: "csqd-api".to_string(),
        status: "ok".to_string(),
    })
}

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

async fn list_review_assignments(
    State(state): State<AppState>,
) -> Result<Json<Vec<ReviewAssignmentSummary>>, ApiError> {
    let assignments = review_assignments::list_summaries(&state.db).await?;

    Ok(Json(assignments))
}

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

async fn list_audit_objects(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditObjectSummary>>, ApiError> {
    let objects = audit_objects::list_summaries(&state.db).await?;

    Ok(Json(objects))
}

async fn get_audit_object(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AuditObjectDetail>, ApiError> {
    let object = audit_objects::find_detail(&state.db, &id).await?;

    Ok(Json(object))
}

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
