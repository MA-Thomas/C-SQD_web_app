use serde::{Deserialize, Serialize};

use crate::common::{Authored, Money, Principal, Provenance, Temporal, Timestamp};
use crate::domain_instantiation::CWECriterionId;
use crate::ids::{
    AuditEpisodeId, AuditSubjectId, CWENodeId, DomainInstantiationId, FactId, UserId,
};
use crate::solicitation::{PaymentScheme, SolicitationEventType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: FactId,
    pub subject_id: AuditSubjectId,
    pub domain_instantiation_id: DomainInstantiationId,
    pub occurred_at: Timestamp,
    pub payload: FactPayload,
    pub status: FactStatus,
    pub provenance: Provenance,
    pub external_refs: Vec<crate::common::ExternalRef>,
}

impl Temporal for Fact {
    fn temporal_anchor(&self) -> Timestamp {
        self.occurred_at
    }
}

impl Authored for Fact {
    fn principal(&self) -> Principal {
        self.provenance.principal.clone()
    }

    fn authored_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeElementReviewRequest {
    pub cwe_node_id: CWENodeId,
    pub submitted_by: Option<UserId>,
    pub solicitation: Option<FactId>,
    pub finding: Finding,
    pub severity: Option<FindingSeverity>,
    pub confidence: Option<ConfidenceLevel>,
    pub limitations: Option<String>,
    pub recommendations: Option<String>,
    pub content: String,
    #[serde(default)]
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeSolicitationRequest {
    pub issued_to: Option<UserId>,
    pub cwe_node_id: CWENodeId,
    pub commission_fact_id: Option<FactId>,
    pub payment_scheme: PaymentScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeSolicitationEventRequest {
    pub solicitation_fact_id: FactId,
    pub event_type: SolicitationEventType,
    pub principal: Option<Principal>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactPayload {
    AuditCommission {
        commissioned_by: Principal,
        scope: Vec<CWECriterionId>,
        funding: Money,
        deadline: Option<Timestamp>,
        confidential: bool,
    },
    ElementReview {
        cwe_criterion: CWECriterionId,
        submitted_by: UserId,
        solicitation: Option<FactId>,
        finding: Finding,
        severity: Option<FindingSeverity>,
        confidence: Option<ConfidenceLevel>,
        #[serde(default)]
        limitations: Option<String>,
        #[serde(default)]
        recommendations: Option<String>,
        content: String,
        featured: bool,
    },
    #[serde(rename = "er_solicitation")]
    ERSolicitation {
        issued_to: UserId,
        cwe_criterion: CWECriterionId,
        commission: FactId,
        payment_scheme: PaymentScheme,
    },
    SolicitationEvent {
        solicitation: FactId,
        event_type: SolicitationEventType,
        principal: Principal,
        note: Option<String>,
    },
    SubmitterResponse {
        responding_to: Vec<FactId>,
        response_type: ResponseType,
        content: String,
        revision_ref: Option<AuditSubjectId>,
    },
    /// A user starting or joining a public audit episode. Prerequisite for
    /// unsolicited synthesis reviews per the UI/UX strategy memo.
    EpisodeParticipation {
        episode: AuditEpisodeId,
        participant: UserId,
        action: ParticipationAction,
        note: Option<String>,
    },
    /// Petition that an existing ElementReview by another author be placed in
    /// the featured set. Affects default UX prominence, not discoverability.
    FeaturePetition {
        element_review: FactId,
        petitioner: UserId,
        rationale: String,
    },
    /// Petition for a new CWE element, or for the applicability of an existing
    /// element to this audit subject.
    #[serde(rename = "cwe_petition")]
    CWEPetition {
        kind: CWEPetitionKind,
        cwe_node: Option<CWENodeId>,
        proposed_label: Option<String>,
        petitioner: UserId,
        rationale: String,
    },
    /// Operator curation act: grants or revokes featured status for an
    /// ElementReview or SynthesisReview. Featured status is derived from the
    /// latest active CurationDecision, keeping curation provenance-bearing
    /// rather than mutable state on an immutable fact.
    CurationDecision {
        target: CurationTarget,
        decision: CurationOutcome,
        decided_by: Principal,
        rationale: Option<String>,
        petitions: Vec<FactId>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipationAction {
    Start,
    Join,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CWEPetitionKind {
    NewElement,
    Applicability,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurationTarget {
    ElementReview {
        fact_id: FactId,
    },
    SynthesisReview {
        review_id: crate::ids::SynthesisReviewId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurationOutcome {
    Feature,
    Unfeature,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactPayloadKind {
    AuditCommission,
    ElementReview,
    #[serde(rename = "er_solicitation")]
    ErSolicitation,
    SolicitationEvent,
    SubmitterResponse,
    EpisodeParticipation,
    FeaturePetition,
    #[serde(rename = "cwe_petition")]
    CwePetition,
    CurationDecision,
}

impl FactPayload {
    pub fn kind(&self) -> FactPayloadKind {
        match self {
            Self::AuditCommission { .. } => FactPayloadKind::AuditCommission,
            Self::ElementReview { .. } => FactPayloadKind::ElementReview,
            Self::ERSolicitation { .. } => FactPayloadKind::ErSolicitation,
            Self::SolicitationEvent { .. } => FactPayloadKind::SolicitationEvent,
            Self::SubmitterResponse { .. } => FactPayloadKind::SubmitterResponse,
            Self::EpisodeParticipation { .. } => FactPayloadKind::EpisodeParticipation,
            Self::FeaturePetition { .. } => FactPayloadKind::FeaturePetition,
            Self::CWEPetition { .. } => FactPayloadKind::CwePetition,
            Self::CurationDecision { .. } => FactPayloadKind::CurationDecision,
        }
    }
}

impl FactPayloadKind {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::AuditCommission => "audit_commission",
            Self::ElementReview => "element_review",
            Self::ErSolicitation => "er_solicitation",
            Self::SolicitationEvent => "solicitation_event",
            Self::SubmitterResponse => "submitter_response",
            Self::EpisodeParticipation => "episode_participation",
            Self::FeaturePetition => "feature_petition",
            Self::CwePetition => "cwe_petition",
            Self::CurationDecision => "curation_decision",
        }
    }
}

impl TryFrom<&str> for FactPayloadKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "audit_commission" => Ok(Self::AuditCommission),
            "element_review" => Ok(Self::ElementReview),
            "er_solicitation" => Ok(Self::ErSolicitation),
            "solicitation_event" => Ok(Self::SolicitationEvent),
            "submitter_response" => Ok(Self::SubmitterResponse),
            "episode_participation" => Ok(Self::EpisodeParticipation),
            "feature_petition" => Ok(Self::FeaturePetition),
            "cwe_petition" => Ok(Self::CwePetition),
            "curation_decision" => Ok(Self::CurationDecision),
            other => Err(format!("unknown fact payload kind: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactStatus {
    Active,
    Superseded {
        superseded_by: Principal,
        superseded_at: Timestamp,
        replaced_by: Option<FactId>,
    },
    Retracted {
        retracted_by: Principal,
        retracted_at: Timestamp,
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finding {
    NonEthicalProblem,
    EthicalProblem,
    NoProblems,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    #[serde(alias = "medium")]
    Moderate,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Accepts,
    Contests,
    PartiallyAccepts,
    RevisesAndResponds,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::common::Money;
    use crate::domain_instantiation::CWECriterionId;
    use crate::ids::{CWENodeId, DomainInstantiationId, FactId, UserId};
    use crate::solicitation::{PaymentCondition, PaymentScheme};

    use super::{ConfidenceLevel, FactPayload, FactPayloadKind};

    #[test]
    fn decodes_medium_confidence_alias_as_moderate() {
        let confidence: ConfidenceLevel = serde_json::from_value(json!("medium")).unwrap();

        assert!(matches!(confidence, ConfidenceLevel::Moderate));
    }

    #[test]
    fn encodes_er_solicitation_payload_kind_with_expected_name() {
        let kind = serde_json::to_value(FactPayloadKind::ErSolicitation).unwrap();

        assert_eq!(kind, json!("er_solicitation"));
    }

    #[test]
    fn encodes_er_solicitation_payload_with_expected_tag() {
        let payload = FactPayload::ERSolicitation {
            issued_to: UserId::new("reviewer-1"),
            cwe_criterion: CWECriterionId {
                domain: DomainInstantiationId::new("domain-1"),
                node_id: CWENodeId::new("criterion-1"),
            },
            commission: FactId::new("fact-1"),
            payment_scheme: PaymentScheme {
                amount: Money {
                    amount: 100.0,
                    currency: "USD".to_string(),
                },
                currency: "USD".to_string(),
                condition: PaymentCondition::OnSubmission,
            },
        };
        let value = serde_json::to_value(payload).unwrap();

        assert!(value.get("er_solicitation").is_some());
    }

    #[test]
    fn decodes_element_review_without_optional_fields() {
        let payload: FactPayload = serde_json::from_value(json!({
            "element_review": {
                "cwe_criterion": {
                    "domain": "domain-1",
                    "node_id": "criterion-1"
                },
                "submitted_by": "reviewer-1",
                "solicitation": null,
                "finding": "inconclusive",
                "severity": null,
                "confidence": "medium",
                "content": "Legacy review payload",
                "featured": false
            }
        }))
        .unwrap();

        assert!(matches!(payload, FactPayload::ElementReview { .. }));
    }

    #[test]
    fn payload_kind_round_trips_through_db_strings() {
        let payload: FactPayload = serde_json::from_value(json!({
            "episode_participation": {
                "episode": "episode-1",
                "participant": "user-1",
                "action": "join",
                "note": null
            }
        }))
        .unwrap();
        let kind = payload.kind();

        assert_eq!(
            FactPayloadKind::try_from(kind.as_db_str()).unwrap(),
            FactPayloadKind::EpisodeParticipation
        );
    }
}
