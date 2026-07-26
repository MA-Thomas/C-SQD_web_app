use std::collections::BTreeSet;
use std::fmt;

use csqd_domain::{
    AccessDecisionId, AuthorityGrantId, IdentityPrincipalId, PolicyId, Timestamp, UserId,
};
use serde::{Deserialize, Serialize};

use crate::{
    AccessDecision, AccessDecisionResult, AssuranceLevel, AuditedPrincipalReference,
    AuditedRepresentation, AuthenticationMethod, AuthorityGrant, AuthorityKind, AuthorityMutation,
    AuthorizationBasis, AuthorizationOutcome, AuthorizationRequest, AuthorizedAction,
    IdentityModelError, IdentityPrincipalKind, IdentityPrincipalStatus, IdentityState,
    NewAccessDecision, ResourceScope, SponsoringParty,
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
    pub request: AuthorizationRequest,
    pub evaluated_at: Timestamp,
    pub conflict_status: ConflictStatus,
}

impl PolicyInput {
    pub fn action(&self) -> AuthorizedAction {
        self.request.action()
    }

    pub fn resource(&self) -> &ResourceScope {
        self.request.resource()
    }
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

    pub(crate) const fn is_allowed(self) -> bool {
        matches!(
            self,
            Self::AllowedAuthenticatedPrincipal
                | Self::AllowedPersonalCapacity
                | Self::AllowedPersonalSponsorship
                | Self::AllowedAuthorityGrant
        )
    }

    pub(crate) const fn is_step_up(self) -> bool {
        matches!(
            self,
            Self::InsufficientAssurance | Self::AuthenticationTooOld
        )
    }

    pub(crate) const fn is_manual_review(self) -> bool {
        matches!(self, Self::UnresolvedConflict)
    }

    pub(crate) const fn is_denial(self) -> bool {
        !self.is_allowed() && !self.is_step_up() && !self.is_manual_review()
    }
}

/// Pure policy output suitable for persistence as an access decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    context: AuthorizationContext,
    actor_reference: AuditedPrincipalReference,
    representation: AuditedRepresentation,
    request: AuthorizationRequest,
    result: AccessDecisionResult,
    policy_id: PolicyId,
    evaluated_at: Timestamp,
}

impl PolicyDecision {
    pub fn context(&self) -> &AuthorizationContext {
        &self.context
    }

    pub fn request(&self) -> &AuthorizationRequest {
        &self.request
    }

    pub fn actor_reference(&self) -> &AuditedPrincipalReference {
        &self.actor_reference
    }

    pub fn representation(&self) -> &AuditedRepresentation {
        &self.representation
    }

    pub fn action(&self) -> AuthorizedAction {
        self.request.action()
    }

    pub fn resource(&self) -> &ResourceScope {
        self.request.resource()
    }

    pub fn outcome(&self) -> AuthorizationOutcome {
        self.result.outcome()
    }

    pub fn policy_id(&self) -> &PolicyId {
        &self.policy_id
    }

    pub fn reason_codes(&self) -> &[PolicyReasonCode] {
        self.result.reason_codes()
    }

    pub fn authorization_basis(&self) -> Option<&AuthorizationBasis> {
        self.result.authorization_basis()
    }

    pub fn evaluated_at(&self) -> &Timestamp {
        &self.evaluated_at
    }

    pub fn authority_grant_id(&self) -> Option<&AuthorityGrantId> {
        match self.result.authorization_basis() {
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
            actor_reference: self.actor_reference.clone(),
            representation: self.representation.clone(),
            authentication_method: self.context.authentication_method.clone(),
            authentication_assurance: self.context.authentication_assurance,
            authenticated_at: self.context.authenticated_at,
            request: self.request.clone(),
            result: self.result.clone(),
            policy_id: self.policy_id.clone(),
            evaluated_at: self.evaluated_at,
        })
    }
}

/// Invalid policy inputs, distinct from ordinary authorization denials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyEvaluationError {
    AuthenticationAfterEvaluation,
    InvalidRequest(IdentityModelError),
}

impl fmt::Display for PolicyEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthenticationAfterEvaluation => {
                formatter.write_str("authentication time must not follow evaluation time")
            }
            Self::InvalidRequest(error) => write!(formatter, "invalid policy request: {error}"),
        }
    }
}

impl std::error::Error for PolicyEvaluationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidRequest(error) => Some(error),
            Self::AuthenticationAfterEvaluation => None,
        }
    }
}

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
            state,
            input,
            configuration,
            PolicyReasonCode::AccountPrincipalMismatch,
        ));
    };
    if actor.id != input.context.actor_principal_id {
        return Ok(denied(
            state,
            input,
            configuration,
            PolicyReasonCode::AccountPrincipalMismatch,
        ));
    }
    if actor.kind != IdentityPrincipalKind::Human {
        return Ok(denied(
            state,
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
                state,
                input,
                configuration,
                PolicyReasonCode::RepresentedOrganizationInactive,
            ));
        }
    }

    let requirement = requirement_for(configuration, input);
    let basis = match authorize(state, input) {
        Ok(basis) => basis,
        Err(reason) => return Ok(denied(state, input, configuration, reason)),
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
            state,
            input,
            configuration,
            AccessDecisionResult::StepUpRequired {
                basis,
                reasons: step_up_reasons,
            },
        ));
    }

    if requirement.manual_review_for_unresolved_conflict
        && input.conflict_status == ConflictStatus::Unresolved
    {
        return Ok(decision(
            state,
            input,
            configuration,
            AccessDecisionResult::ManualReviewRequired {
                basis,
                reasons: vec![PolicyReasonCode::UnresolvedConflict],
            },
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
        state,
        input,
        configuration,
        AccessDecisionResult::Allowed { basis, reason },
    ))
}

fn validate_input(input: &PolicyInput) -> Result<(), PolicyEvaluationError> {
    if input.context.authenticated_at > input.evaluated_at {
        return Err(PolicyEvaluationError::AuthenticationAfterEvaluation);
    }

    input
        .request
        .validate()
        .map_err(PolicyEvaluationError::InvalidRequest)
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

    match input.action() {
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

    match input.action() {
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
        CommissionAudit => find_organization_commission_grant(state, input),
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
    let Some(mutation) = input.request.authority_mutation_target() else {
        return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
    };
    let target = mutation.grant_record();
    match mutation {
        AuthorityMutation::Grant { grant } => {
            if grant.issued_by_principal_id() != &input.context.actor_principal_id
                || grant.issued_at() != &input.evaluated_at
            {
                return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
            }
        }
        AuthorityMutation::Revoke { grant, revocation } => {
            if revocation.grant_id != *grant.id()
                || revocation.revoked_by_principal_id != input.context.actor_principal_id
                || revocation.revoked_at != input.evaluated_at
                || state.authority_grant(grant.id()) != Some(grant.as_ref())
            {
                return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
            }
        }
    }

    let target_actor_is_active_human =
        state
            .principal(target.actor_principal_id())
            .is_some_and(|principal| {
                principal.kind == IdentityPrincipalKind::Human
                    && matches!(principal.status, IdentityPrincipalStatus::Active)
                    && principal.created_at <= input.evaluated_at
            });
    if !target_actor_is_active_human {
        return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
    }
    if input.action() == AuthorizedAction::GrantAuthority
        && target.actor_principal_id() == &input.context.actor_principal_id
    {
        return Err(PolicyReasonCode::SelfEscalationNotAllowed);
    }

    let accepted_issuer_kinds: &[AuthorityKind] = match target.kind() {
        AuthorityKind::PlatformOperator => {
            if target.represented_organization_principal_id().is_some()
                || !matches!(target.scope(), ResourceScope::Platform)
            {
                return Err(PolicyReasonCode::AuthorityTargetNotPermitted);
            }
            &[AuthorityKind::PlatformOperator]
        }
        AuthorityKind::OrganizationAdministrator
        | AuthorityKind::OrganizationRepresentative
        | AuthorityKind::SponsorRepresentative => {
            let (Some(represented_organization), ResourceScope::Organization(organization_id)) = (
                target.represented_organization_principal_id(),
                target.scope(),
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
                target.scope(),
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

fn find_organization_commission_grant(
    state: &IdentityState,
    input: &PolicyInput,
) -> Result<AuthorizationBasis, PolicyReasonCode> {
    let Some(organization_principal_id) =
        input.context.represented_organization_principal_id.as_ref()
    else {
        return Err(PolicyReasonCode::RepresentationNotAllowed);
    };
    let ResourceScope::Organization(organization_id) = input.resource() else {
        return Err(PolicyReasonCode::WrongResourceScope);
    };
    if state.organization_id_for_principal(organization_principal_id, &input.evaluated_at)
        != Some(organization_id)
    {
        return Err(PolicyReasonCode::WrongOrganization);
    }

    let basis = find_grant(
        state,
        input,
        &[AuthorityKind::SponsorRepresentative],
        ScopeRule::ExactOrPlatform,
    )?;
    let AuthorizationBasis::AuthorityGrant(grant_id) = &basis else {
        return Err(PolicyReasonCode::AuthorityMissing);
    };
    let Some(grant) = state.authority_grant(grant_id) else {
        return Err(PolicyReasonCode::AuthorityMissing);
    };
    if grant.supports_organization_commission(
        &input.context.actor_principal_id,
        organization_principal_id,
        organization_id,
    ) {
        Ok(basis)
    } else {
        Err(PolicyReasonCode::AuthorityTargetNotPermitted)
    }
}

fn authorize_sponsored_view(
    state: &IdentityState,
    input: &PolicyInput,
    accepted_kinds: &[AuthorityKind],
) -> Result<AuthorizationBasis, PolicyReasonCode> {
    let ResourceScope::AuditEpisode(episode_id) = input.resource() else {
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
            diagnostic.record(GrantDiagnosticFlag::WrongKind);
            continue;
        }
        diagnostic.record(GrantDiagnosticFlag::KindMatched);

        let platform_override = grant.kind == AuthorityKind::PlatformOperator
            && matches!(grant.scope, ResourceScope::Platform)
            && grant.represented_organization_principal_id.is_none();
        if !platform_override
            && grant.represented_organization_principal_id
                != input.context.represented_organization_principal_id
        {
            diagnostic.record(GrantDiagnosticFlag::WrongOrganization);
            continue;
        }
        diagnostic.record(GrantDiagnosticFlag::OrganizationMatched);

        if !grant.permitted_actions.contains(&input.action()) {
            diagnostic.record(GrantDiagnosticFlag::WrongAction);
            continue;
        }
        diagnostic.record(GrantDiagnosticFlag::ActionMatched);

        if !scope_matches(grant, input.resource(), scope_rule) {
            diagnostic.record(GrantDiagnosticFlag::WrongScope);
            continue;
        }

        if state.grant_is_revoked_at(&grant.id, &input.evaluated_at) {
            diagnostic.record(GrantDiagnosticFlag::Revoked);
            continue;
        }
        if input.evaluated_at < grant.issued_at {
            diagnostic.record(GrantDiagnosticFlag::NotYetValid);
            continue;
        }
        if grant
            .validity
            .as_ref()
            .is_some_and(|validity| input.evaluated_at < validity.valid_from)
        {
            diagnostic.record(GrantDiagnosticFlag::NotYetValid);
            continue;
        }
        if grant.validity.as_ref().is_some_and(|validity| {
            validity
                .valid_until
                .as_ref()
                .is_some_and(|valid_until| &input.evaluated_at >= valid_until)
        }) {
            diagnostic.record(GrantDiagnosticFlag::Expired);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum GrantDiagnosticFlag {
    KindMatched,
    OrganizationMatched,
    ActionMatched,
    WrongKind,
    WrongOrganization,
    WrongAction,
    WrongScope,
    NotYetValid,
    Expired,
    Revoked,
}

#[derive(Debug, Default)]
struct GrantDiagnostic(BTreeSet<GrantDiagnosticFlag>);

impl GrantDiagnostic {
    fn record(&mut self, flag: GrantDiagnosticFlag) {
        self.0.insert(flag);
    }

    fn contains(&self, flag: GrantDiagnosticFlag) -> bool {
        self.0.contains(&flag)
    }

    fn reason(&self) -> PolicyReasonCode {
        if self.contains(GrantDiagnosticFlag::Revoked) {
            PolicyReasonCode::GrantRevoked
        } else if self.contains(GrantDiagnosticFlag::Expired) {
            PolicyReasonCode::GrantExpired
        } else if self.contains(GrantDiagnosticFlag::NotYetValid) {
            PolicyReasonCode::GrantNotYetValid
        } else if self.contains(GrantDiagnosticFlag::WrongScope)
            && self.contains(GrantDiagnosticFlag::ActionMatched)
        {
            PolicyReasonCode::WrongResourceScope
        } else if self.contains(GrantDiagnosticFlag::WrongAction)
            && self.contains(GrantDiagnosticFlag::OrganizationMatched)
        {
            PolicyReasonCode::AuthorityActionNotPermitted
        } else if self.contains(GrantDiagnosticFlag::WrongOrganization)
            && self.contains(GrantDiagnosticFlag::KindMatched)
        {
            PolicyReasonCode::WrongOrganization
        } else if self.contains(GrantDiagnosticFlag::WrongKind) {
            PolicyReasonCode::AuthorityKindNotPermitted
        } else {
            PolicyReasonCode::AuthorityMissing
        }
    }
}

fn authentication_age_seconds(input: &PolicyInput) -> u64 {
    u64::try_from(
        input
            .evaluated_at
            .signed_duration_since(input.context.authenticated_at)
            .num_seconds(),
    )
    .unwrap_or_default()
}

fn denied(
    state: &IdentityState,
    input: &PolicyInput,
    configuration: &InitialPolicyConfiguration,
    reason: PolicyReasonCode,
) -> PolicyDecision {
    decision(
        state,
        input,
        configuration,
        AccessDecisionResult::Denied {
            reasons: vec![reason],
        },
    )
}

fn decision(
    state: &IdentityState,
    input: &PolicyInput,
    configuration: &InitialPolicyConfiguration,
    result: AccessDecisionResult,
) -> PolicyDecision {
    let actor_reference = if state.principal(&input.context.actor_principal_id).is_some() {
        AuditedPrincipalReference::Known(input.context.actor_principal_id.clone())
    } else {
        AuditedPrincipalReference::Unresolved(input.context.actor_principal_id.clone())
    };
    let representation = match &input.context.represented_organization_principal_id {
        None => AuditedRepresentation::None,
        Some(organization_id) if state.principal(organization_id).is_some() => {
            AuditedRepresentation::Known(organization_id.clone())
        }
        Some(organization_id) => AuditedRepresentation::Unresolved(organization_id.clone()),
    };
    PolicyDecision {
        context: input.context.clone(),
        actor_reference,
        representation,
        request: input.request.clone(),
        result,
        policy_id: configuration.policy_id.clone(),
        evaluated_at: input.evaluated_at,
    }
}
