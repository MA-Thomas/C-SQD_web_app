//! Magic-link authentication and cookie sessions.
//!
//! Flow: `request_magic_link` issues a short-lived single-use token for an
//! email address (creating the user on first sign-in). `complete_magic_link`
//! consumes the token and opens a session. Only token hashes are stored.

use chrono::{Duration, Utc};
use csqd_domain::{Role, SessionUser, UserId};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};

use super::{users, RepositoryError};

const MAGIC_LINK_TTL_MINUTES: i64 = 15;
const SESSION_TTL_DAYS: i64 = 30;
/// Maximum sign-in links issuable per email inside one TTL window. A cheap,
/// dependency-free brake on link-request abuse; per-IP limiting belongs at
/// the reverse proxy.
const MAGIC_LINK_MAX_PER_WINDOW: i64 = 3;

pub struct IssuedMagicLink {
    pub email: String,
    /// Raw token; embed in the sign-in URL. Never stored.
    pub token: String,
    pub expires_at: chrono::DateTime<Utc>,
}

pub struct OpenedSession {
    /// Raw session token for the cookie. Never stored.
    pub token: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub user: SessionUser,
}

pub async fn request_magic_link(
    db: &PgPool,
    email: &str,
) -> Result<IssuedMagicLink, RepositoryError> {
    let user = users::find_or_create_by_email(db, email).await?;

    let recent: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM auth_magic_links
        WHERE email = $1
          AND created_at > now() - make_interval(mins => $2)
        "#,
    )
    .bind(&user.email)
    .bind(MAGIC_LINK_TTL_MINUTES as i32)
    .fetch_one(db)
    .await?;

    if recent >= MAGIC_LINK_MAX_PER_WINDOW {
        return Err(RepositoryError::RateLimited(
            "too many sign-in links requested for this address; wait a few minutes and try again"
                .to_string(),
        ));
    }

    let token = generate_token();
    let token_hash = hash_token(&token);
    let expires_at = Utc::now() + Duration::minutes(MAGIC_LINK_TTL_MINUTES);

    sqlx::query(
        r#"
        INSERT INTO auth_magic_links (email, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&user.email)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(db)
    .await?;

    Ok(IssuedMagicLink {
        email: user.email,
        token,
        expires_at,
    })
}

pub async fn complete_magic_link(
    db: &PgPool,
    token: &str,
) -> Result<OpenedSession, RepositoryError> {
    let token_hash = hash_token(token.trim());
    let row = sqlx::query(
        r#"
        UPDATE auth_magic_links
        SET consumed_at = now()
        WHERE token_hash = $1
          AND consumed_at IS NULL
          AND expires_at > now()
        RETURNING email
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| {
        RepositoryError::Unauthorized("sign-in link is invalid or has expired".to_string())
    })?;
    let email: String = row.get("email");
    let user = users::find_or_create_by_email(db, &email).await?;

    open_session(db, &user.id).await
}

pub async fn open_session(db: &PgPool, user_id: &UserId) -> Result<OpenedSession, RepositoryError> {
    let token = generate_token();
    let token_hash = hash_token(&token);
    let expires_at = Utc::now() + Duration::days(SESSION_TTL_DAYS);

    sqlx::query(
        r#"
        INSERT INTO auth_sessions (user_id, token_hash, expires_at)
        VALUES ($1::uuid, $2, $3)
        "#,
    )
    .bind(user_id.as_str())
    .bind(&token_hash)
    .bind(expires_at)
    .execute(db)
    .await?;

    let user = session_user(db, user_id).await?;

    Ok(OpenedSession {
        token,
        expires_at,
        user,
    })
}

pub async fn session_user_for_token(
    db: &PgPool,
    token: &str,
) -> Result<Option<SessionUser>, RepositoryError> {
    let token = token.trim();

    if token.is_empty() {
        return Ok(None);
    }

    let token_hash = hash_token(token);
    let row = sqlx::query(
        r#"
        SELECT user_id::text AS user_id
        FROM auth_sessions
        WHERE token_hash = $1
          AND revoked_at IS NULL
          AND expires_at > now()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => {
            let user_id: String = row.get("user_id");
            let user = session_user(db, &UserId::new(user_id)).await?;

            Ok(Some(user))
        }
        None => Ok(None),
    }
}

pub async fn revoke_session(db: &PgPool, token: &str) -> Result<(), RepositoryError> {
    let token_hash = hash_token(token.trim());

    sqlx::query(
        r#"
        UPDATE auth_sessions
        SET revoked_at = now()
        WHERE token_hash = $1
          AND revoked_at IS NULL
        "#,
    )
    .bind(&token_hash)
    .execute(db)
    .await?;

    Ok(())
}

async fn session_user(db: &PgPool, user_id: &UserId) -> Result<SessionUser, RepositoryError> {
    let user = users::find(db, user_id.as_str()).await?;
    let roles = users::roles_for_user(db, user_id).await?;

    Ok(SessionUser {
        user_id: user.id.clone(),
        display_name: user.display_name.clone(),
        email: user.email.clone(),
        roles,
    })
}

pub fn require_role(session: &SessionUser, role: Role) -> Result<(), RepositoryError> {
    if session.roles.contains(&role) || session.roles.contains(&Role::Operator) {
        Ok(())
    } else {
        Err(RepositoryError::Forbidden(format!(
            "this action requires the {} role",
            role.as_db_str()
        )))
    }
}

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);

    hex::encode(bytes)
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());

    hex::encode(hasher.finalize())
}
