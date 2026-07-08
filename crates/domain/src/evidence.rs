//! Evidence artifacts attached to audit episodes.
//!
//! Per the claim-scoped audits memo, the audit object is a bounded claim and
//! attached papers are evidence artifacts to be examined, not votes to be
//! counted. Attachment is therefore epistemically neutral: whether an
//! artifact supports, challenges, narrows, or fails to bear on the target
//! claim is a *derived* status computed from warrant assertions and the
//! element reviews that scrutinize them — the same pure-function pattern as
//! the evaluation tuple, never stored state.
//!
//! The substrate holds the scholarly object reference as an opaque id; the
//! Academic Publishing adapter enriches it with metadata for display.

use serde::{Deserialize, Serialize};

use crate::common::{Authored, Principal, Temporal, Timestamp};
use crate::fact::{Fact, FactPayload, FactStatus, Finding};
use crate::ids::{AuditEpisodeId, EvidenceArtifactId, FactId};

/// A provenance-bearing attachment of one evidence artifact to one episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeEvidenceArtifact {
    pub id: EvidenceArtifactId,
    pub episode_id: AuditEpisodeId,
    /// Opaque reference into the owning adapter (Academic Publishing:
    /// `scholarly_objects.id`). The substrate never interprets it.
    pub scholarly_object_id: String,
    pub role: EvidenceRole,
    pub note: Option<String>,
    pub attached_by: Principal,
    pub attached_at: Timestamp,
    pub status: EvidenceArtifactStatus,
}

impl EpisodeEvidenceArtifact {
    pub fn is_active(&self) -> bool {
        matches!(self.status, EvidenceArtifactStatus::Active)
    }
}

impl Temporal for EpisodeEvidenceArtifact {
    fn temporal_anchor(&self) -> Timestamp {
        self.attached_at
    }
}

impl Authored for EpisodeEvidenceArtifact {
    fn principal(&self) -> Principal {
        self.attached_by.clone()
    }

    fn authored_at(&self) -> Timestamp {
        self.attached_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachEvidenceArtifactRequest {
    pub scholarly_object_id: String,
    #[serde(default)]
    pub role: EvidenceRole,
    pub note: Option<String>,
    pub attached_by: Option<Principal>,
}

/// Neutral attachment roles. Deliberately *not* support/oppose: bearing on
/// the target claim is an audit outcome, not an intake property.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRole {
    /// Attached for inspection as potentially bearing on the target claim.
    #[default]
    Evidence,
    /// Context that frames the audit without claiming to bear on the target.
    Background,
}

impl EvidenceRole {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Evidence => "evidence",
            Self::Background => "background",
        }
    }
}

impl TryFrom<&str> for EvidenceRole {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "evidence" => Ok(Self::Evidence),
            "background" => Ok(Self::Background),
            other => Err(format!("unknown evidence role: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceArtifactStatus {
    Active,
    Retracted {
        retracted_by: Principal,
        retracted_at: Timestamp,
    },
}

/// Derived audit status of one attached artifact: how far its claimed
/// bearing on the target claim has survived scrutiny. Never stored —
/// recomputed from the episode's active facts, like the evaluation tuple.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactBearing {
    /// No warrants asserted and no reviews target the artifact.
    NotYetInspected,
    /// Warrants have been asserted but no element review has scrutinized
    /// them yet. The artifact's contribution is still unearned.
    WarrantsUnaudited,
    /// At least one active element review found a problem in the artifact or
    /// its warrant links.
    ProblemsFound,
    /// Reviews exist and none found problems, but at least one was
    /// inconclusive.
    Inconclusive,
    /// Every active element review of the artifact and its warrants reported
    /// no problems.
    SurvivesScrutiny,
}

/// Pure function: derive one artifact's bearing from the episode's facts.
///
/// `facts` should be the episode's facts (any order); inactive facts are
/// ignored. A review "targets" the artifact if it names the artifact link
/// directly or names one of the artifact's warrant-assertion facts.
pub fn derive_artifact_bearing(
    artifact_id: &EvidenceArtifactId,
    facts: &[Fact],
) -> ArtifactBearing {
    let active = |fact: &&Fact| matches!(fact.status, FactStatus::Active);

    let warrant_fact_ids: Vec<&FactId> = facts
        .iter()
        .filter(active)
        .filter_map(|fact| match &fact.payload {
            FactPayload::WarrantAssertion {
                evidence_artifact: Some(link),
                ..
            } if link == artifact_id.as_str() => Some(&fact.id),
            _ => None,
        })
        .collect();

    let findings: Vec<&Finding> = facts
        .iter()
        .filter(active)
        .filter_map(|fact| match &fact.payload {
            FactPayload::ElementReview {
                finding,
                evidence_artifact,
                warrant,
                ..
            } => {
                let targets_artifact = evidence_artifact
                    .as_deref()
                    .is_some_and(|link| link == artifact_id.as_str());
                let targets_warrant = warrant
                    .as_ref()
                    .is_some_and(|id| warrant_fact_ids.contains(&id));

                (targets_artifact || targets_warrant).then_some(finding)
            }
            _ => None,
        })
        .collect();

    if findings.is_empty() {
        if warrant_fact_ids.is_empty() {
            return ArtifactBearing::NotYetInspected;
        }

        return ArtifactBearing::WarrantsUnaudited;
    }

    if findings.iter().any(|finding| {
        matches!(
            finding,
            Finding::NonEthicalProblem | Finding::EthicalProblem
        )
    }) {
        return ArtifactBearing::ProblemsFound;
    }

    if findings
        .iter()
        .any(|finding| matches!(finding, Finding::Inconclusive))
    {
        return ArtifactBearing::Inconclusive;
    }

    ArtifactBearing::SurvivesScrutiny
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::common::{Principal, Provenance};
    use crate::fact::{Fact, FactPayload, FactStatus, Finding, InferenceType};
    use crate::ids::{AuditSubjectId, DomainInstantiationId, FactId, UserId};

    use super::{derive_artifact_bearing, ArtifactBearing, EvidenceArtifactId};

    fn fact(id: &str, payload: FactPayload) -> Fact {
        Fact {
            id: FactId::new(id),
            subject_id: AuditSubjectId::new("subject-1"),
            domain_instantiation_id: DomainInstantiationId::new("domain-1"),
            occurred_at: Utc::now(),
            payload,
            status: FactStatus::Active,
            provenance: Provenance {
                source_system: Some("test".to_string()),
                source_document: None,
                imported_at: Utc::now(),
                principal: Principal::Platform,
            },
            external_refs: Vec::new(),
        }
    }

    fn warrant(id: &str, artifact: &str) -> Fact {
        fact(
            id,
            FactPayload::WarrantAssertion {
                asserted_by: UserId::new("user-1"),
                evidence_artifact: Some(artifact.to_string()),
                artifact_claim: "artifact claim".to_string(),
                inference_type: InferenceType::Statistical,
                assumptions: None,
                rationale: None,
            },
        )
    }

    fn review(id: &str, finding: Finding, artifact: Option<&str>, warrant: Option<&str>) -> Fact {
        fact(
            id,
            FactPayload::ElementReview {
                cwe_criterion: crate::domain_instantiation::CWECriterionId {
                    domain: DomainInstantiationId::new("domain-1"),
                    node_id: crate::ids::CWENodeId::new("criterion-1"),
                },
                submitted_by: UserId::new("user-1"),
                solicitation: None,
                finding,
                severity: None,
                confidence: None,
                limitations: None,
                recommendations: None,
                evidence_artifact: artifact.map(str::to_string),
                warrant: warrant.map(FactId::new),
                content: "review".to_string(),
                featured: false,
            },
        )
    }

    #[test]
    fn uninspected_artifact_has_no_bearing() {
        let bearing = derive_artifact_bearing(&EvidenceArtifactId::new("artifact-1"), &[]);

        assert_eq!(bearing, ArtifactBearing::NotYetInspected);
    }

    #[test]
    fn unaudited_warrants_do_not_transfer_credibility() {
        let facts = vec![warrant("fact-1", "artifact-1")];
        let bearing = derive_artifact_bearing(&EvidenceArtifactId::new("artifact-1"), &facts);

        assert_eq!(bearing, ArtifactBearing::WarrantsUnaudited);
    }

    #[test]
    fn problem_in_warrant_review_marks_problems_found() {
        let facts = vec![
            warrant("fact-1", "artifact-1"),
            review("fact-2", Finding::NonEthicalProblem, None, Some("fact-1")),
        ];
        let bearing = derive_artifact_bearing(&EvidenceArtifactId::new("artifact-1"), &facts);

        assert_eq!(bearing, ArtifactBearing::ProblemsFound);
    }

    #[test]
    fn clean_reviews_survive_scrutiny() {
        let facts = vec![
            warrant("fact-1", "artifact-1"),
            review("fact-2", Finding::NoProblems, Some("artifact-1"), None),
        ];
        let bearing = derive_artifact_bearing(&EvidenceArtifactId::new("artifact-1"), &facts);

        assert_eq!(bearing, ArtifactBearing::SurvivesScrutiny);
    }

    #[test]
    fn reviews_of_other_artifacts_are_ignored() {
        let facts = vec![review(
            "fact-1",
            Finding::NonEthicalProblem,
            Some("artifact-2"),
            None,
        )];
        let bearing = derive_artifact_bearing(&EvidenceArtifactId::new("artifact-1"), &facts);

        assert_eq!(bearing, ArtifactBearing::NotYetInspected);
    }
}
