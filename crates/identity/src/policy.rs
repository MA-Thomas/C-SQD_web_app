use std::fmt;

use csqd_domain::{
    AccessDecisionId, AuthorityGrantId, IdentityPrincipalId, PolicyId, Timestamp, UserId,
};
use serde::{Deserialize, Serialize};

use crate::{
    AccessDecision, AssuranceLevel, AuthenticationMethod, AuthorityGrant, AuthorityKind,
    AuthorizationBasis, AuthorizationOutcome, AuthorizedAction, IdentityModelError,
    IdentityPrincipalKind, IdentityPrincipalStatus, IdentityState, NewAccessDecision,
    ResourceScope, SponsoringParty,
};

/// Authenticated identity information supplied by the application boundary.
///
/// Raw sessions, cookies, and provider tokens deliberately do not cross into
/// this crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationContext {
    pub account_id: UserId,
    pub actor_principal_id: IdentityPrincipalId,
    pub represented_organization_principal_id: Option<IdentityPrincipalId>,
    pub authentication_method: AuthenticationMethod,
    pub authentication_assurance: AssuranceLevel,
    pub authenticated_at: Timestamp,
}

/// Conflict information relevant to scholarly and confidential actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStatus {
    Clear,
    Disclosed,
    Unresolved,
}

/// Fully typed input to the initial C-SQD authorization policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyInput {
    pub context: AuthorizationContext,
    pub action: AuthorizedAction,
    pub resource: ResourceScope,
    pub evaluated_at: Timestamp,
    pub conflict_status: ConflictStatus,
    /// Required only when granting or revoking authority.
    pub authority_mutation_target: Option<AuthorityMutationTarget>,
}

/// Complete target of a grant or revocation policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityMutationTarget {
    pub actor_principal_id: IdentityPrincipalId,
    pub represented_organization_principal_id: Option<IdentityPrincipalId>,
    pub kind: AuthorityKind,
    pub scope: ResourceScope,
}

/// Versioned parameters for the initial policy implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialPolicyConfiguration {
    pub policy_id: PolicyId,
    pub sensitive_authentication_max_age_seconds: u64,
    pub confidential_evidence_assurance: AssuranceLevel,
}

impl InitialPolicyConfiguration {
    pub const DEFAULT_SENSITIVE_AUTHENTICATION_MAX_AGE_SECONDS: u64 = 15 * 60;

    pub fn new(policy_id: PolicyId) -> Self {
        Self {
            policy_id,
            sensitive_authentication_max_age_seconds:
                Self::DEFAULT_SENSITIVE_AUTHENTICATION_MAX_AGE_SECONDS,
            confidential_evidence_assurance: AssuranceLevel::High,
        }
    }
}

/// Stable, machine-readable explanations for policy outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyReasonCode {
    AllowedAuthenticatedPrincipal,
    AllowedPersonalCapacity,
    AllowedPersonalSponsorship,
    AllowedAuthorityGrant,
    AccountPrincipalMismatch,
    HumanPrincipalRequired,
    RepresentationNotAllowed,
    RepresentedOrganizationInactive,
    AuthorityMissing,
    AuthorityKindNotPermitted,
    AuthorityActionNotPermitted,
    WrongOrganization,
    WrongResourceScope,
    GrantNotYetValid,
    GrantExpired,
    GrantRevoked,
    InsufficientAssurance,
    AuthenticationTooOld,
    UnresolvedConflict,
    AuthorityTargetNotPermitted,
    SelfEscalationNotAllowed,
}

impl PolicyReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AllowedAuthenticatedPrincipal => "allowed_authenticated_principal",
            Self::AllowedPersonalCapacity => "allowed_personal_capacity",
            Self::AllowedPersonalSponsorship => "allowed_personal_sponsorship",
            Self::AllowedAuthorityGrant => "allowed_authority_grant",
            Self::AccountPrincipalMismatch => "account_principal_mismatch",
            Self::HumanPrincipalRequired => "human_principal_required",
            Self::RepresentationNotAllowed => "representation_not_allowed",
            Self::RepresentedOrganizationInactive => "represented_organization_inactive",
            Self::AuthorityMissing => "authority_missing",
            Self::AuthorityKindNotPermitted => "authority_kind_not_permitted",
            Self::AuthorityActionNotPermitted => "authority_action_not_permitted",
            Self::WrongOrganization => "wrong_organization",
            Self::WrongResourceScope => "wrong_resource_scope",
            Self::GrantNotYetValid => "grant_not_yet_valid",
            Self::GrantExpired => "grant_expired",
            Self::GrantRevoked => "grant_revoked",
            Self::InsufficientAssurance => "insufficient_assurance",
            Self::AuthenticationTooOld => "authentication_too_old",
            Self::UnresolvedConflict => "unresolved_conflict",
            Self::AuthorityTargetNotPermitted => "authority_target_not_permitted",
            Self::SelfEscalationNotAllowed => "self_escalation_not_allowed",
        }
    }
}

/// Pure policy output suitable for persistence as an access decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub context: AuthorizationContext,
    pub action: AuthorizedAction,
    pub resource: ResourceScope,
    pub outcome: AuthorizationOutcome,
    pub policy_id: PolicyId,
    pub reason_codes: Vec<PolicyReasonCode>,
    pub authorization_basis: Option<AuthorizationBasis>,
    pub evaluated_at: Timestamp,
}

impl PolicyDecision {
    pub fn authority_grant_id(&self) -> Option<&AuthorityGrantId> {
        match &self.authorization_basis {
            Some(AuthorizationBasis::AuthorityGrant(grant_id)) => Some(grant_id),
            _ => None,
        }
    }

    pub fn to_access_decision(
        &self,
        id: AccessDecisionId,
    ) -> Result<AccessDecision, IdentityModelError> {
        AccessDecision::new(NewAccessDecision {
            id,
            account_id: self.context.account_id.clone(),
            actor_principal_id: self.context.actor_principal_id.clone(),
            represented_organization_principal_id: self
                .context
                .represented_organization_principal_id
                .clone(),
            authentication_method: self.context.authentication_method.clone(),
            authentication_assurance: self.context.authentication_assurance,
            authenticated_at: self.context.authenticated_at,
            action: self.action,
            scope: self.resource.clone(),
            outcome: self.outcome,
            policy_id: self.policy_id.clone(),
            authorization_basis: self.authorization_basis.clone(),
            reason_codes: self
                .reason_codes
                .iter()
                .map(|reason| reason.as_str().to_string())
                .collect(),
            evaluated_at: self.evaluated_at,
        })
    }
}

/// Invalid policy inputs, distinct from ordinary authorization denials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyEvaluationError {
    AuthenticationAfterEvaluation,
    MissingAuthorityMutationTarget,
    UnexpectedAuthorityMutationTarget,
}

impl fmt::Display for PolicyEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthenticationAfterEvaluation => {
                formatter.write_str("authentication time must not follow evaluation time")
            }
            Self::MissingAuthorityMutationTarget => formatter.write_str(
                "grant-authority and revoke-authority policy inputs require an authority mutation target",
            ),
            Self::UnexpectedAuthorityMutationTarget => formatter.write_str(
                "authority mutation target is only valid for grant-authority or revoke-authority",
            ),
        }
    }
}

impl std::error::Error for PolicyEvaluationError {}

/// Evaluates the initial C-SQD policy without reading the clock or external state.
///
/// For a historical decision, pass an `IdentityState` projected as of
/// `input.evaluated_at`.
pub fn evaluate_access(
    state: &IdentityState,
    configuration: &InitialPolicyConfiguration,
    input: &PolicyInput,
) -> Result<PolicyDecision, PolicyEvaluationError> {
    validate_input(input)?;

    let Some(actor) =
        state.active_principal_for_account(&input.context.account_id, &input.evaluated_at)
    else {
        return Ok(denied(
            input,
            configuration,
            PolicyReasonCode::AccountPrincipalMismatch,
        ));
    };
    if actor.id != input.context.actor_principal_id {
        return Ok(denied(
            input,
            configuration,
            PolicyReasonCode::AccountPrincipalMismatch,
        ));
    }
    if actor.kind != IdentityPrincipalKind::Human {
        return Ok(denied(
            input,
            configuration,
            PolicyReasonCode::HumanPrincipalRequired,
        ));
    }

    if let Some(organization_id) = &input.context.represented_organization_principal_id {
        let organization_is_active = state.principal(organization_id).is_some_and(|principal| {
            principal.kind == IdentityPrincipalKind::Organization
                && matches!(principal.status, crate::IdentityPrincipalStatus::Active)
        });
        if !organization_is_active {
            return Ok(denied(
                input,
                configuration,
                PolicyReasonCode::RepresentedOrganizationInactive,
            ));
        }
    }

    let requirement = requirement_for(configuration, input);
    let basis = match authorize(state, input) {
        Ok(basis) => basis,
        Err(reason) => return Ok(denied(input, configuration, reason)),
    };

    let mut step_up_reasons = Vec::new();
    if input.context.authentication_assurance < requirement.minimum_assurance {
        step_up_reasons.push(PolicyReasonCode::InsufficientAssurance);
    }
    if requirement.requires_recent_authentication
        && authentication_age_seconds(input)
            > configuration.sensitive_authentication_max_age_seconds
    {
        step_up_reasons.push(PolicyReasonCode::AuthenticationTooOld);
    }
    if !step_up_reasons.is_empty() {
        return Ok(decision(
            input,
            configuration,
            AuthorizationOutcome::StepUpRequired,
            step_up_reasons,
            Some(basis),
        ));
    }

    if requirement.manual_review_for_unresolved_conflict
        && input.conflict_status == ConflictStatus::Unresolved
    {
        return Ok(decision(
            input,
            configuration,
            AuthorizationOutcome::ManualReviewRequired,
            vec![PolicyReasonCode::UnresolvedConflict],
            Some(basis),
        ));
    }

    let reason = match basis {
        AuthorizationBasis::AuthenticatedPrincipal => {
            PolicyReasonCode::AllowedAuthenticatedPrincipal
        }
        AuthorizationBasis::PersonalCapacity => PolicyReasonCode::AllowedPersonalCapacity,
        AuthorizationBasis::PersonalSponsorship(_) => PolicyReasonCode::AllowedPersonalSponsorship,
        AuthorizationBasis::AuthorityGrant(_) => PolicyReasonCode::AllowedAuthorityGrant,
    };

    Ok(decision(
        input,
        configuration,
        AuthorizationOutcome::Allowed,
        vec![reason],
        Some(basis),
    ))
}

fn validate_input(input: &PolicyInput) -> Result<(), PolicyEvaluationError> {
    if input.context.authenticated_at > input.evaluated_at {
        return Err(PolicyEvaluationError::AuthenticationAfterEvaluation);
    }

    let targets_authority = matches!(
        input.action,
        AuthorizedAction::GrantAuthority | AuthorizedAction::RevokeAuthority
    );
    match (targets_authority, input.authority_mutation_target.is_some()) {
        (true, false) => Err(PolicyEvaluationError::MissingAuthorityMutationTarget),
        (false, true) => Err(PolicyEvaluationError::UnexpectedAuthorityMutationTarget),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Copy)]
struct PolicyRequirement {
    minimum_assurance: AssuranceLevel,
    requires_recent_authentication: bool,
    manual_review_for_unresolved_conflict: bool,
}

fn requirement_for(
    configuration: &InitialPolicyConfiguration,
    input: &PolicyInput,
) -> PolicyRequirement {
    use AuthorizedAction::{
        AcceptReviewAssignment, CommissionAudit, ExportPrivateAudit, GrantAuthority,
        ManageAccounts, ManageOrganizationMembers, PublishSynthesisReview, RecordInvoice,
        RecordPayment, RecordReviewerPayout, RegisterPublicAuditSubject, RevokeAuthority,
        SubmitElementReview, SubmitSynthesisReview, ViewConfidentialEvidence, ViewSponsoredAudit,
    };

    match input.action {
        RegisterPublicAuditSubject => requirement(AssuranceLevel::Low, false, false),
        CommissionAudit
        | ViewSponsoredAudit
        | AcceptReviewAssignment
        | ManageOrganizationMembers => requirement(AssuranceLevel::Medium, false, false),
        SubmitElementReview | SubmitSynthesisReview => {
            requirement(AssuranceLevel::Medium, false, true)
        }
        ViewConfidentialEvidence => {
            requirement(configuration.confidential_evidence_assurance, true, true)
        }
        PublishSynthesisReview => requirement(AssuranceLevel::High, true, true),
        RecordInvoice | RecordPayment | RecordReviewerPayout | ManageAccounts | GrantAuthority
        | RevokeAuthority | ExportPrivateAudit => requirement(AssuranceLevel::High, true, false),
    }
}

const fn requirement(
    minimum_assurance: AssuranceLevel,
    requires_recent_authentication: bool,
    manual_review_for_unresolved_conflict: bool,
) -> PolicyRequirement {
    PolicyRequirement {
        minimum_assurance,
        requires_recent_authentication,
        manual_review_for_unresolved_conflict,
    }
}

fn authorize(
    state: &IdentityState,
    input: &PolicyInput,
) -> Result<AuthorizationBasis, PolicyReasonCode> {
    use AuthorityKind::{
        EpisodeOperator, EpisodeReviewer, EpisodeSponsor, OrganizationAdministrator,
        OrganizationRepresentative, PlatformOperator, SponsorRepresentative, SynthesisAuthor,
    };
    use AuthorizedAction::{
        AcceptReviewAssignment, CommissionAudit, ExportPrivateAudit, GrantAuthority,
        ManageAccounts, ManageOrganizationMembers, PublishSynthesisReview, RecordInvoice,
        RecordPayment, RecordReviewerPayout, RegisterPublicAuditSubject, RevokeAuthority,
        SubmitElementReview, SubmitSynthesisReview, ViewConfidentialEvidence, ViewSponsoredAudit,
    };

    match input.action {
        RegisterPublicAuditSubject => {
            if input
                .context
                .represented_organization_principal_id
                .is_some()
            {
                Err(PolicyReasonCode::RepresentationNotAllowed)
            } else {
                Ok(AuthorizationBasis::AuthenticatedPrincipal)
            }
        }
        CommissionAudit
            if input
                .context
                .represented_organization_principal_id
                .is_none() =>
        {
            Ok(AuthorizationBasis::PersonalCapacity)
        }
        CommissionAudit => find_grant(
            state,
            input,
            &[SponsorRepresentative],
            ScopeRule::ExactOrPlatform,
        ),
        ViewSponsoredAudit => authorize_sponsored_view(
            state,
            input,
            &[
                EpisodeSponsor,
                SponsorRepresentative,
                OrganizationRepresentative,
                OrganizationAdministrator,
                EpisodeOperator,
                PlatformOperator,
            ],
        ),
        AcceptReviewAssignment | SubmitElementReview => {
            find_grant(state, input, &[EpisodeReviewer], ScopeRule::ExactOrPlatform)
        }
        SubmitSynthesisReview => find_grant(
            state,
            input,
            &[EpisodeReviewer, SynthesisAuthor],
            ScopeRule::ExactOrPlatform,
        ),
        ManageOrganizationMembers => find_grant(
            state,
            input,
            &[OrganizationAdministrator, PlatformOperator],
            ScopeRule::ExactOrPlatform,
        ),
        PublishSynthesisReview
        | RecordInvoice
        | RecordPayment
        | RecordReviewerPayout
        | ExportPrivateAudit => find_grant(
            state,
            input,
            &[EpisodeOperator, PlatformOperator],
            ScopeRule::ExactOrPlatform,
        ),
        ManageAccounts => find_grant(
            state,
            input,
            &[PlatformOperator],
            ScopeRule::ExactOrPlatform,
        ),
        GrantAuthority | RevokeAuthority => authorize_authority_mutation(state, input),
        ViewConfidentialEvidence => find_grant(state, input, &[], ScopeRule::ExactEpisodeOnly),
    }
}

fn authorize_authority_mutation(
    state: &IdentityState,
    input: &PolicyInput,
) -> Result<AuthorizationBasis, PolicyReasonCode> {
    let target = input
        .authority_mutation_target
        .as_ref()
        .expect("policy input validation requires an authority mutation target");

    if input.resource != target.scope {
        return Err(PolicyReasonCode::WrongResourceScope);
    }
    let target_actor_is_active_human =
        state
            .principal(&target.actor_principal_id)
            .is_some_and(|principal| {
                principal.kind == IdentityPrincipalKind::Human
                    && matches!(principal.status, IdentityPrincipalStatus::Active)
                    && principal.created_at <= input.evaluated_at
            });
    if !target_actor_is_active_human {
        return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
    }
    if input.action == AuthorizedAction::GrantAuthority
        && target.actor_principal_id == input.context.actor_principal_id
    {
        return Err(PolicyReasonCode::SelfEscalationNotAllowed);
    }

    let accepted_issuer_kinds: &[AuthorityKind] = match target.kind {
        AuthorityKind::PlatformOperator => {
            if target.represented_organization_principal_id.is_some()
                || !matches!(target.scope, ResourceScope::Platform)
            {
                return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
            }
            &[AuthorityKind::PlatformOperator]
        }
        AuthorityKind::OrganizationAdministrator
        | AuthorityKind::OrganizationRepresentative
        | AuthorityKind::SponsorRepresentative => {
            let (Some(represented_organization), ResourceScope::Organization(organization_id)) = (
                target.represented_organization_principal_id.as_ref(),
                &target.scope,
            ) else {
                return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
            };
            let represented_organization_is_active = state
                .principal(represented_organization)
                .is_some_and(|principal| {
                    principal.kind == IdentityPrincipalKind::Organization
                        && matches!(principal.status, IdentityPrincipalStatus::Active)
                        && principal.created_at <= input.evaluated_at
                });
            if !represented_organization_is_active {
                return Err(PolicyReasonCode::RepresentedOrganizationInactive);
            }
            if state.organization_id_for_principal(represented_organization, &input.evaluated_at)
                != Some(organization_id)
            {
                return Err(PolicyReasonCode::WrongOrganization);
            }
            &[
                AuthorityKind::OrganizationAdministrator,
                AuthorityKind::PlatformOperator,
            ]
        }
        AuthorityKind::EpisodeSponsor
        | AuthorityKind::EpisodeReviewer
        | AuthorityKind::SynthesisAuthor
        | AuthorityKind::EpisodeOperator
        | AuthorityKind::Observer => {
            if matches!(
                target.scope,
                ResourceScope::Platform | ResourceScope::Organization(_)
            ) {
                return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
            }
            &[
                AuthorityKind::EpisodeOperator,
                AuthorityKind::PlatformOperator,
            ]
        }
    };

    find_grant(
        state,
        input,
        accepted_issuer_kinds,
        ScopeRule::ExactOrPlatform,
    )
}

fn authorize_sponsored_view(
    state: &IdentityState,
    input: &PolicyInput,
    accepted_kinds: &[AuthorityKind],
) -> Result<AuthorizationBasis, PolicyReasonCode> {
    let ResourceScope::AuditEpisode(episode_id) = &input.resource else {
        return Err(PolicyReasonCode::WrongResourceScope);
    };

    let sponsorships = state.sponsorships_for_episode(episode_id, &input.evaluated_at);
    if input
        .context
        .represented_organization_principal_id
        .is_none()
    {
        if let Some(sponsorship) = sponsorships.iter().find(|sponsorship| {
            matches!(
                &sponsorship.sponsor,
                SponsoringParty::Individual(sponsor)
                    if sponsor == &input.context.actor_principal_id
            )
        }) {
            return Ok(AuthorizationBasis::PersonalSponsorship(
                sponsorship.id.clone(),
            ));
        }
    }

    let sponsoring_organization_id = input
        .context
        .represented_organization_principal_id
        .as_ref()
        .filter(|organization| {
            sponsorships.iter().any(|sponsorship| {
                matches!(
                    &sponsorship.sponsor,
                    SponsoringParty::Organization(sponsor) if sponsor == *organization
                )
            })
        })
        .and_then(|principal| state.organization_id_for_principal(principal, &input.evaluated_at));

    find_grant(
        state,
        input,
        accepted_kinds,
        ScopeRule::EpisodeOrSponsoringOrganization(sponsoring_organization_id),
    )
}

#[derive(Debug, Clone, Copy)]
enum ScopeRule<'a> {
    ExactOrPlatform,
    ExactEpisodeOnly,
    EpisodeOrSponsoringOrganization(Option<&'a csqd_domain::OrganizationId>),
}

fn find_grant(
    state: &IdentityState,
    input: &PolicyInput,
    accepted_kinds: &[AuthorityKind],
    scope_rule: ScopeRule<'_>,
) -> Result<AuthorizationBasis, PolicyReasonCode> {
    let grants = state.grants_for_actor(&input.context.actor_principal_id);
    if grants.is_empty() {
        return Err(PolicyReasonCode::AuthorityMissing);
    }

    let mut diagnostic = GrantDiagnostic::default();
    for grant in grants {
        if !accepted_kinds.is_empty() && !accepted_kinds.contains(&grant.kind) {
            diagnostic.wrong_kind = true;
            continue;
        }
        diagnostic.kind_matched = true;

        let platform_override = grant.kind == AuthorityKind::PlatformOperator
            && matches!(grant.scope, ResourceScope::Platform)
            && grant.represented_organization_principal_id.is_none();
        if !platform_override
            && grant.represented_organization_principal_id
                != input.context.represented_organization_principal_id
        {
            diagnostic.wrong_organization = true;
            continue;
        }
        diagnostic.organization_matched = true;

        if !grant.permitted_actions.contains(&input.action) {
            diagnostic.wrong_action = true;
            continue;
        }
        diagnostic.action_matched = true;

        if !scope_matches(grant, &input.resource, scope_rule) {
            diagnostic.wrong_scope = true;
            continue;
        }

        if state.grant_is_revoked_at(&grant.id, &input.evaluated_at) {
            diagnostic.revoked = true;
            continue;
        }
        if input.evaluated_at < grant.issued_at {
            diagnostic.not_yet_valid = true;
            continue;
        }
        if grant
            .validity
            .as_ref()
            .is_some_and(|validity| input.evaluated_at < validity.valid_from)
        {
            diagnostic.not_yet_valid = true;
            continue;
        }
        if grant.validity.as_ref().is_some_and(|validity| {
            validity
                .valid_until
                .as_ref()
                .is_some_and(|valid_until| &input.evaluated_at >= valid_until)
        }) {
            diagnostic.expired = true;
            continue;
        }
        if state.grant_is_active_at(&grant.id, &input.evaluated_at) {
            return Ok(AuthorizationBasis::AuthorityGrant(grant.id.clone()));
        }
    }

    Err(diagnostic.reason())
}

fn scope_matches(grant: &AuthorityGrant, requested: &ResourceScope, rule: ScopeRule<'_>) -> bool {
    if matches!(rule, ScopeRule::ExactEpisodeOnly) {
        return matches!(
            (&grant.scope, requested),
            (ResourceScope::AuditEpisode(grant_episode), ResourceScope::AuditEpisode(requested_episode))
                if grant_episode == requested_episode
        );
    }

    if &grant.scope == requested {
        return true;
    }

    match rule {
        ScopeRule::ExactOrPlatform => {
            grant.kind == AuthorityKind::PlatformOperator
                && matches!(grant.scope, ResourceScope::Platform)
        }
        ScopeRule::ExactEpisodeOnly => unreachable!("handled before general exact-scope matching"),
        ScopeRule::EpisodeOrSponsoringOrganization(sponsoring_organization) => {
            let organization_scope = matches!(
                (&grant.scope, sponsoring_organization),
                (ResourceScope::Organization(grant_organization), Some(sponsoring_organization))
                    if grant_organization == sponsoring_organization
            );
            organization_scope
                || (grant.kind == AuthorityKind::PlatformOperator
                    && matches!(grant.scope, ResourceScope::Platform))
        }
    }
}

#[derive(Debug, Default)]
struct GrantDiagnostic {
    kind_matched: bool,
    organization_matched: bool,
    action_matched: bool,
    wrong_kind: bool,
    wrong_organization: bool,
    wrong_action: bool,
    wrong_scope: bool,
    not_yet_valid: bool,
    expired: bool,
    revoked: bool,
}

impl GrantDiagnostic {
    fn reason(&self) -> PolicyReasonCode {
        if self.revoked {
            PolicyReasonCode::GrantRevoked
        } else if self.expired {
            PolicyReasonCode::GrantExpired
        } else if self.not_yet_valid {
            PolicyReasonCode::GrantNotYetValid
        } else if self.wrong_scope && self.action_matched {
            PolicyReasonCode::WrongResourceScope
        } else if self.wrong_action && self.organization_matched {
            PolicyReasonCode::AuthorityActionNotPermitted
        } else if self.wrong_organization && self.kind_matched {
            PolicyReasonCode::WrongOrganization
        } else if self.wrong_kind {
            PolicyReasonCode::AuthorityKindNotPermitted
        } else {
            PolicyReasonCode::AuthorityMissing
        }
    }
}

fn authentication_age_seconds(input: &PolicyInput) -> u64 {
    input
        .evaluated_at
        .signed_duration_since(input.context.authenticated_at)
        .num_seconds() as u64
}

fn denied(
    input: &PolicyInput,
    configuration: &InitialPolicyConfiguration,
    reason: PolicyReasonCode,
) -> PolicyDecision {
    decision(
        input,
        configuration,
        AuthorizationOutcome::Denied,
        vec![reason],
        None,
    )
}

fn decision(
    input: &PolicyInput,
    configuration: &InitialPolicyConfiguration,
    outcome: AuthorizationOutcome,
    reason_codes: Vec<PolicyReasonCode>,
    authorization_basis: Option<AuthorizationBasis>,
) -> PolicyDecision {
    PolicyDecision {
        context: input.context.clone(),
        action: input.action,
        resource: input.resource.clone(),
        outcome,
        policy_id: configuration.policy_id.clone(),
        reason_codes,
        authorization_basis,
        evaluated_at: input.evaluated_at,
    }
}
