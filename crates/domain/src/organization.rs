use serde::{Deserialize, Serialize};

use crate::common::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub org_type: OrganizationType,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationType {
    Biotech,
    VentureCapital,
    Foundation,
    University,
    Journal,
    Regulator,
    Other,
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
            "other" => Ok(Self::Other),
            other => Err(format!("unknown organization type: {other}")),
        }
    }
}
