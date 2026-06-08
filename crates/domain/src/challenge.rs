use serde::{Deserialize, Serialize};

use crate::common::{Principal, Timestamp};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeSummary {
    pub id: String,
    pub challenge_type: ChallengeType,
    pub target: ChallengeTarget,
    pub challenger_review: Option<String>,
    pub initiated_by: Principal,
    pub initiated_at: Timestamp,
    pub election_date: Timestamp,
    pub status: ChallengeStatus,
    pub domain_instantiation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeType {
    Direct,
    Petition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeTarget {
    ElementReview { review_event_id: String },
    SynthesisReview { review_event_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeStatus {
    Open,
    Resolved {
        winner: String,
        decided_at: Timestamp,
    },
    Withdrawn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisReviewRelationSummary {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation_type: SynthesisRelationType,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisRelationType {
    Supersedes,
    Contests {
        scope: ContestationScope,
        rationale: Option<String>,
    },
    RelatedTo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContestationScope {
    Partial,
    Full,
}
