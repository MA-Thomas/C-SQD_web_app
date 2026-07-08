//! The evaluation tuple `E(A | R, T_eval) -> (N, M, S, L, U)`.
//!
//! Per the FEN schema this is a derived view, not stored state: a pure
//! function over immutable `Fact` + `EpisodeMembership` records (and the
//! episode's synthesis reviews), recomputable for any reviewer community
//! filter and reference time.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::audit_episode::EpisodeMembership;
use crate::common::Timestamp;
use crate::domain_instantiation::{EvalTupleConfig, StakesDefinition, UptakeDefinition};
use crate::fact::{Fact, FactPayload, FactStatus, Finding};
use crate::ids::{DomainInstantiationId, TagId, UserId};
use crate::synthesis_review::{NarrativeStatus, SynthesisReview};

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewerCommunityFilter {
    #[serde(default)]
    pub tags: Vec<TagId>,
    #[serde(default)]
    pub domain_instantiation_id: Option<DomainInstantiationId>,
    #[serde(default)]
    pub min_endorsements: Option<u32>,
}

impl ReviewerCommunityFilter {
    pub fn unfiltered(domain_instantiation_id: Option<DomainInstantiationId>) -> Self {
        Self {
            tags: Vec::new(),
            domain_instantiation_id,
            min_endorsements: None,
        }
    }
}

/// The immutable inputs the tuple is computed over.
pub struct EvalTupleObservations<'a> {
    /// Facts paired with the membership that asserts they belong to the episode.
    pub memberships: &'a [(Fact, EpisodeMembership)],
    /// Synthesis reviews authored for the episode.
    pub synthesis_reviews: &'a [SynthesisReview],
}

/// Evaluation context: who is asking, as of when, under which domain config.
pub struct EvalTupleContext<'a> {
    pub community: &'a ReviewerCommunityFilter,
    pub t_eval: Timestamp,
    pub config: &'a EvalTupleConfig,
    /// Reviewer expertise tags, used to apply the community filter and the
    /// expertise weight function. May be empty when identity data is not
    /// loaded; an empty map with an empty `community.tags` means "all
    /// reviewers, unit expertise weight".
    pub reviewer_tags: HashMap<UserId, Vec<TagId>>,
}

/// Pure function over immutable inputs (FEN schema, "The Evaluation Tuple").
///
/// Deterministic for any fixed `(observations, community, t_eval, config)`.
/// Excludes: facts that are not `Active`, memberships that are not active,
/// facts occurring after `t_eval`, reviews by reviewers outside the community
/// filter, and synthesis reviews authored after `t_eval`.
pub fn compute_eval_tuple(
    observations: &EvalTupleObservations<'_>,
    ctx: &EvalTupleContext<'_>,
) -> EvalTuple {
    let mut n: f64 = 0.0;
    let mut m: f64 = 0.0;
    let mut s: f64 = 0.0;
    let mut l: f64 = 0.0;

    for (fact, membership) in observations.memberships {
        if !matches!(fact.status, FactStatus::Active) {
            continue;
        }

        if !membership.is_active() {
            continue;
        }

        if fact.occurred_at > ctx.t_eval {
            continue;
        }

        match &fact.payload {
            FactPayload::AuditCommission { scope, funding, .. } => {
                s = s.max(stakes_signal(
                    &ctx.config.stakes_operationalization,
                    scope.len(),
                    funding.amount,
                ));
            }
            FactPayload::ElementReview {
                finding,
                solicitation,
                submitted_by,
                ..
            } => {
                if !reviewer_in_community(ctx, submitted_by) {
                    continue;
                }

                match finding {
                    Finding::NonEthicalProblem => n += 1.0,
                    Finding::EthicalProblem => m += 1.0,
                    Finding::NoProblems | Finding::Inconclusive => {}
                }

                let solicited_weight = if solicitation.is_some() {
                    ctx.config.l_weight_params.solicited_review_multiplier
                } else {
                    1.0
                };

                l += solicited_weight * expertise_weight(ctx, submitted_by);
            }
            // Warrant assertions are the links under scrutiny, not scrutiny
            // itself: they shape the audit graph but only element reviews of
            // them move N/M/L.
            FactPayload::WarrantAssertion { .. }
            | FactPayload::ERSolicitation { .. }
            | FactPayload::SolicitationEvent { .. }
            | FactPayload::SubmitterResponse { .. }
            | FactPayload::EpisodeParticipation { .. }
            | FactPayload::FeaturePetition { .. }
            | FactPayload::CWEPetition { .. }
            | FactPayload::CurationDecision { .. } => {}
        }
    }

    let u = uptake_signal(
        &ctx.config.uptake_operationalization,
        observations,
        ctx.t_eval,
    );

    EvalTuple {
        n,
        m,
        s,
        l,
        u,
        computed_at: ctx.t_eval,
        community_filter: ctx.community.clone(),
    }
}

fn reviewer_in_community(ctx: &EvalTupleContext<'_>, reviewer: &UserId) -> bool {
    if ctx.community.tags.is_empty() {
        return true;
    }

    ctx.reviewer_tags
        .get(reviewer)
        .map(|tags| tags.iter().any(|tag| ctx.community.tags.contains(tag)))
        .unwrap_or(false)
}

/// Expertise weighting hook. The function is selected by
/// `l_weight_params.expertise_weight_fn`; unknown identifiers fall back to
/// unit weight so that the tuple stays total.
fn expertise_weight(ctx: &EvalTupleContext<'_>, reviewer: &UserId) -> f64 {
    match ctx.config.l_weight_params.expertise_weight_fn.as_str() {
        // Weight grows mildly with the number of (endorsed) expertise tags.
        "academic_tag_endorsement_weight_v1" => {
            let tag_count = ctx
                .reviewer_tags
                .get(reviewer)
                .map(|tags| tags.len())
                .unwrap_or(0);

            1.0 + 0.1 * tag_count.min(5) as f64
        }
        _ => 1.0,
    }
}

/// Stakes operationalization (`S`). MVP proxies, varied per definition so the
/// domain hook is real; richer signals replace the bodies, not the structure.
fn stakes_signal(definition: &StakesDefinition, scope_len: usize, funding_amount: f64) -> f64 {
    let funding_signal = (funding_amount / 10_000.0).clamp(0.0, 2.0);
    let base = scope_len as f64 + funding_signal;

    match definition {
        StakesDefinition::ScientificSignificance => base,
        // Health-consequential and deployment-risk audits weight commissioned
        // scope more heavily until domain-specific signals exist.
        StakesDefinition::PublicHealthConsequentiality => base * 1.25,
        StakesDefinition::DeploymentRiskProfile => base * 1.25,
        StakesDefinition::Custom(_) => base,
    }
}

/// Uptake operationalization (`U`). MVP proxy: visible engagement with the
/// audit record up to `t_eval` (synthesis reviews and submitter responses).
fn uptake_signal(
    definition: &UptakeDefinition,
    observations: &EvalTupleObservations<'_>,
    t_eval: Timestamp,
) -> f64 {
    let synthesis_count = observations
        .synthesis_reviews
        .iter()
        .filter(|review| review.authored_at <= t_eval)
        .filter(|review| {
            matches!(
                review.status,
                NarrativeStatus::Draft | NarrativeStatus::Current
            )
        })
        .count() as f64;
    let response_count = observations
        .memberships
        .iter()
        .filter(|(fact, membership)| {
            matches!(fact.status, FactStatus::Active)
                && membership.is_active()
                && fact.occurred_at <= t_eval
                && matches!(fact.payload, FactPayload::SubmitterResponse { .. })
        })
        .count() as f64;

    match definition {
        UptakeDefinition::CitationImpact
        | UptakeDefinition::DownstreamAdoption
        | UptakeDefinition::DeploymentDecisions
        | UptakeDefinition::Custom(_) => synthesis_count + response_count,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{TimeZone, Utc};

    use crate::audit_episode::{EpisodeMembership, EpisodeMembershipStatus, FactRole};
    use crate::common::{Money, Principal, Provenance, Timestamp};
    use crate::domain_instantiation::{
        CWECriterionId, EvalTupleConfig, ScrutinyWeightParams, StakesDefinition, UptakeDefinition,
    };
    use crate::fact::{Fact, FactPayload, FactStatus, Finding};
    use crate::ids::{
        AuditEpisodeId, AuditSubjectId, CWENodeId, DomainInstantiationId, FactId, MembershipId,
        TagId, UserId,
    };

    use super::{
        compute_eval_tuple, EvalTupleContext, EvalTupleObservations, ReviewerCommunityFilter,
    };

    fn ts(seconds: i64) -> Timestamp {
        Utc.timestamp_opt(seconds, 0).unwrap()
    }

    fn config() -> EvalTupleConfig {
        EvalTupleConfig {
            stakes_operationalization: StakesDefinition::ScientificSignificance,
            uptake_operationalization: UptakeDefinition::CitationImpact,
            l_weight_params: ScrutinyWeightParams {
                solicited_review_multiplier: 1.5,
                expertise_weight_fn: "unit".to_string(),
            },
        }
    }

    fn element_review_fact(
        id: &str,
        occurred_at: Timestamp,
        reviewer: &str,
        finding: Finding,
        solicited: bool,
    ) -> Fact {
        Fact {
            id: FactId::new(id),
            subject_id: AuditSubjectId::new("subject-1"),
            domain_instantiation_id: DomainInstantiationId::new("domain-1"),
            occurred_at,
            payload: FactPayload::ElementReview {
                cwe_criterion: CWECriterionId {
                    domain: DomainInstantiationId::new("domain-1"),
                    node_id: CWENodeId::new("criterion-1"),
                },
                submitted_by: UserId::new(reviewer),
                solicitation: solicited.then(|| FactId::new("solicitation-1")),
                finding,
                severity: None,
                confidence: None,
                limitations: None,
                recommendations: None,
                evidence_artifact: None,
                warrant: None,
                content: "review".to_string(),
                featured: false,
            },
            status: FactStatus::Active,
            provenance: Provenance {
                source_system: None,
                source_document: None,
                imported_at: occurred_at,
                principal: Principal::Platform,
            },
            external_refs: Vec::new(),
        }
    }

    fn membership(fact: &Fact, active: bool) -> EpisodeMembership {
        EpisodeMembership {
            id: MembershipId::new(format!("membership-{}", fact.id.as_str())),
            fact_id: fact.id.clone(),
            episode_id: AuditEpisodeId::new("episode-1"),
            role: FactRole::ElementReview,
            asserted_by: Principal::Platform,
            asserted_at: fact.occurred_at,
            status: if active {
                EpisodeMembershipStatus::Active
            } else {
                EpisodeMembershipStatus::Retracted {
                    retracted_by: Principal::Platform,
                    retracted_at: fact.occurred_at,
                }
            },
        }
    }

    #[test]
    fn counts_problems_and_weights_solicited_reviews() {
        let fact_a = element_review_fact("f1", ts(100), "user-1", Finding::NonEthicalProblem, true);
        let fact_b = element_review_fact("f2", ts(200), "user-2", Finding::EthicalProblem, false);
        let memberships = vec![
            (fact_a.clone(), membership(&fact_a, true)),
            (fact_b.clone(), membership(&fact_b, true)),
        ];
        let cfg = config();
        let community = ReviewerCommunityFilter::default();
        let tuple = compute_eval_tuple(
            &EvalTupleObservations {
                memberships: &memberships,
                synthesis_reviews: &[],
            },
            &EvalTupleContext {
                community: &community,
                t_eval: ts(1_000),
                config: &cfg,
                reviewer_tags: HashMap::new(),
            },
        );

        assert_eq!(tuple.n, 1.0);
        assert_eq!(tuple.m, 1.0);
        assert_eq!(tuple.l, 2.5);
    }

    #[test]
    fn excludes_facts_after_t_eval() {
        let fact = element_review_fact("f1", ts(500), "user-1", Finding::NonEthicalProblem, false);
        let memberships = vec![(fact.clone(), membership(&fact, true))];
        let cfg = config();
        let community = ReviewerCommunityFilter::default();
        let tuple = compute_eval_tuple(
            &EvalTupleObservations {
                memberships: &memberships,
                synthesis_reviews: &[],
            },
            &EvalTupleContext {
                community: &community,
                t_eval: ts(100),
                config: &cfg,
                reviewer_tags: HashMap::new(),
            },
        );

        assert_eq!(tuple.n, 0.0);
        assert_eq!(tuple.l, 0.0);
    }

    #[test]
    fn excludes_retracted_memberships() {
        let fact = element_review_fact("f1", ts(100), "user-1", Finding::NonEthicalProblem, false);
        let memberships = vec![(fact.clone(), membership(&fact, false))];
        let cfg = config();
        let community = ReviewerCommunityFilter::default();
        let tuple = compute_eval_tuple(
            &EvalTupleObservations {
                memberships: &memberships,
                synthesis_reviews: &[],
            },
            &EvalTupleContext {
                community: &community,
                t_eval: ts(1_000),
                config: &cfg,
                reviewer_tags: HashMap::new(),
            },
        );

        assert_eq!(tuple.n, 0.0);
    }

    #[test]
    fn applies_reviewer_community_filter() {
        let fact_a =
            element_review_fact("f1", ts(100), "user-1", Finding::NonEthicalProblem, false);
        let fact_b =
            element_review_fact("f2", ts(100), "user-2", Finding::NonEthicalProblem, false);
        let memberships = vec![
            (fact_a.clone(), membership(&fact_a, true)),
            (fact_b.clone(), membership(&fact_b, true)),
        ];
        let cfg = config();
        let community = ReviewerCommunityFilter {
            tags: vec![TagId::new("statistics")],
            domain_instantiation_id: None,
            min_endorsements: None,
        };
        let mut reviewer_tags = HashMap::new();
        reviewer_tags.insert(UserId::new("user-1"), vec![TagId::new("statistics")]);

        let tuple = compute_eval_tuple(
            &EvalTupleObservations {
                memberships: &memberships,
                synthesis_reviews: &[],
            },
            &EvalTupleContext {
                community: &community,
                t_eval: ts(1_000),
                config: &cfg,
                reviewer_tags,
            },
        );

        assert_eq!(tuple.n, 1.0);
        assert_eq!(tuple.l, 1.0);
    }
}
