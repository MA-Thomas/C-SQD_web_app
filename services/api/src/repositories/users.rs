use csqd_domain::{
    Principal, ReviewerDomainExtension, ReviewerProfile, ReviewerStatus, ReviewerTag, Role,
    SessionUser, TagScope, User, UserId, UserStatus,
};
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

pub async fn find(db: &PgPool, user_id: &str) -> Result<User, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            email,
            display_name,
            roles,
            status,
            status_metadata,
            created_at
        FROM users
        WHERE id::text = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "user",
        id: user_id.to_string(),
    })?;
    let reviewer_profile = find_reviewer_profile(db, user_id).await.ok();

    row_to_user(row, reviewer_profile)
}

pub async fn find_or_create_by_email(db: &PgPool, email: &str) -> Result<User, RepositoryError> {
    let email = email.trim().to_lowercase();

    if email.is_empty() || !email.contains('@') {
        return Err(RepositoryError::Domain(
            "a valid email address is required".to_string(),
        ));
    }

    let display_name = email
        .split('@')
        .next()
        .unwrap_or("member")
        .replace(['.', '_', '-'], " ");
    let row = sqlx::query(
        r#"
        INSERT INTO users (email, display_name, role, roles, status)
        VALUES ($1, $2, 'reader', '{member}', 'active')
        ON CONFLICT (email) DO UPDATE SET updated_at = now()
        RETURNING
            id::text AS id,
            email,
            display_name,
            roles,
            status,
            status_metadata,
            created_at
        "#,
    )
    .bind(&email)
    .bind(display_name.trim())
    .fetch_one(db)
    .await?;

    row_to_user(row, None)
}

pub async fn find_reviewer_profile(
    db: &PgPool,
    user_id: &str,
) -> Result<ReviewerProfile, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            user_id::text AS user_id,
            status,
            domain_extensions
        FROM reviewer_profiles
        WHERE user_id::text = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "reviewer_profile",
        id: user_id.to_string(),
    })?;
    let status: String = row.get("status");
    // Legacy seeds may carry 'candidate'; map onto the FEN grace period.
    let status = match status.as_str() {
        "candidate" => ReviewerStatus::GracePeriod,
        other => ReviewerStatus::try_from(other).map_err(RepositoryError::Domain)?,
    };
    let domain_extensions: Value = row.get("domain_extensions");
    let domain_extensions =
        serde_json::from_value::<Vec<ReviewerDomainExtension>>(domain_extensions)
            .unwrap_or_default();
    let tags = list_reviewer_tags(db, user_id).await?;

    Ok(ReviewerProfile {
        user_id: row.get("user_id"),
        status,
        tags,
        domain_extensions,
    })
}

pub async fn list_reviewer_tags(
    db: &PgPool,
    user_id: &str,
) -> Result<Vec<ReviewerTag>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            label,
            scope,
            domain_instantiation_id::text AS domain_instantiation_id,
            verified
        FROM reviewer_tags
        WHERE user_id::text = $1
        ORDER BY label ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            let scope: String = row.get("scope");
            let domain_instantiation_id: Option<String> = row.get("domain_instantiation_id");
            let scope = match (scope.as_str(), domain_instantiation_id) {
                ("domain", Some(domain_id)) => TagScope::Domain(domain_id.into()),
                _ => TagScope::Global,
            };

            Ok(ReviewerTag {
                id: row.get("id"),
                label: row.get("label"),
                scope,
                verified: row.get("verified"),
            })
        })
        .collect()
}

/// Resolves a `Principal` to a human-readable display name for provenance
/// rendering (timeline entries, report bylines, sponsor views).
pub async fn resolve_principal_display(
    db: &PgPool,
    principal: &Principal,
) -> Result<String, RepositoryError> {
    match principal {
        Principal::Platform => Ok("C-SQD platform".to_string()),
        Principal::AiAssisted { tool_id, .. } => Ok(format!("AI-assisted ({tool_id})")),
        Principal::User { user_id } => {
            let row = sqlx::query(
                r#"
                SELECT display_name
                FROM users
                WHERE id::text = $1
                "#,
            )
            .bind(user_id.as_str())
            .fetch_optional(db)
            .await?;

            Ok(row
                .map(|row| row.get::<String, _>("display_name"))
                .unwrap_or_else(|| format!("User {}", user_id.as_str())))
        }
        Principal::Organization { organization_id } => {
            let row = sqlx::query(
                r#"
                SELECT name
                FROM organizations
                WHERE id::text = $1
                "#,
            )
            .bind(organization_id.as_str())
            .fetch_optional(db)
            .await?;

            Ok(row
                .map(|row| row.get::<String, _>("name"))
                .unwrap_or_else(|| format!("Organization {}", organization_id.as_str())))
        }
    }
}

pub fn session_user_from_user(user: &User) -> SessionUser {
    SessionUser {
        user_id: user.id.clone(),
        display_name: user.display_name.clone(),
        email: user.email.clone(),
        roles: user_roles(user),
    }
}

fn user_roles(user: &User) -> Vec<Role> {
    user.reviewer_profile
        .as_ref()
        .map(|_| vec![Role::Member, Role::Reviewer])
        .unwrap_or_else(|| vec![Role::Member])
}

pub(crate) fn row_to_user(
    row: PgRow,
    reviewer_profile: Option<ReviewerProfile>,
) -> Result<User, RepositoryError> {
    let status: String = row.get("status");
    let status_metadata: Value = row.get("status_metadata");
    let status = match status.as_str() {
        "active" => UserStatus::Active,
        "suspended" => UserStatus::Suspended {
            reason: status_metadata
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("suspended")
                .to_string(),
            until: status_metadata
                .get("until")
                .and_then(Value::as_str)
                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                .map(|value| value.with_timezone(&chrono::Utc)),
        },
        "deactivated" => UserStatus::Deactivated,
        other => {
            return Err(RepositoryError::Domain(format!(
                "unknown user status: {other}"
            )))
        }
    };

    Ok(User {
        id: row.get("id"),
        display_name: row.get("display_name"),
        email: row.get("email"),
        created_at: row.get("created_at"),
        status,
        reviewer_profile,
    })
}

/// Operator view of an account row: identity + roles, for the role-granting
/// panel. Role grants previously required direct SQL, which was
/// provenance-invisible — ironic for this product.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AccountSummary {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_accounts(db: &PgPool) -> Result<Vec<AccountSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            email,
            display_name,
            roles,
            status,
            created_at
        FROM users
        ORDER BY created_at DESC
        LIMIT 500
        "#,
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| AccountSummary {
            id: row.get("id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            roles: row.get("roles"),
            status: row.get("status"),
            created_at: row.get("created_at"),
        })
        .collect())
}

/// Replaces the stored role set. `member` is always retained; roles must be
/// known. The reviewer-profile-derived reviewer role continues to merge in
/// at session time regardless.
pub async fn set_roles(
    db: &PgPool,
    user_id: &str,
    roles: &[String],
) -> Result<AccountSummary, RepositoryError> {
    let mut normalized: Vec<String> = Vec::new();

    for role in roles {
        let role = role.trim().to_lowercase();

        if role.is_empty() {
            continue;
        }

        Role::try_from(role.as_str())
            .map_err(|_| RepositoryError::Domain(format!("unknown role: {role}")))?;

        if !normalized.contains(&role) {
            normalized.push(role);
        }
    }

    if !normalized.iter().any(|role| role == "member") {
        normalized.insert(0, "member".to_string());
    }

    let row = sqlx::query(
        r#"
        UPDATE users
        SET roles = $2, updated_at = now()
        WHERE id::text = $1
        RETURNING
            id::text AS id,
            email,
            display_name,
            roles,
            status,
            created_at
        "#,
    )
    .bind(user_id)
    .bind(&normalized)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "user",
        id: user_id.to_string(),
    })?;

    Ok(AccountSummary {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        roles: row.get("roles"),
        status: row.get("status"),
        created_at: row.get("created_at"),
    })
}

/// Self-service display-name update (the account page / onboarding step).
pub async fn update_display_name(
    db: &PgPool,
    user_id: &str,
    display_name: &str,
) -> Result<User, RepositoryError> {
    let display_name = display_name.trim();

    if display_name.is_empty() || display_name.len() > 120 {
        return Err(RepositoryError::Domain(
            "display name must be between 1 and 120 characters".to_string(),
        ));
    }

    let row = sqlx::query(
        r#"
        UPDATE users
        SET display_name = $2, updated_at = now()
        WHERE id::text = $1
        RETURNING
            id::text AS id,
            email,
            display_name,
            roles,
            status,
            status_metadata,
            created_at
        "#,
    )
    .bind(user_id)
    .bind(display_name)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "user",
        id: user_id.to_string(),
    })?;
    let reviewer_profile = find_reviewer_profile(db, user_id).await.ok();

    row_to_user(row, reviewer_profile)
}

/// Roles stored on the users row (the `roles` text[] column), merged with the
/// reviewer profile signal.
pub async fn roles_for_user(db: &PgPool, user_id: &UserId) -> Result<Vec<Role>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT roles
        FROM users
        WHERE id::text = $1
        "#,
    )
    .bind(user_id.as_str())
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "user",
        id: user_id.as_str().to_string(),
    })?;
    let stored: Vec<String> = row.get("roles");
    let mut roles: Vec<Role> = stored
        .iter()
        .filter_map(|role| Role::try_from(role.as_str()).ok())
        .collect();

    if !roles.contains(&Role::Member) {
        roles.insert(0, Role::Member);
    }

    let has_reviewer_profile = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM reviewer_profiles WHERE user_id::text = $1
        ) AS exists
        "#,
    )
    .bind(user_id.as_str())
    .fetch_one(db)
    .await?
    .get::<bool, _>("exists");

    if has_reviewer_profile && !roles.contains(&Role::Reviewer) {
        roles.push(Role::Reviewer);
    }

    Ok(roles)
}
