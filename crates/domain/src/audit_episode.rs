use serde::{Deserialize, Serialize};

use crate::{
    audit_subject::AuditSubjectType,
    common::{Money, Principal, Timestamp},
    fact::Fact,
    organization::{Organization, OrganizationType},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEpisode {
    pub id: String,
    pub subject_id: String,
    pub domain_instantiation_id: String,
    pub label: String,
    pub status: EpisodeStatus,
    pub authored_by: Principal,
    pub authored_at: Timestamp,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEpisodeSummary {
    pub id: String,
    pub subject_id: String,
    pub domain_instantiation_id: String,
    pub label: String,
    pub status: EpisodeStatus,
    pub authored_by: Principal,
    pub authored_at: Timestamp,
    pub notes: Option<String>,
    pub subject_title: Option<String>,
    pub subject_type: AuditSubjectType,
    pub sponsor_name: Option<String>,
    pub sponsor_organization_type: Option<OrganizationType>,
    pub fact_count: i64,
    pub element_review_count: i64,
    pub synthesis_review_count: i64,
    pub latest_activity_at: Option<Timestamp>,
    pub synthesis_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionAuditEpisodeRequest {
    pub label: String,
    pub sponsor_organization_name: String,
    pub sponsor_organization_type: OrganizationType,
    pub funding: Money,
    #[serde(default)]
    pub scope_cwe_node_ids: Vec<String>,
    pub deadline: Option<Timestamp>,
    #[serde(default)]
    pub confidential: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionAuditEpisodeResult {
    pub organization: Organization,
    pub episode: AuditEpisode,
    pub commission_fact: Fact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeStatus {
    Active,
    SynthesisPending,
    Delivered,
    Closed,
}

impl TryFrom<&str> for EpisodeStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "synthesis_pending" => Ok(Self::SynthesisPending),
            "delivered" => Ok(Self::Delivered),
            "closed" => Ok(Self::Closed),
            other => Err(format!("unknown audit episode status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMembership {
    pub id: String,
    pub fact_id: String,
    pub episode_id: String,
    pub role: FactRole,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
    pub status: EpisodeMembershipStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactRole {
    Commission,
    ElementReview,
    Solicitation,
    SolicitationLifecycle,
    Response,
    Administrative,
    Other,
}

impl TryFrom<&str> for FactRole {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "commission" => Ok(Self::Commission),
            "element_review" => Ok(Self::ElementReview),
            "solicitation" => Ok(Self::Solicitation),
            "solicitation_lifecycle" => Ok(Self::SolicitationLifecycle),
            "response" => Ok(Self::Response),
            "administrative" => Ok(Self::Administrative),
            "other" => Ok(Self::Other),
            other => Err(format!("unknown fact role: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeMembershipStatus {
    Active,
    Retracted {
        retracted_by: Principal,
        retracted_at: Timestamp,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeRelation {
    pub id: String,
    pub source_episode_id: String,
    pub target_episode_id: String,
    pub relation_type: EpisodeRelationType,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeRelationType {
    Supersedes,
    SplitFrom,
    MergedInto,
    RelatedTo,
    PartOf,
}
