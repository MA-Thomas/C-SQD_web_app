use serde::{Deserialize, Serialize};

use crate::common::Timestamp;
use crate::ids::{DomainInstantiationId, TagId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub display_name: String,
    pub email: String,
    pub created_at: Timestamp,
    pub status: UserStatus,
    pub reviewer_profile: Option<ReviewerProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Suspended {
        reason: String,
        until: Option<Timestamp>,
    },
    Deactivated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerProfile {
    pub user_id: UserId,
    pub status: ReviewerStatus,
    pub tags: Vec<ReviewerTag>,
    pub domain_extensions: Vec<ReviewerDomainExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerStatus {
    GracePeriod,
    Active,
    Suspended,
}

impl TryFrom<&str> for ReviewerStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "grace_period" => Ok(Self::GracePeriod),
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            other => Err(format!("unknown reviewer status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerTag {
    pub id: TagId,
    pub label: String,
    pub scope: TagScope,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagScope {
    Global,
    Domain(DomainInstantiationId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerDomainExtension {
    pub domain_instantiation_id: DomainInstantiationId,
    pub data: serde_json::Value,
}

/// Application roles carried by an authenticated session. Roles are
/// context-derived (FEN: "reviewer and submitter are roles that context
/// supplies, not distinct types"); these claims gate backstage surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Member,
    Sponsor,
    Reviewer,
    Operator,
}

impl Role {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Member => "member",
            Self::Sponsor => "sponsor",
            Self::Reviewer => "reviewer",
            Self::Operator => "operator",
        }
    }
}

impl TryFrom<&str> for Role {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "member" => Ok(Self::Member),
            "sponsor" => Ok(Self::Sponsor),
            "reviewer" => Ok(Self::Reviewer),
            "operator" => Ok(Self::Operator),
            other => Err(format!("unknown role: {other}")),
        }
    }
}

/// The authenticated identity exposed to the web app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub user_id: UserId,
    pub display_name: String,
    pub email: String,
    pub roles: Vec<Role>,
}
