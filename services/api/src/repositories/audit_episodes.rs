use csqd_domain::{
    AuditEpisode, AuditEpisodeSummary, AuditSubjectType, CommissionAuditEpisodeRequest,
    CommissionAuditEpisodeResult, CreateEpisodeElementReviewRequest,
    CreateEpisodeSolicitationEventRequest, CreateEpisodeSolicitationRequest,
    CreateSynthesisReviewRequest, DomainConfig, EpisodeStatus, EvalTuple, ExternalRef, Fact,
    FactFinding, FactPayload, FactStatus, NarrativeStatus, Organization, OrganizationType,
    Principal, Provenance, ReviewerCommunityFilter, SynthesisReview, SynthesisReviewSection,
    SynthesisReviewSectionType,
};
use serde_json::{json, Value};
use sqlx::{postgres::PgRow, PgPool, Row};

use super::{enum_json_name, RepositoryError};

const DEMO_REVIEWER_USER_ID: &str = "00000000-0000-0000-0000-000000000002";

pub async fn list_summaries(db: &PgPool) -> Result<Vec<AuditEpisodeSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            ae.id::text AS id,
            ae.subject_id::text AS subject_id,
            ae.domain_instantiation_id::text AS domain_instantiation_id,
            ae.label,
            ae.status,
            ae.authored_by,
            ae.authored_at::text AS authored_at,
            ae.notes,
            aus.title AS subject_title,
            aus.subject_type,
            org.name AS sponsor_name,
            org.org_type AS sponsor_organization_type,
            COUNT(DISTINCT f.id) FILTER (WHERE f.status = 'active') AS fact_count,
            COUNT(DISTINCT f.id) FILTER (
                WHERE f.status = 'active'
                  AND f.payload_kind = 'element_review'
            ) AS element_review_count,
            COUNT(DISTINCT esr.id) FILTER (
                WHERE esr.status IN ('draft', 'current')
            ) AS synthesis_review_count,
            COALESCE(MAX(f.occurred_at)::text, ae.authored_at::text) AS latest_activity_at
        FROM audit_episodes ae
        JOIN audit_subjects aus ON aus.id = ae.subject_id
        LEFT JOIN organizations org
            ON ae.authored_by->'organization'->>'organization_id' = org.id::text
        LEFT JOIN episode_memberships em
            ON em.episode_id = ae.id
           AND em.status = 'active'
        LEFT JOIN facts f ON f.id = em.fact_id
        LEFT JOIN episode_synthesis_reviews esr ON esr.episode_id = ae.id
        GROUP BY ae.id, aus.id, org.id
        ORDER BY latest_activity_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_audit_episode_summary).collect()
}

pub async fn list_for_subject(
    db: &PgPool,
    subject_id: &str,
) -> Result<Vec<AuditEpisode>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            label,
            status,
            authored_by,
            authored_at::text AS authored_at,
            notes
        FROM audit_episodes
        WHERE subject_id::text = $1
        ORDER BY authored_at DESC
        "#,
    )
    .bind(subject_id)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_audit_episode).collect()
}

pub async fn find(db: &PgPool, episode_id: &str) -> Result<AuditEpisode, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            label,
            status,
            authored_by,
            authored_at::text AS authored_at,
            notes
        FROM audit_episodes
        WHERE id::text = $1
        "#,
    )
    .bind(episode_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "audit_episode",
        id: episode_id.to_string(),
    })?;

    row_to_audit_episode(row)
}

pub async fn commission_for_subject(
    db: &PgPool,
    subject_id: &str,
    request: CommissionAuditEpisodeRequest,
) -> Result<CommissionAuditEpisodeResult, RepositoryError> {
    validate_commission_request(&request)?;

    let subject_context = find_subject_context(db, subject_id).await?;
    let scope = validate_cwe_nodes(
        db,
        &request.scope_cwe_node_ids,
        &subject_context.domain_instantiation_id,
    )
    .await?;
    let organization_type = enum_json_name(
        &request.sponsor_organization_type,
        "sponsor organization type",
    )?;
    let mut tx = db.begin().await?;
    let organization_row = sqlx::query(
        r#"
        INSERT INTO organizations (name, org_type)
        VALUES ($1, $2)
        RETURNING
            id::text AS id,
            name,
            org_type,
            created_at::text AS created_at
        "#,
    )
    .bind(request.sponsor_organization_name.trim())
    .bind(organization_type)
    .fetch_one(&mut *tx)
    .await?;
    let organization = row_to_organization(organization_row)?;
    let sponsor_principal = Principal::Organization {
        organization_id: organization.id.clone(),
    };
    let principal_value = serde_json::to_value(&sponsor_principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let episode_row = sqlx::query(
        r#"
        INSERT INTO audit_episodes (
            subject_id,
            domain_instantiation_id,
            label,
            status,
            authored_by,
            notes
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            $3,
            'active',
            $4::jsonb,
            $5
        )
        RETURNING
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            label,
            status,
            authored_by,
            authored_at::text AS authored_at,
            notes
        "#,
    )
    .bind(subject_id)
    .bind(&subject_context.domain_instantiation_id)
    .bind(request.label.trim())
    .bind(&principal_value)
    .bind(request.notes.as_deref().map(str::trim))
    .fetch_one(&mut *tx)
    .await?;
    let episode = row_to_audit_episode(episode_row)?;
    let payload = FactPayload::AuditCommission {
        commissioned_by: sponsor_principal.clone(),
        scope,
        funding: request.funding,
        deadline: request.deadline,
        confidential: request.confidential,
    };
    let payload_value = serde_json::to_value(&payload)
        .map_err(|error| RepositoryError::Domain(format!("invalid fact payload: {error}")))?;
    let provenance = json!({
        "source_system": "csqd_commission_api",
        "source_document": null,
        "imported_at": episode.authored_at.clone(),
        "principal": principal_value.clone(),
    });
    let fact_row = sqlx::query(
        r#"
        INSERT INTO facts (
            subject_id,
            domain_instantiation_id,
            occurred_at,
            payload_kind,
            payload,
            status,
            provenance,
            external_refs
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            now(),
            'audit_commission',
            $3::jsonb,
            'active',
            $4::jsonb,
            '[]'::jsonb
        )
        RETURNING
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            occurred_at::text AS occurred_at,
            payload,
            status,
            status_metadata,
            provenance,
            external_refs
        "#,
    )
    .bind(subject_id)
    .bind(&subject_context.domain_instantiation_id)
    .bind(&payload_value)
    .bind(&provenance)
    .fetch_one(&mut *tx)
    .await?;
    let commission_fact = row_to_fact(fact_row)?;

    sqlx::query(
        r#"
        INSERT INTO episode_memberships (
            fact_id,
            episode_id,
            role,
            asserted_by,
            status
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            'commission',
            $3::jsonb,
            'active'
        )
        "#,
    )
    .bind(&commission_fact.id)
    .bind(&episode.id)
    .bind(&principal_value)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(CommissionAuditEpisodeResult {
        organization,
        episode,
        commission_fact,
    })
}

pub async fn create_element_review_fact(
    db: &PgPool,
    episode_id: &str,
    request: CreateEpisodeElementReviewRequest,
) -> Result<Fact, RepositoryError> {
    validate_create_element_review_fact_request(&request)?;

    let episode_context = find_episode_context(db, episode_id).await?;
    validate_cwe_nodes(
        db,
        std::slice::from_ref(&request.cwe_node_id),
        &episode_context.domain_instantiation_id,
    )
    .await?;

    let submitted_by = request
        .submitted_by
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEMO_REVIEWER_USER_ID)
        .to_string();
    let principal = Principal::User {
        user_id: submitted_by.clone(),
    };
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::ElementReview {
        cwe_criterion: csqd_domain::CWECriterionId {
            domain: episode_context.domain_instantiation_id.clone(),
            node_id: request.cwe_node_id,
        },
        submitted_by,
        solicitation: request.solicitation,
        finding: request.finding,
        severity: request.severity,
        confidence: request.confidence,
        limitations: trimmed_optional(request.limitations),
        recommendations: trimmed_optional(request.recommendations),
        content: request.content.trim().to_string(),
        featured: request.featured,
    };
    let payload_value = serde_json::to_value(&payload)
        .map_err(|error| RepositoryError::Domain(format!("invalid fact payload: {error}")))?;
    let provenance = json!({
        "source_system": "csqd_episode_review_api",
        "source_document": null,
        "imported_at": episode_context.now,
        "principal": principal_value.clone(),
    });

    let mut tx = db.begin().await?;
    let fact_row = sqlx::query(
        r#"
        INSERT INTO facts (
            subject_id,
            domain_instantiation_id,
            occurred_at,
            payload_kind,
            payload,
            status,
            provenance,
            external_refs
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            now(),
            'element_review',
            $3::jsonb,
            'active',
            $4::jsonb,
            '[]'::jsonb
        )
        RETURNING
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            occurred_at::text AS occurred_at,
            payload,
            status,
            status_metadata,
            provenance,
            external_refs
        "#,
    )
    .bind(&episode_context.subject_id)
    .bind(&episode_context.domain_instantiation_id)
    .bind(&payload_value)
    .bind(&provenance)
    .fetch_one(&mut *tx)
    .await?;
    let fact = row_to_fact(fact_row)?;

    sqlx::query(
        r#"
        INSERT INTO episode_memberships (
            fact_id,
            episode_id,
            role,
            asserted_by,
            status
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            'element_review',
            $3::jsonb,
            'active'
        )
        "#,
    )
    .bind(&fact.id)
    .bind(episode_id)
    .bind(&principal_value)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(fact)
}

pub async fn create_solicitation_fact(
    db: &PgPool,
    episode_id: &str,
    request: CreateEpisodeSolicitationRequest,
) -> Result<Fact, RepositoryError> {
    validate_create_solicitation_fact_request(&request)?;

    let episode_context = find_episode_context(db, episode_id).await?;
    validate_cwe_nodes(
        db,
        std::slice::from_ref(&request.cwe_node_id),
        &episode_context.domain_instantiation_id,
    )
    .await?;

    let commission_fact_id = match request.commission_fact_id.as_deref().map(str::trim) {
        Some(value) if !value.is_empty() => {
            validate_episode_fact_kind(db, episode_id, value, "audit_commission").await?;
            value.to_string()
        }
        _ => find_episode_commission_fact_id(db, episode_id).await?,
    };
    let issued_to = request
        .issued_to
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEMO_REVIEWER_USER_ID)
        .to_string();
    let principal = Principal::Platform;
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::ERSolicitation {
        issued_to,
        cwe_criterion: csqd_domain::CWECriterionId {
            domain: episode_context.domain_instantiation_id.clone(),
            node_id: request.cwe_node_id,
        },
        commission: commission_fact_id,
        payment_scheme: request.payment_scheme,
    };
    let payload_value = serde_json::to_value(&payload)
        .map_err(|error| RepositoryError::Domain(format!("invalid fact payload: {error}")))?;
    let provenance = json!({
        "source_system": "csqd_solicitation_api",
        "source_document": null,
        "imported_at": episode_context.now,
        "principal": principal_value.clone(),
    });

    let mut tx = db.begin().await?;
    let fact_row = sqlx::query(
        r#"
        INSERT INTO facts (
            subject_id,
            domain_instantiation_id,
            occurred_at,
            payload_kind,
            payload,
            status,
            provenance,
            external_refs
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            now(),
            'er_solicitation',
            $3::jsonb,
            'active',
            $4::jsonb,
            '[]'::jsonb
        )
        RETURNING
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            occurred_at::text AS occurred_at,
            payload,
            status,
            status_metadata,
            provenance,
            external_refs
        "#,
    )
    .bind(&episode_context.subject_id)
    .bind(&episode_context.domain_instantiation_id)
    .bind(&payload_value)
    .bind(&provenance)
    .fetch_one(&mut *tx)
    .await?;
    let fact = row_to_fact(fact_row)?;

    sqlx::query(
        r#"
        INSERT INTO episode_memberships (
            fact_id,
            episode_id,
            role,
            asserted_by,
            status
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            'solicitation',
            $3::jsonb,
            'active'
        )
        "#,
    )
    .bind(&fact.id)
    .bind(episode_id)
    .bind(&principal_value)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(fact)
}

pub async fn create_solicitation_event_fact(
    db: &PgPool,
    episode_id: &str,
    request: CreateEpisodeSolicitationEventRequest,
) -> Result<Fact, RepositoryError> {
    validate_create_solicitation_event_fact_request(&request)?;

    let episode_context = find_episode_context(db, episode_id).await?;
    validate_episode_fact_kind(
        db,
        episode_id,
        &request.solicitation_fact_id,
        "er_solicitation",
    )
    .await?;

    let principal = request.principal.unwrap_or(Principal::Platform);
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::SolicitationEvent {
        solicitation: request.solicitation_fact_id,
        event_type: request.event_type,
        principal: principal.clone(),
        note: trimmed_optional(request.note),
    };
    let payload_value = serde_json::to_value(&payload)
        .map_err(|error| RepositoryError::Domain(format!("invalid fact payload: {error}")))?;
    let provenance = json!({
        "source_system": "csqd_solicitation_event_api",
        "source_document": null,
        "imported_at": episode_context.now,
        "principal": principal_value.clone(),
    });

    let mut tx = db.begin().await?;
    let fact_row = sqlx::query(
        r#"
        INSERT INTO facts (
            subject_id,
            domain_instantiation_id,
            occurred_at,
            payload_kind,
            payload,
            status,
            provenance,
            external_refs
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            now(),
            'solicitation_event',
            $3::jsonb,
            'active',
            $4::jsonb,
            '[]'::jsonb
        )
        RETURNING
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            occurred_at::text AS occurred_at,
            payload,
            status,
            status_metadata,
            provenance,
            external_refs
        "#,
    )
    .bind(&episode_context.subject_id)
    .bind(&episode_context.domain_instantiation_id)
    .bind(&payload_value)
    .bind(&provenance)
    .fetch_one(&mut *tx)
    .await?;
    let fact = row_to_fact(fact_row)?;

    sqlx::query(
        r#"
        INSERT INTO episode_memberships (
            fact_id,
            episode_id,
            role,
            asserted_by,
            status
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            'solicitation_lifecycle',
            $3::jsonb,
            'active'
        )
        "#,
    )
    .bind(&fact.id)
    .bind(episode_id)
    .bind(&principal_value)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(fact)
}

pub async fn list_synthesis_reviews(
    db: &PgPool,
    episode_id: &str,
) -> Result<Vec<SynthesisReview>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            episode_id::text AS episode_id,
            submitted_by::text AS submitted_by,
            authored_at::text AS authored_at,
            status,
            summary,
            featured
        FROM episode_synthesis_reviews
        WHERE episode_id::text = $1
        ORDER BY authored_at DESC
        "#,
    )
    .bind(episode_id)
    .fetch_all(db)
    .await?;

    let mut reviews = Vec::with_capacity(rows.len());
    for row in rows {
        reviews.push(row_to_synthesis_review(db, row).await?);
    }

    Ok(reviews)
}

pub async fn create_synthesis_review(
    db: &PgPool,
    episode_id: &str,
    request: CreateSynthesisReviewRequest,
) -> Result<SynthesisReview, RepositoryError> {
    validate_create_synthesis_review_request(db, episode_id, &request).await?;

    let submitted_by = request
        .submitted_by
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEMO_REVIEWER_USER_ID)
        .to_string();
    let status = enum_json_name(&request.status, "narrative status")?;
    let mut tx = db.begin().await?;

    let review_row = sqlx::query(
        r#"
        INSERT INTO episode_synthesis_reviews (
            episode_id,
            submitted_by,
            status,
            summary,
            featured
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            $3,
            $4,
            $5
        )
        RETURNING
            id::text AS id,
            episode_id::text AS episode_id,
            submitted_by::text AS submitted_by,
            authored_at::text AS authored_at,
            status,
            summary,
            featured
        "#,
    )
    .bind(episode_id)
    .bind(&submitted_by)
    .bind(status)
    .bind(request.summary.trim())
    .bind(request.featured)
    .fetch_one(&mut *tx)
    .await?;

    let review_id: String = review_row.get("id");
    for section in &request.sections {
        let section_type = enum_json_name(&section.section_type, "synthesis section type")?;
        let referenced_facts: Vec<String> = section
            .referenced_facts
            .iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();

        sqlx::query(
            r#"
            INSERT INTO episode_synthesis_sections (
                review_id,
                section_type,
                content,
                referenced_facts
            )
            VALUES (
                $1::uuid,
                $2,
                $3,
                ARRAY(SELECT value::uuid FROM unnest($4::text[]) AS value)
            )
            "#,
        )
        .bind(&review_id)
        .bind(section_type)
        .bind(section.content.trim())
        .bind(&referenced_facts)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    row_to_synthesis_review(db, review_row).await
}

pub async fn compute_eval_tuple(
    db: &PgPool,
    episode_id: &str,
) -> Result<EvalTuple, RepositoryError> {
    let episode_context = find_episode_context(db, episode_id).await?;
    let config_row = sqlx::query(
        r#"
        SELECT config
        FROM domain_instantiations
        WHERE id::text = $1
        "#,
    )
    .bind(&episode_context.domain_instantiation_id)
    .fetch_one(db)
    .await?;
    let config_value: Value = config_row.get("config");
    let config = serde_json::from_value::<DomainConfig>(config_value)
        .map_err(|error| RepositoryError::Domain(format!("invalid domain config: {error}")))?;
    let facts = list_facts_for_episode(db, episode_id).await?;
    let synthesis_count = count_synthesis_reviews(db, episode_id).await?;
    let mut n: f64 = 0.0;
    let mut m: f64 = 0.0;
    let mut l: f64 = 0.0;
    let mut s: f64 = 0.0;

    for fact in facts {
        if !matches!(fact.status, FactStatus::Active) {
            continue;
        }

        match fact.payload {
            FactPayload::AuditCommission { scope, funding, .. } => {
                let funding_signal = (funding.amount / 10_000.0).clamp(0.0, 2.0);
                s = s.max(scope.len() as f64 + funding_signal);
            }
            FactPayload::ElementReview {
                finding,
                solicitation,
                ..
            } => {
                match finding {
                    FactFinding::NonEthicalProblem => n += 1.0,
                    FactFinding::EthicalProblem => m += 1.0,
                    FactFinding::NoProblems | FactFinding::Inconclusive => {}
                }

                l += if solicitation.is_some() {
                    config
                        .eval_tuple_config
                        .l_weight_params
                        .solicited_review_multiplier
                } else {
                    1.0
                };
            }
            FactPayload::ERSolicitation { .. }
            | FactPayload::SolicitationEvent { .. }
            | FactPayload::SubmitterResponse { .. } => {}
        }
    }

    Ok(EvalTuple {
        n,
        m,
        s,
        l,
        u: synthesis_count as f64,
        computed_at: episode_context.now,
        community_filter: ReviewerCommunityFilter {
            tags: Vec::new(),
            domain_instantiation_id: Some(episode_context.domain_instantiation_id),
            min_endorsements: None,
        },
    })
}

pub async fn list_facts_for_subject(
    db: &PgPool,
    subject_id: &str,
) -> Result<Vec<Fact>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            occurred_at::text AS occurred_at,
            payload,
            status,
            status_metadata,
            provenance,
            external_refs
        FROM facts
        WHERE subject_id::text = $1
        ORDER BY occurred_at DESC
        LIMIT 200
        "#,
    )
    .bind(subject_id)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_fact).collect()
}

pub async fn list_facts_for_episode(
    db: &PgPool,
    episode_id: &str,
) -> Result<Vec<Fact>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            f.id::text AS id,
            f.subject_id::text AS subject_id,
            f.domain_instantiation_id::text AS domain_instantiation_id,
            f.occurred_at::text AS occurred_at,
            f.payload,
            f.status,
            f.status_metadata,
            f.provenance,
            f.external_refs
        FROM episode_memberships em
        JOIN facts f ON f.id = em.fact_id
        WHERE em.episode_id::text = $1
          AND em.status = 'active'
        ORDER BY f.occurred_at DESC
        "#,
    )
    .bind(episode_id)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_fact).collect()
}

struct SubjectContext {
    domain_instantiation_id: String,
}

struct EpisodeContext {
    subject_id: String,
    domain_instantiation_id: String,
    now: String,
}

async fn find_subject_context(
    db: &PgPool,
    subject_id: &str,
) -> Result<SubjectContext, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT domain_instantiation_id::text AS domain_instantiation_id
        FROM audit_subjects
        WHERE id::text = $1
        "#,
    )
    .bind(subject_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "audit_subject",
        id: subject_id.to_string(),
    })?;

    Ok(SubjectContext {
        domain_instantiation_id: row.get("domain_instantiation_id"),
    })
}

async fn find_episode_context(
    db: &PgPool,
    episode_id: &str,
) -> Result<EpisodeContext, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            now()::text AS now
        FROM audit_episodes
        WHERE id::text = $1
        "#,
    )
    .bind(episode_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "audit_episode",
        id: episode_id.to_string(),
    })?;

    Ok(EpisodeContext {
        subject_id: row.get("subject_id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        now: row.get("now"),
    })
}

async fn validate_cwe_nodes(
    db: &PgPool,
    cwe_node_ids: &[String],
    domain_instantiation_id: &str,
) -> Result<Vec<csqd_domain::CWECriterionId>, RepositoryError> {
    let mut criteria = Vec::with_capacity(cwe_node_ids.len());

    for node_id in cwe_node_ids {
        let node_id = node_id.trim();
        if node_id.is_empty() {
            return Err(RepositoryError::Domain(
                "CWE criterion id cannot be empty".to_string(),
            ));
        }

        let exists = sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM cwe_nodes
                WHERE id::text = $1
                  AND domain_instantiation_id::text = $2
            ) AS exists
            "#,
        )
        .bind(node_id)
        .bind(domain_instantiation_id)
        .fetch_one(db)
        .await?
        .get::<bool, _>("exists");

        if !exists {
            return Err(RepositoryError::NotFound {
                entity: "cwe_node",
                id: node_id.to_string(),
            });
        }

        criteria.push(csqd_domain::CWECriterionId {
            domain: domain_instantiation_id.to_string(),
            node_id: node_id.to_string(),
        });
    }

    Ok(criteria)
}

async fn find_episode_commission_fact_id(
    db: &PgPool,
    episode_id: &str,
) -> Result<String, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT f.id::text AS id
        FROM episode_memberships em
        JOIN facts f ON f.id = em.fact_id
        WHERE em.episode_id::text = $1
          AND em.status = 'active'
          AND em.role = 'commission'
          AND f.status = 'active'
          AND f.payload_kind = 'audit_commission'
        ORDER BY f.occurred_at ASC
        LIMIT 1
        "#,
    )
    .bind(episode_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "audit_commission_fact",
        id: episode_id.to_string(),
    })?;

    Ok(row.get("id"))
}

async fn validate_episode_fact_kind(
    db: &PgPool,
    episode_id: &str,
    fact_id: &str,
    payload_kind: &str,
) -> Result<(), RepositoryError> {
    let exists = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM episode_memberships em
            JOIN facts f ON f.id = em.fact_id
            WHERE em.episode_id::text = $1
              AND f.id::text = $2
              AND f.payload_kind = $3
              AND em.status = 'active'
              AND f.status = 'active'
        ) AS exists
        "#,
    )
    .bind(episode_id)
    .bind(fact_id.trim())
    .bind(payload_kind)
    .fetch_one(db)
    .await?
    .get::<bool, _>("exists");

    if exists {
        Ok(())
    } else {
        Err(RepositoryError::NotFound {
            entity: "episode_fact",
            id: fact_id.to_string(),
        })
    }
}

async fn validate_episode_fact_references(
    db: &PgPool,
    episode_id: &str,
    fact_ids: &[String],
) -> Result<(), RepositoryError> {
    for fact_id in fact_ids {
        let fact_id = fact_id.trim();
        if fact_id.is_empty() {
            continue;
        }

        let exists = sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM episode_memberships em
                JOIN facts f ON f.id = em.fact_id
                WHERE em.episode_id::text = $1
                  AND f.id::text = $2
                  AND em.status = 'active'
                  AND f.status = 'active'
            ) AS exists
            "#,
        )
        .bind(episode_id)
        .bind(fact_id)
        .fetch_one(db)
        .await?
        .get::<bool, _>("exists");

        if !exists {
            return Err(RepositoryError::NotFound {
                entity: "episode_fact",
                id: fact_id.to_string(),
            });
        }
    }

    Ok(())
}

async fn count_synthesis_reviews(db: &PgPool, episode_id: &str) -> Result<i64, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) AS count
        FROM episode_synthesis_reviews
        WHERE episode_id::text = $1
          AND status IN ('draft', 'current')
        "#,
    )
    .bind(episode_id)
    .fetch_one(db)
    .await?;

    Ok(row.get("count"))
}

fn row_to_organization(row: PgRow) -> Result<Organization, RepositoryError> {
    let org_type: String = row.get("org_type");

    Ok(Organization {
        id: row.get("id"),
        name: row.get("name"),
        org_type: OrganizationType::try_from(org_type.as_str()).map_err(RepositoryError::Domain)?,
        created_at: row.get("created_at"),
    })
}

fn row_to_audit_episode(row: PgRow) -> Result<AuditEpisode, RepositoryError> {
    let status: String = row.get("status");
    let authored_by: Value = row.get("authored_by");

    Ok(AuditEpisode {
        id: row.get("id"),
        subject_id: row.get("subject_id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        label: row.get("label"),
        status: EpisodeStatus::try_from(status.as_str()).map_err(RepositoryError::Domain)?,
        authored_by: serde_json::from_value::<Principal>(authored_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        authored_at: row.get("authored_at"),
        notes: row.get("notes"),
    })
}

fn row_to_audit_episode_summary(row: PgRow) -> Result<AuditEpisodeSummary, RepositoryError> {
    let status: String = row.get("status");
    let authored_by: Value = row.get("authored_by");
    let subject_type: String = row.get("subject_type");
    let sponsor_organization_type: Option<String> = row.get("sponsor_organization_type");
    let element_review_count = row.get("element_review_count");
    let synthesis_review_count = row.get("synthesis_review_count");

    Ok(AuditEpisodeSummary {
        id: row.get("id"),
        subject_id: row.get("subject_id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        label: row.get("label"),
        status: EpisodeStatus::try_from(status.as_str()).map_err(RepositoryError::Domain)?,
        authored_by: serde_json::from_value::<Principal>(authored_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        authored_at: row.get("authored_at"),
        notes: row.get("notes"),
        subject_title: row.get("subject_title"),
        subject_type: AuditSubjectType::try_from(subject_type.as_str())
            .map_err(RepositoryError::Domain)?,
        sponsor_name: row.get("sponsor_name"),
        sponsor_organization_type: sponsor_organization_type
            .as_deref()
            .map(OrganizationType::try_from)
            .transpose()
            .map_err(RepositoryError::Domain)?,
        fact_count: row.get("fact_count"),
        element_review_count,
        synthesis_review_count,
        latest_activity_at: row.get("latest_activity_at"),
        synthesis_ready: element_review_count > 0 && synthesis_review_count == 0,
    })
}

fn validate_commission_request(
    request: &CommissionAuditEpisodeRequest,
) -> Result<(), RepositoryError> {
    if request.label.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "audit episode label cannot be empty".to_string(),
        ));
    }

    if request.sponsor_organization_name.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "audit commission requires a sponsor organization".to_string(),
        ));
    }

    if request.scope_cwe_node_ids.is_empty() {
        return Err(RepositoryError::Domain(
            "audit commission requires at least one CWE criterion".to_string(),
        ));
    }

    if request.funding.amount <= 0.0 {
        return Err(RepositoryError::Domain(
            "audit commission funding amount must be positive".to_string(),
        ));
    }

    if request.funding.currency.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "audit commission funding currency cannot be empty".to_string(),
        ));
    }

    Ok(())
}

fn validate_create_element_review_fact_request(
    request: &CreateEpisodeElementReviewRequest,
) -> Result<(), RepositoryError> {
    if request.cwe_node_id.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "element review requires a CWE criterion".to_string(),
        ));
    }

    if request.content.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "element review content cannot be empty".to_string(),
        ));
    }

    Ok(())
}

fn validate_create_solicitation_fact_request(
    request: &CreateEpisodeSolicitationRequest,
) -> Result<(), RepositoryError> {
    if request.cwe_node_id.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "solicitation requires a CWE criterion".to_string(),
        ));
    }

    if request.payment_scheme.amount.amount <= 0.0 {
        return Err(RepositoryError::Domain(
            "solicitation payment amount must be positive".to_string(),
        ));
    }

    if request.payment_scheme.currency.trim().is_empty()
        || request.payment_scheme.amount.currency.trim().is_empty()
    {
        return Err(RepositoryError::Domain(
            "solicitation payment currency cannot be empty".to_string(),
        ));
    }

    Ok(())
}

fn validate_create_solicitation_event_fact_request(
    request: &CreateEpisodeSolicitationEventRequest,
) -> Result<(), RepositoryError> {
    if request.solicitation_fact_id.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "solicitation event requires a solicitation fact".to_string(),
        ));
    }

    Ok(())
}

async fn validate_create_synthesis_review_request(
    db: &PgPool,
    episode_id: &str,
    request: &CreateSynthesisReviewRequest,
) -> Result<(), RepositoryError> {
    find_episode_context(db, episode_id).await?;

    if request.summary.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "synthesis review summary cannot be empty".to_string(),
        ));
    }

    for section in &request.sections {
        if section.content.trim().is_empty() {
            return Err(RepositoryError::Domain(
                "synthesis review section content cannot be empty".to_string(),
            ));
        }

        validate_episode_fact_references(db, episode_id, &section.referenced_facts).await?;
    }

    Ok(())
}

fn trimmed_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn row_to_synthesis_review(
    db: &PgPool,
    row: PgRow,
) -> Result<SynthesisReview, RepositoryError> {
    let status: String = row.get("status");
    let review_id: String = row.get("id");
    let section_rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            review_id::text AS review_id,
            section_type,
            content,
            ARRAY(SELECT value::text FROM unnest(referenced_facts) AS value) AS referenced_facts
        FROM episode_synthesis_sections
        WHERE review_id::text = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(&review_id)
    .fetch_all(db)
    .await?;
    let sections = section_rows
        .into_iter()
        .map(row_to_synthesis_section)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SynthesisReview {
        id: review_id,
        episode_id: row.get("episode_id"),
        submitted_by: row.get("submitted_by"),
        authored_at: row.get("authored_at"),
        status: NarrativeStatus::try_from(status.as_str()).map_err(RepositoryError::Domain)?,
        summary: row.get("summary"),
        sections,
        featured: row.get("featured"),
    })
}

fn row_to_synthesis_section(row: PgRow) -> Result<SynthesisReviewSection, RepositoryError> {
    let section_type: String = row.get("section_type");

    Ok(SynthesisReviewSection {
        id: row.get("id"),
        review_id: row.get("review_id"),
        section_type: SynthesisReviewSectionType::try_from(section_type.as_str())
            .map_err(RepositoryError::Domain)?,
        content: row.get("content"),
        referenced_facts: row.get("referenced_facts"),
    })
}

fn row_to_fact(row: PgRow) -> Result<Fact, RepositoryError> {
    let occurred_at: String = row.get("occurred_at");
    let payload: Value = row.get("payload");
    let status: String = row.get("status");
    let status_metadata: Value = row.get("status_metadata");
    let provenance: Value = row.get("provenance");
    let external_refs: Value = row.get("external_refs");

    Ok(Fact {
        id: row.get("id"),
        subject_id: row.get("subject_id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        occurred_at: occurred_at.clone(),
        payload: serde_json::from_value::<FactPayload>(payload)
            .map_err(|error| RepositoryError::Domain(format!("invalid fact payload: {error}")))?,
        status: fact_status_from_row(&status, status_metadata, &occurred_at)?,
        provenance: provenance_from_row(provenance, &occurred_at)?,
        external_refs: serde_json::from_value::<Vec<ExternalRef>>(external_refs)
            .map_err(|error| RepositoryError::Domain(format!("invalid external refs: {error}")))?,
    })
}

fn fact_status_from_row(
    status: &str,
    metadata: Value,
    occurred_at: &str,
) -> Result<FactStatus, RepositoryError> {
    match status {
        "active" => Ok(FactStatus::Active),
        "superseded" => Ok(FactStatus::Superseded {
            superseded_by: metadata_principal(&metadata, "superseded_by")?,
            superseded_at: metadata_timestamp(&metadata, "superseded_at", occurred_at),
            replaced_by: metadata
                .get("replaced_by")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        }),
        "retracted" => Ok(FactStatus::Retracted {
            retracted_by: metadata_principal(&metadata, "retracted_by")?,
            retracted_at: metadata_timestamp(&metadata, "retracted_at", occurred_at),
            reason: metadata
                .get("reason")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        }),
        other => Err(RepositoryError::Domain(format!(
            "unknown fact status: {other}"
        ))),
    }
}

fn provenance_from_row(value: Value, occurred_at: &str) -> Result<Provenance, RepositoryError> {
    if let Ok(provenance) = serde_json::from_value::<Provenance>(value.clone()) {
        return Ok(provenance);
    }

    let principal = match value.get("principal") {
        Some(principal) => serde_json::from_value::<Principal>(principal.clone())
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        None => Principal::Platform,
    };

    Ok(Provenance {
        source_system: value
            .get("source_system")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        source_document: value
            .get("source_document")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        imported_at: value
            .get("imported_at")
            .and_then(Value::as_str)
            .unwrap_or(occurred_at)
            .to_string(),
        principal,
    })
}

fn metadata_principal(metadata: &Value, key: &str) -> Result<Principal, RepositoryError> {
    match metadata.get(key) {
        Some(value) => serde_json::from_value::<Principal>(value.clone())
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}"))),
        None => Ok(Principal::Platform),
    }
}

fn metadata_timestamp(metadata: &Value, key: &str, fallback: &str) -> String {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}
