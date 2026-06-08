use serde::{Deserialize, Serialize};

use crate::{
    common::{ExternalRef, Principal, Timestamp},
    domain_instantiation::DomainType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditObjectSummary {
    pub id: String,
    pub domain_instantiation_id: String,
    pub domain_type: DomainType,
    pub domain_name: String,
    pub object_type: String,
    pub title: String,
    pub status: AuditObjectStatus,
    pub submission_tier: SubmissionTier,
    pub submitted_by: Option<String>,
    pub submitted_at: Timestamp,
    pub review_event_count: i64,
    pub active_element_review_count: i64,
    pub active_synthesis_review_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditObjectDetail {
    pub id: String,
    pub domain_instantiation_id: String,
    pub domain_type: DomainType,
    pub domain_name: String,
    pub object_type: String,
    pub title: String,
    pub status: AuditObjectStatus,
    pub submission_tier: SubmissionTier,
    pub submitted_by: Option<String>,
    pub submitted_at: Timestamp,
    pub external_refs: Vec<ExternalRef>,
    pub relations: Vec<AuditObjectRelationSummary>,
    pub review_event_count: i64,
    pub active_element_review_count: i64,
    pub active_synthesis_review_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditObjectStatus {
    Active,
    Revised,
    Withdrawn,
}

impl TryFrom<&str> for AuditObjectStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "revised" => Ok(Self::Revised),
            "withdrawn" => Ok(Self::Withdrawn),
            other => Err(format!("unknown audit object status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionTier {
    Tier0,
    Tier1,
    Tier2,
    Tier3Plus,
}

impl TryFrom<&str> for SubmissionTier {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "tier0" => Ok(Self::Tier0),
            "tier1" => Ok(Self::Tier1),
            "tier2" => Ok(Self::Tier2),
            "tier3_plus" => Ok(Self::Tier3Plus),
            other => Err(format!("unknown submission tier: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditObjectRelationSummary {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation_type: AuditObjectRelationType,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditObjectRelationType {
    Supersedes,
    Revises,
    SplitFrom,
    MergedInto,
    RelatedTo,
}

impl TryFrom<&str> for AuditObjectRelationType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "supersedes" => Ok(Self::Supersedes),
            "revises" => Ok(Self::Revises),
            "split_from" => Ok(Self::SplitFrom),
            "merged_into" => Ok(Self::MergedInto),
            "related_to" => Ok(Self::RelatedTo),
            other => Err(format!("unknown audit object relation type: {other}")),
        }
    }
}
