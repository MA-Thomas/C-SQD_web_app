use serde::{Deserialize, Serialize};

use crate::{
    common::{Money, Principal, Provenance, Timestamp},
    domain_instantiation::CWECriterionId,
    solicitation::{PaymentScheme, SolicitationEventType},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub subject_id: String,
    pub domain_instantiation_id: String,
    pub occurred_at: Timestamp,
    pub payload: FactPayload,
    pub status: FactStatus,
    pub provenance: Provenance,
    pub external_refs: Vec<crate::common::ExternalRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeElementReviewRequest {
    pub cwe_node_id: String,
    pub submitted_by: Option<String>,
    pub solicitation: Option<String>,
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
    pub issued_to: Option<String>,
    pub cwe_node_id: String,
    pub commission_fact_id: Option<String>,
    pub payment_scheme: PaymentScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeSolicitationEventRequest {
    pub solicitation_fact_id: String,
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
        submitted_by: String,
        solicitation: Option<String>,
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
        issued_to: String,
        cwe_criterion: CWECriterionId,
        commission: String,
        payment_scheme: PaymentScheme,
    },
    SolicitationEvent {
        solicitation: String,
        event_type: SolicitationEventType,
        principal: Principal,
        note: Option<String>,
    },
    SubmitterResponse {
        responding_to: Vec<String>,
        response_type: ResponseType,
        content: String,
        revision_ref: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactPayloadKind {
    AuditCommission,
    ElementReview,
    #[serde(rename = "er_solicitation")]
    ErSolicitation,
    SolicitationEvent,
    SubmitterResponse,
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
        replaced_by: Option<String>,
    },
    Retracted {
        retracted_by: Principal,
        retracted_at: Timestamp,
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finding {
    NonEthicalProblem,
    EthicalProblem,
    NoProblems,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    #[serde(alias = "medium")]
    Moderate,
    High,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        common::Money,
        domain_instantiation::CWECriterionId,
        solicitation::{PaymentCondition, PaymentScheme},
    };

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
            issued_to: "reviewer-1".to_string(),
            cwe_criterion: CWECriterionId {
                domain: "domain-1".to_string(),
                node_id: "criterion-1".to_string(),
            },
            commission: "fact-1".to_string(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Accepts,
    Contests,
    PartiallyAccepts,
    RevisesAndResponds,
}
