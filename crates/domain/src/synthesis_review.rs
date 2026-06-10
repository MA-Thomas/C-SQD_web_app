use serde::{Deserialize, Serialize};

use crate::common::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisReview {
    pub id: String,
    pub episode_id: String,
    pub submitted_by: String,
    pub authored_at: Timestamp,
    pub status: NarrativeStatus,
    pub summary: String,
    pub sections: Vec<SynthesisReviewSection>,
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSynthesisReviewRequest {
    pub submitted_by: Option<String>,
    #[serde(default = "default_narrative_status")]
    pub status: NarrativeStatus,
    pub summary: String,
    #[serde(default)]
    pub sections: Vec<CreateSynthesisReviewSectionRequest>,
    #[serde(default)]
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSynthesisReviewSectionRequest {
    pub section_type: SynthesisReviewSectionType,
    pub content: String,
    #[serde(default)]
    pub referenced_facts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NarrativeStatus {
    Draft,
    Current,
    Superseded,
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
    pub id: String,
    pub review_id: String,
    pub section_type: SynthesisReviewSectionType,
    pub content: String,
    pub referenced_facts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisReviewSectionType {
    Summary,
    MethodologicalAssessment,
    EthicalAssessment,
    EvidenceIntegration,
    Recommendations,
    OpenQuestions,
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
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation_type: NarrativeRelationType,
    pub asserted_by: crate::common::Principal,
    pub asserted_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NarrativeRelationType {
    Supersedes,
    Contests(ContestationInfo),
    RelatedTo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestationInfo {
    pub scope: ContestationScope,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContestationScope {
    Partial,
    Full,
}

fn default_narrative_status() -> NarrativeStatus {
    NarrativeStatus::Current
}
