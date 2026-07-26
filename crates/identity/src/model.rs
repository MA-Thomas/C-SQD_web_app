use std::collections::BTreeSet;

use csqd_domain::{
    AccessDecisionId, AccountPrincipalLinkId, AuditEpisodeId, AuditSubjectId,
    AuthenticationIdentityId, AuthorityGrantId, AuthorityRevocationId, DomainInstantiationId,
    IdentityAssertionId, IdentityPrincipalId, OrganizationId, OrganizationMembershipId, PolicyId,
    Principal, SponsorshipId, SynthesisReviewId, Timestamp, UserId,
};
use serde::{Deserialize, Serialize};

use crate::IdentityModelError;

/// A durable actor represented in identity and provenance records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityPrincipal {
    pub id: IdentityPrincipalId,
    pub kind: IdentityPrincipalKind,
    pub status: IdentityPrincipalStatus,
    pub display_name: String,
    pub created_at: Timestamp,
    pub created_by: Principal,
}

impl IdentityPrincipal {
    pub fn new(
        id: IdentityPrincipalId,
        kind: IdentityPrincipalKind,
        display_name: impl Into<String>,
        created_at: Timestamp,
        created_by: Principal,
    ) -> Result<Self, IdentityModelError> {
        let principal = Self {
            id,
            kind,
            status: IdentityPrincipalStatus::Active,
            display_name: required_text(display_name, "display_name")?,
            created_at,
            created_by,
        };
        principal.validate()?;

        Ok(principal)
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        ensure_text(&self.display_name, "display_name")?;
        if let IdentityPrincipalStatus::Superseded { reason, .. } = &self.status {
            ensure_text(reason, "supersession_reason")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityPrincipalKind {
    Human,
    Organization,
    SystemAgent,
    Device,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityPrincipalStatus {
    Active,
    Disputed,
    Superseded {
        by: IdentityPrincipalId,
        reason: String,
    },
    Deactivated,
}

/// Current lifecycle state of an account-to-principal link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkStatus {
    Active,
    Disputed,
    Superseded { by: AccountPrincipalLinkId },
    Deactivated,
}

/// Connects a login account to the durable human principal it authenticates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountPrincipalLink {
    pub id: AccountPrincipalLinkId,
    pub account_id: UserId,
    pub principal_id: IdentityPrincipalId,
    pub status: LinkStatus,
    pub established_by: Principal,
    pub established_at: Timestamp,
}

/// One-to-one correspondence between an organization business record and its
/// durable identity principal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationPrincipalLink {
    pub organization_id: OrganizationId,
    pub principal_id: IdentityPrincipalId,
    pub established_by: Principal,
    pub established_at: Timestamp,
}

/// External or local authentication identity attached to an account.
///
/// Credential secrets and raw tokens are deliberately not part of this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticationIdentity {
    pub id: AuthenticationIdentityId,
    pub account_id: UserId,
    pub kind: AuthenticationIdentityKind,
    pub status: AuthenticationIdentityStatus,
    pub established_at: Timestamp,
}

impl AuthenticationIdentity {
    pub fn new(
        id: AuthenticationIdentityId,
        account_id: UserId,
        kind: AuthenticationIdentityKind,
        established_at: Timestamp,
    ) -> Result<Self, IdentityModelError> {
        let identity = Self {
            id,
            account_id,
            kind,
            status: AuthenticationIdentityStatus::Active,
            established_at,
        };
        identity.validate()?;

        Ok(identity)
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        self.kind.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationIdentityKind {
    MagicLinkEmail { email: String },
    OidcSubject { issuer: String, subject: String },
    Passkey { credential_id: String },
}

impl AuthenticationIdentityKind {
    fn validate(&self) -> Result<(), IdentityModelError> {
        match self {
            Self::MagicLinkEmail { email } => ensure_text(email, "email"),
            Self::OidcSubject { issuer, subject } => {
                ensure_text(issuer, "issuer")?;
                ensure_text(subject, "subject")
            }
            Self::Passkey { credential_id } => ensure_text(credential_id, "credential_id"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationIdentityStatus {
    Active,
    Revoked,
    Superseded { by: AuthenticationIdentityId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationMethod {
    MagicLink,
    Oidc,
    Passkey,
    MultiFactor,
    Other(String),
}

impl AuthenticationMethod {
    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        if let Self::Other(label) = self {
            ensure_text(label, "authentication_method")
        } else {
            Ok(())
        }
    }
}

/// Ordinal confidence used by authorization policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Evidence-backed claim about a principal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityAssertion {
    pub id: IdentityAssertionId,
    pub subject_principal_id: IdentityPrincipalId,
    pub kind: IdentityAssertionKind,
    pub assurance: AssuranceLevel,
    pub status: AssertionStatus,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
    pub validity: Option<ValidityPeriod>,
    pub evidence_refs: Vec<String>,
}

impl IdentityAssertion {
    pub fn new(spec: NewIdentityAssertion) -> Result<Self, IdentityModelError> {
        let mut assertion = Self {
            id: spec.id,
            subject_principal_id: spec.subject_principal_id,
            kind: spec.kind,
            assurance: spec.assurance,
            status: AssertionStatus::Active,
            asserted_by: spec.asserted_by,
            asserted_at: spec.asserted_at,
            validity: spec.validity,
            evidence_refs: spec.evidence_refs,
        };
        assertion.validate()?;
        assertion.evidence_refs = std::mem::take(&mut assertion.evidence_refs)
            .into_iter()
            .map(|value| required_text(value, "evidence_ref"))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(assertion)
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        self.kind.validate()?;
        if let Some(validity) = &self.validity {
            validity.validate()?;
        }
        for evidence_ref in &self.evidence_refs {
            ensure_text(evidence_ref, "evidence_ref")?;
        }

        Ok(())
    }
}

/// Validated creation parameters for an identity assertion.
#[derive(Debug, Clone, PartialEq)]
pub struct NewIdentityAssertion {
    pub id: IdentityAssertionId,
    pub subject_principal_id: IdentityPrincipalId,
    pub kind: IdentityAssertionKind,
    pub assurance: AssuranceLevel,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
    pub validity: Option<ValidityPeriod>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityAssertionKind {
    VerifiedEmail {
        email: String,
    },
    OidcSubject {
        issuer: String,
        subject: String,
    },
    Orcid {
        identifier: String,
    },
    InstitutionalAffiliation {
        organization_id: OrganizationId,
    },
    OrganizationMembership {
        membership_id: OrganizationMembershipId,
    },
    ReviewerExpertise {
        label: String,
    },
    ConflictDisclosure {
        disclosure_ref: String,
    },
    AuthenticatorAssertion {
        method: AuthenticationMethod,
    },
    Other(String),
}

impl IdentityAssertionKind {
    fn validate(&self) -> Result<(), IdentityModelError> {
        match self {
            Self::VerifiedEmail { email } => ensure_text(email, "email"),
            Self::OidcSubject { issuer, subject } => {
                ensure_text(issuer, "issuer")?;
                ensure_text(subject, "subject")
            }
            Self::Orcid { identifier } => ensure_text(identifier, "identifier"),
            Self::ReviewerExpertise { label } => ensure_text(label, "label"),
            Self::ConflictDisclosure { disclosure_ref } => {
                ensure_text(disclosure_ref, "disclosure_ref")
            }
            Self::Other(label) => ensure_text(label, "other"),
            Self::AuthenticatorAssertion {
                method: AuthenticationMethod::Other(label),
            } => ensure_text(label, "authentication_method"),
            Self::InstitutionalAffiliation { .. }
            | Self::OrganizationMembership { .. }
            | Self::AuthenticatorAssertion { .. } => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionStatus {
    Active,
    Disputed,
    Superseded { by: IdentityAssertionId },
    Revoked,
}

/// An optional validity window with an inclusive start and exclusive end.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityPeriod {
    pub valid_from: Timestamp,
    pub valid_until: Option<Timestamp>,
}

impl ValidityPeriod {
    pub fn new(
        valid_from: Timestamp,
        valid_until: Option<Timestamp>,
    ) -> Result<Self, IdentityModelError> {
        let period = Self {
            valid_from,
            valid_until,
        };
        period.validate()?;

        Ok(period)
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        if self
            .valid_until
            .as_ref()
            .is_some_and(|valid_until| valid_until <= &self.valid_from)
        {
            Err(IdentityModelError::InvalidValidityPeriod)
        } else {
            Ok(())
        }
    }

    pub fn contains(&self, at: &Timestamp) -> bool {
        at >= &self.valid_from
            && self
                .valid_until
                .as_ref()
                .is_none_or(|valid_until| at < valid_until)
    }
}

/// A human's verified or asserted relationship to an organization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationMembership {
    pub id: OrganizationMembershipId,
    pub member_principal_id: IdentityPrincipalId,
    pub organization_principal_id: IdentityPrincipalId,
    pub organization_id: OrganizationId,
    pub role: OrganizationMembershipRole,
    pub assurance: AssuranceLevel,
    pub status: OrganizationMembershipStatus,
    pub validity: Option<ValidityPeriod>,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
}

impl OrganizationMembership {
    pub fn new(spec: NewOrganizationMembership) -> Result<Self, IdentityModelError> {
        let membership = Self {
            id: spec.id,
            member_principal_id: spec.member_principal_id,
            organization_principal_id: spec.organization_principal_id,
            organization_id: spec.organization_id,
            role: spec.role,
            assurance: spec.assurance,
            status: spec.status,
            validity: spec.validity,
            asserted_by: spec.asserted_by,
            asserted_at: spec.asserted_at,
        };
        membership.validate()?;

        Ok(membership)
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        if self.member_principal_id == self.organization_principal_id {
            return Err(IdentityModelError::InconsistentMembership(
                "member and organization must be distinct principals",
            ));
        }
        if let OrganizationMembershipRole::Other(label) = &self.role {
            ensure_text(label, "membership_role")?;
        }
        if let Some(validity) = &self.validity {
            validity.validate()?;
        }

        Ok(())
    }
}

/// Validated creation parameters for an organization membership.
#[derive(Debug, Clone, PartialEq)]
pub struct NewOrganizationMembership {
    pub id: OrganizationMembershipId,
    pub member_principal_id: IdentityPrincipalId,
    pub organization_principal_id: IdentityPrincipalId,
    pub organization_id: OrganizationId,
    pub role: OrganizationMembershipRole,
    pub assurance: AssuranceLevel,
    pub status: OrganizationMembershipStatus,
    pub validity: Option<ValidityPeriod>,
    pub asserted_by: Principal,
    pub asserted_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationMembershipRole {
    Member,
    Administrator,
    CommissioningRepresentative,
    BillingContact,
    AuthorizedSignatory,
    ReviewerAffiliate,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationMembershipStatus {
    Invited,
    Active,
    Revoked,
    Expired,
    Superseded { by: OrganizationMembershipId },
}

/// The person or organization that owns or funds a commission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SponsoringParty {
    Individual(IdentityPrincipalId),
    Organization(IdentityPrincipalId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SponsorVisibility {
    Named,
    Generic,
    Confidential,
}

/// Connects a commissioned episode to its sponsor and human actor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sponsorship {
    pub(crate) id: SponsorshipId,
    pub(crate) episode_id: AuditEpisodeId,
    pub(crate) sponsor: SponsoringParty,
    pub(crate) actor_principal_id: IdentityPrincipalId,
    pub(crate) represented_organization_principal_id: Option<IdentityPrincipalId>,
    pub(crate) authority_grant_id: Option<AuthorityGrantId>,
    pub(crate) visibility: SponsorVisibility,
    pub(crate) created_at: Timestamp,
}

impl Sponsorship {
    pub fn id(&self) -> &SponsorshipId {
        &self.id
    }

    pub fn episode_id(&self) -> &AuditEpisodeId {
        &self.episode_id
    }

    pub fn sponsor(&self) -> &SponsoringParty {
        &self.sponsor
    }

    pub fn actor_principal_id(&self) -> &IdentityPrincipalId {
        &self.actor_principal_id
    }

    pub fn represented_organization_principal_id(&self) -> Option<&IdentityPrincipalId> {
        self.represented_organization_principal_id.as_ref()
    }

    pub fn authority_grant_id(&self) -> Option<&AuthorityGrantId> {
        self.authority_grant_id.as_ref()
    }

    pub fn visibility(&self) -> SponsorVisibility {
        self.visibility
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn individual(
        id: SponsorshipId,
        episode_id: AuditEpisodeId,
        human_principal_id: IdentityPrincipalId,
        visibility: SponsorVisibility,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            episode_id,
            sponsor: SponsoringParty::Individual(human_principal_id.clone()),
            actor_principal_id: human_principal_id,
            represented_organization_principal_id: None,
            authority_grant_id: None,
            visibility,
            created_at,
        }
    }

    pub fn organization(spec: NewOrganizationSponsorship) -> Result<Self, IdentityModelError> {
        if spec.organization_principal_id == spec.actor_principal_id {
            return Err(IdentityModelError::InconsistentSponsorship(
                "organization sponsor and human actor must be distinct principals",
            ));
        }

        Ok(Self {
            id: spec.id,
            episode_id: spec.episode_id,
            sponsor: SponsoringParty::Organization(spec.organization_principal_id.clone()),
            actor_principal_id: spec.actor_principal_id,
            represented_organization_principal_id: Some(spec.organization_principal_id),
            authority_grant_id: Some(spec.authority_grant_id),
            visibility: spec.visibility,
            created_at: spec.created_at,
        })
    }

    /// Validates invariants after deserialization or other struct construction.
    pub fn validate(&self) -> Result<(), IdentityModelError> {
        match &self.sponsor {
            SponsoringParty::Individual(individual) => {
                if individual != &self.actor_principal_id {
                    return Err(IdentityModelError::InconsistentSponsorship(
                        "individual sponsor must also be the commissioning actor",
                    ));
                }
                if self.represented_organization_principal_id.is_some()
                    || self.authority_grant_id.is_some()
                {
                    return Err(IdentityModelError::InconsistentSponsorship(
                        "individual sponsorship cannot carry organization authority",
                    ));
                }
            }
            SponsoringParty::Organization(organization) => {
                if self.represented_organization_principal_id.as_ref() != Some(organization) {
                    return Err(IdentityModelError::InconsistentSponsorship(
                        "organization sponsor must match represented organization",
                    ));
                }
                if self.authority_grant_id.is_none() {
                    return Err(IdentityModelError::InconsistentSponsorship(
                        "organization sponsorship requires an authority grant",
                    ));
                }
                if organization == &self.actor_principal_id {
                    return Err(IdentityModelError::InconsistentSponsorship(
                        "organization sponsor and human actor must be distinct principals",
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Validated creation parameters for organization sponsorship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewOrganizationSponsorship {
    pub id: SponsorshipId,
    pub episode_id: AuditEpisodeId,
    pub organization_principal_id: IdentityPrincipalId,
    pub actor_principal_id: IdentityPrincipalId,
    pub authority_grant_id: AuthorityGrantId,
    pub visibility: SponsorVisibility,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityKind {
    PlatformOperator,
    OrganizationAdministrator,
    OrganizationRepresentative,
    SponsorRepresentative,
    EpisodeSponsor,
    EpisodeReviewer,
    SynthesisAuthor,
    EpisodeOperator,
    Observer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceScope {
    Platform,
    Domain(DomainInstantiationId),
    Organization(OrganizationId),
    AuditSubject(AuditSubjectId),
    AuditEpisode(AuditEpisodeId),
    SynthesisReview(SynthesisReviewId),
    CommercialRecord(AuditEpisodeId),
    PrivateEvidenceCollection(AuditEpisodeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizedAction {
    RegisterPublicAuditSubject,
    CommissionAudit,
    ManageOrganizationMembers,
    ViewSponsoredAudit,
    AcceptReviewAssignment,
    SubmitElementReview,
    SubmitSynthesisReview,
    ViewConfidentialEvidence,
    PublishSynthesisReview,
    RecordInvoice,
    RecordPayment,
    RecordReviewerPayout,
    ManageAccounts,
    GrantAuthority,
    RevokeAuthority,
    ExportPrivateAudit,
}

/// Immutable grant of authority; revocation is represented separately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityGrant {
    pub(crate) id: AuthorityGrantId,
    pub(crate) actor_principal_id: IdentityPrincipalId,
    pub(crate) represented_organization_principal_id: Option<IdentityPrincipalId>,
    pub(crate) kind: AuthorityKind,
    pub(crate) scope: ResourceScope,
    pub(crate) permitted_actions: Vec<AuthorizedAction>,
    pub(crate) issued_by_principal_id: IdentityPrincipalId,
    pub(crate) issued_at: Timestamp,
    pub(crate) validity: Option<ValidityPeriod>,
    pub(crate) evidence_refs: Vec<String>,
}

impl AuthorityGrant {
    pub fn id(&self) -> &AuthorityGrantId {
        &self.id
    }

    pub fn actor_principal_id(&self) -> &IdentityPrincipalId {
        &self.actor_principal_id
    }

    pub fn represented_organization_principal_id(&self) -> Option<&IdentityPrincipalId> {
        self.represented_organization_principal_id.as_ref()
    }

    pub fn kind(&self) -> &AuthorityKind {
        &self.kind
    }

    pub fn scope(&self) -> &ResourceScope {
        &self.scope
    }

    pub fn permitted_actions(&self) -> &[AuthorizedAction] {
        &self.permitted_actions
    }

    pub fn issued_by_principal_id(&self) -> &IdentityPrincipalId {
        &self.issued_by_principal_id
    }

    pub fn issued_at(&self) -> &Timestamp {
        &self.issued_at
    }

    pub fn validity(&self) -> Option<&ValidityPeriod> {
        self.validity.as_ref()
    }

    pub fn evidence_refs(&self) -> &[String] {
        &self.evidence_refs
    }

    pub fn new(spec: NewAuthorityGrant) -> Result<Self, IdentityModelError> {
        let evidence_refs = spec
            .evidence_refs
            .into_iter()
            .map(|value| required_text(value, "evidence_ref"))
            .collect::<Result<Vec<_>, _>>()?;

        let grant = Self {
            id: spec.id,
            actor_principal_id: spec.actor_principal_id,
            represented_organization_principal_id: spec.represented_organization_principal_id,
            kind: spec.kind,
            scope: spec.scope,
            permitted_actions: spec.permitted_actions,
            issued_by_principal_id: spec.issued_by_principal_id,
            issued_at: spec.issued_at,
            validity: spec.validity,
            evidence_refs,
        };
        grant.validate()?;

        Ok(grant)
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        if self.permitted_actions.is_empty() {
            return Err(IdentityModelError::EmptyCollection("permitted_actions"));
        }
        let unique_actions = self
            .permitted_actions
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if unique_actions.len() != self.permitted_actions.len() {
            return Err(IdentityModelError::DuplicateCollectionValue(
                "permitted_actions",
            ));
        }
        if let Some(validity) = &self.validity {
            validity.validate()?;
        }
        for evidence_ref in &self.evidence_refs {
            ensure_text(evidence_ref, "evidence_ref")?;
        }

        Ok(())
    }

    pub(crate) fn supports_organization_commission(
        &self,
        actor_principal_id: &IdentityPrincipalId,
        organization_principal_id: &IdentityPrincipalId,
        organization_id: &OrganizationId,
    ) -> bool {
        self.kind == AuthorityKind::SponsorRepresentative
            && &self.actor_principal_id == actor_principal_id
            && self.represented_organization_principal_id.as_ref()
                == Some(organization_principal_id)
            && matches!(
                &self.scope,
                ResourceScope::Organization(scoped_organization_id)
                    if scoped_organization_id == organization_id
            )
            && self
                .permitted_actions
                .contains(&AuthorizedAction::CommissionAudit)
    }
}

/// Validated creation parameters for an immutable authority grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAuthorityGrant {
    pub id: AuthorityGrantId,
    pub actor_principal_id: IdentityPrincipalId,
    pub represented_organization_principal_id: Option<IdentityPrincipalId>,
    pub kind: AuthorityKind,
    pub scope: ResourceScope,
    pub permitted_actions: Vec<AuthorizedAction>,
    pub issued_by_principal_id: IdentityPrincipalId,
    pub issued_at: Timestamp,
    pub validity: Option<ValidityPeriod>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityRevocation {
    pub id: AuthorityRevocationId,
    pub grant_id: AuthorityGrantId,
    pub revoked_by_principal_id: IdentityPrincipalId,
    pub revoked_at: Timestamp,
    pub reason: String,
}

impl AuthorityRevocation {
    pub fn new(
        id: AuthorityRevocationId,
        grant_id: AuthorityGrantId,
        revoked_by_principal_id: IdentityPrincipalId,
        revoked_at: Timestamp,
        reason: impl Into<String>,
    ) -> Result<Self, IdentityModelError> {
        let revocation = Self {
            id,
            grant_id,
            revoked_by_principal_id,
            revoked_at,
            reason: required_text(reason, "reason")?,
        };
        revocation.validate()?;

        Ok(revocation)
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        ensure_text(&self.reason, "reason")
    }
}

/// The complete authority mutation evaluated by policy and retained in the
/// resulting audit decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mutation_type", rename_all = "snake_case")]
pub enum AuthorityMutation {
    Grant {
        grant: Box<AuthorityGrant>,
    },
    Revoke {
        grant: Box<AuthorityGrant>,
        revocation: AuthorityRevocation,
    },
}

impl AuthorityMutation {
    pub fn grant(grant: AuthorityGrant) -> Self {
        Self::Grant {
            grant: Box::new(grant),
        }
    }

    pub fn revoke(grant: AuthorityGrant, revocation: AuthorityRevocation) -> Self {
        Self::Revoke {
            grant: Box::new(grant),
            revocation,
        }
    }

    pub fn grant_record(&self) -> &AuthorityGrant {
        match self {
            Self::Grant { grant } | Self::Revoke { grant, .. } => grant,
        }
    }

    pub fn revocation(&self) -> Option<&AuthorityRevocation> {
        match self {
            Self::Grant { .. } => None,
            Self::Revoke { revocation, .. } => Some(revocation),
        }
    }

    pub fn action(&self) -> AuthorizedAction {
        match self {
            Self::Grant { .. } => AuthorizedAction::GrantAuthority,
            Self::Revoke { .. } => AuthorizedAction::RevokeAuthority,
        }
    }

    pub fn scope(&self) -> &ResourceScope {
        self.grant_record().scope()
    }
}

/// A policy request whose shape makes authority-mutation targets mandatory and
/// keeps them distinct from ordinary access checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "request_type", rename_all = "snake_case")]
pub enum AuthorizationRequest {
    Access {
        action: AuthorizedAction,
        resource: ResourceScope,
    },
    AuthorityMutation {
        mutation: AuthorityMutation,
    },
}

impl AuthorizationRequest {
    pub fn access(
        action: AuthorizedAction,
        resource: ResourceScope,
    ) -> Result<Self, IdentityModelError> {
        let request = Self::Access { action, resource };
        request.validate()?;
        Ok(request)
    }

    pub fn authority_mutation(mutation: AuthorityMutation) -> Result<Self, IdentityModelError> {
        let request = Self::AuthorityMutation { mutation };
        request.validate()?;
        Ok(request)
    }

    pub fn action(&self) -> AuthorizedAction {
        match self {
            Self::Access { action, .. } => *action,
            Self::AuthorityMutation { mutation } => mutation.action(),
        }
    }

    pub fn resource(&self) -> &ResourceScope {
        match self {
            Self::Access { resource, .. } => resource,
            Self::AuthorityMutation { mutation } => mutation.scope(),
        }
    }

    pub fn authority_mutation_target(&self) -> Option<&AuthorityMutation> {
        match self {
            Self::Access { .. } => None,
            Self::AuthorityMutation { mutation } => Some(mutation),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        match self {
            Self::Access { action, .. } => {
                if matches!(
                    action,
                    AuthorizedAction::GrantAuthority | AuthorizedAction::RevokeAuthority
                ) {
                    Err(IdentityModelError::InconsistentAccessDecision(
                        "authority actions require a complete mutation target",
                    ))
                } else {
                    Ok(())
                }
            }
            Self::AuthorityMutation { mutation } => {
                mutation.grant_record().validate()?;
                if let Some(revocation) = mutation.revocation() {
                    revocation.validate()?;
                    if &revocation.grant_id != mutation.grant_record().id() {
                        return Err(IdentityModelError::InconsistentAccessDecision(
                            "revocation must target the retained authority grant",
                        ));
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationOutcome {
    Allowed,
    Denied,
    StepUpRequired,
    ManualReviewRequired,
}

/// The identity or authority relationship that supports an authorization
/// decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationBasis {
    AuthenticatedPrincipal,
    PersonalCapacity,
    PersonalSponsorship(SponsorshipId),
    AuthorityGrant(AuthorityGrantId),
}

/// Principal reference retained by an authorization decision. `Known` means
/// the identifier existed in the replayed identity graph at evaluation time;
/// `Unresolved` preserves an attempted identifier without fabricating a
/// foreign-key-safe principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditedPrincipalReference {
    Known(IdentityPrincipalId),
    Unresolved(IdentityPrincipalId),
}

impl AuditedPrincipalReference {
    pub fn principal_id(&self) -> &IdentityPrincipalId {
        match self {
            Self::Known(id) | Self::Unresolved(id) => id,
        }
    }

    pub fn known_principal_id(&self) -> Option<&IdentityPrincipalId> {
        match self {
            Self::Known(id) => Some(id),
            Self::Unresolved(_) => None,
        }
    }
}

/// Explicit representation context retained by an authorization decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditedRepresentation {
    None,
    Known(IdentityPrincipalId),
    Unresolved(IdentityPrincipalId),
}

impl AuditedRepresentation {
    pub fn principal_id(&self) -> Option<&IdentityPrincipalId> {
        match self {
            Self::None => None,
            Self::Known(id) | Self::Unresolved(id) => Some(id),
        }
    }

    pub fn known_principal_id(&self) -> Option<&IdentityPrincipalId> {
        match self {
            Self::Known(id) => Some(id),
            Self::None | Self::Unresolved(_) => None,
        }
    }
}

/// Structurally valid authorization result. The enum prevents denied outcomes
/// from carrying a basis and requires every non-denied outcome to retain one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum AccessDecisionResult {
    Allowed {
        basis: AuthorizationBasis,
        reason: crate::PolicyReasonCode,
    },
    Denied {
        reasons: Vec<crate::PolicyReasonCode>,
    },
    StepUpRequired {
        basis: AuthorizationBasis,
        reasons: Vec<crate::PolicyReasonCode>,
    },
    ManualReviewRequired {
        basis: AuthorizationBasis,
        reasons: Vec<crate::PolicyReasonCode>,
    },
}

impl AccessDecisionResult {
    pub fn outcome(&self) -> AuthorizationOutcome {
        match self {
            Self::Allowed { .. } => AuthorizationOutcome::Allowed,
            Self::Denied { .. } => AuthorizationOutcome::Denied,
            Self::StepUpRequired { .. } => AuthorizationOutcome::StepUpRequired,
            Self::ManualReviewRequired { .. } => AuthorizationOutcome::ManualReviewRequired,
        }
    }

    pub fn authorization_basis(&self) -> Option<&AuthorizationBasis> {
        match self {
            Self::Allowed { basis, .. }
            | Self::StepUpRequired { basis, .. }
            | Self::ManualReviewRequired { basis, .. } => Some(basis),
            Self::Denied { .. } => None,
        }
    }

    pub fn reason_codes(&self) -> &[crate::PolicyReasonCode] {
        match self {
            Self::Allowed { reason, .. } => std::slice::from_ref(reason),
            Self::Denied { reasons }
            | Self::StepUpRequired { reasons, .. }
            | Self::ManualReviewRequired { reasons, .. } => reasons,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        let reasons = self.reason_codes();
        if reasons.is_empty() {
            return Err(IdentityModelError::EmptyCollection("reason_codes"));
        }
        let valid_categories = match self {
            Self::Allowed { reason, .. } => reason.is_allowed(),
            Self::Denied { reasons } => reasons.iter().all(|reason| reason.is_denial()),
            Self::StepUpRequired { reasons, .. } => {
                reasons.iter().all(|reason| reason.is_step_up())
            }
            Self::ManualReviewRequired { reasons, .. } => {
                reasons.iter().all(|reason| reason.is_manual_review())
            }
        };
        if !valid_categories {
            return Err(IdentityModelError::InconsistentAccessDecision(
                "reason codes must agree with the authorization outcome",
            ));
        }

        Ok(())
    }
}

/// Auditable result of evaluating a versioned policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessDecision {
    pub(crate) id: AccessDecisionId,
    pub(crate) account_id: UserId,
    pub(crate) actor_reference: AuditedPrincipalReference,
    pub(crate) representation: AuditedRepresentation,
    pub(crate) authentication_method: AuthenticationMethod,
    pub(crate) authentication_assurance: AssuranceLevel,
    pub(crate) authenticated_at: Timestamp,
    pub(crate) request: AuthorizationRequest,
    pub(crate) result: AccessDecisionResult,
    pub(crate) policy_id: PolicyId,
    pub(crate) evaluated_at: Timestamp,
}

impl AccessDecision {
    pub fn id(&self) -> &AccessDecisionId {
        &self.id
    }

    pub fn account_id(&self) -> &UserId {
        &self.account_id
    }

    pub fn actor_principal_id(&self) -> &IdentityPrincipalId {
        self.actor_reference.principal_id()
    }

    pub fn actor_reference(&self) -> &AuditedPrincipalReference {
        &self.actor_reference
    }

    pub fn represented_organization_principal_id(&self) -> Option<&IdentityPrincipalId> {
        self.representation.principal_id()
    }

    pub fn representation(&self) -> &AuditedRepresentation {
        &self.representation
    }

    pub fn authentication_method(&self) -> &AuthenticationMethod {
        &self.authentication_method
    }

    pub fn authentication_assurance(&self) -> AssuranceLevel {
        self.authentication_assurance
    }

    pub fn authenticated_at(&self) -> &Timestamp {
        &self.authenticated_at
    }

    pub fn action(&self) -> AuthorizedAction {
        self.request.action()
    }

    pub fn scope(&self) -> &ResourceScope {
        self.request.resource()
    }

    pub fn request(&self) -> &AuthorizationRequest {
        &self.request
    }

    pub fn outcome(&self) -> AuthorizationOutcome {
        self.result.outcome()
    }

    pub fn policy_id(&self) -> &PolicyId {
        &self.policy_id
    }

    pub fn authorization_basis(&self) -> Option<&AuthorizationBasis> {
        self.result.authorization_basis()
    }

    pub fn reason_codes(&self) -> &[crate::PolicyReasonCode] {
        self.result.reason_codes()
    }

    pub fn evaluated_at(&self) -> &Timestamp {
        &self.evaluated_at
    }

    pub fn new(spec: NewAccessDecision) -> Result<Self, IdentityModelError> {
        let decision = Self {
            id: spec.id,
            account_id: spec.account_id,
            actor_reference: spec.actor_reference,
            representation: spec.representation,
            authentication_method: spec.authentication_method,
            authentication_assurance: spec.authentication_assurance,
            authenticated_at: spec.authenticated_at,
            request: spec.request,
            result: spec.result,
            policy_id: spec.policy_id,
            evaluated_at: spec.evaluated_at,
        };
        decision.validate()?;

        Ok(decision)
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityModelError> {
        self.authentication_method.validate()?;
        self.request.validate()?;
        self.result.validate()?;
        if self.outcome() != AuthorizationOutcome::Denied {
            if matches!(
                self.actor_reference,
                AuditedPrincipalReference::Unresolved(_)
            ) {
                return Err(IdentityModelError::InconsistentAccessDecision(
                    "non-denied decisions require a known actor principal",
                ));
            }
            if matches!(self.representation, AuditedRepresentation::Unresolved(_)) {
                return Err(IdentityModelError::InconsistentAccessDecision(
                    "non-denied decisions require a known represented organization",
                ));
            }
        }
        if self.authenticated_at > self.evaluated_at {
            return Err(IdentityModelError::InconsistentAccessDecision(
                "authentication time must not follow evaluation time",
            ));
        }
        Ok(())
    }
}

/// Validated creation parameters for an auditable access decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAccessDecision {
    pub id: AccessDecisionId,
    pub account_id: UserId,
    pub actor_reference: AuditedPrincipalReference,
    pub representation: AuditedRepresentation,
    pub authentication_method: AuthenticationMethod,
    pub authentication_assurance: AssuranceLevel,
    pub authenticated_at: Timestamp,
    pub request: AuthorizationRequest,
    pub result: AccessDecisionResult,
    pub policy_id: PolicyId,
    pub evaluated_at: Timestamp,
}

fn required_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, IdentityModelError> {
    let value = value.into();
    ensure_text(&value, field)?;

    Ok(value.trim().to_string())
}

fn ensure_text(value: &str, field: &'static str) -> Result<(), IdentityModelError> {
    if value.trim().is_empty() {
        Err(IdentityModelError::EmptyField(field))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use csqd_domain::{
        AccessDecisionId, AuditEpisodeId, AuthenticationIdentityId, AuthorityGrantId,
        IdentityAssertionId, IdentityPrincipalId, PolicyId, Principal, SponsorshipId, UserId,
    };
    use serde_json::json;

    use super::*;

    fn timestamp(hour: u32) -> Timestamp {
        Utc.with_ymd_and_hms(2026, 7, 26, hour, 0, 0)
            .single()
            .unwrap()
    }

    fn principal_id(value: &str) -> IdentityPrincipalId {
        IdentityPrincipalId::new(value)
    }

    #[test]
    fn identity_principal_trims_name_and_rejects_blank_name() {
        let principal = IdentityPrincipal::new(
            principal_id("principal-1"),
            IdentityPrincipalKind::Human,
            "  Ada Reviewer  ",
            timestamp(1),
            Principal::Platform,
        )
        .unwrap();

        assert_eq!(principal.display_name, "Ada Reviewer");
        assert_eq!(
            IdentityPrincipal::new(
                principal_id("principal-2"),
                IdentityPrincipalKind::Human,
                "   ",
                timestamp(1),
                Principal::Platform,
            ),
            Err(IdentityModelError::EmptyField("display_name"))
        );
    }

    #[test]
    fn enum_labels_are_stable_snake_case() {
        assert_eq!(
            serde_json::to_value(IdentityPrincipalKind::SystemAgent).unwrap(),
            json!("system_agent")
        );
        assert_eq!(
            serde_json::to_value(AssuranceLevel::VeryHigh).unwrap(),
            json!("very_high")
        );
        assert_eq!(
            serde_json::to_value(AuthorityKind::EpisodeSponsor).unwrap(),
            json!("episode_sponsor")
        );
        assert_eq!(
            serde_json::to_value(AuthorizedAction::RecordReviewerPayout).unwrap(),
            json!("record_reviewer_payout")
        );
        assert_eq!(
            serde_json::to_value(AuthorizationOutcome::StepUpRequired).unwrap(),
            json!("step_up_required")
        );
        assert_eq!(
            serde_json::to_value(SponsorVisibility::Confidential).unwrap(),
            json!("confidential")
        );
    }

    #[test]
    fn authentication_identity_validates_public_identifiers_without_credentials() {
        let identity = AuthenticationIdentity::new(
            AuthenticationIdentityId::new("auth-identity-1"),
            UserId::new("user-1"),
            AuthenticationIdentityKind::OidcSubject {
                issuer: "https://identity.example.test".to_string(),
                subject: "subject-123".to_string(),
            },
            timestamp(1),
        )
        .unwrap();

        assert_eq!(identity.status, AuthenticationIdentityStatus::Active);
        assert_eq!(
            serde_json::to_value(&identity.kind).unwrap(),
            json!({
                "oidc_subject": {
                    "issuer": "https://identity.example.test",
                    "subject": "subject-123"
                }
            })
        );

        assert_eq!(
            AuthenticationIdentity::new(
                AuthenticationIdentityId::new("auth-identity-2"),
                UserId::new("user-1"),
                AuthenticationIdentityKind::Passkey {
                    credential_id: " ".to_string(),
                },
                timestamp(1),
            ),
            Err(IdentityModelError::EmptyField("credential_id"))
        );
    }

    #[test]
    fn data_carrying_variants_have_stable_wire_shapes() {
        assert_eq!(
            serde_json::to_value(SponsoringParty::Individual(principal_id("human-1"))).unwrap(),
            json!({"individual": "human-1"})
        );
        assert_eq!(
            serde_json::to_value(ResourceScope::AuditEpisode(AuditEpisodeId::new(
                "episode-1"
            )))
            .unwrap(),
            json!({"audit_episode": "episode-1"})
        );
        assert_eq!(
            serde_json::to_value(AuthenticationMethod::Other("saml".to_string())).unwrap(),
            json!({"other": "saml"})
        );
    }

    #[test]
    fn validity_period_requires_end_after_start() {
        assert!(ValidityPeriod::new(timestamp(1), Some(timestamp(2))).is_ok());
        assert_eq!(
            ValidityPeriod::new(timestamp(1), Some(timestamp(1))),
            Err(IdentityModelError::InvalidValidityPeriod)
        );
        assert_eq!(
            ValidityPeriod::new(timestamp(2), Some(timestamp(1))),
            Err(IdentityModelError::InvalidValidityPeriod)
        );
    }

    #[test]
    fn assertion_validates_payload_and_evidence_labels() {
        let invalid = IdentityAssertion::new(NewIdentityAssertion {
            id: IdentityAssertionId::new("assertion-1"),
            subject_principal_id: principal_id("human-1"),
            kind: IdentityAssertionKind::VerifiedEmail {
                email: "  ".to_string(),
            },
            assurance: AssuranceLevel::Medium,
            asserted_by: Principal::Platform,
            asserted_at: timestamp(1),
            validity: None,
            evidence_refs: vec![],
        });

        assert_eq!(invalid, Err(IdentityModelError::EmptyField("email")));

        let valid = IdentityAssertion::new(NewIdentityAssertion {
            id: IdentityAssertionId::new("assertion-2"),
            subject_principal_id: principal_id("human-1"),
            kind: IdentityAssertionKind::Orcid {
                identifier: "0000-0001-2345-6789".to_string(),
            },
            assurance: AssuranceLevel::Medium,
            asserted_by: Principal::Platform,
            asserted_at: timestamp(1),
            validity: None,
            evidence_refs: vec!["  orcid-api-event-1  ".to_string()],
        })
        .unwrap();

        assert_eq!(valid.evidence_refs, vec!["orcid-api-event-1"]);
    }

    #[test]
    fn individual_sponsorship_uses_same_sponsor_and_actor_without_organization() {
        let human = principal_id("human-sponsor");
        let sponsorship = Sponsorship::individual(
            SponsorshipId::new("sponsorship-1"),
            AuditEpisodeId::new("episode-1"),
            human.clone(),
            SponsorVisibility::Generic,
            timestamp(1),
        );

        assert_eq!(
            sponsorship.sponsor,
            SponsoringParty::Individual(human.clone())
        );
        assert_eq!(sponsorship.actor_principal_id, human);
        assert_eq!(sponsorship.represented_organization_principal_id, None);
        assert_eq!(sponsorship.authority_grant_id, None);
        assert_eq!(sponsorship.validate(), Ok(()));
    }

    #[test]
    fn organization_sponsorship_requires_distinct_actor_and_authority() {
        let organization = principal_id("organization-1");
        let human = principal_id("human-1");
        let grant = AuthorityGrantId::new("grant-1");
        let sponsorship = Sponsorship::organization(NewOrganizationSponsorship {
            id: SponsorshipId::new("sponsorship-1"),
            episode_id: AuditEpisodeId::new("episode-1"),
            organization_principal_id: organization.clone(),
            actor_principal_id: human.clone(),
            authority_grant_id: grant.clone(),
            visibility: SponsorVisibility::Named,
            created_at: timestamp(1),
        })
        .unwrap();

        assert_eq!(
            sponsorship.sponsor,
            SponsoringParty::Organization(organization.clone())
        );
        assert_eq!(sponsorship.actor_principal_id, human);
        assert_eq!(
            sponsorship.represented_organization_principal_id,
            Some(organization.clone())
        );
        assert_eq!(sponsorship.authority_grant_id, Some(grant));
        assert_eq!(sponsorship.validate(), Ok(()));

        assert_eq!(
            Sponsorship::organization(NewOrganizationSponsorship {
                id: SponsorshipId::new("sponsorship-2"),
                episode_id: AuditEpisodeId::new("episode-2"),
                organization_principal_id: organization.clone(),
                actor_principal_id: organization,
                authority_grant_id: AuthorityGrantId::new("grant-2"),
                visibility: SponsorVisibility::Named,
                created_at: timestamp(1),
            }),
            Err(IdentityModelError::InconsistentSponsorship(
                "organization sponsor and human actor must be distinct principals"
            ))
        );
    }

    #[test]
    fn sponsorship_validation_rejects_cross_mode_state() {
        let inconsistent = Sponsorship {
            id: SponsorshipId::new("sponsorship-invalid"),
            episode_id: AuditEpisodeId::new("episode-1"),
            sponsor: SponsoringParty::Individual(principal_id("human-1")),
            actor_principal_id: principal_id("human-1"),
            represented_organization_principal_id: Some(principal_id("organization-1")),
            authority_grant_id: Some(AuthorityGrantId::new("grant-1")),
            visibility: SponsorVisibility::Confidential,
            created_at: timestamp(1),
        };

        assert_eq!(
            inconsistent.validate(),
            Err(IdentityModelError::InconsistentSponsorship(
                "individual sponsorship cannot carry organization authority"
            ))
        );
    }

    #[test]
    fn authority_grant_requires_at_least_one_action() {
        let base = NewAuthorityGrant {
            id: AuthorityGrantId::new("grant-1"),
            actor_principal_id: principal_id("human-1"),
            represented_organization_principal_id: None,
            kind: AuthorityKind::EpisodeReviewer,
            scope: ResourceScope::AuditEpisode(AuditEpisodeId::new("episode-1")),
            permitted_actions: vec![],
            issued_by_principal_id: principal_id("operator-1"),
            issued_at: timestamp(1),
            validity: None,
            evidence_refs: vec![],
        };
        let result = AuthorityGrant::new(base.clone());

        assert_eq!(
            result,
            Err(IdentityModelError::EmptyCollection("permitted_actions"))
        );
        assert_eq!(
            AuthorityGrant::new(NewAuthorityGrant {
                permitted_actions: vec![
                    AuthorizedAction::SubmitElementReview,
                    AuthorizedAction::SubmitElementReview,
                ],
                ..base
            }),
            Err(IdentityModelError::DuplicateCollectionValue(
                "permitted_actions"
            ))
        );
    }

    #[test]
    fn access_decision_serializes_typed_policy_and_reason_codes() {
        let decision = AccessDecision::new(NewAccessDecision {
            id: AccessDecisionId::new("decision-1"),
            account_id: UserId::new("account-1"),
            actor_reference: AuditedPrincipalReference::Known(principal_id("human-1")),
            representation: AuditedRepresentation::None,
            authentication_method: AuthenticationMethod::MagicLink,
            authentication_assurance: AssuranceLevel::Medium,
            authenticated_at: timestamp(0),
            request: AuthorizationRequest::Access {
                action: AuthorizedAction::CommissionAudit,
                resource: ResourceScope::Platform,
            },
            result: AccessDecisionResult::Allowed {
                basis: AuthorizationBasis::PersonalCapacity,
                reason: crate::PolicyReasonCode::AllowedPersonalCapacity,
            },
            policy_id: PolicyId::new("policy-commission-v1"),
            evaluated_at: timestamp(1),
        })
        .unwrap();
        let json = serde_json::to_value(decision).unwrap();

        assert_eq!(json["policy_id"], json!("policy-commission-v1"));
        assert_eq!(json["result"]["basis"], json!("personal_capacity"));
        assert_eq!(json["result"]["reason"], json!("allowed_personal_capacity"));
    }

    #[test]
    fn access_decision_requires_explainable_outcome_and_basis() {
        let base = NewAccessDecision {
            id: AccessDecisionId::new("decision-invalid"),
            account_id: UserId::new("account-1"),
            actor_reference: AuditedPrincipalReference::Known(principal_id("human-1")),
            representation: AuditedRepresentation::None,
            authentication_method: AuthenticationMethod::MagicLink,
            authentication_assurance: AssuranceLevel::Medium,
            authenticated_at: timestamp(0),
            request: AuthorizationRequest::Access {
                action: AuthorizedAction::CommissionAudit,
                resource: ResourceScope::Platform,
            },
            result: AccessDecisionResult::Denied { reasons: vec![] },
            policy_id: PolicyId::new("policy-commission-v1"),
            evaluated_at: timestamp(1),
        };
        assert_eq!(
            AccessDecision::new(base.clone()),
            Err(IdentityModelError::EmptyCollection("reason_codes"))
        );

        let inconsistent = NewAccessDecision {
            result: AccessDecisionResult::StepUpRequired {
                basis: AuthorizationBasis::PersonalCapacity,
                reasons: vec![crate::PolicyReasonCode::AuthorityMissing],
            },
            ..base.clone()
        };
        assert_eq!(
            AccessDecision::new(inconsistent),
            Err(IdentityModelError::InconsistentAccessDecision(
                "reason codes must agree with the authorization outcome"
            ))
        );

        let unresolved_actor = NewAccessDecision {
            actor_reference: AuditedPrincipalReference::Unresolved(principal_id("human-1")),
            result: AccessDecisionResult::Allowed {
                basis: AuthorizationBasis::PersonalCapacity,
                reason: crate::PolicyReasonCode::AllowedPersonalCapacity,
            },
            ..base
        };
        assert_eq!(
            AccessDecision::new(unresolved_actor),
            Err(IdentityModelError::InconsistentAccessDecision(
                "non-denied decisions require a known actor principal"
            ))
        );
    }
}
