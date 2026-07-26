use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use csqd_domain::{
    AccessDecisionId, AccountPrincipalLinkId, AuthenticationIdentityId, AuthorityGrantId,
    AuthorityRevocationId, IdentityAssertionId, IdentityEventId, IdentityPrincipalId,
    OrganizationId, OrganizationMembershipId, SponsorshipId, Timestamp, UserId,
};

use crate::{
    AccessDecision, AccountPrincipalLink, AssertionStatus, AuthenticationIdentity,
    AuthenticationIdentityStatus, AuthorityGrant, AuthorityKind, AuthorityRevocation,
    AuthorizationBasis, IdentityAssertion, IdentityEvent, IdentityEventPayload,
    IdentityEventValidationError, IdentityPrincipal, IdentityPrincipalKind,
    IdentityPrincipalStatus, LinkStatus, OrganizationMembership, OrganizationMembershipStatus,
    OrganizationPrincipalLink, ResourceScope, SponsoringParty, Sponsorship,
};

/// Deterministic materialization of an ordered identity event ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityState {
    projected_through_sequence: Option<u64>,
    as_of: Option<Timestamp>,
    event_count: usize,
    principals: BTreeMap<IdentityPrincipalId, IdentityPrincipal>,
    account_links: BTreeMap<AccountPrincipalLinkId, AccountPrincipalLink>,
    organization_principal_links: BTreeMap<OrganizationId, OrganizationPrincipalLink>,
    authentication_identities: BTreeMap<AuthenticationIdentityId, AuthenticationIdentity>,
    assertions: BTreeMap<IdentityAssertionId, IdentityAssertion>,
    memberships: BTreeMap<OrganizationMembershipId, OrganizationMembership>,
    sponsorships: BTreeMap<SponsorshipId, Sponsorship>,
    authority_grants: BTreeMap<AuthorityGrantId, AuthorityGrant>,
    authority_revocations: BTreeMap<AuthorityRevocationId, AuthorityRevocation>,
    access_decisions: BTreeMap<AccessDecisionId, AccessDecision>,
}

impl IdentityState {
    fn empty(as_of: Option<Timestamp>) -> Self {
        Self {
            projected_through_sequence: None,
            as_of,
            event_count: 0,
            principals: BTreeMap::new(),
            account_links: BTreeMap::new(),
            organization_principal_links: BTreeMap::new(),
            authentication_identities: BTreeMap::new(),
            assertions: BTreeMap::new(),
            memberships: BTreeMap::new(),
            sponsorships: BTreeMap::new(),
            authority_grants: BTreeMap::new(),
            authority_revocations: BTreeMap::new(),
            access_decisions: BTreeMap::new(),
        }
    }

    pub fn projected_through_sequence(&self) -> Option<u64> {
        self.projected_through_sequence
    }

    pub fn as_of(&self) -> Option<&Timestamp> {
        self.as_of.as_ref()
    }

    pub fn event_count(&self) -> usize {
        self.event_count
    }

    pub fn principal(&self, id: &IdentityPrincipalId) -> Option<&IdentityPrincipal> {
        self.principals.get(id)
    }

    pub fn account_link(&self, id: &AccountPrincipalLinkId) -> Option<&AccountPrincipalLink> {
        self.account_links.get(id)
    }

    pub fn authentication_identity(
        &self,
        id: &AuthenticationIdentityId,
    ) -> Option<&AuthenticationIdentity> {
        self.authentication_identities.get(id)
    }

    pub fn organization_principal_for(
        &self,
        organization_id: &OrganizationId,
        at: &Timestamp,
    ) -> Option<&IdentityPrincipal> {
        self.organization_principal_links
            .get(organization_id)
            .filter(|link| &link.established_at <= at)
            .and_then(|link| self.principals.get(&link.principal_id))
            .filter(|principal| {
                matches!(principal.status, IdentityPrincipalStatus::Active)
                    && &principal.created_at <= at
            })
    }

    pub fn organization_id_for_principal(
        &self,
        principal_id: &IdentityPrincipalId,
        at: &Timestamp,
    ) -> Option<&OrganizationId> {
        self.organization_principal_links
            .iter()
            .find_map(|(organization_id, link)| {
                (&link.principal_id == principal_id && &link.established_at <= at)
                    .then_some(organization_id)
            })
    }

    pub fn assertion(&self, id: &IdentityAssertionId) -> Option<&IdentityAssertion> {
        self.assertions.get(id)
    }

    pub fn membership(&self, id: &OrganizationMembershipId) -> Option<&OrganizationMembership> {
        self.memberships.get(id)
    }

    pub fn sponsorship(&self, id: &SponsorshipId) -> Option<&Sponsorship> {
        self.sponsorships.get(id)
    }

    pub fn authority_grant(&self, id: &AuthorityGrantId) -> Option<&AuthorityGrant> {
        self.authority_grants.get(id)
    }

    pub fn grants_for_actor(&self, principal_id: &IdentityPrincipalId) -> Vec<&AuthorityGrant> {
        self.authority_grants
            .values()
            .filter(|grant| &grant.actor_principal_id == principal_id)
            .collect()
    }

    pub fn sponsorships_for_episode(
        &self,
        episode_id: &csqd_domain::AuditEpisodeId,
        at: &Timestamp,
    ) -> Vec<&Sponsorship> {
        self.sponsorships
            .values()
            .filter(|sponsorship| {
                &sponsorship.episode_id == episode_id
                    && &sponsorship.created_at <= at
                    && match &sponsorship.sponsor {
                        SponsoringParty::Individual(sponsor) => {
                            self.principal_is_active_at(sponsor, at)
                        }
                        SponsoringParty::Organization(sponsor) => {
                            self.principal_is_active_at(sponsor, at)
                                && self.organization_id_for_principal(sponsor, at).is_some()
                        }
                    }
            })
            .collect()
    }

    pub fn access_decision(&self, id: &AccessDecisionId) -> Option<&AccessDecision> {
        self.access_decisions.get(id)
    }

    pub fn active_principal_for_account(
        &self,
        account_id: &UserId,
        at: &Timestamp,
    ) -> Option<&IdentityPrincipal> {
        self.account_links
            .values()
            .find(|link| {
                &link.account_id == account_id
                    && matches!(link.status, LinkStatus::Active)
                    && &link.established_at <= at
            })
            .and_then(|link| self.principals.get(&link.principal_id))
            .filter(|principal| {
                matches!(principal.status, IdentityPrincipalStatus::Active)
                    && &principal.created_at <= at
            })
    }

    pub fn active_assertions_for(
        &self,
        principal_id: &IdentityPrincipalId,
        at: &Timestamp,
    ) -> Vec<&IdentityAssertion> {
        self.assertions
            .values()
            .filter(|assertion| {
                &assertion.subject_principal_id == principal_id
                    && matches!(assertion.status, AssertionStatus::Active)
                    && &assertion.asserted_at <= at
                    && validity_contains(assertion.validity.as_ref(), at)
                    && self.principal_is_active_at(&assertion.subject_principal_id, at)
            })
            .collect()
    }

    pub fn active_memberships_for(
        &self,
        principal_id: &IdentityPrincipalId,
        at: &Timestamp,
    ) -> Vec<&OrganizationMembership> {
        self.memberships
            .values()
            .filter(|membership| {
                &membership.member_principal_id == principal_id
                    && matches!(membership.status, OrganizationMembershipStatus::Active)
                    && &membership.asserted_at <= at
                    && validity_contains(membership.validity.as_ref(), at)
                    && self.principal_is_active_at(&membership.member_principal_id, at)
                    && self.principal_is_active_at(&membership.organization_principal_id, at)
                    && self.organization_id_for_principal(&membership.organization_principal_id, at)
                        == Some(&membership.organization_id)
            })
            .collect()
    }

    pub fn active_grants_for(
        &self,
        principal_id: &IdentityPrincipalId,
        at: &Timestamp,
    ) -> Vec<&AuthorityGrant> {
        self.authority_grants
            .values()
            .filter(|grant| {
                &grant.actor_principal_id == principal_id && self.grant_is_active_at(&grant.id, at)
            })
            .collect()
    }

    pub fn grant_is_active_at(&self, grant_id: &AuthorityGrantId, at: &Timestamp) -> bool {
        let Some(grant) = self.authority_grants.get(grant_id) else {
            return false;
        };

        &grant.issued_at <= at
            && self.principal_is_active_at(&grant.actor_principal_id, at)
            && grant
                .represented_organization_principal_id
                .as_ref()
                .is_none_or(|organization| {
                    self.principal_is_active_at(organization, at)
                        && self
                            .organization_id_for_principal(organization, at)
                            .is_some()
                })
            && validity_contains(grant.validity.as_ref(), at)
            && !self
                .authority_revocations
                .values()
                .any(|revocation| &revocation.grant_id == grant_id && &revocation.revoked_at <= at)
    }

    pub fn grant_is_revoked_at(&self, grant_id: &AuthorityGrantId, at: &Timestamp) -> bool {
        self.authority_revocations
            .values()
            .any(|revocation| &revocation.grant_id == grant_id && &revocation.revoked_at <= at)
    }

    pub fn sponsorships_for(&self, principal_id: &IdentityPrincipalId) -> Vec<&Sponsorship> {
        self.sponsorships
            .values()
            .filter(|sponsorship| match &sponsorship.sponsor {
                SponsoringParty::Individual(sponsor) | SponsoringParty::Organization(sponsor) => {
                    sponsor == principal_id
                }
            })
            .collect()
    }

    fn principal_is_active_at(&self, id: &IdentityPrincipalId, at: &Timestamp) -> bool {
        self.principals.get(id).is_some_and(|principal| {
            matches!(principal.status, IdentityPrincipalStatus::Active)
                && &principal.created_at <= at
        })
    }

    fn validate_access_decision(
        &self,
        decision: &AccessDecision,
    ) -> Result<(), IdentityProjectionError> {
        let account_principal = self
            .active_principal_for_account(&decision.account_id, &decision.evaluated_at)
            .ok_or_else(|| IdentityProjectionError::InvalidAccessDecision {
                id: decision.id.to_string(),
                reason: "account does not resolve to an active principal",
            })?;
        if account_principal.id != decision.actor_principal_id {
            return Err(IdentityProjectionError::InvalidAccessDecision {
                id: decision.id.to_string(),
                reason: "account and actor principal do not match",
            });
        }
        if let Some(organization) = &decision.represented_organization_principal_id {
            if !self.principal_is_active_at(organization, &decision.evaluated_at)
                || self
                    .organization_id_for_principal(organization, &decision.evaluated_at)
                    .is_none()
            {
                return Err(IdentityProjectionError::InvalidAccessDecision {
                    id: decision.id.to_string(),
                    reason: "represented organization is inactive or unlinked",
                });
            }
        }

        match &decision.authorization_basis {
            None => Ok(()),
            Some(AuthorizationBasis::AuthenticatedPrincipal)
                if decision.action == crate::AuthorizedAction::RegisterPublicAuditSubject
                    && decision.represented_organization_principal_id.is_none() =>
            {
                Ok(())
            }
            Some(AuthorizationBasis::PersonalCapacity)
                if decision.action == crate::AuthorizedAction::CommissionAudit
                    && decision.represented_organization_principal_id.is_none() =>
            {
                Ok(())
            }
            Some(
                AuthorizationBasis::AuthenticatedPrincipal | AuthorizationBasis::PersonalCapacity,
            ) => Err(IdentityProjectionError::InvalidAccessDecision {
                id: decision.id.to_string(),
                reason: "authorization basis does not support the action or representation",
            }),
            Some(AuthorizationBasis::PersonalSponsorship(sponsorship_id)) => {
                let sponsorship = self.sponsorships.get(sponsorship_id).ok_or_else(|| {
                    IdentityProjectionError::InvalidAccessDecision {
                        id: decision.id.to_string(),
                        reason: "personal sponsorship basis does not exist",
                    }
                })?;
                let valid = decision.action == crate::AuthorizedAction::ViewSponsoredAudit
                    && decision.represented_organization_principal_id.is_none()
                    && sponsorship.created_at <= decision.evaluated_at
                    && matches!(
                        (&sponsorship.sponsor, &decision.scope),
                        (
                            SponsoringParty::Individual(sponsor),
                            ResourceScope::AuditEpisode(episode_id)
                        ) if sponsor == &decision.actor_principal_id
                            && episode_id == &sponsorship.episode_id
                    );
                if valid {
                    Ok(())
                } else {
                    Err(IdentityProjectionError::InvalidAccessDecision {
                        id: decision.id.to_string(),
                        reason: "personal sponsorship basis does not match actor and resource",
                    })
                }
            }
            Some(AuthorizationBasis::AuthorityGrant(grant_id)) => {
                let grant = self.authority_grants.get(grant_id).ok_or_else(|| {
                    IdentityProjectionError::InvalidAccessDecision {
                        id: decision.id.to_string(),
                        reason: "authority grant basis does not exist",
                    }
                })?;
                let organization_matches = grant.kind == AuthorityKind::PlatformOperator
                    && matches!(grant.scope, ResourceScope::Platform)
                    || grant.represented_organization_principal_id
                        == decision.represented_organization_principal_id;
                let scope_matches = grant.scope == decision.scope
                    || grant.kind == AuthorityKind::PlatformOperator
                        && matches!(grant.scope, ResourceScope::Platform)
                    || self.organization_grant_supports_episode(
                        grant,
                        &decision.scope,
                        &decision.evaluated_at,
                    );
                if grant.actor_principal_id == decision.actor_principal_id
                    && grant.permitted_actions.contains(&decision.action)
                    && organization_matches
                    && scope_matches
                    && self.grant_is_active_at(grant_id, &decision.evaluated_at)
                {
                    Ok(())
                } else {
                    Err(IdentityProjectionError::InvalidAccessDecision {
                        id: decision.id.to_string(),
                        reason: "authority grant basis does not support the decision",
                    })
                }
            }
        }
    }

    fn organization_grant_supports_episode(
        &self,
        grant: &AuthorityGrant,
        resource: &ResourceScope,
        at: &Timestamp,
    ) -> bool {
        let (
            Some(represented_organization),
            ResourceScope::Organization(grant_organization_id),
            ResourceScope::AuditEpisode(episode_id),
        ) = (
            grant.represented_organization_principal_id.as_ref(),
            &grant.scope,
            resource,
        )
        else {
            return false;
        };
        self.organization_id_for_principal(represented_organization, at)
            == Some(grant_organization_id)
            && self
                .sponsorships_for_episode(episode_id, at)
                .iter()
                .any(|sponsorship| {
                    matches!(
                        &sponsorship.sponsor,
                        SponsoringParty::Organization(sponsor)
                            if sponsor == represented_organization
                    )
                })
    }

    fn apply(&mut self, event: &IdentityEvent) -> Result<(), IdentityProjectionError> {
        match &event.payload {
            IdentityEventPayload::PrincipalCreated { principal } => {
                insert_unique(
                    &mut self.principals,
                    principal.id.clone(),
                    principal.clone(),
                    "identity principal",
                )?;
            }
            IdentityEventPayload::PrincipalStatusChanged {
                principal_id,
                status,
            } => {
                validate_principal_supersession_target(
                    &self.principals,
                    principal_id,
                    status,
                    &event.recorded_at,
                )?;
                let principal = self.principals.get_mut(principal_id).ok_or_else(|| {
                    IdentityProjectionError::UnknownTarget {
                        entity: "identity principal",
                        id: principal_id.to_string(),
                    }
                })?;
                require_transition(
                    "identity principal",
                    principal_id.as_str(),
                    principal_status_label(&principal.status),
                    principal_status_label(status),
                    principal_transition_allowed(&principal.status, status),
                )?;
                principal.status = status.clone();
            }
            IdentityEventPayload::AccountPrincipalLinked { link } => {
                require_principal_kind(
                    &self.principals,
                    &link.principal_id,
                    IdentityPrincipalKind::Human,
                )?;
                if self.account_links.values().any(|existing| {
                    existing.account_id == link.account_id
                        && matches!(existing.status, LinkStatus::Active)
                }) {
                    return Err(IdentityProjectionError::ConflictingActiveAccountLink(
                        link.account_id.to_string(),
                    ));
                }
                insert_unique(
                    &mut self.account_links,
                    link.id.clone(),
                    link.clone(),
                    "account principal link",
                )?;
            }
            IdentityEventPayload::AccountPrincipalLinkStatusChanged { link_id, status } => {
                validate_link_supersession_target(
                    &self.account_links,
                    link_id,
                    status,
                    &event.recorded_at,
                )?;
                let link = self.account_links.get_mut(link_id).ok_or_else(|| {
                    IdentityProjectionError::UnknownTarget {
                        entity: "account principal link",
                        id: link_id.to_string(),
                    }
                })?;
                require_transition(
                    "account principal link",
                    link_id.as_str(),
                    link_status_label(&link.status),
                    link_status_label(status),
                    link_transition_allowed(&link.status, status),
                )?;
                link.status = status.clone();
            }
            IdentityEventPayload::OrganizationPrincipalLinked { link } => {
                require_principal_kind(
                    &self.principals,
                    &link.principal_id,
                    IdentityPrincipalKind::Organization,
                )?;
                if self
                    .organization_principal_links
                    .values()
                    .any(|existing| existing.principal_id == link.principal_id)
                {
                    return Err(
                        IdentityProjectionError::ConflictingOrganizationPrincipalLink {
                            organization_id: link.organization_id.to_string(),
                            principal_id: link.principal_id.to_string(),
                        },
                    );
                }
                insert_unique(
                    &mut self.organization_principal_links,
                    link.organization_id.clone(),
                    link.clone(),
                    "organization principal link",
                )?;
            }
            IdentityEventPayload::AuthenticationIdentityAdded { identity } => {
                insert_unique(
                    &mut self.authentication_identities,
                    identity.id.clone(),
                    identity.clone(),
                    "authentication identity",
                )?;
            }
            IdentityEventPayload::AuthenticationIdentityStatusChanged {
                identity_id,
                status,
            } => {
                validate_authentication_supersession_target(
                    &self.authentication_identities,
                    identity_id,
                    status,
                    &event.recorded_at,
                )?;
                let identity = self
                    .authentication_identities
                    .get_mut(identity_id)
                    .ok_or_else(|| IdentityProjectionError::UnknownTarget {
                        entity: "authentication identity",
                        id: identity_id.to_string(),
                    })?;
                require_transition(
                    "authentication identity",
                    identity_id.as_str(),
                    authentication_status_label(&identity.status),
                    authentication_status_label(status),
                    authentication_transition_allowed(&identity.status, status),
                )?;
                identity.status = status.clone();
            }
            IdentityEventPayload::AssertionRecorded { assertion } => {
                require_principal(&self.principals, &assertion.subject_principal_id)?;
                insert_unique(
                    &mut self.assertions,
                    assertion.id.clone(),
                    assertion.clone(),
                    "identity assertion",
                )?;
            }
            IdentityEventPayload::AssertionStatusChanged {
                assertion_id,
                status,
            } => {
                validate_assertion_supersession_target(
                    &self.assertions,
                    assertion_id,
                    status,
                    &event.recorded_at,
                )?;
                let assertion = self.assertions.get_mut(assertion_id).ok_or_else(|| {
                    IdentityProjectionError::UnknownTarget {
                        entity: "identity assertion",
                        id: assertion_id.to_string(),
                    }
                })?;
                require_transition(
                    "identity assertion",
                    assertion_id.as_str(),
                    assertion_status_label(&assertion.status),
                    assertion_status_label(status),
                    assertion_transition_allowed(&assertion.status, status),
                )?;
                assertion.status = status.clone();
            }
            IdentityEventPayload::OrganizationMembershipRecorded { membership } => {
                require_principal_kind(
                    &self.principals,
                    &membership.member_principal_id,
                    IdentityPrincipalKind::Human,
                )?;
                require_principal_kind(
                    &self.principals,
                    &membership.organization_principal_id,
                    IdentityPrincipalKind::Organization,
                )?;
                require_organization_principal_link(
                    &self.organization_principal_links,
                    &membership.organization_id,
                    &membership.organization_principal_id,
                )?;
                insert_unique(
                    &mut self.memberships,
                    membership.id.clone(),
                    membership.clone(),
                    "organization membership",
                )?;
            }
            IdentityEventPayload::OrganizationMembershipStatusChanged {
                membership_id,
                status,
            } => {
                validate_membership_supersession_target(
                    &self.memberships,
                    membership_id,
                    status,
                    &event.recorded_at,
                )?;
                let membership = self.memberships.get_mut(membership_id).ok_or_else(|| {
                    IdentityProjectionError::UnknownTarget {
                        entity: "organization membership",
                        id: membership_id.to_string(),
                    }
                })?;
                require_transition(
                    "organization membership",
                    membership_id.as_str(),
                    membership_status_label(&membership.status),
                    membership_status_label(status),
                    membership_transition_allowed(&membership.status, status),
                )?;
                membership.status = status.clone();
            }
            IdentityEventPayload::SponsorshipRecorded { sponsorship } => {
                require_principal_kind(
                    &self.principals,
                    &sponsorship.actor_principal_id,
                    IdentityPrincipalKind::Human,
                )?;
                match &sponsorship.sponsor {
                    SponsoringParty::Individual(sponsor) => require_principal_kind(
                        &self.principals,
                        sponsor,
                        IdentityPrincipalKind::Human,
                    )?,
                    SponsoringParty::Organization(sponsor) => require_principal_kind(
                        &self.principals,
                        sponsor,
                        IdentityPrincipalKind::Organization,
                    )?,
                }
                if let SponsoringParty::Organization(sponsor) = &sponsorship.sponsor {
                    require_organization_principal(&self.organization_principal_links, sponsor)?;
                }
                if let Some(grant_id) = &sponsorship.authority_grant_id {
                    let grant = self.authority_grants.get(grant_id).ok_or_else(|| {
                        IdentityProjectionError::UnknownTarget {
                            entity: "authority grant",
                            id: grant_id.to_string(),
                        }
                    })?;
                    if grant.actor_principal_id != sponsorship.actor_principal_id
                        || grant.represented_organization_principal_id
                            != sponsorship.represented_organization_principal_id
                    {
                        return Err(IdentityProjectionError::SponsorshipGrantMismatch(
                            sponsorship.id.to_string(),
                        ));
                    }
                    if !grant
                        .permitted_actions
                        .contains(&crate::AuthorizedAction::CommissionAudit)
                        || !self.grant_is_active_at(grant_id, &sponsorship.created_at)
                    {
                        return Err(IdentityProjectionError::SponsorshipGrantMismatch(
                            sponsorship.id.to_string(),
                        ));
                    }
                }
                insert_unique(
                    &mut self.sponsorships,
                    sponsorship.id.clone(),
                    sponsorship.clone(),
                    "sponsorship",
                )?;
            }
            IdentityEventPayload::AuthorityGranted { grant } => {
                require_principal(&self.principals, &grant.actor_principal_id)?;
                require_principal(&self.principals, &grant.issued_by_principal_id)?;
                if let Some(organization) = &grant.represented_organization_principal_id {
                    require_principal_kind(
                        &self.principals,
                        organization,
                        IdentityPrincipalKind::Organization,
                    )?;
                    let organization_id = require_organization_principal(
                        &self.organization_principal_links,
                        organization,
                    )?;
                    if let ResourceScope::Organization(scoped_organization_id) = &grant.scope {
                        if scoped_organization_id != organization_id {
                            return Err(IdentityProjectionError::GrantOrganizationScopeMismatch(
                                grant.id.to_string(),
                            ));
                        }
                    }
                } else if matches!(grant.scope, ResourceScope::Organization(_)) {
                    return Err(IdentityProjectionError::GrantOrganizationScopeMismatch(
                        grant.id.to_string(),
                    ));
                }
                insert_unique(
                    &mut self.authority_grants,
                    grant.id.clone(),
                    grant.clone(),
                    "authority grant",
                )?;
            }
            IdentityEventPayload::AuthorityRevoked { revocation } => {
                require_principal(&self.principals, &revocation.revoked_by_principal_id)?;
                if !self.authority_grants.contains_key(&revocation.grant_id) {
                    return Err(IdentityProjectionError::UnknownTarget {
                        entity: "authority grant",
                        id: revocation.grant_id.to_string(),
                    });
                }
                if self
                    .authority_revocations
                    .values()
                    .any(|existing| existing.grant_id == revocation.grant_id)
                {
                    return Err(IdentityProjectionError::GrantAlreadyRevoked(
                        revocation.grant_id.to_string(),
                    ));
                }
                insert_unique(
                    &mut self.authority_revocations,
                    revocation.id.clone(),
                    revocation.clone(),
                    "authority revocation",
                )?;
            }
            IdentityEventPayload::AccessDecisionRecorded { decision } => {
                require_principal(&self.principals, &decision.actor_principal_id)?;
                if decision.evaluated_at != event.recorded_at {
                    return Err(IdentityProjectionError::InvalidAccessDecision {
                        id: decision.id.to_string(),
                        reason: "decision evaluation time must equal its ledger recording time",
                    });
                }
                self.validate_access_decision(decision)?;
                insert_unique(
                    &mut self.access_decisions,
                    decision.id.clone(),
                    decision.clone(),
                    "access decision",
                )?;
            }
        }

        self.projected_through_sequence = Some(event.append_sequence);
        self.event_count += 1;

        Ok(())
    }
}

/// Replays every event in append-sequence order.
pub fn project_identity_state(
    events: &[IdentityEvent],
) -> Result<IdentityState, IdentityProjectionError> {
    replay(events, None)
}

/// Replays the prefix of the ledger recorded at or before `as_of`.
pub fn project_identity_state_at(
    events: &[IdentityEvent],
    as_of: Timestamp,
) -> Result<IdentityState, IdentityProjectionError> {
    replay(events, Some(as_of))
}

fn replay(
    events: &[IdentityEvent],
    as_of: Option<Timestamp>,
) -> Result<IdentityState, IdentityProjectionError> {
    let mut ordered = events.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|event| event.append_sequence);
    validate_ledger(&ordered)?;

    let mut state = IdentityState::empty(as_of);
    for event in ordered {
        if state
            .as_of
            .as_ref()
            .is_some_and(|as_of| &event.recorded_at > as_of)
        {
            continue;
        }
        state.apply(event)?;
    }

    Ok(state)
}

fn validate_ledger(events: &[&IdentityEvent]) -> Result<(), IdentityProjectionError> {
    let mut ids = BTreeSet::new();
    let mut sequences = BTreeSet::new();
    let mut previous: Option<&IdentityEvent> = None;

    for event in events {
        event
            .validate()
            .map_err(|source| IdentityProjectionError::InvalidEvent {
                event_id: event.id.clone(),
                source,
            })?;
        if !ids.insert(event.id.clone()) {
            return Err(IdentityProjectionError::DuplicateEventId(event.id.clone()));
        }
        if !sequences.insert(event.append_sequence) {
            return Err(IdentityProjectionError::DuplicateAppendSequence(
                event.append_sequence,
            ));
        }
        if let Some(previous) = previous {
            if event.recorded_at < previous.recorded_at {
                return Err(IdentityProjectionError::NonMonotonicRecordedAt {
                    previous_event_id: previous.id.clone(),
                    event_id: event.id.clone(),
                });
            }
        }
        previous = Some(event);
    }

    Ok(())
}

fn validity_contains(validity: Option<&crate::ValidityPeriod>, at: &Timestamp) -> bool {
    validity.is_none_or(|validity| validity.contains(at))
}

fn require_principal(
    principals: &BTreeMap<IdentityPrincipalId, IdentityPrincipal>,
    id: &IdentityPrincipalId,
) -> Result<(), IdentityProjectionError> {
    if principals.contains_key(id) {
        Ok(())
    } else {
        Err(IdentityProjectionError::UnknownTarget {
            entity: "identity principal",
            id: id.to_string(),
        })
    }
}

fn require_principal_kind(
    principals: &BTreeMap<IdentityPrincipalId, IdentityPrincipal>,
    id: &IdentityPrincipalId,
    expected: IdentityPrincipalKind,
) -> Result<(), IdentityProjectionError> {
    let principal = principals
        .get(id)
        .ok_or_else(|| IdentityProjectionError::UnknownTarget {
            entity: "identity principal",
            id: id.to_string(),
        })?;
    if principal.kind == expected {
        Ok(())
    } else {
        Err(IdentityProjectionError::PrincipalKindMismatch {
            id: id.to_string(),
            expected,
            actual: principal.kind,
        })
    }
}

fn require_organization_principal_link(
    links: &BTreeMap<OrganizationId, OrganizationPrincipalLink>,
    organization_id: &OrganizationId,
    principal_id: &IdentityPrincipalId,
) -> Result<(), IdentityProjectionError> {
    match links.get(organization_id) {
        Some(link) if &link.principal_id == principal_id => Ok(()),
        _ => Err(IdentityProjectionError::OrganizationPrincipalLinkMismatch {
            organization_id: organization_id.to_string(),
            principal_id: principal_id.to_string(),
        }),
    }
}

fn require_organization_principal<'a>(
    links: &'a BTreeMap<OrganizationId, OrganizationPrincipalLink>,
    principal_id: &IdentityPrincipalId,
) -> Result<&'a OrganizationId, IdentityProjectionError> {
    links
        .iter()
        .find_map(|(organization_id, link)| {
            (&link.principal_id == principal_id).then_some(organization_id)
        })
        .ok_or_else(
            || IdentityProjectionError::OrganizationPrincipalLinkMismatch {
                organization_id: "<unlinked>".to_string(),
                principal_id: principal_id.to_string(),
            },
        )
}

fn insert_unique<K, V>(
    values: &mut BTreeMap<K, V>,
    id: K,
    value: V,
    entity: &'static str,
) -> Result<(), IdentityProjectionError>
where
    K: Ord + ToString,
{
    if values.contains_key(&id) {
        return Err(IdentityProjectionError::DuplicateEntity {
            entity,
            id: id.to_string(),
        });
    }
    values.insert(id, value);

    Ok(())
}

fn require_transition(
    entity: &'static str,
    id: &str,
    from: &'static str,
    to: &'static str,
    allowed: bool,
) -> Result<(), IdentityProjectionError> {
    if allowed {
        Ok(())
    } else {
        Err(IdentityProjectionError::InvalidStatusTransition {
            entity,
            id: id.to_string(),
            from,
            to,
        })
    }
}

fn principal_transition_allowed(
    from: &IdentityPrincipalStatus,
    to: &IdentityPrincipalStatus,
) -> bool {
    match from {
        IdentityPrincipalStatus::Active => !matches!(to, IdentityPrincipalStatus::Active),
        IdentityPrincipalStatus::Disputed => !matches!(to, IdentityPrincipalStatus::Disputed),
        IdentityPrincipalStatus::Superseded { .. } | IdentityPrincipalStatus::Deactivated => false,
    }
}

fn link_transition_allowed(from: &LinkStatus, to: &LinkStatus) -> bool {
    match from {
        LinkStatus::Active => !matches!(to, LinkStatus::Active),
        LinkStatus::Disputed => !matches!(to, LinkStatus::Disputed),
        LinkStatus::Superseded { .. } | LinkStatus::Deactivated => false,
    }
}

fn authentication_transition_allowed(
    from: &AuthenticationIdentityStatus,
    to: &AuthenticationIdentityStatus,
) -> bool {
    matches!(from, AuthenticationIdentityStatus::Active)
        && !matches!(to, AuthenticationIdentityStatus::Active)
}

fn assertion_transition_allowed(from: &AssertionStatus, to: &AssertionStatus) -> bool {
    match from {
        AssertionStatus::Active => !matches!(to, AssertionStatus::Active),
        AssertionStatus::Disputed => !matches!(to, AssertionStatus::Disputed),
        AssertionStatus::Superseded { .. } | AssertionStatus::Revoked => false,
    }
}

fn membership_transition_allowed(
    from: &OrganizationMembershipStatus,
    to: &OrganizationMembershipStatus,
) -> bool {
    match from {
        OrganizationMembershipStatus::Invited => !matches!(
            to,
            OrganizationMembershipStatus::Invited | OrganizationMembershipStatus::Superseded { .. }
        ),
        OrganizationMembershipStatus::Active => !matches!(
            to,
            OrganizationMembershipStatus::Invited | OrganizationMembershipStatus::Active
        ),
        OrganizationMembershipStatus::Revoked
        | OrganizationMembershipStatus::Expired
        | OrganizationMembershipStatus::Superseded { .. } => false,
    }
}

fn validate_principal_supersession_target(
    values: &BTreeMap<IdentityPrincipalId, IdentityPrincipal>,
    current_id: &IdentityPrincipalId,
    status: &IdentityPrincipalStatus,
    at: &Timestamp,
) -> Result<(), IdentityProjectionError> {
    if let IdentityPrincipalStatus::Superseded { by, .. } = status {
        let current =
            values
                .get(current_id)
                .ok_or_else(|| IdentityProjectionError::UnknownTarget {
                    entity: "identity principal",
                    id: current_id.to_string(),
                })?;
        let target = values
            .get(by)
            .ok_or_else(|| IdentityProjectionError::UnknownTarget {
                entity: "identity principal",
                id: by.to_string(),
            })?;
        require_supersession(
            "identity principal",
            current_id.as_str(),
            by.as_str(),
            current_id != by
                && current.kind == target.kind
                && &target.created_at <= at
                && matches!(target.status, IdentityPrincipalStatus::Active),
            "replacement must be a distinct active principal of the same kind",
        )?;
    }
    Ok(())
}

fn validate_link_supersession_target(
    values: &BTreeMap<AccountPrincipalLinkId, AccountPrincipalLink>,
    current_id: &AccountPrincipalLinkId,
    status: &LinkStatus,
    at: &Timestamp,
) -> Result<(), IdentityProjectionError> {
    if let LinkStatus::Superseded { by } = status {
        let current = require_map_value(values, current_id, "account principal link")?;
        let target = require_map_value(values, by, "account principal link")?;
        require_supersession(
            "account principal link",
            current_id.as_str(),
            by.as_str(),
            current_id != by
                && current.account_id == target.account_id
                && &target.established_at <= at
                && matches!(target.status, LinkStatus::Active),
            "replacement must be a distinct active link for the same account",
        )?;
    }
    Ok(())
}

fn validate_authentication_supersession_target(
    values: &BTreeMap<AuthenticationIdentityId, AuthenticationIdentity>,
    current_id: &AuthenticationIdentityId,
    status: &AuthenticationIdentityStatus,
    at: &Timestamp,
) -> Result<(), IdentityProjectionError> {
    if let AuthenticationIdentityStatus::Superseded { by } = status {
        let current = require_map_value(values, current_id, "authentication identity")?;
        let target = require_map_value(values, by, "authentication identity")?;
        require_supersession(
            "authentication identity",
            current_id.as_str(),
            by.as_str(),
            current_id != by
                && current.account_id == target.account_id
                && std::mem::discriminant(&current.kind) == std::mem::discriminant(&target.kind)
                && &target.established_at <= at
                && matches!(target.status, AuthenticationIdentityStatus::Active),
            "replacement must be a distinct active identity of the same mechanism for the same account",
        )?;
    }
    Ok(())
}

fn validate_assertion_supersession_target(
    values: &BTreeMap<IdentityAssertionId, IdentityAssertion>,
    current_id: &IdentityAssertionId,
    status: &AssertionStatus,
    at: &Timestamp,
) -> Result<(), IdentityProjectionError> {
    if let AssertionStatus::Superseded { by } = status {
        let current = require_map_value(values, current_id, "identity assertion")?;
        let target = require_map_value(values, by, "identity assertion")?;
        require_supersession(
            "identity assertion",
            current_id.as_str(),
            by.as_str(),
            current_id != by
                && current.subject_principal_id == target.subject_principal_id
                && std::mem::discriminant(&current.kind) == std::mem::discriminant(&target.kind)
                && &target.asserted_at <= at
                && matches!(target.status, AssertionStatus::Active),
            "replacement must be a distinct active assertion of the same kind for the same subject",
        )?;
    }
    Ok(())
}

fn validate_membership_supersession_target(
    values: &BTreeMap<OrganizationMembershipId, OrganizationMembership>,
    current_id: &OrganizationMembershipId,
    status: &OrganizationMembershipStatus,
    at: &Timestamp,
) -> Result<(), IdentityProjectionError> {
    if let OrganizationMembershipStatus::Superseded { by } = status {
        let current = require_map_value(values, current_id, "organization membership")?;
        let target = require_map_value(values, by, "organization membership")?;
        require_supersession(
            "organization membership",
            current_id.as_str(),
            by.as_str(),
            current_id != by
                && current.member_principal_id == target.member_principal_id
                && current.organization_principal_id == target.organization_principal_id
                && current.organization_id == target.organization_id
                && &target.asserted_at <= at
                && matches!(target.status, OrganizationMembershipStatus::Active),
            "replacement must be a distinct active membership for the same member and organization",
        )?;
    }
    Ok(())
}

fn require_map_value<'a, K, V>(
    values: &'a BTreeMap<K, V>,
    id: &K,
    entity: &'static str,
) -> Result<&'a V, IdentityProjectionError>
where
    K: Ord + ToString,
{
    values
        .get(id)
        .ok_or_else(|| IdentityProjectionError::UnknownTarget {
            entity,
            id: id.to_string(),
        })
}

fn require_supersession(
    entity: &'static str,
    id: &str,
    target_id: &str,
    allowed: bool,
    reason: &'static str,
) -> Result<(), IdentityProjectionError> {
    if allowed {
        Ok(())
    } else {
        Err(IdentityProjectionError::InvalidSupersessionTarget {
            entity,
            id: id.to_string(),
            target_id: target_id.to_string(),
            reason,
        })
    }
}

fn principal_status_label(status: &IdentityPrincipalStatus) -> &'static str {
    match status {
        IdentityPrincipalStatus::Active => "active",
        IdentityPrincipalStatus::Disputed => "disputed",
        IdentityPrincipalStatus::Superseded { .. } => "superseded",
        IdentityPrincipalStatus::Deactivated => "deactivated",
    }
}

fn link_status_label(status: &LinkStatus) -> &'static str {
    match status {
        LinkStatus::Active => "active",
        LinkStatus::Disputed => "disputed",
        LinkStatus::Superseded { .. } => "superseded",
        LinkStatus::Deactivated => "deactivated",
    }
}

fn authentication_status_label(status: &AuthenticationIdentityStatus) -> &'static str {
    match status {
        AuthenticationIdentityStatus::Active => "active",
        AuthenticationIdentityStatus::Revoked => "revoked",
        AuthenticationIdentityStatus::Superseded { .. } => "superseded",
    }
}

fn assertion_status_label(status: &AssertionStatus) -> &'static str {
    match status {
        AssertionStatus::Active => "active",
        AssertionStatus::Disputed => "disputed",
        AssertionStatus::Superseded { .. } => "superseded",
        AssertionStatus::Revoked => "revoked",
    }
}

fn membership_status_label(status: &OrganizationMembershipStatus) -> &'static str {
    match status {
        OrganizationMembershipStatus::Invited => "invited",
        OrganizationMembershipStatus::Active => "active",
        OrganizationMembershipStatus::Revoked => "revoked",
        OrganizationMembershipStatus::Expired => "expired",
        OrganizationMembershipStatus::Superseded { .. } => "superseded",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityProjectionError {
    InvalidEvent {
        event_id: IdentityEventId,
        source: IdentityEventValidationError,
    },
    DuplicateEventId(IdentityEventId),
    DuplicateAppendSequence(u64),
    NonMonotonicRecordedAt {
        previous_event_id: IdentityEventId,
        event_id: IdentityEventId,
    },
    DuplicateEntity {
        entity: &'static str,
        id: String,
    },
    UnknownTarget {
        entity: &'static str,
        id: String,
    },
    PrincipalKindMismatch {
        id: String,
        expected: IdentityPrincipalKind,
        actual: IdentityPrincipalKind,
    },
    ConflictingActiveAccountLink(String),
    ConflictingOrganizationPrincipalLink {
        organization_id: String,
        principal_id: String,
    },
    OrganizationPrincipalLinkMismatch {
        organization_id: String,
        principal_id: String,
    },
    GrantOrganizationScopeMismatch(String),
    SponsorshipGrantMismatch(String),
    InvalidAccessDecision {
        id: String,
        reason: &'static str,
    },
    InvalidSupersessionTarget {
        entity: &'static str,
        id: String,
        target_id: String,
        reason: &'static str,
    },
    InvalidStatusTransition {
        entity: &'static str,
        id: String,
        from: &'static str,
        to: &'static str,
    },
    GrantAlreadyRevoked(String),
}

impl fmt::Display for IdentityProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEvent { event_id, source } => {
                write!(formatter, "invalid event {event_id}: {source}")
            }
            Self::DuplicateEventId(id) => write!(formatter, "duplicate identity event id: {id}"),
            Self::DuplicateAppendSequence(sequence) => {
                write!(formatter, "duplicate identity append sequence: {sequence}")
            }
            Self::NonMonotonicRecordedAt {
                previous_event_id,
                event_id,
            } => write!(
                formatter,
                "identity event {event_id} was recorded before prior event {previous_event_id}"
            ),
            Self::DuplicateEntity { entity, id } => {
                write!(formatter, "duplicate {entity} id: {id}")
            }
            Self::UnknownTarget { entity, id } => write!(formatter, "unknown {entity}: {id}"),
            Self::PrincipalKindMismatch {
                id,
                expected,
                actual,
            } => write!(
                formatter,
                "identity principal {id} has kind {actual:?}, expected {expected:?}"
            ),
            Self::ConflictingActiveAccountLink(account_id) => {
                write!(
                    formatter,
                    "account {account_id} already has an active principal link"
                )
            }
            Self::ConflictingOrganizationPrincipalLink {
                organization_id,
                principal_id,
            } => write!(
                formatter,
                "organization {organization_id} or principal {principal_id} is already linked"
            ),
            Self::OrganizationPrincipalLinkMismatch {
                organization_id,
                principal_id,
            } => write!(
                formatter,
                "organization {organization_id} is not linked to principal {principal_id}"
            ),
            Self::GrantOrganizationScopeMismatch(id) => write!(
                formatter,
                "authority grant {id} has an inconsistent organization scope"
            ),
            Self::SponsorshipGrantMismatch(id) => write!(
                formatter,
                "sponsorship {id} does not match its authority grant actor and organization"
            ),
            Self::InvalidAccessDecision { id, reason } => {
                write!(formatter, "invalid access decision {id}: {reason}")
            }
            Self::InvalidSupersessionTarget {
                entity,
                id,
                target_id,
                reason,
            } => write!(
                formatter,
                "invalid {entity} supersession for {id} -> {target_id}: {reason}"
            ),
            Self::InvalidStatusTransition {
                entity,
                id,
                from,
                to,
            } => write!(
                formatter,
                "invalid {entity} status transition for {id}: {from} -> {to}"
            ),
            Self::GrantAlreadyRevoked(id) => {
                write!(formatter, "authority grant {id} is already revoked")
            }
        }
    }
}

impl std::error::Error for IdentityProjectionError {}
