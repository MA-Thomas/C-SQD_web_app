use serde::{Deserialize, Serialize};

use crate::common::{Authored, Principal, Temporal, Timestamp};
use crate::ids::{AuditEpisodeId, FactId, RelationId, SectionId, SynthesisReviewId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisReview {
    pub id: SynthesisReviewId,
    pub episode_id: AuditEpisodeId,
    pub submitted_by: UserId,
    pub authored_at: Timestamp,
    pub status: NarrativeStatus,
    pub summary: String,
    pub sections: Vec<SynthesisReviewSection>,
    pub featured: bool,
    /// True when the review was produced outside any commissioned
    /// solicitation: the author started or joined a public episode and
    /// submitted on their own initiative (UI/UX strategy memo).
    #[serde(default)]
    pub unsolicited: bool,
}

impl Temporal for SynthesisReview {
    fn temporal_anchor(&self) -> Timestamp {
        self.authored_at
    }
}

impl Authored for SynthesisReview {
    fn principal(&self) -> Principal {
        Principal::User {
            user_id: self.submitted_by.clone(),
        }
    }

    fn authored_at(&self) -> Timestamp {
        self.authored_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSynthesisReviewRequest {
    pub submitted_by: Option<UserId>,
    #[serde(default = "default_narrative_status")]
    pub status: NarrativeStatus,
    pub summary: String,
    #[serde(default)]
    pub sections: Vec<CreateSynthesisReviewSectionRequest>,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub unsolicited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSynthesisReviewSectionRequest {
    pub section_type: SynthesisReviewSectionType,
    pub content: String,
    #[serde(default)]
    pub referenced_facts: Vec<FactId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NarrativeStatus {
    Draft,
    Current,
    Superseded,
}

impl NarrativeStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Current => "current",
            Self::Superseded => "superseded",
        }
    }
}

impl TryFrom<&str> for NarrativeStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "draft" => Ok(Self::Draft),
            "current" => Ok(Self::Current),
            "superseded" => Ok(Self::Superseded),
            other => Err(format!("unknown narrative status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisReviewSection {
    pub id: SectionId,
    pub review_id: SynthesisReviewId,
    pub section_type: SynthesisReviewSectionType,
    pub content: String,
    pub referenced_facts: Vec<FactId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisReviewSectionType {
    Summary,
    MethodologicalAssessment,
    EthicalAssessment,
    EvidenceIntegration,
    Recommendations,
    OpenQuestions,
}

impl SynthesisReviewSectionType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::MethodologicalAssessment => "methodological_assessment",
            Self::EthicalAssessment => "ethical_assessment",
            Self::EvidenceIntegration => "evidence_integration",
            Self::Recommendations => "recommendations",
            Self::OpenQuestions => "open_questions",
        }
    }
}

impl TryFrom<&str> for SynthesisReviewSectionType {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisReviewRelation {
    pub id: RelationId,
    pub source: SynthesisReviewId,
    pub target: SynthesisReviewId,
    pub relation_type: NarrativeRelationType,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
}

impl Temporal for SynthesisReviewRelation {
    fn temporal_anchor(&self) -> Timestamp {
        self.asserted_at
    }
}

impl Authored for SynthesisReviewRelation {
    fn principal(&self) -> Principal {
        self.asserted_by.clone()
    }

    fn authored_at(&self) -> Timestamp {
        self.asserted_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSynthesisReviewRelationRequest {
    pub target: SynthesisReviewId,
    pub relation_type: NarrativeRelationType,
    pub asserted_by: Option<Principal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NarrativeRelationType {
    Supersedes,
    Contests(ContestationInfo),
    RelatedTo,
}

impl NarrativeRelationType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::Contests(_) => "contests",
            Self::RelatedTo => "related_to",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContestationInfo {
    pub scope: ContestationScope,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContestationScope {
    Partial,
    Full,
}

fn default_narrative_status() -> NarrativeStatus {
    NarrativeStatus::Current
}
