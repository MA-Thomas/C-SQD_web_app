use csqd_domain::{
    AuditSubject, AuditSubjectType, CreateAuditSubjectRequest, ExternalRef, Principal,
    ScopeCondition,
};
use serde_json::{json, Value};
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

const SUBJECT_COLUMNS: &str = r#"
            id::text AS id,
            domain_instantiation_id::text AS domain_instantiation_id,
            subject_type,
            subject_type_detail,
            title,
            claim_statement,
            scope_conditions,
            external_refs,
            registered_by,
            registered_at
"#;

pub async fn list(db: &PgPool) -> Result<Vec<AuditSubject>, RepositoryError> {
    let rows = sqlx::query(&format!(
        r#"
        SELECT {SUBJECT_COLUMNS}
        FROM audit_subjects
        ORDER BY updated_at DESC, registered_at DESC
        LIMIT 50
        "#
    ))
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_audit_subject).collect()
}

pub async fn create(
    db: &PgPool,
    request: CreateAuditSubjectRequest,
) -> Result<AuditSubject, RepositoryError> {
    validate_create_audit_subject_request(&request)?;
    ensure_domain_instantiation_exists(db, request.domain_instantiation_id.as_str()).await?;

    let external_refs = serde_json::to_value(&request.external_refs)
        .map_err(|error| RepositoryError::Domain(format!("invalid external refs: {error}")))?;
    let scope_conditions = serde_json::to_value(&request.scope_conditions)
        .map_err(|error| RepositoryError::Domain(format!("invalid scope conditions: {error}")))?;
    let registered_by = serde_json::to_value(request.registered_by.unwrap_or(Principal::Platform))
        .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?;

    let row = sqlx::query(&format!(
        r#"
        INSERT INTO audit_subjects (
            domain_instantiation_id,
            subject_type,
            subject_type_detail,
            title,
            claim_statement,
            scope_conditions,
            external_refs,
            registered_by
        )
        VALUES (
            $1::uuid,
            $2,
            $3,
            $4,
            $5,
            $6::jsonb,
            $7::jsonb,
            $8::jsonb
        )
        RETURNING {SUBJECT_COLUMNS}
        "#
    ))
    .bind(request.domain_instantiation_id.as_str())
    .bind(request.subject_type.db_kind())
    .bind(request.subject_type.db_detail())
    .bind(request.title.as_deref().map(str::trim))
    .bind(request.claim_statement.as_deref().map(str::trim))
    .bind(scope_conditions)
    .bind(external_refs)
    .bind(registered_by)
    .fetch_one(db)
    .await?;

    row_to_audit_subject(row)
}

pub async fn ensure_academic_for_scholarly_object(
    db: &PgPool,
    scholarly_object_id: &str,
) -> Result<String, RepositoryError> {
    let object = sqlx::query(
        r#"
        SELECT
            so.id::text AS id,
            so.object_type,
            so.doi,
            so.title,
            so.authors,
            so.abstract,
            so.license,
            so.canonical_url,
            COALESCE(j.name, 'Unknown source') AS source_name
        FROM scholarly_objects so
        LEFT JOIN journals j ON j.id = so.journal_id
        WHERE so.id::text = $1
        "#,
    )
    .bind(scholarly_object_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "scholarly_object",
        id: scholarly_object_id.to_string(),
    })?;

    let domain_instantiation_id = academic_domain_instantiation_id(db).await?;
    let object_type: String = object.get("object_type");
    let subject_type = academic_subject_type_for_object_type(&object_type);
    let title: String = object.get("title");
    let doi: Option<String> = object.get("doi");
    let canonical_url: String = object.get("canonical_url");
    let authors: Value = object.get("authors");
    let abstract_text: Option<String> = object.get("abstract");
    let license: Option<String> = object.get("license");
    let source_name: String = object.get("source_name");
    let mut external_refs = Vec::new();

    if let Some(doi) = doi.as_deref().filter(|value| !value.trim().is_empty()) {
        external_refs.push(json!({
            "system": "doi",
            "resource_type": "scholarly_work",
            "resource_id": doi,
            "uri": format!("https://doi.org/{doi}")
        }));
    }

    external_refs.push(json!({
        "system": "url",
        "resource_type": "canonical_url",
        "resource_id": canonical_url.clone(),
        "uri": canonical_url.clone()
    }));

    let row = sqlx::query(
        r#"
        INSERT INTO audit_subjects (
            domain_instantiation_id,
            subject_type,
            title,
            external_refs,
            registered_by,
            source_entity_type,
            source_entity_id,
            metadata
        )
        VALUES (
            $1::uuid,
            $2,
            $3,
            $4::jsonb,
            '"platform"'::jsonb,
            'scholarly_object',
            $5::uuid,
            $6::jsonb
        )
        ON CONFLICT (source_entity_type, source_entity_id) DO UPDATE SET
            domain_instantiation_id = EXCLUDED.domain_instantiation_id,
            subject_type = EXCLUDED.subject_type,
            title = EXCLUDED.title,
            external_refs = EXCLUDED.external_refs,
            metadata = EXCLUDED.metadata,
            updated_at = now()
        RETURNING id::text AS id
        "#,
    )
    .bind(&domain_instantiation_id)
    .bind(subject_type)
    .bind(&title)
    .bind(json!(external_refs))
    .bind(scholarly_object_id)
    .bind(json!({
        "source": "academic_publishing_intake",
        "scholarly_object_id": scholarly_object_id,
        "object_type": object_type,
        "authors": authors,
        "abstract": abstract_text,
        "license": license,
        "canonical_url": canonical_url,
        "source_name": source_name,
    }))
    .fetch_one(db)
    .await?;

    Ok(row.get("id"))
}

pub async fn find(db: &PgPool, subject_id: &str) -> Result<AuditSubject, RepositoryError> {
    let row = sqlx::query(&format!(
        r#"
        SELECT {SUBJECT_COLUMNS}
        FROM audit_subjects
        WHERE id::text = $1
        "#
    ))
    .bind(subject_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "audit_subject",
        id: subject_id.to_string(),
    })?;

    row_to_audit_subject(row)
}

fn validate_create_audit_subject_request(
    request: &CreateAuditSubjectRequest,
) -> Result<(), RepositoryError> {
    if request.domain_instantiation_id.as_str().trim().is_empty() {
        return Err(RepositoryError::Domain(
            "audit subject requires a domain instantiation".to_string(),
        ));
    }

    if matches!(request.title.as_deref(), Some(title) if title.trim().is_empty()) {
        return Err(RepositoryError::Domain(
            "audit subject title cannot be empty when provided".to_string(),
        ));
    }

    // A scoped claim is the audit object: it must be stated precisely enough
    // that reviewers can ask what would count as support, challenge,
    // limitation, or non-applicability (claim-scoped audits memo).
    if matches!(request.subject_type, AuditSubjectType::ScopedClaim)
        && request
            .claim_statement
            .as_deref()
            .map_or(true, |value| value.trim().is_empty())
    {
        return Err(RepositoryError::Domain(
            "a scoped-claim audit subject requires a claim statement".to_string(),
        ));
    }

    if request
        .scope_conditions
        .iter()
        .any(|condition| condition.label.trim().is_empty() || condition.value.trim().is_empty())
    {
        return Err(RepositoryError::Domain(
            "scope conditions require both a label and a value".to_string(),
        ));
    }

    Ok(())
}

async fn ensure_domain_instantiation_exists(
    db: &PgPool,
    domain_instantiation_id: &str,
) -> Result<(), RepositoryError> {
    let exists = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM domain_instantiations
            WHERE id::text = $1
        ) AS exists
        "#,
    )
    .bind(domain_instantiation_id)
    .fetch_one(db)
    .await?
    .get::<bool, _>("exists");

    if exists {
        Ok(())
    } else {
        Err(RepositoryError::NotFound {
            entity: "domain_instantiation",
            id: domain_instantiation_id.to_string(),
        })
    }
}

async fn academic_domain_instantiation_id(db: &PgPool) -> Result<String, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id
        FROM domain_instantiations
        WHERE domain_type = 'academic_publishing'
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "domain_instantiation",
        id: "academic_publishing".to_string(),
    })?;

    Ok(row.get("id"))
}

fn academic_subject_type_for_object_type(object_type: &str) -> &'static str {
    match object_type {
        "article" => "research_manuscript",
        "preprint" => "preprint",
        "dataset" => "dataset",
        "software" => "code_repository",
        "protocol" => "clinical_trial_protocol",
        "report" => "technical_report",
        _ => "other",
    }
}

fn row_to_audit_subject(row: PgRow) -> Result<AuditSubject, RepositoryError> {
    let subject_type: String = row.get("subject_type");
    let subject_type_detail: Option<String> = row.get("subject_type_detail");
    let scope_conditions: Value = row.get("scope_conditions");
    let external_refs: Value = row.get("external_refs");
    let registered_by: Value = row.get("registered_by");

    Ok(AuditSubject {
        id: row.get("id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        subject_type: AuditSubjectType::from_db(
            subject_type.as_str(),
            subject_type_detail.as_deref(),
        )
        .map_err(RepositoryError::Domain)?,
        title: row.get("title"),
        claim_statement: row.get("claim_statement"),
        scope_conditions: serde_json::from_value::<Vec<ScopeCondition>>(scope_conditions).map_err(
            |error| RepositoryError::Domain(format!("invalid scope conditions: {error}")),
        )?,
        external_refs: serde_json::from_value::<Vec<ExternalRef>>(external_refs)
            .map_err(|error| RepositoryError::Domain(format!("invalid external refs: {error}")))?,
        registered_by: serde_json::from_value::<Principal>(registered_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
        registered_at: row.get("registered_at"),
    })
}
