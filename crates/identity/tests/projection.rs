use chrono::{TimeZone, Utc};
use csqd_domain::{
    AccessDecisionId, AccountPrincipalLinkId, AuthorityGrantId, AuthorityRevocationId,
    IdentityAssertionId, IdentityEventId, IdentityPrincipalId, OrganizationId,
    OrganizationMembershipId, PolicyId, Principal, Timestamp, UserId,
};
use csqd_identity::{
    project_identity_state, project_identity_state_at, AccessDecisionResult, AccountPrincipalLink,
    AssertionStatus, AssuranceLevel, AuthenticationMethod, AuthorityGrant, AuthorityKind,
    AuthorityRevocation, AuthorizationBasis, AuthorizationRequest, AuthorizedAction,
    IdentityAssertion, IdentityAssertionKind, IdentityEvent, IdentityEventPayload,
    IdentityPrincipal, IdentityPrincipalKind, IdentityProjectionError, LinkStatus,
    NewAccessDecision, NewAuthorityGrant, NewIdentityAssertion, NewOrganizationMembership,
    NewOrganizationSponsorship, OrganizationMembership, OrganizationMembershipRole,
    OrganizationMembershipStatus, OrganizationPrincipalLink, PolicyReasonCode, ResourceScope,
    SponsorVisibility, Sponsorship, ValidityPeriod,
};

fn at(hour: u32) -> Timestamp {
    Utc.with_ymd_and_hms(2026, 7, 26, hour, 0, 0)
        .single()
        .unwrap()
}

fn event(sequence: u64, recorded_at: Timestamp, payload: IdentityEventPayload) -> IdentityEvent {
    IdentityEvent::new(
        IdentityEventId::new(format!("event-{sequence}")),
        sequence,
        recorded_at,
        Principal::Platform,
        payload,
    )
    .unwrap()
}

fn principal_event(sequence: u64, id: &str, kind: IdentityPrincipalKind) -> IdentityEvent {
    let principal = IdentityPrincipal::new(
        IdentityPrincipalId::new(id),
        kind,
        format!("{id} display name"),
        at(0),
        Principal::Platform,
    )
    .unwrap();

    event(
        sequence,
        at(0),
        IdentityEventPayload::PrincipalCreated { principal },
    )
}

#[test]
fn grant_replay_distinguishes_revocation_and_expiration() {
    let actor_id = IdentityPrincipalId::new("actor");
    let issuer_id = IdentityPrincipalId::new("issuer");
    let revocable_grant_id = AuthorityGrantId::new("grant-revocable");
    let expiring_grant_id = AuthorityGrantId::new("grant-expiring");
    let revocable_grant = AuthorityGrant::new(NewAuthorityGrant {
        id: revocable_grant_id.clone(),
        actor_principal_id: actor_id.clone(),
        represented_organization_principal_id: None,
        kind: AuthorityKind::EpisodeReviewer,
        scope: ResourceScope::Platform,
        permitted_actions: vec![AuthorizedAction::AcceptReviewAssignment],
        issued_by_principal_id: issuer_id.clone(),
        issued_at: at(1),
        validity: Some(ValidityPeriod::new(at(1), Some(at(5))).unwrap()),
        evidence_refs: vec!["reviewer-agreement-1".into()],
    })
    .unwrap();
    let expiring_grant = AuthorityGrant::new(NewAuthorityGrant {
        id: expiring_grant_id.clone(),
        actor_principal_id: actor_id,
        represented_organization_principal_id: None,
        kind: AuthorityKind::Observer,
        scope: ResourceScope::Platform,
        permitted_actions: vec![AuthorizedAction::ViewSponsoredAudit],
        issued_by_principal_id: issuer_id.clone(),
        issued_at: at(1),
        validity: Some(ValidityPeriod::new(at(1), Some(at(4))).unwrap()),
        evidence_refs: vec![],
    })
    .unwrap();
    let revocation = AuthorityRevocation::new(
        AuthorityRevocationId::new("revocation-1"),
        revocable_grant_id.clone(),
        issuer_id,
        at(3),
        "assignment withdrawn",
    )
    .unwrap();
    let events = vec![
        principal_event(1, "actor", IdentityPrincipalKind::Human),
        principal_event(2, "issuer", IdentityPrincipalKind::Human),
        event(
            3,
            at(1),
            IdentityEventPayload::AuthorityGranted {
                grant: revocable_grant,
            },
        ),
        event(
            4,
            at(1),
            IdentityEventPayload::AuthorityGranted {
                grant: expiring_grant,
            },
        ),
        event(
            5,
            at(3),
            IdentityEventPayload::AuthorityRevoked { revocation },
        ),
    ];

    let current = project_identity_state(&events).unwrap();
    assert!(current.grant_is_active_at(&revocable_grant_id, &at(2)));
    assert!(!current.grant_is_active_at(&revocable_grant_id, &at(3)));
    assert!(current.grant_is_active_at(&expiring_grant_id, &at(3)));
    assert!(!current.grant_is_active_at(&expiring_grant_id, &at(4)));

    let before_revocation = project_identity_state_at(&events, at(2)).unwrap();
    assert_eq!(before_revocation.projected_through_sequence(), Some(4));
    assert!(before_revocation.grant_is_active_at(&revocable_grant_id, &at(2)));
}

#[test]
fn superseded_assertion_has_a_replayable_history() {
    let principal_id = IdentityPrincipalId::new("reviewer");
    let first_id = IdentityAssertionId::new("assertion-1");
    let replacement_id = IdentityAssertionId::new("assertion-2");
    let first = IdentityAssertion::new(NewIdentityAssertion {
        id: first_id.clone(),
        subject_principal_id: principal_id.clone(),
        kind: IdentityAssertionKind::ReviewerExpertise {
            label: "causal inference".into(),
        },
        assurance: AssuranceLevel::Medium,
        asserted_by: Principal::Platform,
        asserted_at: at(1),
        validity: None,
        evidence_refs: vec!["cv-v1".into()],
    })
    .unwrap();
    let replacement = IdentityAssertion::new(NewIdentityAssertion {
        id: replacement_id.clone(),
        subject_principal_id: principal_id.clone(),
        kind: IdentityAssertionKind::ReviewerExpertise {
            label: "causal inference".into(),
        },
        assurance: AssuranceLevel::High,
        asserted_by: Principal::Platform,
        asserted_at: at(2),
        validity: None,
        evidence_refs: vec!["cv-v2".into()],
    })
    .unwrap();
    let events = vec![
        principal_event(1, "reviewer", IdentityPrincipalKind::Human),
        event(
            2,
            at(1),
            IdentityEventPayload::AssertionRecorded { assertion: first },
        ),
        event(
            3,
            at(2),
            IdentityEventPayload::AssertionRecorded {
                assertion: replacement,
            },
        ),
        event(
            4,
            at(3),
            IdentityEventPayload::AssertionStatusChanged {
                assertion_id: first_id.clone(),
                status: AssertionStatus::Superseded {
                    by: replacement_id.clone(),
                },
            },
        ),
    ];

    let historical = project_identity_state_at(&events, at(1)).unwrap();
    assert_eq!(
        historical.active_assertions_for(&principal_id, &at(1))[0].id,
        first_id
    );

    let current = project_identity_state(&events).unwrap();
    assert!(matches!(
        current.assertion(&first_id).unwrap().status,
        AssertionStatus::Superseded { ref by } if by == &replacement_id
    ));
    assert_eq!(
        current.active_assertions_for(&principal_id, &at(3))[0].id,
        replacement_id
    );
}

#[test]
fn disputed_account_link_stops_authenticating_the_principal() {
    let account_id = UserId::new("account-1");
    let principal_id = IdentityPrincipalId::new("human-1");
    let link_id = AccountPrincipalLinkId::new("link-1");
    let link = AccountPrincipalLink {
        id: link_id.clone(),
        account_id: account_id.clone(),
        principal_id: principal_id.clone(),
        status: LinkStatus::Active,
        established_by: Principal::Platform,
        established_at: at(1),
    };
    let events = vec![
        principal_event(1, "human-1", IdentityPrincipalKind::Human),
        event(
            2,
            at(1),
            IdentityEventPayload::AccountPrincipalLinked { link },
        ),
        event(
            3,
            at(2),
            IdentityEventPayload::AccountPrincipalLinkStatusChanged {
                link_id,
                status: LinkStatus::Disputed,
            },
        ),
    ];

    let historical = project_identity_state_at(&events, at(1)).unwrap();
    assert_eq!(
        historical
            .active_principal_for_account(&account_id, &at(1))
            .unwrap()
            .id,
        principal_id
    );

    let current = project_identity_state(&events).unwrap();
    assert!(current
        .active_principal_for_account(&account_id, &at(2))
        .is_none());
}

#[test]
fn membership_queries_respect_distinct_validity_windows() {
    let member_id = IdentityPrincipalId::new("member");
    let first_organization_id = IdentityPrincipalId::new("organization-1");
    let second_organization_id = IdentityPrincipalId::new("organization-2");
    let first_membership = OrganizationMembership::new(NewOrganizationMembership {
        id: OrganizationMembershipId::new("membership-1"),
        member_principal_id: member_id.clone(),
        organization_principal_id: first_organization_id.clone(),
        organization_id: OrganizationId::new("organization-record-1"),
        role: OrganizationMembershipRole::ReviewerAffiliate,
        assurance: AssuranceLevel::High,
        status: OrganizationMembershipStatus::Active,
        validity: Some(ValidityPeriod::new(at(1), Some(at(3))).unwrap()),
        asserted_by: Principal::Platform,
        asserted_at: at(1),
    })
    .unwrap();
    let second_membership = OrganizationMembership::new(NewOrganizationMembership {
        id: OrganizationMembershipId::new("membership-2"),
        member_principal_id: member_id.clone(),
        organization_principal_id: second_organization_id.clone(),
        organization_id: OrganizationId::new("organization-record-2"),
        role: OrganizationMembershipRole::ReviewerAffiliate,
        assurance: AssuranceLevel::High,
        status: OrganizationMembershipStatus::Active,
        validity: Some(ValidityPeriod::new(at(3), Some(at(5))).unwrap()),
        asserted_by: Principal::Platform,
        asserted_at: at(1),
    })
    .unwrap();
    let events = vec![
        principal_event(1, "member", IdentityPrincipalKind::Human),
        principal_event(2, "organization-1", IdentityPrincipalKind::Organization),
        principal_event(3, "organization-2", IdentityPrincipalKind::Organization),
        event(
            4,
            at(1),
            IdentityEventPayload::OrganizationPrincipalLinked {
                link: OrganizationPrincipalLink {
                    organization_id: OrganizationId::new("organization-record-1"),
                    principal_id: first_organization_id.clone(),
                    established_by: Principal::Platform,
                    established_at: at(1),
                },
            },
        ),
        event(
            5,
            at(1),
            IdentityEventPayload::OrganizationPrincipalLinked {
                link: OrganizationPrincipalLink {
                    organization_id: OrganizationId::new("organization-record-2"),
                    principal_id: second_organization_id.clone(),
                    established_by: Principal::Platform,
                    established_at: at(1),
                },
            },
        ),
        event(
            6,
            at(1),
            IdentityEventPayload::OrganizationMembershipRecorded {
                membership: first_membership,
            },
        ),
        event(
            7,
            at(1),
            IdentityEventPayload::OrganizationMembershipRecorded {
                membership: second_membership,
            },
        ),
    ];

    let state = project_identity_state(&events).unwrap();
    assert_eq!(
        state.active_memberships_for(&member_id, &at(2))[0].organization_principal_id,
        first_organization_id
    );
    assert_eq!(
        state.active_memberships_for(&member_id, &at(4))[0].organization_principal_id,
        second_organization_id
    );
    assert!(state.active_memberships_for(&member_id, &at(5)).is_empty());
}

#[test]
fn replay_is_deterministic_for_shuffled_input() {
    let ordered = vec![
        principal_event(1, "principal-1", IdentityPrincipalKind::Human),
        principal_event(2, "principal-2", IdentityPrincipalKind::Organization),
        principal_event(3, "principal-3", IdentityPrincipalKind::SystemAgent),
    ];
    let shuffled = vec![ordered[2].clone(), ordered[0].clone(), ordered[1].clone()];

    assert_eq!(
        project_identity_state(&ordered).unwrap(),
        project_identity_state(&shuffled).unwrap()
    );
}

#[test]
fn replay_rejects_duplicate_event_identity_and_sequence() {
    let first = principal_event(1, "principal-1", IdentityPrincipalKind::Human);
    let mut duplicate_id = principal_event(2, "principal-2", IdentityPrincipalKind::Human);
    duplicate_id.id = first.id.clone();
    assert!(matches!(
        project_identity_state(&[first.clone(), duplicate_id]),
        Err(IdentityProjectionError::DuplicateEventId(_))
    ));

    let mut duplicate_sequence =
        principal_event(1, "principal-3", IdentityPrincipalKind::Organization);
    duplicate_sequence.id = IdentityEventId::new("event-duplicate-sequence");
    assert_eq!(
        project_identity_state(&[first, duplicate_sequence]),
        Err(IdentityProjectionError::DuplicateAppendSequence(1))
    );
}

#[test]
fn replay_rejects_unknown_targets_and_non_monotonic_timestamps() {
    let unknown_target = event(
        1,
        at(1),
        IdentityEventPayload::PrincipalStatusChanged {
            principal_id: IdentityPrincipalId::new("missing"),
            status: csqd_identity::IdentityPrincipalStatus::Disputed,
        },
    );
    assert!(matches!(
        project_identity_state(&[unknown_target]),
        Err(IdentityProjectionError::UnknownTarget {
            entity: "identity principal",
            ..
        })
    ));

    let first = principal_event(1, "principal-1", IdentityPrincipalKind::Human);
    let second_principal = IdentityPrincipal::new(
        IdentityPrincipalId::new("principal-2"),
        IdentityPrincipalKind::Human,
        "Principal 2",
        at(0),
        Principal::Platform,
    )
    .unwrap();
    let second = event(
        2,
        at(0),
        IdentityEventPayload::PrincipalCreated {
            principal: second_principal,
        },
    );
    let mut late_first = first;
    late_first.recorded_at = at(1);

    assert!(matches!(
        project_identity_state(&[late_first, second]),
        Err(IdentityProjectionError::NonMonotonicRecordedAt { .. })
    ));
}

#[test]
fn future_issued_grant_is_inactive_until_issuance() {
    let grant_id = AuthorityGrantId::new("future-grant");
    let grant = AuthorityGrant::new(NewAuthorityGrant {
        id: grant_id.clone(),
        actor_principal_id: IdentityPrincipalId::new("actor"),
        represented_organization_principal_id: None,
        kind: AuthorityKind::EpisodeReviewer,
        scope: ResourceScope::AuditEpisode(csqd_domain::AuditEpisodeId::new("episode")),
        permitted_actions: vec![AuthorizedAction::SubmitElementReview],
        issued_by_principal_id: IdentityPrincipalId::new("issuer"),
        issued_at: at(5),
        validity: None,
        evidence_refs: vec![],
    })
    .unwrap();
    let events = vec![
        principal_event(1, "actor", IdentityPrincipalKind::Human),
        principal_event(2, "issuer", IdentityPrincipalKind::Human),
        event(3, at(1), IdentityEventPayload::AuthorityGranted { grant }),
    ];

    let state = project_identity_state(&events).unwrap();
    assert!(!state.grant_is_active_at(&grant_id, &at(4)));
    assert!(state.grant_is_active_at(&grant_id, &at(5)));
}

#[test]
fn organization_grant_scope_must_match_the_linked_business_record() {
    let grant = AuthorityGrant::new(NewAuthorityGrant {
        id: AuthorityGrantId::new("cross-organization-grant"),
        actor_principal_id: IdentityPrincipalId::new("actor"),
        represented_organization_principal_id: Some(IdentityPrincipalId::new("organization")),
        kind: AuthorityKind::SponsorRepresentative,
        scope: ResourceScope::Organization(OrganizationId::new("wrong-organization-record")),
        permitted_actions: vec![AuthorizedAction::ViewSponsoredAudit],
        issued_by_principal_id: IdentityPrincipalId::new("issuer"),
        issued_at: at(1),
        validity: None,
        evidence_refs: vec![],
    })
    .unwrap();
    let events = vec![
        principal_event(1, "actor", IdentityPrincipalKind::Human),
        principal_event(2, "issuer", IdentityPrincipalKind::Human),
        principal_event(3, "organization", IdentityPrincipalKind::Organization),
        event(
            4,
            at(1),
            IdentityEventPayload::OrganizationPrincipalLinked {
                link: OrganizationPrincipalLink {
                    organization_id: OrganizationId::new("organization-record"),
                    principal_id: IdentityPrincipalId::new("organization"),
                    established_by: Principal::Platform,
                    established_at: at(1),
                },
            },
        ),
        event(5, at(1), IdentityEventPayload::AuthorityGranted { grant }),
    ];

    assert!(matches!(
        project_identity_state(&events),
        Err(IdentityProjectionError::GrantOrganizationScopeMismatch(id))
            if id == "cross-organization-grant"
    ));
}

#[test]
fn organization_sponsorship_requires_the_exact_commissioning_grant_semantics() {
    let grant = AuthorityGrant::new(NewAuthorityGrant {
        id: AuthorityGrantId::new("reviewer-shaped-commission-grant"),
        actor_principal_id: IdentityPrincipalId::new("actor"),
        represented_organization_principal_id: Some(IdentityPrincipalId::new("organization")),
        kind: AuthorityKind::EpisodeReviewer,
        scope: ResourceScope::Organization(OrganizationId::new("organization-record")),
        permitted_actions: vec![AuthorizedAction::CommissionAudit],
        issued_by_principal_id: IdentityPrincipalId::new("issuer"),
        issued_at: at(1),
        validity: None,
        evidence_refs: vec![],
    })
    .unwrap();
    let sponsorship = Sponsorship::organization(NewOrganizationSponsorship {
        id: csqd_domain::SponsorshipId::new("invalid-organization-sponsorship"),
        episode_id: csqd_domain::AuditEpisodeId::new("episode"),
        organization_principal_id: IdentityPrincipalId::new("organization"),
        actor_principal_id: IdentityPrincipalId::new("actor"),
        authority_grant_id: AuthorityGrantId::new("reviewer-shaped-commission-grant"),
        visibility: SponsorVisibility::Named,
        created_at: at(1),
    })
    .unwrap();
    let events = vec![
        principal_event(1, "actor", IdentityPrincipalKind::Human),
        principal_event(2, "issuer", IdentityPrincipalKind::Human),
        principal_event(3, "organization", IdentityPrincipalKind::Organization),
        event(
            4,
            at(0),
            IdentityEventPayload::OrganizationPrincipalLinked {
                link: OrganizationPrincipalLink {
                    organization_id: OrganizationId::new("organization-record"),
                    principal_id: IdentityPrincipalId::new("organization"),
                    established_by: Principal::Platform,
                    established_at: at(0),
                },
            },
        ),
        event(5, at(1), IdentityEventPayload::AuthorityGranted { grant }),
        event(
            6,
            at(1),
            IdentityEventPayload::SponsorshipRecorded { sponsorship },
        ),
    ];

    assert!(matches!(
        project_identity_state(&events),
        Err(IdentityProjectionError::SponsorshipGrantMismatch(id))
            if id == "invalid-organization-sponsorship"
    ));
}

#[test]
fn supersession_requires_semantic_lineage() {
    let first = IdentityAssertion::new(NewIdentityAssertion {
        id: IdentityAssertionId::new("assertion-1"),
        subject_principal_id: IdentityPrincipalId::new("subject-1"),
        kind: IdentityAssertionKind::ReviewerExpertise {
            label: "causal inference".into(),
        },
        assurance: AssuranceLevel::Medium,
        asserted_by: Principal::Platform,
        asserted_at: at(1),
        validity: None,
        evidence_refs: vec![],
    })
    .unwrap();
    let unrelated = IdentityAssertion::new(NewIdentityAssertion {
        id: IdentityAssertionId::new("assertion-2"),
        subject_principal_id: IdentityPrincipalId::new("subject-2"),
        kind: IdentityAssertionKind::ReviewerExpertise {
            label: "causal inference".into(),
        },
        assurance: AssuranceLevel::High,
        asserted_by: Principal::Platform,
        asserted_at: at(2),
        validity: None,
        evidence_refs: vec![],
    })
    .unwrap();
    let events = vec![
        principal_event(1, "subject-1", IdentityPrincipalKind::Human),
        principal_event(2, "subject-2", IdentityPrincipalKind::Human),
        event(
            3,
            at(1),
            IdentityEventPayload::AssertionRecorded { assertion: first },
        ),
        event(
            4,
            at(2),
            IdentityEventPayload::AssertionRecorded {
                assertion: unrelated,
            },
        ),
        event(
            5,
            at(3),
            IdentityEventPayload::AssertionStatusChanged {
                assertion_id: IdentityAssertionId::new("assertion-1"),
                status: AssertionStatus::Superseded {
                    by: IdentityAssertionId::new("assertion-2"),
                },
            },
        ),
    ];

    assert!(matches!(
        project_identity_state(&events),
        Err(IdentityProjectionError::InvalidSupersessionTarget {
            entity: "identity assertion",
            ..
        })
    ));
}

#[test]
fn access_decision_rejects_a_nonexistent_authority_basis() {
    let decision = csqd_identity::AccessDecision::new(NewAccessDecision {
        id: AccessDecisionId::new("decision"),
        account_id: UserId::new("account"),
        actor_reference: csqd_identity::AuditedPrincipalReference::Known(IdentityPrincipalId::new(
            "actor",
        )),
        representation: csqd_identity::AuditedRepresentation::None,
        authentication_method: AuthenticationMethod::MagicLink,
        authentication_assurance: AssuranceLevel::Medium,
        authenticated_at: at(0),
        request: AuthorizationRequest::Access {
            action: AuthorizedAction::SubmitElementReview,
            resource: ResourceScope::AuditEpisode(csqd_domain::AuditEpisodeId::new("episode")),
        },
        result: AccessDecisionResult::Allowed {
            basis: AuthorizationBasis::AuthorityGrant(AuthorityGrantId::new("missing-grant")),
            reason: PolicyReasonCode::AllowedAuthorityGrant,
        },
        policy_id: PolicyId::new("policy"),
        evaluated_at: at(1),
    })
    .unwrap();
    let events = vec![
        principal_event(1, "actor", IdentityPrincipalKind::Human),
        event(
            2,
            at(0),
            IdentityEventPayload::AccountPrincipalLinked {
                link: AccountPrincipalLink {
                    id: AccountPrincipalLinkId::new("link"),
                    account_id: UserId::new("account"),
                    principal_id: IdentityPrincipalId::new("actor"),
                    status: csqd_identity::LinkStatus::Active,
                    established_by: Principal::Platform,
                    established_at: at(0),
                },
            },
        ),
        event(
            3,
            at(1),
            IdentityEventPayload::AccessDecisionRecorded { decision },
        ),
    ];

    assert!(matches!(
        project_identity_state(&events),
        Err(IdentityProjectionError::InvalidAccessDecision {
            id,
            reason: "authority grant basis does not exist"
        }) if id == "decision"
    ));
}
