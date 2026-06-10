use serde::{Deserialize, Serialize};

use crate::common::{ExternalRef, Principal, Timestamp};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSubject {
    pub id: String,
    pub domain_instantiation_id: String,
    pub subject_type: AuditSubjectType,
    pub title: Option<String>,
    pub external_refs: Vec<ExternalRef>,
    pub registered_by: Principal,
    pub registered_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditSubjectRequest {
    pub domain_instantiation_id: String,
    pub subject_type: AuditSubjectType,
    pub title: Option<String>,
    #[serde(default)]
    pub external_refs: Vec<ExternalRef>,
    pub registered_by: Option<Principal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSubjectType {
    ResearchManuscript,
    Preprint,
    Dataset,
    CodeRepository,
    ClinicalTrialProtocol,
    AiModelEvaluation,
    Benchmark,
    PolicyDocument,
    GrantProposal,
    TechnicalReport,
    Other,
}

impl TryFrom<&str> for AuditSubjectType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "research_manuscript" => Ok(Self::ResearchManuscript),
            "preprint" => Ok(Self::Preprint),
            "dataset" => Ok(Self::Dataset),
            "code_repository" => Ok(Self::CodeRepository),
            "clinical_trial_protocol" => Ok(Self::ClinicalTrialProtocol),
            "ai_model_evaluation" => Ok(Self::AiModelEvaluation),
            "benchmark" => Ok(Self::Benchmark),
            "policy_document" => Ok(Self::PolicyDocument),
            "grant_proposal" => Ok(Self::GrantProposal),
            "technical_report" => Ok(Self::TechnicalReport),
            "other" => Ok(Self::Other),
            other => Err(format!("unknown audit subject type: {other}")),
        }
    }
}
