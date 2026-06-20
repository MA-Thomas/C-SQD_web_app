use serde::{Deserialize, Serialize};

use crate::common::Timestamp;
use crate::ids::OrganizationId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub org_type: OrganizationType,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationType {
    Biotech,
    VentureCapital,
    Foundation,
    University,
    Journal,
    Regulator,
    /// Carries the custom label per the FEN schema (`Other(String)`).
    Other(String),
}

impl OrganizationType {
    pub fn db_kind(&self) -> &'static str {
        match self {
            Self::Biotech => "biotech",
            Self::VentureCapital => "venture_capital",
            Self::Foundation => "foundation",
            Self::University => "university",
            Self::Journal => "journal",
            Self::Regulator => "regulator",
            Self::Other(_) => "other",
        }
    }

    pub fn db_detail(&self) -> Option<&str> {
        match self {
            Self::Other(label) if !label.is_empty() => Some(label),
            _ => None,
        }
    }

    pub fn from_db(kind: &str, detail: Option<&str>) -> Result<Self, String> {
        match kind {
            "other" => Ok(Self::Other(detail.unwrap_or_default().to_string())),
            other => Self::try_from(other),
        }
    }
}

impl TryFrom<&str> for OrganizationType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "biotech" => Ok(Self::Biotech),
            "venture_capital" => Ok(Self::VentureCapital),
            "foundation" => Ok(Self::Foundation),
            "university" => Ok(Self::University),
            "journal" => Ok(Self::Journal),
            "regulator" => Ok(Self::Regulator),
            "other" => Ok(Self::Other(String::new())),
            other => Err(format!("unknown organization type: {other}")),
        }
    }
}
