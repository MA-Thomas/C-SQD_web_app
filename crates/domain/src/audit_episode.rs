use serde::{Deserialize, Serialize};

use crate::audit_subject::AuditSubjectType;
use crate::common::{Authored, Money, Principal, Temporal, Timestamp};
use crate::fact::Fact;
use crate::ids::{
    AuditEpisodeId, AuditSubjectId, CWENodeId, DomainInstantiationId, FactId, MembershipId,
    RelationId,
};
use crate::organization::{Organization, OrganizationType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEpisode {
    pub id: AuditEpisodeId,
    pub subject_id: AuditSubjectId,
    pub domain_instantiation_id: DomainInstantiationId,
    pub label: String,
    pub status: EpisodeStatus,
    pub authored_by: Principal,
    pub authored_at: Timestamp,
    pub notes: Option<String>,
}

impl Temporal for AuditEpisode {
    fn temporal_anchor(&self) -> Timestamp {
        self.authored_at
    }
}

impl Authored for AuditEpisode {
    fn principal(&self) -> Principal {
        self.authored_by.clone()
    }

    fn authored_at(&self) -> Timestamp {
        self.authored_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEpisodeSummary {
    pub id: AuditEpisodeId,
    pub subject_id: AuditSubjectId,
    pub domain_instantiation_id: DomainInstantiationId,
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
    pub scope_cwe_node_ids: Vec<CWENodeId>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub id: MembershipId,
    pub fact_id: FactId,
    pub episode_id: AuditEpisodeId,
    pub role: FactRole,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
    pub status: EpisodeMembershipStatus,
}

impl EpisodeMembership {
    pub fn is_active(&self) -> bool {
        matches!(self.status, EpisodeMembershipStatus::Active)
    }
}

impl Temporal for EpisodeMembership {
    fn temporal_anchor(&self) -> Timestamp {
        self.asserted_at
    }
}

impl Authored for EpisodeMembership {
    fn principal(&self) -> Principal {
        self.asserted_by.clone()
    }

    fn authored_at(&self) -> Timestamp {
        self.asserted_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactRole {
    Commission,
    ElementReview,
    Solicitation,
    SolicitationLifecycle,
    Response,
    Participation,
    Petition,
    Curation,
    Administrative,
    Other,
}

impl FactRole {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Commission => "commission",
            Self::ElementReview => "element_review",
            Self::Solicitation => "solicitation",
            Self::SolicitationLifecycle => "solicitation_lifecycle",
            Self::Response => "response",
            Self::Participation => "participation",
            Self::Petition => "petition",
            Self::Curation => "curation",
            Self::Administrative => "administrative",
            Self::Other => "other",
        }
    }
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
            "participation" => Ok(Self::Participation),
            "petition" => Ok(Self::Petition),
            "curation" => Ok(Self::Curation),
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
    pub id: RelationId,
    pub source_episode_id: AuditEpisodeId,
    pub target_episode_id: AuditEpisodeId,
    pub relation_type: EpisodeRelationType,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
}

impl Temporal for EpisodeRelation {
    fn temporal_anchor(&self) -> Timestamp {
        self.asserted_at
    }
}

impl Authored for EpisodeRelation {
    fn principal(&self) -> Principal {
        self.asserted_by.clone()
    }

    fn authored_at(&self) -> Timestamp {
        self.asserted_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeRelationRequest {
    pub target_episode_id: AuditEpisodeId,
    pub relation_type: EpisodeRelationType,
    pub asserted_by: Option<Principal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeRelationType {
    Supersedes,
    SplitFrom,
    MergedInto,
    RelatedTo,
    PartOf,
}

impl EpisodeRelationType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::SplitFrom => "split_from",
            Self::MergedInto => "merged_into",
            Self::RelatedTo => "related_to",
            Self::PartOf => "part_of",
        }
    }
}

impl TryFrom<&str> for EpisodeRelationType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "supersedes" => Ok(Self::Supersedes),
            "split_from" => Ok(Self::SplitFrom),
            "merged_into" => Ok(Self::MergedInto),
            "related_to" => Ok(Self::RelatedTo),
            "part_of" => Ok(Self::PartOf),
            other => Err(format!("unknown episode relation type: {other}")),
        }
    }
}
