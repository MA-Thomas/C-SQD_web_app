use std::env;

use chrono::{Duration, Utc};
use csqd_api::repositories::identity::{
    active_grants_for, create_account_principal, grant_authority, load_events,
    record_access_decision, replace_authority_grant, LedgerMetadata,
};
use csqd_domain::{
    AccessDecisionId, AccountPrincipalLinkId, AuthorityGrantId, AuthorityRevocationId,
    IdentityEventId, IdentityPrincipalId, PolicyId, Principal, UserId,
};
use csqd_identity::{
    project_identity_state, AccessDecision, AccessDecisionResult, AccountPrincipalLink,
    AssuranceLevel, AuditedPrincipalReference, AuditedRepresentation, AuthenticationMethod,
    AuthorityGrant, AuthorityKind, AuthorityRevocation, AuthorizationRequest, AuthorizedAction,
    IdentityPrincipal, IdentityPrincipalKind, LinkStatus, NewAccessDecision, NewAuthorityGrant,
    PolicyReasonCode, ResourceScope,
};
use rand::Rng;
use sqlx::{Connection, Executor, PgConnection, PgPool, Row};

const MIGRATIONS: [&str; 5] = [
    include_str!("../../../db/migrations/000001_initial_schema.sql"),
    include_str!("../../../db/migrations/000002_fen_alignment.sql"),
    include_str!("../../../db/migrations/000003_claim_scoped_audits.sql"),
    include_str!("../../../db/migrations/000004_commercial_lifecycle.sql"),
    include_str!("../../../db/migrations/000005_identity_persistence.sql"),
];

const SEEDS: [&str; 3] = [
    include_str!("../../../db/seeds/000001_demo_data.sql"),
    include_str!("../../../db/seeds/000002_status_showcase.sql"),
    include_str!("../../../db/seeds/000003_claim_scoped_demo.sql"),
];

struct TestDatabase {
    admin_url: String,
    database_name: String,
    pool: PgPool,
}

impl TestDatabase {
    async fn create() -> Self {
        let admin_url = env::var("CSQD_TEST_DATABASE_ADMIN_URL").expect(
            "CSQD_TEST_DATABASE_ADMIN_URL must name a PostgreSQL admin database used only for tests",
        );
        let suffix: u64 = rand::thread_rng().gen();
        let database_name = format!("csqd_identity_test_{suffix:016x}");
        let mut admin = PgConnection::connect(&admin_url)
            .await
            .expect("test database administrator should connect");
        admin
            .execute(format!("CREATE DATABASE {database_name}").as_str())
            .await
            .expect("disposable test database should be created");
        drop(admin);

        let (prefix, _) = admin_url
            .rsplit_once('/')
            .expect("admin URL should end with a database name");
        let database_url = format!("{prefix}/{database_name}");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("disposable test database should connect");

        Self {
            admin_url,
            database_name,
            pool,
        }
    }

    async fn apply_migrations(&self, through: usize) {
        for migration in MIGRATIONS.iter().take(through) {
            self.pool
                .execute(*migration)
                .await
                .expect("migration should apply");
        }
    }

    async fn apply_seeds(&self) {
        for seed in SEEDS {
            self.pool.execute(seed).await.expect("seed should apply");
        }
    }

    async fn destroy(self) {
        self.pool.close().await;
        let mut admin = PgConnection::connect(&self.admin_url)
            .await
            .expect("test database administrator should reconnect");
        admin
            .execute(format!("DROP DATABASE {} WITH (FORCE)", self.database_name).as_str())
            .await
            .expect("disposable test database should be removed");
    }
}

#[tokio::test]
#[ignore = "requires CSQD_TEST_DATABASE_ADMIN_URL and creates a disposable database"]
async fn migration_upgrades_seeded_database_and_backfill_is_idempotent() {
    let database = TestDatabase::create().await;
    database.apply_migrations(4).await;
    database.apply_seeds().await;
    database
        .pool
        .execute(MIGRATIONS[4])
        .await
        .expect("identity migration should upgrade seeded database");

    let before = identity_counts(&database.pool).await;
    database
        .pool
        .execute(MIGRATIONS[4])
        .await
        .expect("identity migration should be idempotent");
    let after = identity_counts(&database.pool).await;
    assert_eq!(before, after);

    let coverage = sqlx::query(
        r#"
        SELECT
            (SELECT count(*) FROM users) AS users,
            (SELECT count(*) FROM account_principal_links WHERE status = 'active') AS account_links,
            (SELECT count(*) FROM organizations) AS organizations,
            (SELECT count(*) FROM organization_principal_links) AS organization_links,
            (
                SELECT count(*)
                FROM audit_episodes
                WHERE authored_by ? 'organization'
            ) AS organization_episodes,
            (
                SELECT count(*)
                FROM audit_episode_sponsorships
                WHERE legacy_backfill_status = 'actor_attribution_required'
            ) AS legacy_sponsorships
        "#,
    )
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(
        coverage.get::<i64, _>("users"),
        coverage.get::<i64, _>("account_links")
    );
    assert_eq!(
        coverage.get::<i64, _>("organizations"),
        coverage.get::<i64, _>("organization_links")
    );
    assert_eq!(
        coverage.get::<i64, _>("organization_episodes"),
        coverage.get::<i64, _>("legacy_sponsorships")
    );

    let events = load_events(&database.pool).await.unwrap();
    let state = project_identity_state(&events).expect("backfilled ledger should replay");
    assert_eq!(state.event_count(), events.len());

    database.destroy().await;
}

#[tokio::test]
#[ignore = "requires CSQD_TEST_DATABASE_ADMIN_URL and creates a disposable database"]
async fn database_constraints_reject_invalid_and_duplicate_identity_state() {
    let database = TestDatabase::create().await;
    database.apply_migrations(5).await;

    let invalid_kind = sqlx::query(
        r#"
        INSERT INTO identity_principals (
            kind, status, display_name, created_at, created_by, record
        )
        VALUES ('person', 'active', 'Invalid', now(), '"platform"', '{}')
        "#,
    )
    .execute(&database.pool)
    .await;
    assert!(invalid_kind.is_err());

    sqlx::query(
        r#"
        INSERT INTO users (id, email, display_name, role)
        VALUES (
            '10000000-0000-0000-0000-000000000001',
            'constraint-test@csqd.local',
            'Constraint Test',
            'reader'
        )
        "#,
    )
    .execute(&database.pool)
    .await
    .unwrap();
    for id in [
        "10000000-0000-0000-0000-000000000011",
        "10000000-0000-0000-0000-000000000012",
    ] {
        sqlx::query(
            r#"
            INSERT INTO identity_principals (
                id, kind, status, display_name, created_at, created_by, record
            )
            VALUES ($1::uuid, 'human', 'active', 'Human', now(), '"platform"', '{}')
            "#,
        )
        .bind(id)
        .execute(&database.pool)
        .await
        .unwrap();
    }
    sqlx::query(
        r#"
        INSERT INTO account_principal_links (
            account_id, principal_id, status, established_by, established_at, record
        )
        VALUES (
            '10000000-0000-0000-0000-000000000001',
            '10000000-0000-0000-0000-000000000011',
            'active',
            '"platform"',
            now(),
            '{}'
        )
        "#,
    )
    .execute(&database.pool)
    .await
    .unwrap();
    let duplicate = sqlx::query(
        r#"
        INSERT INTO account_principal_links (
            account_id, principal_id, status, established_by, established_at, record
        )
        VALUES (
            '10000000-0000-0000-0000-000000000001',
            '10000000-0000-0000-0000-000000000012',
            'active',
            '"platform"',
            now(),
            '{}'
        )
        "#,
    )
    .execute(&database.pool)
    .await;
    assert!(duplicate.is_err());

    let invalid_foreign_key = sqlx::query(
        r#"
        INSERT INTO authority_revocations (
            grant_id, revoked_by_principal_id, revoked_at, reason, record
        )
        VALUES (
            '10000000-0000-0000-0000-000000000099',
            '10000000-0000-0000-0000-000000000011',
            now(),
            'invalid target',
            '{}'
        )
        "#,
    )
    .execute(&database.pool)
    .await;
    assert!(invalid_foreign_key.is_err());

    let invalid_audited_reference = sqlx::query(
        r#"
        INSERT INTO identity_access_decisions (
            id,
            account_id,
            actor_reference,
            representation_reference,
            action,
            scope,
            outcome,
            policy_id,
            reason_codes,
            evaluated_at,
            record
        )
        VALUES (
            '10000000-0000-0000-0000-000000000021',
            '10000000-0000-0000-0000-000000000001',
            '[]',
            '"none"',
            'manage_accounts',
            '"platform"',
            'denied',
            'identity-policy-v1',
            '["account_principal_mismatch"]',
            now(),
            '{}'
        )
        "#,
    )
    .execute(&database.pool)
    .await;
    assert!(invalid_audited_reference.is_err());

    database.destroy().await;
}

#[tokio::test]
#[ignore = "requires CSQD_TEST_DATABASE_ADMIN_URL and creates a disposable database"]
async fn repository_grant_query_matches_replay_and_failed_replacement_rolls_back() {
    let database = TestDatabase::create().await;
    database.apply_migrations(5).await;

    let account_id = UserId::new("20000000-0000-0000-0000-000000000001");
    sqlx::query(
        r#"
        INSERT INTO users (id, email, display_name, role)
        VALUES ($1::uuid, 'repository-test@csqd.local', 'Repository Test', 'reader')
        "#,
    )
    .bind(account_id.as_str())
    .execute(&database.pool)
    .await
    .unwrap();

    let created_at = Utc::now();
    let actor_id = IdentityPrincipalId::new("20000000-0000-0000-0000-000000000011");
    let principal = IdentityPrincipal::new(
        actor_id.clone(),
        IdentityPrincipalKind::Human,
        "Repository Test",
        created_at,
        Principal::Platform,
    )
    .unwrap();
    let link = AccountPrincipalLink {
        id: AccountPrincipalLinkId::new("20000000-0000-0000-0000-000000000021"),
        account_id,
        principal_id: actor_id.clone(),
        status: LinkStatus::Active,
        established_by: Principal::Platform,
        established_at: created_at,
    };
    create_account_principal(
        &database.pool,
        &principal,
        &link,
        metadata("20000000-0000-0000-0000-000000000031", created_at),
        metadata("20000000-0000-0000-0000-000000000032", created_at),
    )
    .await
    .unwrap();

    let issuer_id = IdentityPrincipalId::new(
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT id::text
            FROM identity_principals
            WHERE source_entity_type = 'platform_authority_service'
            "#,
        )
        .fetch_one(&database.pool)
        .await
        .unwrap(),
    );
    let issued_at = created_at + Duration::milliseconds(1);
    let original = authority_grant(
        "20000000-0000-0000-0000-000000000041",
        &actor_id,
        &issuer_id,
        issued_at,
    );
    grant_authority(
        &database.pool,
        &original,
        metadata("20000000-0000-0000-0000-000000000051", issued_at),
    )
    .await
    .unwrap();

    let query_grants = active_grants_for(&database.pool, actor_id.as_str(), &issued_at)
        .await
        .unwrap();
    let events = load_events(&database.pool).await.unwrap();
    let state = project_identity_state(&events).unwrap();
    let projected_grants = state.active_grants_for(&actor_id, &issued_at);
    assert_eq!(
        query_grants
            .iter()
            .map(|grant| grant.id().as_str())
            .collect::<Vec<_>>(),
        projected_grants
            .iter()
            .map(|grant| grant.id().as_str())
            .collect::<Vec<_>>()
    );

    let event_count_before: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_events")
        .fetch_one(&database.pool)
        .await
        .unwrap();
    let failed_at = issued_at + Duration::milliseconds(1);
    let failed_revocation = AuthorityRevocation::new(
        AuthorityRevocationId::new("20000000-0000-0000-0000-000000000061"),
        original.id().clone(),
        issuer_id.clone(),
        failed_at,
        "replacement test",
    )
    .unwrap();
    let duplicate_replacement =
        authority_grant(original.id().as_str(), &actor_id, &issuer_id, failed_at);
    let failed = replace_authority_grant(
        &database.pool,
        &failed_revocation,
        metadata("20000000-0000-0000-0000-000000000071", failed_at),
        &duplicate_replacement,
        metadata("20000000-0000-0000-0000-000000000072", failed_at),
    )
    .await;
    assert!(failed.is_err());
    let event_count_after: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_events")
        .fetch_one(&database.pool)
        .await
        .unwrap();
    let revocation_count: i64 = sqlx::query_scalar("SELECT count(*) FROM authority_revocations")
        .fetch_one(&database.pool)
        .await
        .unwrap();
    assert_eq!(event_count_before, event_count_after);
    assert_eq!(revocation_count, 0);

    let replaced_at = failed_at + Duration::milliseconds(1);
    let revocation = AuthorityRevocation::new(
        AuthorityRevocationId::new("20000000-0000-0000-0000-000000000062"),
        original.id().clone(),
        issuer_id.clone(),
        replaced_at,
        "rotate grant",
    )
    .unwrap();
    let replacement = authority_grant(
        "20000000-0000-0000-0000-000000000042",
        &actor_id,
        &issuer_id,
        replaced_at,
    );
    replace_authority_grant(
        &database.pool,
        &revocation,
        metadata("20000000-0000-0000-0000-000000000073", replaced_at),
        &replacement,
        metadata("20000000-0000-0000-0000-000000000074", replaced_at),
    )
    .await
    .unwrap();

    let active = active_grants_for(&database.pool, actor_id.as_str(), &replaced_at)
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id(), replacement.id());

    database.destroy().await;
}

#[tokio::test]
#[ignore = "requires CSQD_TEST_DATABASE_ADMIN_URL and creates a disposable database"]
async fn denied_decision_persists_unresolved_principals_in_canonical_json() {
    let database = TestDatabase::create().await;
    database.apply_migrations(5).await;

    let account_id = UserId::new("30000000-0000-0000-0000-000000000001");
    sqlx::query(
        r#"
        INSERT INTO users (id, email, display_name, role)
        VALUES ($1::uuid, 'denial-test@csqd.local', 'Denial Test', 'reader')
        "#,
    )
    .bind(account_id.as_str())
    .execute(&database.pool)
    .await
    .unwrap();

    let evaluated_at = Utc::now();
    let decision_id = AccessDecisionId::new("30000000-0000-0000-0000-000000000011");
    let decision = AccessDecision::new(NewAccessDecision {
        id: decision_id.clone(),
        account_id,
        actor_reference: AuditedPrincipalReference::Unresolved(IdentityPrincipalId::new(
            "unresolved-actor",
        )),
        representation: AuditedRepresentation::Unresolved(IdentityPrincipalId::new(
            "unresolved-organization",
        )),
        authentication_method: AuthenticationMethod::MagicLink,
        authentication_assurance: AssuranceLevel::Low,
        authenticated_at: evaluated_at,
        request: AuthorizationRequest::access(
            AuthorizedAction::ManageAccounts,
            ResourceScope::Platform,
        )
        .unwrap(),
        result: AccessDecisionResult::Denied {
            reasons: vec![PolicyReasonCode::AccountPrincipalMismatch],
        },
        policy_id: PolicyId::new("identity-policy-v1"),
        evaluated_at,
    })
    .unwrap();
    record_access_decision(
        &database.pool,
        &decision,
        metadata("30000000-0000-0000-0000-000000000021", evaluated_at),
    )
    .await
    .unwrap();

    let row = sqlx::query(
        r#"
        SELECT
            actor_reference,
            representation_reference,
            reason_codes,
            record
        FROM identity_access_decisions
        WHERE id = $1::uuid
        "#,
    )
    .bind(decision_id.as_str())
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(
        row.get::<serde_json::Value, _>("actor_reference"),
        serde_json::json!({"unresolved": "unresolved-actor"})
    );
    assert_eq!(
        row.get::<serde_json::Value, _>("representation_reference"),
        serde_json::json!({"unresolved": "unresolved-organization"})
    );
    assert_eq!(
        row.get::<serde_json::Value, _>("reason_codes"),
        serde_json::json!(["account_principal_mismatch"])
    );
    let record = row.get::<serde_json::Value, _>("record");
    assert_eq!(
        record["actor_reference"],
        serde_json::json!({"unresolved": "unresolved-actor"})
    );
    assert_eq!(
        record["representation"],
        serde_json::json!({"unresolved": "unresolved-organization"})
    );
    let resolved_actor_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM identity_access_decision_actor_principals WHERE decision_id = $1::uuid",
    )
    .bind(decision_id.as_str())
    .fetch_one(&database.pool)
    .await
    .unwrap();
    let resolved_organization_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM identity_access_decision_organization_principals WHERE decision_id = $1::uuid",
    )
    .bind(decision_id.as_str())
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(resolved_actor_rows, 0);
    assert_eq!(resolved_organization_rows, 0);

    let state = project_identity_state(&load_events(&database.pool).await.unwrap()).unwrap();
    assert!(state.access_decision(&decision_id).is_some());

    database.destroy().await;
}

async fn identity_counts(pool: &PgPool) -> (i64, i64, i64, i64, i64, i64) {
    let row = sqlx::query(
        r#"
        SELECT
            (SELECT count(*) FROM identity_principals) AS principals,
            (SELECT count(*) FROM account_principal_links) AS account_links,
            (SELECT count(*) FROM organization_principal_links) AS organization_links,
            (SELECT count(*) FROM identity_assertions) AS assertions,
            (SELECT count(*) FROM authority_grants) AS grants,
            (SELECT count(*) FROM identity_events) AS events
        "#,
    )
    .fetch_one(pool)
    .await
    .unwrap();
    (
        row.get("principals"),
        row.get("account_links"),
        row.get("organization_links"),
        row.get("assertions"),
        row.get("grants"),
        row.get("events"),
    )
}

fn metadata(id: &str, recorded_at: chrono::DateTime<Utc>) -> LedgerMetadata {
    LedgerMetadata {
        event_id: IdentityEventId::new(id),
        recorded_at,
        recorded_by: Principal::Platform,
    }
}

fn authority_grant(
    id: &str,
    actor_id: &IdentityPrincipalId,
    issuer_id: &IdentityPrincipalId,
    issued_at: chrono::DateTime<Utc>,
) -> AuthorityGrant {
    AuthorityGrant::new(NewAuthorityGrant {
        id: AuthorityGrantId::new(id),
        actor_principal_id: actor_id.clone(),
        represented_organization_principal_id: None,
        kind: AuthorityKind::PlatformOperator,
        scope: ResourceScope::Platform,
        permitted_actions: vec![AuthorizedAction::ManageAccounts],
        issued_by_principal_id: issuer_id.clone(),
        issued_at,
        validity: None,
        evidence_refs: vec!["postgres integration test".to_string()],
    })
    .unwrap()
}
