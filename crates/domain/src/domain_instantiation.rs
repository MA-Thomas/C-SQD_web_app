use serde::{Deserialize, Serialize};

use crate::common::{Principal, Timestamp};
use crate::ids::{CWENodeId, CommunityId, DomainInstantiationId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInstantiationSummary {
    pub id: DomainInstantiationId,
    pub domain_type: DomainType,
    pub name: String,
    pub created_at: Timestamp,
    pub governed_by: Principal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInstantiationDetail {
    pub id: DomainInstantiationId,
    pub domain_type: DomainType,
    pub name: String,
    pub config: DomainConfig,
    pub cwe_nodes: Vec<CWENode>,
    pub created_at: Timestamp,
    pub governed_by: Principal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainType {
    AcademicPublishing,
    ClinicalTrialReview,
    AiAuditing,
    PolicyReview,
    /// Carries the custom domain label per the FEN schema (`Custom(String)`).
    Custom(String),
}

impl DomainType {
    pub fn db_kind(&self) -> &'static str {
        match self {
            Self::AcademicPublishing => "academic_publishing",
            Self::ClinicalTrialReview => "clinical_trial_review",
            Self::AiAuditing => "ai_auditing",
            Self::PolicyReview => "policy_review",
            Self::Custom(_) => "custom",
        }
    }

    pub fn db_detail(&self) -> Option<&str> {
        match self {
            Self::Custom(label) if !label.is_empty() => Some(label),
            _ => None,
        }
    }

    pub fn from_db(kind: &str, detail: Option<&str>) -> Result<Self, String> {
        match kind {
            "custom" => Ok(Self::Custom(detail.unwrap_or_default().to_string())),
            other => Self::try_from(other),
        }
    }
}

impl TryFrom<&str> for DomainType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "academic_publishing" => Ok(Self::AcademicPublishing),
            "clinical_trial_review" => Ok(Self::ClinicalTrialReview),
            "ai_auditing" => Ok(Self::AiAuditing),
            "policy_review" => Ok(Self::PolicyReview),
            "custom" => Ok(Self::Custom(String::new())),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StakesDefinition {
    ScientificSignificance,
    PublicHealthConsequentiality,
    DeploymentRiskProfile,
    /// Carries the custom operationalization label (`Custom(String)`).
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UptakeDefinition {
    CitationImpact,
    DownstreamAdoption,
    DeploymentDecisions,
    /// Carries the custom operationalization label (`Custom(String)`).
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrutinyWeightParams {
    pub solicited_review_multiplier: f64,
    pub expertise_weight_fn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CWENode {
    pub id: CWENodeId,
    pub domain_instantiation_id: DomainInstantiationId,
    pub parent: Option<CWENodeId>,
    pub label: String,
    pub description: String,
    pub source: CWESource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CWESource {
    BaseTaxonomy,
    /// Community extensions carry the proposing community id (deferred post-MVP).
    CommunityExtension {
        community_id: CommunityId,
    },
}

impl CWESource {
    pub fn db_kind(&self) -> &'static str {
        match self {
            Self::BaseTaxonomy => "base_taxonomy",
            Self::CommunityExtension { .. } => "community_extension",
        }
    }

    pub fn community_id(&self) -> Option<&CommunityId> {
        match self {
            Self::CommunityExtension { community_id } => Some(community_id),
            Self::BaseTaxonomy => None,
        }
    }

    pub fn from_db(kind: &str, community_id: Option<&str>) -> Result<Self, String> {
        match kind {
            "base_taxonomy" => Ok(Self::BaseTaxonomy),
            "community_extension" => Ok(Self::CommunityExtension {
                community_id: CommunityId::new(community_id.unwrap_or_default()),
            }),
            other => Err(format!("unknown CWE source: {other}")),
        }
    }
}

/// A CWE criterion id carries its domain context implicitly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CWECriterionId {
    pub domain: DomainInstantiationId,
    pub node_id: CWENodeId,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CWESource, DomainConfig, DomainType, StakesDefinition, UptakeDefinition};

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

    #[test]
    fn decodes_custom_stakes_with_label() {
        let stakes: StakesDefinition =
            serde_json::from_value(json!({ "custom": "licensing_decision_risk" })).unwrap();

        assert!(
            matches!(stakes, StakesDefinition::Custom(label) if label == "licensing_decision_risk")
        );
    }

    #[test]
    fn round_trips_community_extension_source() {
        let source = CWESource::from_db("community_extension", Some("community-1")).unwrap();

        assert_eq!(
            source.community_id().map(|id| id.as_str()),
            Some("community-1")
        );
        assert_eq!(source.db_kind(), "community_extension");
    }

    #[test]
    fn custom_domain_type_round_trips_through_db_columns() {
        let domain = DomainType::from_db("custom", Some("Internal Diligence")).unwrap();

        assert_eq!(domain.db_kind(), "custom");
        assert_eq!(domain.db_detail(), Some("Internal Diligence"));
    }
}
