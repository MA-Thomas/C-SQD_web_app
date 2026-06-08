use serde::{Deserialize, Serialize};

use crate::{
    common::{Money, Principal, Timestamp},
    domain_instantiation::CWECriterionId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERSolicitationSummary {
    pub id: String,
    pub audit_object_id: String,
    pub cwe_criterion: CWECriterionId,
    pub issued_to: String,
    pub payment_scheme: PaymentScheme,
    pub issued_at: Timestamp,
    pub domain_instantiation_id: String,
    pub current_state: Option<SolicitationEventType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentScheme {
    pub amount: Money,
    pub currency: String,
    pub condition: PaymentCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentCondition {
    OnSubmission,
    OnAcceptance,
    Tiered {
        on_submission: Money,
        on_acceptance: Money,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolicitationEvent {
    pub id: String,
    pub solicitation_id: String,
    pub event_type: SolicitationEventType,
    pub occurred_at: Timestamp,
    pub principal: Principal,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolicitationEventType {
    Issued,
    Accepted,
    Declined,
    Expired,
    Completed,
    PenaltyFlagged,
}

impl TryFrom<&str> for SolicitationEventType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "issued" => Ok(Self::Issued),
            "accepted" => Ok(Self::Accepted),
            "declined" => Ok(Self::Declined),
            "expired" => Ok(Self::Expired),
            "completed" => Ok(Self::Completed),
            "penalty_flagged" => Ok(Self::PenaltyFlagged),
            other => Err(format!("unknown solicitation event type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PenaltySeverity {
    Moderate,
    Severe,
    EthicsViolation,
}
