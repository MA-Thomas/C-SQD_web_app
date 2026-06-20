use chrono::{DateTime, Utc};
use csqd_domain::{
    compute_eval_tuple as compute_eval_tuple_pure, AuditEpisode, AuditEpisodeId,
    AuditEpisodeSummary, AuditSubjectType, CWECriterionId, CWENodeId, CWEPetitionKind,
    CommissionAuditEpisodeRequest, CommissionAuditEpisodeResult, CreateEpisodeElementReviewRequest,
    CreateEpisodeSolicitationEventRequest, CreateEpisodeSolicitationRequest,
    CreateSynthesisReviewRequest, CurationOutcome, CurationTarget, DomainConfig,
    DomainInstantiationId, EpisodeMembership, EpisodeMembershipStatus, EpisodeStatus, EvalTuple,
    EvalTupleContext, EvalTupleObservations, ExternalRef, Fact, FactId, FactPayload, FactRole,
    FactStatus, NarrativeStatus, Organization, OrganizationType, ParticipationAction, Principal,
    Provenance, ReviewerCommunityFilter, SynthesisReview, SynthesisReviewSection,
    SynthesisReviewSectionType, TagId, UserId,
};
use serde_json::{json, Value};
use sqlx::{postgres::PgRow, PgPool, Postgres, Row, Transaction};

use super::RepositoryError;

const DEMO_REVIEWER_USER_ID: &str = "00000000-0000-0000-0000-000000000002";

const FACT_COLUMNS: &str = r#"
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            occurred_at,
            payload,
            status,
            status_metadata,
            provenance,
            external_refs
"#;

const EPISODE_COLUMNS: &str = r#"
            id::text AS id,
            subject_id::text AS subject_id,
            domain_instantiation_id::text AS domain_instantiation_id,
            label,
            status,
            authored_by,
            authored_at,
            notes
"#;

const SYNTHESIS_COLUMNS: &str = r#"
            id::text AS id,
            episode_id::text AS episode_id,
            submitted_by::text AS submitted_by,
            authored_at,
            status,
            summary,
            featured,
            unsolicited
"#;

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
            ae.authored_at,
            ae.notes,
            aus.title AS subject_title,
            aus.subject_type,
            aus.subject_type_detail,
            org.name AS sponsor_name,
            org.org_type AS sponsor_organization_type,
            org.org_type_detail AS sponsor_organization_type_detail,
            COUNT(DISTINCT f.id) FILTER (WHERE f.status = 'active') AS fact_count,
            COUNT(DISTINCT f.id) FILTER (
                WHERE f.status = 'active'
                  AND f.payload_kind = 'element_review'
            ) AS element_review_count,
            COUNT(DISTINCT esr.id) FILTER (
                WHERE esr.status IN ('draft', 'current')
            ) AS synthesis_review_count,
            COALESCE(MAX(f.occurred_at), ae.authored_at) AS latest_activity_at
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
    let rows = sqlx::query(&format!(
        r#"
        SELECT {EPISODE_COLUMNS}
        FROM audit_episodes
        WHERE subject_id::text = $1
        ORDER BY authored_at DESC
        "#
    ))
    .bind(subject_id)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_audit_episode).collect()
}

pub async fn find(db: &PgPool, episode_id: &str) -> Result<AuditEpisode, RepositoryError> {
    let row = sqlx::query(&format!(
        r#"
        SELECT {EPISODE_COLUMNS}
        FROM audit_episodes
        WHERE id::text = $1
        "#
    ))
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
    let mut tx = db.begin().await?;
    let organization_row = sqlx::query(
        r#"
        INSERT INTO organizations (name, org_type, org_type_detail)
        VALUES ($1, $2, $3)
        RETURNING
            id::text AS id,
            name,
            org_type,
            org_type_detail,
            created_at
        "#,
    )
    .bind(request.sponsor_organization_name.trim())
    .bind(request.sponsor_organization_type.db_kind())
    .bind(request.sponsor_organization_type.db_detail())
    .fetch_one(&mut *tx)
    .await?;
    let organization = row_to_organization(organization_row)?;
    let sponsor_principal = Principal::Organization {
        organization_id: organization.id.clone(),
    };
    let principal_value = serde_json::to_value(&sponsor_principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let episode_row = sqlx::query(&format!(
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
        RETURNING {EPISODE_COLUMNS}
        "#
    ))
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
    let commission_fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: subject_id.to_string(),
            domain_instantiation_id: subject_context.domain_instantiation_id.clone(),
            episode_id: episode.id.as_str().to_string(),
            payload: &payload,
            role: FactRole::Commission,
            principal_value: &principal_value,
            source_system: "csqd_commission_api",
        },
    )
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
        .clone()
        .filter(|value| !value.as_str().trim().is_empty())
        .unwrap_or_else(|| UserId::new(DEMO_REVIEWER_USER_ID));

    if let Some(solicitation) = &request.solicitation {
        validate_episode_fact_kind(db, episode_id, solicitation.as_str(), "er_solicitation")
            .await?;
    }

    let principal = Principal::User {
        user_id: submitted_by.clone(),
    };
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::ElementReview {
        cwe_criterion: CWECriterionId {
            domain: DomainInstantiationId::new(episode_context.domain_instantiation_id.clone()),
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

    let mut tx = db.begin().await?;
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: episode_context.subject_id.clone(),
            domain_instantiation_id: episode_context.domain_instantiation_id.clone(),
            episode_id: episode_id.to_string(),
            payload: &payload,
            role: FactRole::ElementReview,
            principal_value: &principal_value,
            source_system: "csqd_episode_review_api",
        },
    )
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

    let commission_fact_id = match &request.commission_fact_id {
        Some(value) if !value.as_str().trim().is_empty() => {
            validate_episode_fact_kind(db, episode_id, value.as_str(), "audit_commission").await?;
            value.clone()
        }
        _ => find_episode_commission_fact_id(db, episode_id).await?,
    };
    let issued_to = request
        .issued_to
        .clone()
        .filter(|value| !value.as_str().trim().is_empty())
        .unwrap_or_else(|| UserId::new(DEMO_REVIEWER_USER_ID));
    let principal = Principal::Platform;
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::ERSolicitation {
        issued_to,
        cwe_criterion: CWECriterionId {
            domain: DomainInstantiationId::new(episode_context.domain_instantiation_id.clone()),
            node_id: request.cwe_node_id,
        },
        commission: commission_fact_id,
        payment_scheme: request.payment_scheme,
    };

    let mut tx = db.begin().await?;
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: episode_context.subject_id.clone(),
            domain_instantiation_id: episode_context.domain_instantiation_id.clone(),
            episode_id: episode_id.to_string(),
            payload: &payload,
            role: FactRole::Solicitation,
            principal_value: &principal_value,
            source_system: "csqd_solicitation_api",
        },
    )
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
        request.solicitation_fact_id.as_str(),
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

    let mut tx = db.begin().await?;
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: episode_context.subject_id.clone(),
            domain_instantiation_id: episode_context.domain_instantiation_id.clone(),
            episode_id: episode_id.to_string(),
            payload: &payload,
            role: FactRole::SolicitationLifecycle,
            principal_value: &principal_value,
            source_system: "csqd_solicitation_event_api",
        },
    )
    .await?;
    tx.commit().await?;

    Ok(fact)
}

/// Starts a new public audit episode for a subject, authored by `participant`,
/// and records the `EpisodeParticipation { action: Start }` fact.
pub async fn start_public_episode(
    db: &PgPool,
    subject_id: &str,
    participant: &UserId,
    label: &str,
    notes: Option<String>,
) -> Result<(AuditEpisode, Fact), RepositoryError> {
    if label.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "public audit episode label cannot be empty".to_string(),
        ));
    }

    let subject_context = find_subject_context(db, subject_id).await?;
    let principal = Principal::User {
        user_id: participant.clone(),
    };
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;

    let mut tx = db.begin().await?;
    let episode_row = sqlx::query(&format!(
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
        RETURNING {EPISODE_COLUMNS}
        "#
    ))
    .bind(subject_id)
    .bind(&subject_context.domain_instantiation_id)
    .bind(label.trim())
    .bind(&principal_value)
    .bind(notes.as_deref().map(str::trim))
    .fetch_one(&mut *tx)
    .await?;
    let episode = row_to_audit_episode(episode_row)?;
    let payload = FactPayload::EpisodeParticipation {
        episode: episode.id.clone(),
        participant: participant.clone(),
        action: ParticipationAction::Start,
        note: None,
    };
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: subject_id.to_string(),
            domain_instantiation_id: subject_context.domain_instantiation_id.clone(),
            episode_id: episode.id.as_str().to_string(),
            payload: &payload,
            role: FactRole::Participation,
            principal_value: &principal_value,
            source_system: "csqd_public_episode_api",
        },
    )
    .await?;
    tx.commit().await?;

    Ok((episode, fact))
}

/// Joins an existing episode by recording an `EpisodeParticipation` fact.
/// Idempotent for a participant within an episode: repeat joins return the
/// existing active participation fact instead of adding duplicate provenance.
pub async fn join_public_episode(
    db: &PgPool,
    episode_id: &str,
    participant: &UserId,
    note: Option<String>,
) -> Result<Fact, RepositoryError> {
    let episode_context = find_episode_context(db, episode_id).await?;
    let existing = sqlx::query(&format!(
        r#"
        SELECT {FACT_COLUMNS}
        FROM facts
        WHERE id = (
            SELECT f.id
            FROM episode_memberships em
            JOIN facts f ON f.id = em.fact_id
            WHERE em.episode_id::text = $1
              AND em.status = 'active'
              AND f.status = 'active'
              AND f.payload_kind = 'episode_participation'
              AND f.payload->'episode_participation'->>'participant' = $2
            ORDER BY f.occurred_at ASC
            LIMIT 1
        )
        "#
    ))
    .bind(episode_id)
    .bind(participant.as_str())
    .fetch_optional(db)
    .await?;

    if let Some(row) = existing {
        return row_to_fact(row);
    }

    let principal = Principal::User {
        user_id: participant.clone(),
    };
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::EpisodeParticipation {
        episode: AuditEpisodeId::new(episode_id),
        participant: participant.clone(),
        action: ParticipationAction::Join,
        note: trimmed_optional(note),
    };

    let mut tx = db.begin().await?;
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: episode_context.subject_id.clone(),
            domain_instantiation_id: episode_context.domain_instantiation_id.clone(),
            episode_id: episode_id.to_string(),
            payload: &payload,
            role: FactRole::Participation,
            principal_value: &principal_value,
            source_system: "csqd_public_episode_api",
        },
    )
    .await?;
    tx.commit().await?;

    Ok(fact)
}

/// Records a submitter response (acceptance, contestation, ...) against
/// existing facts in the episode. The challenged facts are preserved; the
/// response is a provenance-bearing record.
pub async fn create_submitter_response_fact(
    db: &PgPool,
    episode_id: &str,
    responder: &UserId,
    responding_to: Vec<FactId>,
    response_type: csqd_domain::FactResponseType,
    content: String,
) -> Result<Fact, RepositoryError> {
    if content.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "response content cannot be empty".to_string(),
        ));
    }

    if responding_to.is_empty() {
        return Err(RepositoryError::Domain(
            "response must reference at least one fact".to_string(),
        ));
    }

    let episode_context = find_episode_context(db, episode_id).await?;
    let reference_ids: Vec<String> = responding_to
        .iter()
        .map(|fact_id| fact_id.as_str().to_string())
        .collect();
    validate_episode_fact_references(db, episode_id, &reference_ids).await?;

    let principal = Principal::User {
        user_id: responder.clone(),
    };
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::SubmitterResponse {
        responding_to,
        response_type,
        content: content.trim().to_string(),
        revision_ref: None,
    };

    let mut tx = db.begin().await?;
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: episode_context.subject_id.clone(),
            domain_instantiation_id: episode_context.domain_instantiation_id.clone(),
            episode_id: episode_id.to_string(),
            payload: &payload,
            role: FactRole::Response,
            principal_value: &principal_value,
            source_system: "csqd_response_api",
        },
    )
    .await?;
    tx.commit().await?;

    Ok(fact)
}

/// Petition to feature someone else's ElementReview.
pub async fn create_feature_petition_fact(
    db: &PgPool,
    episode_id: &str,
    petitioner: &UserId,
    element_review: FactId,
    rationale: String,
) -> Result<Fact, RepositoryError> {
    if rationale.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "feature petition rationale cannot be empty".to_string(),
        ));
    }

    let episode_context = find_episode_context(db, episode_id).await?;
    validate_episode_fact_kind(db, episode_id, element_review.as_str(), "element_review").await?;

    let principal = Principal::User {
        user_id: petitioner.clone(),
    };
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::FeaturePetition {
        element_review,
        petitioner: petitioner.clone(),
        rationale: rationale.trim().to_string(),
    };

    let mut tx = db.begin().await?;
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: episode_context.subject_id.clone(),
            domain_instantiation_id: episode_context.domain_instantiation_id.clone(),
            episode_id: episode_id.to_string(),
            payload: &payload,
            role: FactRole::Petition,
            principal_value: &principal_value,
            source_system: "csqd_petition_api",
        },
    )
    .await?;
    tx.commit().await?;

    Ok(fact)
}

/// Petition for a new CWE element or for applicability of an existing one.
pub async fn create_cwe_petition_fact(
    db: &PgPool,
    episode_id: &str,
    petitioner: &UserId,
    kind: CWEPetitionKind,
    cwe_node: Option<CWENodeId>,
    proposed_label: Option<String>,
    rationale: String,
) -> Result<Fact, RepositoryError> {
    if rationale.trim().is_empty() {
        return Err(RepositoryError::Domain(
            "CWE petition rationale cannot be empty".to_string(),
        ));
    }

    let episode_context = find_episode_context(db, episode_id).await?;

    match (&kind, &cwe_node, &proposed_label) {
        (CWEPetitionKind::Applicability, None, _) => {
            return Err(RepositoryError::Domain(
                "applicability petition requires an existing CWE node".to_string(),
            ));
        }
        (CWEPetitionKind::NewElement, _, None) => {
            return Err(RepositoryError::Domain(
                "new element petition requires a proposed label".to_string(),
            ));
        }
        _ => {}
    }

    if let Some(node) = &cwe_node {
        validate_cwe_nodes(
            db,
            std::slice::from_ref(node),
            &episode_context.domain_instantiation_id,
        )
        .await?;
    }

    let principal = Principal::User {
        user_id: petitioner.clone(),
    };
    let principal_value = serde_json::to_value(&principal)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::CWEPetition {
        kind,
        cwe_node,
        proposed_label: trimmed_optional(proposed_label),
        petitioner: petitioner.clone(),
        rationale: rationale.trim().to_string(),
    };

    let mut tx = db.begin().await?;
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: episode_context.subject_id.clone(),
            domain_instantiation_id: episode_context.domain_instantiation_id.clone(),
            episode_id: episode_id.to_string(),
            payload: &payload,
            role: FactRole::Petition,
            principal_value: &principal_value,
            source_system: "csqd_petition_api",
        },
    )
    .await?;
    tx.commit().await?;

    Ok(fact)
}

/// Operator curation decision granting or revoking featured status. Featured
/// display state is derived from the latest active CurationDecision for a
/// target, not from mutable flags.
pub async fn create_curation_decision_fact(
    db: &PgPool,
    episode_id: &str,
    decided_by: Principal,
    target: CurationTarget,
    decision: CurationOutcome,
    rationale: Option<String>,
    petitions: Vec<FactId>,
) -> Result<Fact, RepositoryError> {
    let episode_context = find_episode_context(db, episode_id).await?;

    if let CurationTarget::ElementReview { fact_id } = &target {
        validate_episode_fact_kind(db, episode_id, fact_id.as_str(), "element_review").await?;
    }

    for petition in &petitions {
        validate_episode_fact_kind(db, episode_id, petition.as_str(), "feature_petition").await?;
    }

    let principal_value = serde_json::to_value(&decided_by)
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;
    let payload = FactPayload::CurationDecision {
        target,
        decision,
        decided_by: decided_by.clone(),
        rationale: trimmed_optional(rationale),
        petitions,
    };

    let mut tx = db.begin().await?;
    let fact = insert_fact_with_membership(
        &mut tx,
        InsertEpisodeFact {
            subject_id: episode_context.subject_id.clone(),
            domain_instantiation_id: episode_context.domain_instantiation_id.clone(),
            episode_id: episode_id.to_string(),
            payload: &payload,
            role: FactRole::Curation,
            principal_value: &principal_value,
            source_system: "csqd_curation_api",
        },
    )
    .await?;
    tx.commit().await?;

    Ok(fact)
}

pub async fn list_synthesis_reviews(
    db: &PgPool,
    episode_id: &str,
) -> Result<Vec<SynthesisReview>, RepositoryError> {
    let rows = sqlx::query(&format!(
        r#"
        SELECT {SYNTHESIS_COLUMNS}
        FROM episode_synthesis_reviews
        WHERE episode_id::text = $1
        ORDER BY authored_at DESC
        "#
    ))
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
        .clone()
        .filter(|value| !value.as_str().trim().is_empty())
        .unwrap_or_else(|| UserId::new(DEMO_REVIEWER_USER_ID));

    // Unsolicited synthesis reviews must come from an episode participant
    // (memo: start or join a public AuditEpisode first).
    if request.unsolicited {
        let participates = sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM episode_memberships em
                JOIN facts f ON f.id = em.fact_id
                WHERE em.episode_id::text = $1
                  AND em.status = 'active'
                  AND f.status = 'active'
                  AND f.payload_kind = 'episode_participation'
                  AND f.payload->'episode_participation'->>'participant' = $2
            ) AS exists
            "#,
        )
        .bind(episode_id)
        .bind(submitted_by.as_str())
        .fetch_one(db)
        .await?
        .get::<bool, _>("exists");

        if !participates {
            return Err(RepositoryError::Forbidden(
                "unsolicited synthesis reviews require starting or joining the public episode first"
                    .to_string(),
            ));
        }
    }

    let mut tx = db.begin().await?;

    let review_row = sqlx::query(&format!(
        r#"
        INSERT INTO episode_synthesis_reviews (
            episode_id,
            submitted_by,
            status,
            summary,
            featured,
            unsolicited
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            $3,
            $4,
            $5,
            $6
        )
        RETURNING {SYNTHESIS_COLUMNS}
        "#
    ))
    .bind(episode_id)
    .bind(submitted_by.as_str())
    .bind(request.status.as_db_str())
    .bind(request.summary.trim())
    .bind(request.featured)
    .bind(request.unsolicited)
    .fetch_one(&mut *tx)
    .await?;

    let review_id: String = review_row.get("id");
    for section in &request.sections {
        let referenced_facts: Vec<String> = section
            .referenced_facts
            .iter()
            .map(|value| value.as_str().trim().to_string())
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
        .bind(section.section_type.as_db_str())
        .bind(section.content.trim())
        .bind(&referenced_facts)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    row_to_synthesis_review(db, review_row).await
}

/// Parameters for a tuple recomputation request.
#[derive(Debug, Default, Clone)]
pub struct EvalTupleQuery {
    pub t_eval: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub min_endorsements: Option<u32>,
}

pub async fn compute_eval_tuple(
    db: &PgPool,
    episode_id: &str,
    query: EvalTupleQuery,
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
    let memberships = list_fact_memberships_for_episode(db, episode_id).await?;
    let synthesis_reviews = list_synthesis_reviews(db, episode_id).await?;
    let reviewer_tags = load_reviewer_tags(db).await?;
    let community = ReviewerCommunityFilter {
        tags: query.tags.into_iter().map(TagId::new).collect(),
        domain_instantiation_id: Some(DomainInstantiationId::new(
            episode_context.domain_instantiation_id.clone(),
        )),
        min_endorsements: query.min_endorsements,
    };
    let t_eval = query.t_eval.unwrap_or(episode_context.now);

    Ok(compute_eval_tuple_pure(
        &EvalTupleObservations {
            memberships: &memberships,
            synthesis_reviews: &synthesis_reviews,
        },
        &EvalTupleContext {
            community: &community,
            t_eval,
            config: &config.eval_tuple_config,
            reviewer_tags,
        },
    ))
}

async fn load_reviewer_tags(
    db: &PgPool,
) -> Result<std::collections::HashMap<UserId, Vec<TagId>>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT user_id::text AS user_id, label
        FROM reviewer_tags
        "#,
    )
    .fetch_all(db)
    .await?;
    let mut map: std::collections::HashMap<UserId, Vec<TagId>> = std::collections::HashMap::new();

    for row in rows {
        let user_id: String = row.get("user_id");
        let label: String = row.get("label");

        map.entry(UserId::new(user_id))
            .or_default()
            .push(TagId::new(label));
    }

    Ok(map)
}

pub async fn list_facts_for_subject(
    db: &PgPool,
    subject_id: &str,
) -> Result<Vec<Fact>, RepositoryError> {
    let rows = sqlx::query(&format!(
        r#"
        SELECT {FACT_COLUMNS}
        FROM facts
        WHERE subject_id::text = $1
        ORDER BY occurred_at DESC
        LIMIT 200
        "#
    ))
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
            f.occurred_at,
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

/// Facts paired with the membership that asserts episode inclusion. Includes
/// retracted memberships so derived views can honor membership status.
pub async fn list_fact_memberships_for_episode(
    db: &PgPool,
    episode_id: &str,
) -> Result<Vec<(Fact, EpisodeMembership)>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            f.id::text AS id,
            f.subject_id::text AS subject_id,
            f.domain_instantiation_id::text AS domain_instantiation_id,
            f.occurred_at,
            f.payload,
            f.status,
            f.status_metadata,
            f.provenance,
            f.external_refs,
            em.id::text AS membership_id,
            em.fact_id::text AS membership_fact_id,
            em.episode_id::text AS membership_episode_id,
            em.role AS membership_role,
            em.asserted_by AS membership_asserted_by,
            em.asserted_at AS membership_asserted_at,
            em.status AS membership_status,
            em.status_metadata AS membership_status_metadata
        FROM episode_memberships em
        JOIN facts f ON f.id = em.fact_id
        WHERE em.episode_id::text = $1
        ORDER BY f.occurred_at ASC
        "#,
    )
    .bind(episode_id)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            let membership = row_to_membership(&row)?;
            let fact = row_to_fact(row)?;

            Ok((fact, membership))
        })
        .collect()
}

struct SubjectContext {
    domain_instantiation_id: String,
}

struct EpisodeContext {
    subject_id: String,
    domain_instantiation_id: String,
    now: DateTime<Utc>,
}

struct InsertEpisodeFact<'a> {
    subject_id: String,
    domain_instantiation_id: String,
    episode_id: String,
    payload: &'a FactPayload,
    role: FactRole,
    principal_value: &'a Value,
    source_system: &'static str,
}

async fn insert_fact_with_membership(
    tx: &mut Transaction<'_, Postgres>,
    params: InsertEpisodeFact<'_>,
) -> Result<Fact, RepositoryError> {
    let payload_value = serde_json::to_value(params.payload)
        .map_err(|error| RepositoryError::Domain(format!("invalid fact payload: {error}")))?;
    let provenance = json!({
        "source_system": params.source_system,
        "source_document": null,
        "imported_at": Utc::now(),
        "principal": params.principal_value,
    });
    let fact_row = sqlx::query(&format!(
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
            $3,
            $4::jsonb,
            'active',
            $5::jsonb,
            '[]'::jsonb
        )
        RETURNING {FACT_COLUMNS}
        "#
    ))
    .bind(&params.subject_id)
    .bind(&params.domain_instantiation_id)
    .bind(params.payload.kind().as_db_str())
    .bind(&payload_value)
    .bind(&provenance)
    .fetch_one(&mut **tx)
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
            $3,
            $4::jsonb,
            'active'
        )
        "#,
    )
    .bind(fact.id.as_str())
    .bind(&params.episode_id)
    .bind(params.role.as_db_str())
    .bind(params.principal_value)
    .execute(&mut **tx)
    .await?;

    Ok(fact)
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
            now() AS now
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
    cwe_node_ids: &[CWENodeId],
    domain_instantiation_id: &str,
) -> Result<Vec<CWECriterionId>, RepositoryError> {
    let mut criteria = Vec::with_capacity(cwe_node_ids.len());

    for node_id in cwe_node_ids {
        let node_id = node_id.as_str().trim();
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

        criteria.push(CWECriterionId {
            domain: DomainInstantiationId::new(domain_instantiation_id),
            node_id: CWENodeId::new(node_id),
        });
    }

    Ok(criteria)
}

async fn find_episode_commission_fact_id(
    db: &PgPool,
    episode_id: &str,
) -> Result<FactId, RepositoryError> {
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

fn row_to_organization(row: PgRow) -> Result<Organization, RepositoryError> {
    let org_type: String = row.get("org_type");
    let org_type_detail: Option<String> = row.get("org_type_detail");

    Ok(Organization {
        id: row.get("id"),
        name: row.get("name"),
        org_type: OrganizationType::from_db(org_type.as_str(), org_type_detail.as_deref())
            .map_err(RepositoryError::Domain)?,
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
    let subject_type_detail: Option<String> = row.get("subject_type_detail");
    let sponsor_organization_type: Option<String> = row.get("sponsor_organization_type");
    let sponsor_organization_type_detail: Option<String> =
        row.get("sponsor_organization_type_detail");
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
        subject_type: AuditSubjectType::from_db(
            subject_type.as_str(),
            subject_type_detail.as_deref(),
        )
        .map_err(RepositoryError::Domain)?,
        sponsor_name: row.get("sponsor_name"),
        sponsor_organization_type: sponsor_organization_type
            .as_deref()
            .map(|kind| {
                OrganizationType::from_db(kind, sponsor_organization_type_detail.as_deref())
            })
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
    if request.cwe_node_id.as_str().trim().is_empty() {
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
    if request.cwe_node_id.as_str().trim().is_empty() {
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
    if request.solicitation_fact_id.as_str().trim().is_empty() {
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

        let referenced: Vec<String> = section
            .referenced_facts
            .iter()
            .map(|fact_id| fact_id.as_str().to_string())
            .collect();
        validate_episode_fact_references(db, episode_id, &referenced).await?;
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
        id: review_id.into(),
        episode_id: row.get("episode_id"),
        submitted_by: row.get("submitted_by"),
        authored_at: row.get("authored_at"),
        status: NarrativeStatus::try_from(status.as_str()).map_err(RepositoryError::Domain)?,
        summary: row.get("summary"),
        sections,
        featured: row.get("featured"),
        unsolicited: row.get("unsolicited"),
    })
}

fn row_to_synthesis_section(row: PgRow) -> Result<SynthesisReviewSection, RepositoryError> {
    let section_type: String = row.get("section_type");
    let referenced_facts: Vec<String> = row.get("referenced_facts");

    Ok(SynthesisReviewSection {
        id: row.get("id"),
        review_id: row.get("review_id"),
        section_type: SynthesisReviewSectionType::try_from(section_type.as_str())
            .map_err(RepositoryError::Domain)?,
        content: row.get("content"),
        referenced_facts: referenced_facts.into_iter().map(FactId::new).collect(),
    })
}

pub(crate) fn row_to_fact(row: PgRow) -> Result<Fact, RepositoryError> {
    let occurred_at: DateTime<Utc> = row.get("occurred_at");
    let payload: Value = row.get("payload");
    let status: String = row.get("status");
    let status_metadata: Value = row.get("status_metadata");
    let provenance: Value = row.get("provenance");
    let external_refs: Value = row.get("external_refs");

    Ok(Fact {
        id: row.get("id"),
        subject_id: row.get("subject_id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        occurred_at,
        payload: serde_json::from_value::<FactPayload>(payload)
            .map_err(|error| RepositoryError::Domain(format!("invalid fact payload: {error}")))?,
        status: fact_status_from_row(&status, status_metadata, occurred_at)?,
        provenance: provenance_from_row(provenance, occurred_at)?,
        external_refs: serde_json::from_value::<Vec<ExternalRef>>(external_refs)
            .map_err(|error| RepositoryError::Domain(format!("invalid external refs: {error}")))?,
    })
}

fn row_to_membership(row: &PgRow) -> Result<EpisodeMembership, RepositoryError> {
    let status: String = row.get("membership_status");
    let status_metadata: Value = row.get("membership_status_metadata");
    let asserted_by: Value = row.get("membership_asserted_by");
    let asserted_at: DateTime<Utc> = row.get("membership_asserted_at");
    let role: String = row.get("membership_role");
    let membership_status = match status.as_str() {
        "active" => EpisodeMembershipStatus::Active,
        "retracted" => EpisodeMembershipStatus::Retracted {
            retracted_by: metadata_principal(&status_metadata, "retracted_by")?,
            retracted_at: metadata_timestamp(&status_metadata, "retracted_at", asserted_at),
        },
        other => {
            return Err(RepositoryError::Domain(format!(
                "unknown membership status: {other}"
            )))
        }
    };

    Ok(EpisodeMembership {
        id: row.get("membership_id"),
        fact_id: row.get("membership_fact_id"),
        episode_id: row.get("membership_episode_id"),
        role: csqd_domain::FactRole::try_from(role.as_str()).map_err(RepositoryError::Domain)?,
        asserted_by: serde_json::from_value::<Principal>(asserted_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        asserted_at,
        status: membership_status,
    })
}

fn fact_status_from_row(
    status: &str,
    metadata: Value,
    occurred_at: DateTime<Utc>,
) -> Result<FactStatus, RepositoryError> {
    match status {
        "active" => Ok(FactStatus::Active),
        "superseded" => Ok(FactStatus::Superseded {
            superseded_by: metadata_principal(&metadata, "superseded_by")?,
            superseded_at: metadata_timestamp(&metadata, "superseded_at", occurred_at),
            replaced_by: metadata
                .get("replaced_by")
                .and_then(Value::as_str)
                .map(FactId::from),
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

fn provenance_from_row(
    value: Value,
    occurred_at: DateTime<Utc>,
) -> Result<Provenance, RepositoryError> {
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
        imported_at: metadata_timestamp(&value, "imported_at", occurred_at),
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

fn metadata_timestamp(metadata: &Value, key: &str, fallback: DateTime<Utc>) -> DateTime<Utc> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or(fallback)
}
