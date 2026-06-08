use serde::{Deserialize, Serialize};

use crate::common::{Principal, Timestamp};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInstantiationSummary {
    pub id: String,
    pub domain_type: DomainType,
    pub name: String,
    pub created_at: Timestamp,
    pub governed_by: Principal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInstantiationDetail {
    pub id: String,
    pub domain_type: DomainType,
    pub name: String,
    pub config: DomainConfig,
    pub cwe_nodes: Vec<CWENode>,
    pub created_at: Timestamp,
    pub governed_by: Principal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainType {
    AcademicPublishing,
    ClinicalTrialReview,
    AiAuditing,
    PolicyReview,
    Custom,
}

impl TryFrom<&str> for DomainType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "academic_publishing" => Ok(Self::AcademicPublishing),
            "clinical_trial_review" => Ok(Self::ClinicalTrialReview),
            "ai_auditing" => Ok(Self::AiAuditing),
            "policy_review" => Ok(Self::PolicyReview),
            "custom" => Ok(Self::Custom),
            other => Err(format!("unknown domain type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub phase_config: PhaseConfig,
    pub eval_tuple_config: EvalTupleConfig,
    pub audit_object_types: Vec<String>,
    pub reviewer_concurrency: ReviewerConcurrencyLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseConfig {
    pub public_review_duration_seconds: Option<i64>,
    pub response_rounds_permitted: u32,
    pub synthesis_significance_threshold: f64,
    pub anonymity_rules: AnonymityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymityConfig {
    pub blind_submitter: bool,
    pub blind_reviewer: bool,
    pub reviewer_reidentification_delay_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerConcurrencyLimits {
    pub max_active_element_reviews: u32,
    pub max_active_synthesis_reviews: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalTupleConfig {
    pub stakes_operationalization: StakesDefinition,
    pub uptake_operationalization: UptakeDefinition,
    pub l_weight_params: ScrutinyWeightParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StakesDefinition {
    ScientificSignificance,
    PublicHealthConsequentiality,
    DeploymentRiskProfile,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UptakeDefinition {
    CitationImpact,
    DownstreamAdoption,
    DeploymentDecisions,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrutinyWeightParams {
    pub solicited_review_multiplier: f64,
    pub bounty_triggered_multiplier: f64,
    pub expertise_weight_fn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CWENode {
    pub id: String,
    pub domain_instantiation_id: String,
    pub parent: Option<String>,
    pub label: String,
    pub description: String,
    pub source: CWESource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CWESource {
    BaseTaxonomy,
    CommunityExtension,
    VerifiedTag,
}

impl TryFrom<&str> for CWESource {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "base_taxonomy" => Ok(Self::BaseTaxonomy),
            "community_extension" => Ok(Self::CommunityExtension),
            "verified_tag" => Ok(Self::VerifiedTag),
            other => Err(format!("unknown CWE source: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CWECriterionId {
    pub domain: String,
    pub node_id: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{DomainConfig, StakesDefinition, UptakeDefinition};

    #[test]
    fn decodes_seeded_academic_domain_config_shape() {
        let config: DomainConfig = serde_json::from_value(json!({
            "phase_config": {
                "public_review_duration_seconds": 2592000,
                "response_rounds_permitted": 2,
                "synthesis_significance_threshold": 0.65,
                "anonymity_rules": {
                    "blind_submitter": true,
                    "blind_reviewer": true,
                    "reviewer_reidentification_delay_seconds": 2592000
                }
            },
            "eval_tuple_config": {
                "stakes_operationalization": "scientific_significance",
                "uptake_operationalization": "citation_impact",
                "l_weight_params": {
                    "solicited_review_multiplier": 1.5,
                    "bounty_triggered_multiplier": 2.0,
                    "expertise_weight_fn": "academic_tag_endorsement_weight_v1"
                }
            },
            "audit_object_types": ["article", "preprint"],
            "reviewer_concurrency": {
                "max_active_element_reviews": 5,
                "max_active_synthesis_reviews": 2
            }
        }))
        .unwrap();

        assert!(matches!(
            config.eval_tuple_config.stakes_operationalization,
            StakesDefinition::ScientificSignificance
        ));
        assert!(matches!(
            config.eval_tuple_config.uptake_operationalization,
            UptakeDefinition::CitationImpact
        ));
        assert_eq!(config.reviewer_concurrency.max_active_element_reviews, 5);
    }
}
