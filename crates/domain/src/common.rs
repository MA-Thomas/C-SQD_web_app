use serde::{Deserialize, Serialize};

pub type Timestamp = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Principal {
    User {
        user_id: String,
    },
    Organization {
        organization_id: String,
    },
    Platform,
    AiAssisted {
        tool_id: String,
        supervised_by: Option<String>,
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

    use super::Principal;

    #[test]
    fn decodes_platform_principal_from_database_json() {
        let principal: Principal = serde_json::from_value(json!("platform")).unwrap();

        assert!(matches!(principal, Principal::Platform));
    }

    #[test]
    fn decodes_user_principal_from_database_json() {
        let principal: Principal =
            serde_json::from_value(json!({ "user": { "user_id": "user-1" } })).unwrap();

        assert!(matches!(principal, Principal::User { user_id } if user_id == "user-1"));
    }
}
