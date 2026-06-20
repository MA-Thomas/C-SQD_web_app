use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use csqd_domain::{
    ApiHealth, ArticleAccessSummary, ArticleRetrievalResult, ArticleRetrievalSet, AuditEpisode,
    AuditEpisodeSummary, AuditSubject, CommissionAuditEpisodeRequest, CommissionAuditEpisodeResult,
    CreateAuditSubjectRequest, CreateEpisodeElementReviewRequest,
    CreateEpisodeSolicitationEventRequest, CreateEpisodeSolicitationRequest,
    CreateSynthesisReviewRequest, DomainInstantiationDetail, DomainInstantiationSummary, EvalTuple,
    Fact, LibraryItemSummary, ProblemAreaWorkSummary, ScholarlyObjectDetail,
    ScholarlyObjectSummary, SynthesisReview,
};
use serde::Deserialize;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    error::ApiError,
    repositories::{
        article_access, article_retrieval, audit_episodes, audit_subjects, doi_retrieval,
        domain_instantiations, pubmed_retrieval, scholarly_objects, title_retrieval, user_library,
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
struct ProblemAreaBrowseQuery {
    query: Option<String>,
    cwe_node_id: Option<String>,
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
        .route("/api/audit-episodes", get(list_audit_episodes))
        .route("/api/audit-episodes/:id", get(get_audit_episode))
        .route("/api/audit-episodes/:id/facts", get(list_facts_for_episode))
        .route(
            "/api/audit-episodes/:id/facts/element-review",
            post(create_episode_element_review),
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
            "/api/audit-episodes/:id/synthesis-reviews",
            get(list_synthesis_reviews).post(create_synthesis_review),
        )
        .route("/api/audit-episodes/:id/eval-tuple", get(get_eval_tuple))
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
        .route(
            "/api/peer-review/problem-area-works",
            get(browse_problem_area_works),
        )
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

async fn list_audit_subjects(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditSubject>>, ApiError> {
    let subjects = audit_subjects::list(&state.db).await?;

    Ok(Json(subjects))
}

async fn create_audit_subject(
    State(state): State<AppState>,
    Json(request): Json<CreateAuditSubjectRequest>,
) -> Result<Json<AuditSubject>, ApiError> {
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

async fn list_audit_episodes_for_subject(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<AuditEpisode>>, ApiError> {
    let episodes = audit_episodes::list_for_subject(&state.db, &id).await?;

    Ok(Json(episodes))
}

async fn commission_audit_episode(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<CommissionAuditEpisodeRequest>,
) -> Result<Json<CommissionAuditEpisodeResult>, ApiError> {
    let result = audit_episodes::commission_for_subject(&state.db, &id, request).await?;

    Ok(Json(result))
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

async fn create_episode_element_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<CreateEpisodeElementReviewRequest>,
) -> Result<Json<Fact>, ApiError> {
    let fact = audit_episodes::create_element_review_fact(&state.db, &id, request).await?;

    Ok(Json(fact))
}

async fn create_episode_solicitation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<CreateEpisodeSolicitationRequest>,
) -> Result<Json<Fact>, ApiError> {
    let fact = audit_episodes::create_solicitation_fact(&state.db, &id, request).await?;

    Ok(Json(fact))
}

async fn create_episode_solicitation_event(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<CreateEpisodeSolicitationEventRequest>,
) -> Result<Json<Fact>, ApiError> {
    let fact = audit_episodes::create_solicitation_event_fact(&state.db, &id, request).await?;

    Ok(Json(fact))
}

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
    Json(request): Json<CreateSynthesisReviewRequest>,
) -> Result<Json<SynthesisReview>, ApiError> {
    let review = audit_episodes::create_synthesis_review(&state.db, &id, request).await?;

    Ok(Json(review))
}

async fn get_eval_tuple(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<EvalTuple>, ApiError> {
    let eval_tuple = audit_episodes::compute_eval_tuple(&state.db, &id).await?;

    Ok(Json(eval_tuple))
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
