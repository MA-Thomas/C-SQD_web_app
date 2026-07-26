//! PostgreSQL persistence for the pure `csqd-identity` event model.
//!
//! The ledger is append-only and canonical for replay. Relational tables are
//! transactionally maintained query indexes; they never replace event
//! validation by the Rust projection.

use csqd_domain::{IdentityEventId, Principal, Timestamp};
use csqd_identity::{
    project_identity_state, AccessDecision, AccountPrincipalLink, AuditedPrincipalReference,
    AuditedRepresentation, AuthorityGrant, AuthorityRevocation, IdentityEvent,
    IdentityEventPayload, IdentityPrincipal, IdentityState, OrganizationPrincipalLink,
};
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgConnection, PgPool, Row};

use super::RepositoryError;

/// Provenance assigned to one newly appended identity event.
#[derive(Debug, Clone)]
pub struct LedgerMetadata {
    pub event_id: IdentityEventId,
    pub recorded_at: Timestamp,
    pub recorded_by: Principal,
}

/// Loads and validates the complete ordered identity ledger.
pub async fn load_events(db: &PgPool) -> Result<Vec<IdentityEvent>, RepositoryError> {
    let mut connection = db.acquire().await?;
    load_events_from(&mut connection).await
}

/// Rebuilds identity state from the persisted event ledger.
pub async fn replay(db: &PgPool) -> Result<IdentityState, RepositoryError> {
    let events = load_events(db).await?;
    project_identity_state(&events).map_err(|error| {
        RepositoryError::Domain(format!("invalid persisted identity event ledger: {error}"))
    })
}

/// Returns grants active under the same temporal rules as `IdentityState`.
///
/// The query excludes grants for inactive actors, inactive or unlinked
/// represented organizations, grants outside their validity window, and
/// grants revoked at or before `at`.
pub async fn active_grants_for(
    db: &PgPool,
    actor_principal_id: &str,
    at: &Timestamp,
) -> Result<Vec<AuthorityGrant>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT g.record
        FROM authority_grants AS g
        JOIN identity_principals AS actor
          ON actor.id = g.actor_principal_id
        LEFT JOIN identity_principals AS represented
          ON represented.id = g.represented_organization_principal_id
        LEFT JOIN organization_principal_links AS organization_link
          ON organization_link.principal_id = represented.id
        WHERE g.actor_principal_id = $1::uuid
          AND g.issued_at <= $2
          AND actor.status = 'active'
          AND actor.created_at <= $2
          AND (
              g.represented_organization_principal_id IS NULL
              OR (
                  represented.status = 'active'
                  AND represented.created_at <= $2
                  AND organization_link.principal_id IS NOT NULL
              )
          )
          AND (g.valid_from IS NULL OR g.valid_from <= $2)
          AND (g.valid_until IS NULL OR $2 < g.valid_until)
          AND NOT EXISTS (
              SELECT 1
              FROM authority_revocations AS r
              WHERE r.grant_id = g.id
                AND r.revoked_at <= $2
          )
        ORDER BY g.issued_at, g.id
        "#,
    )
    .bind(actor_principal_id)
    .bind(at)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| deserialize_record(row.get("record"), "authority grant"))
        .collect()
}

/// Atomically creates a human principal and its account link, including both
/// events. Projection replay occurs before relational indexes are committed.
pub async fn create_account_principal(
    db: &PgPool,
    principal: &IdentityPrincipal,
    link: &AccountPrincipalLink,
    principal_event: LedgerMetadata,
    link_event: LedgerMetadata,
) -> Result<(IdentityEvent, IdentityEvent), RepositoryError> {
    let mut tx = db.begin().await?;
    lock_ledger(&mut tx).await?;

    let principal_event = append_event(
        &mut tx,
        principal_event,
        IdentityEventPayload::PrincipalCreated {
            principal: principal.clone(),
        },
    )
    .await?;
    let link_event = append_event(
        &mut tx,
        link_event,
        IdentityEventPayload::AccountPrincipalLinked { link: link.clone() },
    )
    .await?;
    validate_transaction_ledger(&mut tx).await?;

    let principal_record = serialize_record(principal, "identity principal")?;
    sqlx::query(
        r#"
        INSERT INTO identity_principals (
            id, kind, status, display_name, created_at, created_by, record
        )
        VALUES (
            $1::uuid, $2, 'active', $3, $4, $5::jsonb, $6::jsonb
        )
        "#,
    )
    .bind(principal.id.as_str())
    .bind(enum_name(&principal.kind, "identity principal kind")?)
    .bind(&principal.display_name)
    .bind(principal.created_at)
    .bind(serialize_record(
        &principal.created_by,
        "principal creator",
    )?)
    .bind(principal_record)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO account_principal_links (
            id, account_id, principal_id, status, established_by,
            established_at, record
        )
        VALUES (
            $1::uuid, $2::uuid, $3::uuid, 'active', $4::jsonb, $5, $6::jsonb
        )
        "#,
    )
    .bind(link.id.as_str())
    .bind(link.account_id.as_str())
    .bind(link.principal_id.as_str())
    .bind(serialize_record(&link.established_by, "link provenance")?)
    .bind(link.established_at)
    .bind(serialize_record(link, "account principal link")?)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((principal_event, link_event))
}

/// Atomically creates an organization principal and its one-to-one business
/// record link.
pub async fn create_organization_principal(
    db: &PgPool,
    principal: &IdentityPrincipal,
    link: &OrganizationPrincipalLink,
    principal_event: LedgerMetadata,
    link_event: LedgerMetadata,
) -> Result<(IdentityEvent, IdentityEvent), RepositoryError> {
    let mut tx = db.begin().await?;
    lock_ledger(&mut tx).await?;

    let principal_event = append_event(
        &mut tx,
        principal_event,
        IdentityEventPayload::PrincipalCreated {
            principal: principal.clone(),
        },
    )
    .await?;
    let link_event = append_event(
        &mut tx,
        link_event,
        IdentityEventPayload::OrganizationPrincipalLinked { link: link.clone() },
    )
    .await?;
    validate_transaction_ledger(&mut tx).await?;

    sqlx::query(
        r#"
        INSERT INTO identity_principals (
            id, kind, status, display_name, created_at, created_by, record
        )
        VALUES (
            $1::uuid, $2, 'active', $3, $4, $5::jsonb, $6::jsonb
        )
        "#,
    )
    .bind(principal.id.as_str())
    .bind(enum_name(&principal.kind, "identity principal kind")?)
    .bind(&principal.display_name)
    .bind(principal.created_at)
    .bind(serialize_record(
        &principal.created_by,
        "principal creator",
    )?)
    .bind(serialize_record(principal, "identity principal")?)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO organization_principal_links (
            organization_id, principal_id, established_by, established_at, record
        )
        VALUES ($1::uuid, $2::uuid, $3::jsonb, $4, $5::jsonb)
        "#,
    )
    .bind(link.organization_id.as_str())
    .bind(link.principal_id.as_str())
    .bind(serialize_record(&link.established_by, "link provenance")?)
    .bind(link.established_at)
    .bind(serialize_record(link, "organization principal link")?)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((principal_event, link_event))
}

/// Appends and indexes one immutable authority grant in a transaction.
pub async fn grant_authority(
    db: &PgPool,
    grant: &AuthorityGrant,
    metadata: LedgerMetadata,
) -> Result<IdentityEvent, RepositoryError> {
    let mut tx = db.begin().await?;
    lock_ledger(&mut tx).await?;
    let event = append_event(
        &mut tx,
        metadata,
        IdentityEventPayload::AuthorityGranted {
            grant: grant.clone(),
        },
    )
    .await?;
    validate_transaction_ledger(&mut tx).await?;
    insert_grant(&mut tx, grant).await?;
    tx.commit().await?;
    Ok(event)
}

/// Appends and indexes one immutable revocation in a transaction.
pub async fn revoke_authority(
    db: &PgPool,
    revocation: &AuthorityRevocation,
    metadata: LedgerMetadata,
) -> Result<IdentityEvent, RepositoryError> {
    let mut tx = db.begin().await?;
    lock_ledger(&mut tx).await?;
    let event = append_event(
        &mut tx,
        metadata,
        IdentityEventPayload::AuthorityRevoked {
            revocation: revocation.clone(),
        },
    )
    .await?;
    validate_transaction_ledger(&mut tx).await?;
    insert_revocation(&mut tx, revocation).await?;
    tx.commit().await?;
    Ok(event)
}

/// Revokes one grant and creates its replacement as one atomic ledger change.
pub async fn replace_authority_grant(
    db: &PgPool,
    revocation: &AuthorityRevocation,
    revocation_metadata: LedgerMetadata,
    replacement: &AuthorityGrant,
    replacement_metadata: LedgerMetadata,
) -> Result<(IdentityEvent, IdentityEvent), RepositoryError> {
    let mut tx = db.begin().await?;
    lock_ledger(&mut tx).await?;
    let revocation_event = append_event(
        &mut tx,
        revocation_metadata,
        IdentityEventPayload::AuthorityRevoked {
            revocation: revocation.clone(),
        },
    )
    .await?;
    let replacement_event = append_event(
        &mut tx,
        replacement_metadata,
        IdentityEventPayload::AuthorityGranted {
            grant: replacement.clone(),
        },
    )
    .await?;
    validate_transaction_ledger(&mut tx).await?;
    insert_revocation(&mut tx, revocation).await?;
    insert_grant(&mut tx, replacement).await?;
    tx.commit().await?;
    Ok((revocation_event, replacement_event))
}

/// Persists an authorization decision only if replay validates its exact basis
/// against the prior ledger prefix.
pub async fn record_access_decision(
    db: &PgPool,
    decision: &AccessDecision,
    metadata: LedgerMetadata,
) -> Result<IdentityEvent, RepositoryError> {
    let mut tx = db.begin().await?;
    lock_ledger(&mut tx).await?;
    let event = append_event(
        &mut tx,
        metadata,
        IdentityEventPayload::AccessDecisionRecorded {
            decision: decision.clone(),
        },
    )
    .await?;
    validate_transaction_ledger(&mut tx).await?;

    sqlx::query(
        r#"
        INSERT INTO identity_access_decisions (
            id, account_id, actor_reference, representation_reference,
            action, scope, outcome,
            policy_id, reason_codes, evaluated_at, record
        )
        VALUES (
            $1::uuid, $2::uuid, $3::jsonb, $4::jsonb, $5, $6::jsonb, $7,
            $8, $9::jsonb, $10, $11::jsonb
        )
        "#,
    )
    .bind(decision.id().as_str())
    .bind(decision.account_id().as_str())
    .bind(serialize_record(
        decision.actor_reference(),
        "audited actor reference",
    )?)
    .bind(serialize_record(
        decision.representation(),
        "audited representation",
    )?)
    .bind(enum_name(&decision.action(), "authorized action")?)
    .bind(serialize_record(decision.scope(), "resource scope")?)
    .bind(enum_name(&decision.outcome(), "authorization outcome")?)
    .bind(decision.policy_id().as_str())
    .bind(serialize_record(decision.reason_codes(), "reason codes")?)
    .bind(decision.evaluated_at())
    .bind(serialize_record(decision, "access decision")?)
    .execute(&mut *tx)
    .await?;

    if let AuditedPrincipalReference::Known(principal_id) = decision.actor_reference() {
        sqlx::query(
            r#"
            INSERT INTO identity_access_decision_actor_principals (
                decision_id, principal_id
            )
            VALUES ($1::uuid, $2::uuid)
            "#,
        )
        .bind(decision.id().as_str())
        .bind(principal_id.as_str())
        .execute(&mut *tx)
        .await?;
    }

    if let AuditedRepresentation::Known(principal_id) = decision.representation() {
        sqlx::query(
            r#"
            INSERT INTO identity_access_decision_organization_principals (
                decision_id, principal_id
            )
            VALUES ($1::uuid, $2::uuid)
            "#,
        )
        .bind(decision.id().as_str())
        .bind(principal_id.as_str())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(event)
}

async fn lock_ledger(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), RepositoryError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext('csqd_identity_event_ledger'))")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn append_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    metadata: LedgerMetadata,
    payload: IdentityEventPayload,
) -> Result<IdentityEvent, RepositoryError> {
    let payload_value = serialize_record(&payload, "identity event payload")?;
    let row = sqlx::query(
        r#"
        INSERT INTO identity_events (id, recorded_at, recorded_by, payload)
        VALUES ($1::uuid, $2, $3::jsonb, $4::jsonb)
        RETURNING append_sequence
        "#,
    )
    .bind(metadata.event_id.as_str())
    .bind(metadata.recorded_at)
    .bind(serialize_record(&metadata.recorded_by, "event recorder")?)
    .bind(payload_value)
    .fetch_one(&mut **tx)
    .await?;
    let sequence: i64 = row.get("append_sequence");
    let append_sequence = u64::try_from(sequence).map_err(|_| {
        RepositoryError::Domain(format!("invalid identity append sequence: {sequence}"))
    })?;

    IdentityEvent::new(
        metadata.event_id,
        append_sequence,
        metadata.recorded_at,
        metadata.recorded_by,
        payload,
    )
    .map_err(|error| RepositoryError::Domain(format!("invalid identity event: {error}")))
}

async fn validate_transaction_ledger(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), RepositoryError> {
    let events = load_events_from(tx).await?;
    project_identity_state(&events).map_err(|error| {
        RepositoryError::Domain(format!("identity event would invalidate ledger: {error}"))
    })?;
    Ok(())
}

async fn load_events_from(
    connection: &mut PgConnection,
) -> Result<Vec<IdentityEvent>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            append_sequence,
            recorded_at,
            recorded_by,
            payload
        FROM identity_events
        ORDER BY append_sequence
        "#,
    )
    .fetch_all(connection)
    .await?;

    rows.into_iter()
        .map(|row| {
            let sequence: i64 = row.get("append_sequence");
            let append_sequence = u64::try_from(sequence).map_err(|_| {
                RepositoryError::Domain(format!(
                    "invalid persisted identity append sequence: {sequence}"
                ))
            })?;
            let payload =
                deserialize_record::<IdentityEventPayload>(row.get("payload"), "event payload")?;
            let recorded_by =
                deserialize_record::<Principal>(row.get("recorded_by"), "event recorder")?;
            IdentityEvent::new(
                IdentityEventId::new(row.get::<String, _>("id")),
                append_sequence,
                row.get("recorded_at"),
                recorded_by,
                payload,
            )
            .map_err(|error| {
                RepositoryError::Domain(format!("invalid persisted identity event: {error}"))
            })
        })
        .collect()
}

async fn insert_grant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    grant: &AuthorityGrant,
) -> Result<(), RepositoryError> {
    let (valid_from, valid_until) = grant
        .validity()
        .map(|period| (Some(&period.valid_from), period.valid_until.as_ref()))
        .unwrap_or((None, None));
    sqlx::query(
        r#"
        INSERT INTO authority_grants (
            id, actor_principal_id, represented_organization_principal_id,
            authority_kind, scope, permitted_actions, issued_by_principal_id,
            issued_at, valid_from, valid_until, evidence_refs, record
        )
        VALUES (
            $1::uuid, $2::uuid, $3::uuid, $4, $5::jsonb, $6::jsonb,
            $7::uuid, $8, $9, $10, $11::jsonb, $12::jsonb
        )
        "#,
    )
    .bind(grant.id().as_str())
    .bind(grant.actor_principal_id().as_str())
    .bind(
        grant
            .represented_organization_principal_id()
            .map(|id| id.as_str()),
    )
    .bind(enum_name(grant.kind(), "authority kind")?)
    .bind(serialize_record(grant.scope(), "resource scope")?)
    .bind(serialize_record(
        grant.permitted_actions(),
        "permitted actions",
    )?)
    .bind(grant.issued_by_principal_id().as_str())
    .bind(grant.issued_at())
    .bind(valid_from)
    .bind(valid_until)
    .bind(serialize_record(grant.evidence_refs(), "grant evidence")?)
    .bind(serialize_record(grant, "authority grant")?)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_revocation(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    revocation: &AuthorityRevocation,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO authority_revocations (
            id, grant_id, revoked_by_principal_id, revoked_at, reason, record
        )
        VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6::jsonb)
        "#,
    )
    .bind(revocation.id.as_str())
    .bind(revocation.grant_id.as_str())
    .bind(revocation.revoked_by_principal_id.as_str())
    .bind(revocation.revoked_at)
    .bind(&revocation.reason)
    .bind(serialize_record(revocation, "authority revocation")?)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn enum_name<T: Serialize>(value: &T, label: &str) -> Result<String, RepositoryError> {
    match serialize_record(value, label)? {
        Value::String(value) => Ok(value),
        other => Err(RepositoryError::Domain(format!(
            "{label} should serialize as a string, got {other}"
        ))),
    }
}

fn serialize_record<T: Serialize + ?Sized>(
    value: &T,
    label: &str,
) -> Result<Value, RepositoryError> {
    serde_json::to_value(value)
        .map_err(|error| RepositoryError::Domain(format!("invalid {label}: {error}")))
}

fn deserialize_record<T: serde::de::DeserializeOwned>(
    value: Value,
    label: &str,
) -> Result<T, RepositoryError> {
    serde_json::from_value(value)
        .map_err(|error| RepositoryError::Domain(format!("invalid persisted {label}: {error}")))
}
