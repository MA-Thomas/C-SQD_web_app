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
    /// Optional evidence-artifact target: which attached artifact this
    /// review inspects (claim-scoped audits memo).
    #[serde(default)]
    pub evidence_artifact: Option<String>,
    /// Optional warrant target: the WarrantAssertion fact whose claim-bearing
    /// link this review scrutinizes.
    #[serde(default)]
    pub warrant: Option<FactId>,
    #[serde(default)]
    pub featured: bool,
}

/// Request to assert a warrant link: why an attached evidence artifact is
/// supposed to bear on the target claim. Warrants are facts so they are
/// authored, timestamped, and challengeable — the central audit question is
/// whether these links survive scrutiny.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeWarrantRequest {
    pub asserted_by: Option<UserId>,
    /// The `episode_evidence_artifacts` link this warrant runs through.
    pub evidence_artifact: Option<String>,
    /// The claim the artifact itself actually makes.
    pub artifact_claim: String,
    /// The kind of inference connecting the artifact claim to the target claim.
    pub inference_type: InferenceType,
    /// The assumptions required for the artifact claim to bear on the target.
    #[serde(default)]
    pub assumptions: Option<String>,
    pub rationale: Option<String>,
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

/// Operator request to record an invoice against an episode's commission.
/// The commission fact is resolved from the episode when not supplied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceIssuedRequest {
    #[serde(default)]
    pub commission_fact_id: Option<FactId>,
    pub amount: Money,
    #[serde(default)]
    pub invoice_ref: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Operator request to record a sponsor payment. An active PaymentReceived
/// fact is what makes an episode count as funded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentReceivedRequest {
    #[serde(default)]
    pub commission_fact_id: Option<FactId>,
    pub amount: Money,
    #[serde(default)]
    pub invoice_fact_id: Option<FactId>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Operator request to record a reviewer payout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReviewerPayoutRequest {
    pub paid_to: UserId,
    pub amount: Money,
    #[serde(default)]
    pub solicitation_fact_id: Option<FactId>,
    #[serde(default)]
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
        /// Which attached evidence artifact this review inspects, when the
        /// review targets an artifact rather than the subject as a whole
        /// (claim-scoped audits memo).
        #[serde(default)]
        evidence_artifact: Option<String>,
        /// The WarrantAssertion fact under scrutiny, when the review audits a
        /// specific claim-bearing link.
        #[serde(default)]
        warrant: Option<FactId>,
        content: String,
        featured: bool,
    },
    /// A warrant link: the assertion that an attached evidence artifact makes
    /// a claim that bears on the target claim via a stated inference. Papers
    /// do not vote — their evidentiary contribution is earned when these
    /// links survive element-review scrutiny.
    WarrantAssertion {
        asserted_by: UserId,
        /// The `episode_evidence_artifacts` link this warrant runs through.
        evidence_artifact: Option<String>,
        /// The claim the artifact itself actually makes.
        artifact_claim: String,
        /// The kind of inference connecting the artifact claim to the target.
        inference_type: InferenceType,
        /// Assumptions required for the artifact claim to bear on the target.
        #[serde(default)]
        assumptions: Option<String>,
        rationale: Option<String>,
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
    /// Commercial lifecycle facts. Money movement around a commissioned
    /// audit is recorded as provenance-bearing administrative acts on the
    /// audit record itself — never as mutable billing state, and never as
    /// input to the evaluation tuple. An episode counts as *funded* when an
    /// active `PaymentReceived` fact exists for its commission: a derived
    /// view, like everything else.
    InvoiceIssued {
        /// The `AuditCommission` fact this invoice bills.
        commission: FactId,
        /// The sponsor being invoiced.
        issued_to: Principal,
        amount: Money,
        /// External invoice identifier (accounting system, PDF ref).
        invoice_ref: Option<String>,
        note: Option<String>,
    },
    PaymentReceived {
        /// The `AuditCommission` fact this payment funds.
        commission: FactId,
        /// The sponsor the payment came from.
        received_from: Principal,
        amount: Money,
        /// The `InvoiceIssued` fact this payment settles, when invoiced.
        invoice: Option<FactId>,
        note: Option<String>,
    },
    ReviewerPayout {
        paid_to: UserId,
        amount: Money,
        /// The `ERSolicitation` fact this payout compensates, when solicited.
        solicitation: Option<FactId>,
        note: Option<String>,
    },
}

/// How an artifact claim is supposed to connect to the target claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceType {
    Statistical,
    Causal,
    Mechanistic,
    ExternalValidity,
    Other,
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
    WarrantAssertion,
    InvoiceIssued,
    PaymentReceived,
    ReviewerPayout,
}

impl FactPayload {
    pub fn kind(&self) -> FactPayloadKind {
        match self {
            Self::AuditCommission { .. } => FactPayloadKind::AuditCommission,
            Self::ElementReview { .. } => FactPayloadKind::ElementReview,
            Self::WarrantAssertion { .. } => FactPayloadKind::WarrantAssertion,
            Self::ERSolicitation { .. } => FactPayloadKind::ErSolicitation,
            Self::SolicitationEvent { .. } => FactPayloadKind::SolicitationEvent,
            Self::SubmitterResponse { .. } => FactPayloadKind::SubmitterResponse,
            Self::EpisodeParticipation { .. } => FactPayloadKind::EpisodeParticipation,
            Self::FeaturePetition { .. } => FactPayloadKind::FeaturePetition,
            Self::CWEPetition { .. } => FactPayloadKind::CwePetition,
            Self::CurationDecision { .. } => FactPayloadKind::CurationDecision,
            Self::InvoiceIssued { .. } => FactPayloadKind::InvoiceIssued,
            Self::PaymentReceived { .. } => FactPayloadKind::PaymentReceived,
            Self::ReviewerPayout { .. } => FactPayloadKind::ReviewerPayout,
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
            Self::WarrantAssertion => "warrant_assertion",
            Self::InvoiceIssued => "invoice_issued",
            Self::PaymentReceived => "payment_received",
            Self::ReviewerPayout => "reviewer_payout",
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
            "warrant_assertion" => Ok(Self::WarrantAssertion),
            "invoice_issued" => Ok(Self::InvoiceIssued),
            "payment_received" => Ok(Self::PaymentReceived),
            "reviewer_payout" => Ok(Self::ReviewerPayout),
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
