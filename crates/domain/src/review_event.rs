use serde::{Deserialize, Serialize};

use crate::{
    common::{Money, Principal, Provenance, Timestamp},
    domain_instantiation::CWECriterionId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEvent {
    pub id: String,
    pub audit_object_id: String,
    pub domain_instantiation_id: String,
    pub occurred_at: Timestamp,
    pub payload: ReviewEventPayload,
    pub status: ReviewEventStatus,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEventSummary {
    pub id: String,
    pub audit_object_id: String,
    pub domain_instantiation_id: String,
    pub occurred_at: Timestamp,
    pub payload_kind: ReviewEventPayloadKind,
    pub status: ReviewEventStatus,
    pub submitted_by: Option<String>,
    pub finding: Option<Finding>,
    pub severity: Option<FindingSeverity>,
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewEventPayload {
    ElementReview {
        cwe_criterion: CWECriterionId,
        submitted_by: String,
        solicitation: Option<String>,
        finding: Finding,
        severity: Option<FindingSeverity>,
        content: String,
        featured: bool,
    },
    SynthesisReview {
        submitted_by: String,
        content: String,
        sections: Vec<SynthesisSection>,
        featured: bool,
    },
    SubmitterResponse {
        responding_to: Vec<String>,
        response_type: ResponseType,
        content: String,
        revision_ref: Option<String>,
    },
    BountyPosting {
        posted_by: Principal,
        cwe_criterion: Option<CWECriterionId>,
        target_review: Option<String>,
        amount: Money,
        conditions: String,
        expires_at: Option<Timestamp>,
    },
    BountySubmission {
        bounty_id: String,
        submitted_review: String,
        submitted_by: String,
    },
    BountyAdjudication {
        bounty_id: String,
        submission_id: String,
        outcome: AdjudicationOutcome,
        adjudicated_by: Principal,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewEventPayloadKind {
    ElementReview,
    SynthesisReview,
    SubmitterResponse,
    BountyPosting,
    BountySubmission,
    BountyAdjudication,
}

impl TryFrom<&str> for ReviewEventPayloadKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "element_review" => Ok(Self::ElementReview),
            "synthesis_review" => Ok(Self::SynthesisReview),
            "submitter_response" => Ok(Self::SubmitterResponse),
            "bounty_posting" => Ok(Self::BountyPosting),
            "bounty_submission" => Ok(Self::BountySubmission),
            "bounty_adjudication" => Ok(Self::BountyAdjudication),
            other => Err(format!("unknown review event payload kind: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewEventStatus {
    Active,
    Superseded,
    Retracted,
}

impl TryFrom<&str> for ReviewEventStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "superseded" => Ok(Self::Superseded),
            "retracted" => Ok(Self::Retracted),
            other => Err(format!("unknown review event status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finding {
    NonEthicalProblem,
    EthicalProblem,
    NoProblems,
    Inconclusive,
}

impl TryFrom<&str> for Finding {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "non_ethical_problem" => Ok(Self::NonEthicalProblem),
            "ethical_problem" => Ok(Self::EthicalProblem),
            "no_problems" => Ok(Self::NoProblems),
            "inconclusive" => Ok(Self::Inconclusive),
            other => Err(format!("unknown finding: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

impl TryFrom<&str> for FindingSeverity {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "minor" => Ok(Self::Minor),
            "moderate" => Ok(Self::Moderate),
            "major" => Ok(Self::Major),
            "critical" => Ok(Self::Critical),
            other => Err(format!("unknown finding severity: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Accepts,
    Contests,
    PartiallyAccepts,
    RevisesAndResponds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjudicationOutcome {
    Awarded,
    Rejected,
    PartiallyAwarded,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEventMembership {
    pub id: String,
    pub review_event_id: String,
    pub audit_object_id: String,
    pub role: ReviewEventRole,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
    pub status: MembershipStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewEventRole {
    ElementReview,
    SynthesisReview,
    SubmitterResponse,
    BountyPosting,
    BountySubmission,
    BountyAdjudication,
}

impl TryFrom<&str> for ReviewEventRole {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "element_review" => Ok(Self::ElementReview),
            "synthesis_review" => Ok(Self::SynthesisReview),
            "submitter_response" => Ok(Self::SubmitterResponse),
            "bounty_posting" => Ok(Self::BountyPosting),
            "bounty_submission" => Ok(Self::BountySubmission),
            "bounty_adjudication" => Ok(Self::BountyAdjudication),
            other => Err(format!("unknown review event role: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipStatus {
    Active,
    Retracted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisSection {
    pub id: String,
    pub review_event_id: String,
    pub section_type: SynthesisSectionType,
    pub content: String,
    pub referenced_reviews: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisSectionType {
    Summary,
    MethodologicalAssessment,
    EthicalAssessment,
    EvidenceIntegration,
    Recommendations,
    OpenQuestions,
}

impl TryFrom<&str> for SynthesisSectionType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "summary" => Ok(Self::Summary),
            "methodological_assessment" => Ok(Self::MethodologicalAssessment),
            "ethical_assessment" => Ok(Self::EthicalAssessment),
            "evidence_integration" => Ok(Self::EvidenceIntegration),
            "recommendations" => Ok(Self::Recommendations),
            "open_questions" => Ok(Self::OpenQuestions),
            other => Err(format!("unknown synthesis section type: {other}")),
        }
    }
}
