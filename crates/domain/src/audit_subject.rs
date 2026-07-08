use serde::{Deserialize, Serialize};

use crate::common::{ExternalRef, Principal, Timestamp};
use crate::ids::{AuditSubjectId, DomainInstantiationId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSubject {
    pub id: AuditSubjectId,
    pub domain_instantiation_id: DomainInstantiationId,
    pub subject_type: AuditSubjectType,
    pub title: Option<String>,
    /// The bounded target claim under audit (claim-scoped audits memo).
    /// Usually present for `ScopedClaim` subjects; stated precisely enough
    /// that reviewers can ask what would count as support, challenge,
    /// limitation, or non-applicability.
    #[serde(default)]
    pub claim_statement: Option<String>,
    /// The explicit conditions under which the claim is being evaluated
    /// (population, intervention, measurement, outcome, timeframe, ...).
    #[serde(default)]
    pub scope_conditions: Vec<ScopeCondition>,
    pub external_refs: Vec<ExternalRef>,
    pub registered_by: Principal,
    pub registered_at: Timestamp,
}

/// One named condition bounding the claim under audit, e.g.
/// `{ label: "population", value: "adults aged 40-70 in Z" }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopeCondition {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditSubjectRequest {
    pub domain_instantiation_id: DomainInstantiationId,
    pub subject_type: AuditSubjectType,
    pub title: Option<String>,
    #[serde(default)]
    pub claim_statement: Option<String>,
    #[serde(default)]
    pub scope_conditions: Vec<ScopeCondition>,
    #[serde(default)]
    pub external_refs: Vec<ExternalRef>,
    pub registered_by: Option<Principal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSubjectType {
    /// A bounded target claim (or claim-warrant bundle) under audit. The
    /// default subject kind per the claim-scoped audits memo: papers attach
    /// to episodes as evidence artifacts rather than being the epistemic
    /// target themselves.
    ScopedClaim,
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
    /// Carries the custom label per the FEN schema (`Other(String)`).
    Other(String),
}

impl AuditSubjectType {
    /// Database column representation: the enum kind string.
    pub fn db_kind(&self) -> &'static str {
        match self {
            Self::ScopedClaim => "scoped_claim",
            Self::ResearchManuscript => "research_manuscript",
            Self::Preprint => "preprint",
            Self::Dataset => "dataset",
            Self::CodeRepository => "code_repository",
            Self::ClinicalTrialProtocol => "clinical_trial_protocol",
            Self::AiModelEvaluation => "ai_model_evaluation",
            Self::Benchmark => "benchmark",
            Self::PolicyDocument => "policy_document",
            Self::GrantProposal => "grant_proposal",
            Self::TechnicalReport => "technical_report",
            Self::Other(_) => "other",
        }
    }

    /// Database detail column: only `Other` carries one.
    pub fn db_detail(&self) -> Option<&str> {
        match self {
            Self::Other(label) if !label.is_empty() => Some(label),
            _ => None,
        }
    }

    /// Reconstructs from the kind + detail database columns.
    pub fn from_db(kind: &str, detail: Option<&str>) -> Result<Self, String> {
        match kind {
            "other" => Ok(Self::Other(detail.unwrap_or_default().to_string())),
            other => Self::try_from(other),
        }
    }
}

impl TryFrom<&str> for AuditSubjectType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "scoped_claim" => Ok(Self::ScopedClaim),
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
            "other" => Ok(Self::Other(String::new())),
            other => Err(format!("unknown audit subject type: {other}")),
        }
    }
}
