use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{OrganizationId, UserId};

/// All FEN entities anchor to real instants. RFC3339 on the wire.
pub type Timestamp = DateTime<Utc>;

/// Places heterogeneous entities on a single timeline (FEN `Temporal` trait).
pub trait Temporal {
    fn temporal_anchor(&self) -> Timestamp;
}

/// Answers provenance queries across entity types (FEN `Authored` trait).
///
/// Deviation from the schema document: `principal` returns an owned
/// `Principal` rather than `&Principal` so that entities which record only a
/// `UserId` author (e.g. `SynthesisReview`) can participate without storing a
/// redundant field.
pub trait Authored {
    fn principal(&self) -> Principal;
    fn authored_at(&self) -> Timestamp;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Principal {
    User {
        user_id: UserId,
    },
    Organization {
        organization_id: OrganizationId,
    },
    Platform,
    AiAssisted {
        tool_id: String,
        supervised_by: Option<UserId>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source_system: Option<String>,
    pub source_document: Option<String>,
    pub imported_at: Timestamp,
    pub principal: Principal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalRef {
    pub system: ExternalSystem,
    pub resource_type: Option<String>,
    pub resource_id: String,
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSystem {
    Doi,
    Arxiv,
    ClinicalTrialsGov,
    Orcid,
    Pubmed,
    Pmc,
    Url,
    Other,
}

impl TryFrom<&str> for ExternalSystem {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "doi" => Ok(Self::Doi),
            "arxiv" => Ok(Self::Arxiv),
            "clinical_trials_gov" => Ok(Self::ClinicalTrialsGov),
            "orcid" => Ok(Self::Orcid),
            "pubmed" => Ok(Self::Pubmed),
            "pmc" => Ok(Self::Pmc),
            "url" => Ok(Self::Url),
            "other" => Ok(Self::Other),
            other => Err(format!("unknown external system: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Principal, Timestamp};

    #[test]
    fn decodes_platform_principal_from_database_json() {
        let principal: Principal = serde_json::from_value(json!("platform")).unwrap();

        assert!(matches!(principal, Principal::Platform));
    }

    #[test]
    fn decodes_user_principal_from_database_json() {
        let principal: Principal =
            serde_json::from_value(json!({ "user": { "user_id": "user-1" } })).unwrap();

        assert!(matches!(principal, Principal::User { user_id } if user_id.as_str() == "user-1"));
    }

    #[test]
    fn decodes_rfc3339_timestamp() {
        let ts: Timestamp = serde_json::from_value(json!("2026-01-15T00:00:00Z")).unwrap();

        assert!(ts.timestamp() > 0);
    }
}
