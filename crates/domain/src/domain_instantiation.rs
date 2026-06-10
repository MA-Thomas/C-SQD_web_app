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
    pub eval_tuple_config: EvalTupleConfig,
    #[serde(default)]
    pub audit_subject_types: Vec<String>,
    #[serde(default)]
    pub phase_config: Option<PhaseConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseConfig {
    pub public_review_duration: Option<i64>,
    pub response_rounds_permitted: u32,
    pub synthesis_significance_threshold: f64,
    pub anonymity_rules: AnonymityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymityConfig {
    pub blind_submitter: bool,
    pub blind_reviewer: bool,
    pub reviewer_reidentification_delay: Option<i64>,
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
}

impl TryFrom<&str> for CWESource {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "base_taxonomy" => Ok(Self::BaseTaxonomy),
            "community_extension" => Ok(Self::CommunityExtension),
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
            "phase_config": null,
            "eval_tuple_config": {
                "stakes_operationalization": "scientific_significance",
                "uptake_operationalization": "citation_impact",
                "l_weight_params": {
                    "solicited_review_multiplier": 1.5,
                    "expertise_weight_fn": "academic_tag_endorsement_weight_v1"
                }
            },
            "audit_subject_types": ["research_manuscript", "preprint"]
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
        assert!(config.phase_config.is_none());
        assert_eq!(
            config.audit_subject_types,
            ["research_manuscript", "preprint"]
        );
    }
}
