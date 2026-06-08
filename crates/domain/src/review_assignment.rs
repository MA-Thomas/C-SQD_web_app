use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAssignmentSummary {
    pub id: String,
    pub scholarly_object_id: String,
    pub scholarly_object_title: String,
    pub scholarly_object_canonical_url: String,
    pub reviewer_display_name: String,
    pub assignment_type: ReviewAssignmentType,
    pub compensation_status: CompensationStatus,
    pub state: ReviewAssignmentState,
    pub due_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAssignmentType {
    ElementReview,
    SynthesisReview,
    ErrorClaimReview,
}

impl TryFrom<&str> for ReviewAssignmentType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "element_review" => Ok(Self::ElementReview),
            "synthesis_review" => Ok(Self::SynthesisReview),
            "error_claim_review" => Ok(Self::ErrorClaimReview),
            other => Err(format!("unknown review assignment type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompensationStatus {
    Unpaid,
    Eligible,
    Approved,
    Paid,
}

impl TryFrom<&str> for CompensationStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "unpaid" => Ok(Self::Unpaid),
            "eligible" => Ok(Self::Eligible),
            "approved" => Ok(Self::Approved),
            "paid" => Ok(Self::Paid),
            other => Err(format!("unknown compensation status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAssignmentState {
    Created,
    Offered,
    Accepted,
    Declined,
    InProgress,
    Submitted,
    QualityControl,
    Published,
    Canceled,
}

impl TryFrom<&str> for ReviewAssignmentState {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "created" => Ok(Self::Created),
            "offered" => Ok(Self::Offered),
            "accepted" => Ok(Self::Accepted),
            "declined" => Ok(Self::Declined),
            "in_progress" => Ok(Self::InProgress),
            "submitted" => Ok(Self::Submitted),
            "quality_control" => Ok(Self::QualityControl),
            "published" => Ok(Self::Published),
            "canceled" => Ok(Self::Canceled),
            other => Err(format!("unknown review assignment state: {other}")),
        }
    }
}
