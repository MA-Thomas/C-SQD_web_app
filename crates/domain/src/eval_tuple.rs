use serde::{Deserialize, Serialize};

use crate::common::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalTuple {
    pub n: f64,
    pub m: f64,
    pub s: f64,
    pub l: f64,
    pub u: f64,
    pub computed_at: Timestamp,
    pub community_filter: ReviewerCommunityFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerCommunityFilter {
    pub tags: Vec<String>,
    pub domain_instantiation_id: Option<String>,
    pub min_endorsements: Option<u32>,
}
