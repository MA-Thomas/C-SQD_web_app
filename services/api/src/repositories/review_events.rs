use csqd_domain::{
    CreateElementReviewRequest, Provenance, ReviewEvent, ReviewEventPayload, ReviewEventStatus,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};

use super::{audit_objects, RepositoryError};

const DEMO_REVIEWER_USER_ID: &str = "00000000-0000-0000-0000-000000000002";

pub async fn create_element_review_for_scholarly_object(
    db: &PgPool,
    scholarly_object_id: &str,
    request: CreateElementReviewRequest,
) -> Result<ReviewEvent, RepositoryError> {
    validate_create_element_review_request(&request)?;

    let audit_object_id =
        audit_objects::ensure_academic_for_scholarly_object(db, scholarly_object_id).await?;
    let audit_context = find_audit_object_context(db, &audit_object_id).await?;
    validate_cwe_node(
        db,
        &request.cwe_node_id,
        &audit_context.domain_instantiation_id,
    )
    .await?;

    let cwe_criterion = csqd_domain::CWECriterionId {
        domain: audit_context.domain_instantiation_id.clone(),
        node_id: request.cwe_node_id,
    };
    let payload = ReviewEventPayload::ElementReview {
        cwe_criterion,
        submitted_by: DEMO_REVIEWER_USER_ID.to_string(),
        solicitation: request.solicitation,
        finding: request.finding,
        severity: request.severity,
        confidence: request.confidence,
        content: request.content.trim().to_string(),
        featured: false,
    };
    let payload_value = serde_json::to_value(&payload)
        .map_err(|error| RepositoryError::Domain(format!("invalid review payload: {error}")))?;
    let principal_value = json!({
        "user": {
            "user_id": DEMO_REVIEWER_USER_ID
        }
    });
    let mut tx = db.begin().await?;
    let event_row = sqlx::query(
        r#"
        INSERT INTO review_events (
            audit_object_id,
            domain_instantiation_id,
            payload_kind,
            payload,
            status,
            provenance
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            'element_review',
            $3::jsonb,
            'active',
            jsonb_build_object(
                'source_system', 'csqd_web_review_workspace',
                'source_document', NULL,
                'imported_at', now()::text,
                'principal', $4::jsonb
            )
        )
        RETURNING
            id::text AS id,
            occurred_at::text AS occurred_at,
            provenance
        "#,
    )
    .bind(&audit_object_id)
    .bind(&audit_context.domain_instantiation_id)
    .bind(&payload_value)
    .bind(&principal_value)
    .fetch_one(&mut *tx)
    .await?;
    let review_event_id: String = event_row.get("id");
    let occurred_at: String = event_row.get("occurred_at");
    let provenance_value: Value = event_row.get("provenance");

    sqlx::query(
        r#"
        INSERT INTO review_event_memberships (
            review_event_id,
            audit_object_id,
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
    .bind(&review_event_id)
    .bind(&audit_object_id)
    .bind(&principal_value)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO user_library_items (
            user_id,
            audit_object_id,
            added_reason
        )
        VALUES ($1::uuid, $2::uuid, 'review_created')
        ON CONFLICT (user_id, audit_object_id) DO UPDATE SET
            added_reason = 'review_created',
            archived = false,
            updated_at = now()
        "#,
    )
    .bind(DEMO_REVIEWER_USER_ID)
    .bind(&audit_object_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let provenance = serde_json::from_value::<Provenance>(provenance_value)
        .map_err(|error| RepositoryError::Domain(format!("invalid provenance: {error}")))?;

    Ok(ReviewEvent {
        id: review_event_id,
        audit_object_id,
        domain_instantiation_id: audit_context.domain_instantiation_id,
        occurred_at,
        payload,
        status: ReviewEventStatus::Active,
        provenance,
    })
}

struct AuditObjectContext {
    domain_instantiation_id: String,
}

async fn find_audit_object_context(
    db: &PgPool,
    audit_object_id: &str,
) -> Result<AuditObjectContext, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT domain_instantiation_id::text AS domain_instantiation_id
        FROM audit_objects
        WHERE id::text = $1
        "#,
    )
    .bind(audit_object_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "audit_object",
        id: audit_object_id.to_string(),
    })?;

    Ok(AuditObjectContext {
        domain_instantiation_id: row.get("domain_instantiation_id"),
    })
}

async fn validate_cwe_node(
    db: &PgPool,
    cwe_node_id: &str,
    domain_instantiation_id: &str,
) -> Result<(), RepositoryError> {
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
    .bind(cwe_node_id)
    .bind(domain_instantiation_id)
    .fetch_one(db)
    .await?
    .get::<bool, _>("exists");

    if exists {
        Ok(())
    } else {
        Err(RepositoryError::NotFound {
            entity: "cwe_node",
            id: cwe_node_id.to_string(),
        })
    }
}

fn validate_create_element_review_request(
    request: &CreateElementReviewRequest,
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

#[cfg(test)]
mod tests {
    use csqd_domain::{CreateElementReviewRequest, Finding};

    use super::validate_create_element_review_request;

    #[test]
    fn rejects_empty_element_review_content() {
        let request = CreateElementReviewRequest {
            cwe_node_id: "criterion-1".to_string(),
            finding: Finding::Inconclusive,
            severity: None,
            confidence: None,
            content: " ".to_string(),
            solicitation: None,
        };

        assert!(validate_create_element_review_request(&request).is_err());
    }

    #[test]
    fn accepts_minimal_inconclusive_element_review() {
        let request = CreateElementReviewRequest {
            cwe_node_id: "criterion-1".to_string(),
            finding: Finding::Inconclusive,
            severity: None,
            confidence: None,
            content:
                "The available source does not provide enough detail to assess this criterion."
                    .to_string(),
            solicitation: None,
        };

        assert!(validate_create_element_review_request(&request).is_ok());
    }
}
