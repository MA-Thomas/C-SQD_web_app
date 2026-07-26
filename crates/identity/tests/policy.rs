use chrono::{TimeZone, Utc};
use csqd_domain::{
    AccessDecisionId, AuditEpisodeId, AuditSubjectId, AuthorityGrantId, AuthorityRevocationId,
    DomainInstantiationId, IdentityEventId, IdentityPrincipalId, OrganizationId, PolicyId,
    Principal, SponsorshipId, Timestamp, UserId,
};
use csqd_identity::{
    evaluate_access, project_identity_state, AccountPrincipalLink, AssuranceLevel,
    AuthenticationMethod, AuthorityGrant, AuthorityKind, AuthorityMutationTarget,
    AuthorityRevocation, AuthorizationBasis, AuthorizationContext, AuthorizationOutcome,
    AuthorizedAction, ConflictStatus, IdentityEvent, IdentityEventPayload, IdentityPrincipal,
    IdentityPrincipalKind, InitialPolicyConfiguration, LinkStatus, NewAuthorityGrant,
    NewOrganizationSponsorship, OrganizationPrincipalLink, PolicyEvaluationError, PolicyInput,
    PolicyReasonCode, ResourceScope, SponsorVisibility, Sponsorship, ValidityPeriod,
};

const ACTOR: &str = "actor";
const ISSUER: &str = "issuer";
const ACCOUNT: &str = "account";
const ORGANIZATION: &str = "organization";
const OTHER_ORGANIZATION: &str = "other-organization";
const EPISODE: &str = "episode";
const OTHER_EPISODE: &str = "other-episode";
const TARGET_ACTOR: &str = "target-actor";

fn at(hour: u32, minute: u32) -> Timestamp {
    Utc.with_ymd_and_hms(2026, 7, 26, hour, minute, 0)
        .single()
        .unwrap()
}

struct Ledger {
    events: Vec<IdentityEvent>,
    next_sequence: u64,
}

impl Ledger {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            next_sequence: 1,
        }
    }

    fn push_at(&mut self, recorded_at: Timestamp, payload: IdentityEventPayload) {
        let sequence = self.next_sequence;
        self.events.push(
            IdentityEvent::new(
                IdentityEventId::new(format!("event-{sequence}")),
                sequence,
                recorded_at,
                Principal::Platform,
                payload,
            )
            .unwrap(),
        );
        self.next_sequence += 1;
    }

    fn principal(&mut self, id: &str, kind: IdentityPrincipalKind) {
        let principal = IdentityPrincipal::new(
            IdentityPrincipalId::new(id),
            kind,
            format!("{id} display name"),
            at(0, 0),
            Principal::Platform,
        )
        .unwrap();
        self.push_at(
            at(0, 0),
            IdentityEventPayload::PrincipalCreated { principal },
        );
    }

    fn link_actor_account(&mut self) {
        self.push_at(
            at(0, 0),
            IdentityEventPayload::AccountPrincipalLinked {
                link: AccountPrincipalLink {
                    id: csqd_domain::AccountPrincipalLinkId::new("actor-link"),
                    account_id: UserId::new(ACCOUNT),
                    principal_id: IdentityPrincipalId::new(ACTOR),
                    status: LinkStatus::Active,
                    established_by: Principal::Platform,
                    established_at: at(0, 0),
                },
            },
        );
    }

    fn organization_link(&mut self, organization_id: &str, principal_id: &str) {
        self.push_at(
            at(0, 0),
            IdentityEventPayload::OrganizationPrincipalLinked {
                link: OrganizationPrincipalLink {
                    organization_id: OrganizationId::new(organization_id),
                    principal_id: IdentityPrincipalId::new(principal_id),
                    established_by: Principal::Platform,
                    established_at: at(0, 0),
                },
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn grant(
        &mut self,
        id: &str,
        represented_organization: Option<&str>,
        kind: AuthorityKind,
        scope: ResourceScope,
        actions: Vec<AuthorizedAction>,
        validity: Option<ValidityPeriod>,
    ) {
        let grant = AuthorityGrant::new(NewAuthorityGrant {
            id: AuthorityGrantId::new(id),
            actor_principal_id: IdentityPrincipalId::new(ACTOR),
            represented_organization_principal_id: represented_organization
                .map(IdentityPrincipalId::new),
            kind,
            scope,
            permitted_actions: actions,
            issued_by_principal_id: IdentityPrincipalId::new(ISSUER),
            issued_at: at(0, 0),
            validity,
            evidence_refs: vec!["policy-test-fixture".into()],
        })
        .unwrap();
        self.push_at(at(0, 0), IdentityEventPayload::AuthorityGranted { grant });
    }

    fn base() -> Self {
        let mut ledger = Self::new();
        ledger.principal(ACTOR, IdentityPrincipalKind::Human);
        ledger.principal(ISSUER, IdentityPrincipalKind::Human);
        ledger.principal(ORGANIZATION, IdentityPrincipalKind::Organization);
        ledger.principal(OTHER_ORGANIZATION, IdentityPrincipalKind::Organization);
        ledger.principal(TARGET_ACTOR, IdentityPrincipalKind::Human);
        ledger.organization_link("organization-record", ORGANIZATION);
        ledger.organization_link("other-organization-record", OTHER_ORGANIZATION);
        ledger.link_actor_account();
        ledger
    }
}

fn configuration() -> InitialPolicyConfiguration {
    InitialPolicyConfiguration::new(PolicyId::new("initial-policy-v1"))
}

fn input(
    action: AuthorizedAction,
    resource: ResourceScope,
    assurance: AssuranceLevel,
    represented_organization: Option<&str>,
) -> PolicyInput {
    PolicyInput {
        context: AuthorizationContext {
            account_id: UserId::new(ACCOUNT),
            actor_principal_id: IdentityPrincipalId::new(ACTOR),
            represented_organization_principal_id: represented_organization
                .map(IdentityPrincipalId::new),
            authentication_method: AuthenticationMethod::MultiFactor,
            authentication_assurance: assurance,
            authenticated_at: at(3, 50),
        },
        action,
        resource,
        evaluated_at: at(4, 0),
        conflict_status: ConflictStatus::Clear,
        authority_mutation_target: None,
    }
}

fn reviewer_grant(ledger: &mut Ledger, id: &str, episode: &str, validity: Option<ValidityPeriod>) {
    ledger.grant(
        id,
        None,
        AuthorityKind::EpisodeReviewer,
        ResourceScope::AuditEpisode(AuditEpisodeId::new(episode)),
        vec![
            AuthorizedAction::AcceptReviewAssignment,
            AuthorizedAction::SubmitElementReview,
            AuthorizedAction::SubmitSynthesisReview,
        ],
        validity,
    );
}

#[test]
fn initial_policy_matrix_allows_each_documented_path() {
    let mut ledger = Ledger::base();
    ledger.grant(
        "sponsor-representative",
        Some(ORGANIZATION),
        AuthorityKind::SponsorRepresentative,
        ResourceScope::Organization(OrganizationId::new("organization-record")),
        vec![AuthorizedAction::CommissionAudit],
        None,
    );
    ledger.grant(
        "sponsoring-organization-viewer",
        Some(ORGANIZATION),
        AuthorityKind::SponsorRepresentative,
        ResourceScope::Organization(OrganizationId::new("organization-record")),
        vec![
            AuthorizedAction::CommissionAudit,
            AuthorizedAction::ViewSponsoredAudit,
        ],
        None,
    );
    reviewer_grant(&mut ledger, "reviewer", EPISODE, None);
    ledger.grant(
        "operator",
        None,
        AuthorityKind::PlatformOperator,
        ResourceScope::Platform,
        vec![
            AuthorizedAction::PublishSynthesisReview,
            AuthorizedAction::RecordInvoice,
            AuthorizedAction::RecordPayment,
            AuthorizedAction::RecordReviewerPayout,
            AuthorizedAction::ManageAccounts,
            AuthorizedAction::ManageOrganizationMembers,
            AuthorizedAction::GrantAuthority,
            AuthorizedAction::RevokeAuthority,
            AuthorizedAction::ExportPrivateAudit,
        ],
        None,
    );
    ledger.grant(
        "confidential-evidence",
        None,
        AuthorityKind::Observer,
        ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
        vec![AuthorizedAction::ViewConfidentialEvidence],
        None,
    );
    ledger.push_at(
        at(0, 0),
        IdentityEventPayload::SponsorshipRecorded {
            sponsorship: Sponsorship::individual(
                SponsorshipId::new("personal-sponsorship"),
                AuditEpisodeId::new(EPISODE),
                IdentityPrincipalId::new(ACTOR),
                SponsorVisibility::Named,
                at(0, 0),
            ),
        },
    );
    ledger.push_at(
        at(0, 0),
        IdentityEventPayload::SponsorshipRecorded {
            sponsorship: Sponsorship::organization(NewOrganizationSponsorship {
                id: SponsorshipId::new("organization-sponsorship"),
                episode_id: AuditEpisodeId::new(OTHER_EPISODE),
                organization_principal_id: IdentityPrincipalId::new(ORGANIZATION),
                actor_principal_id: IdentityPrincipalId::new(ACTOR),
                authority_grant_id: AuthorityGrantId::new("sponsoring-organization-viewer"),
                visibility: SponsorVisibility::Named,
                created_at: at(0, 0),
            })
            .unwrap(),
        },
    );
    let state = project_identity_state(&ledger.events).unwrap();
    let policy = configuration();

    let personal_view = input(
        AuthorizedAction::ViewSponsoredAudit,
        ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
        AssuranceLevel::Medium,
        None,
    );
    let personal_view_record = evaluate_access(&state, &policy, &personal_view)
        .unwrap()
        .to_access_decision(AccessDecisionId::new("personal-view-decision"))
        .unwrap();
    assert_eq!(
        personal_view_record.authorization_basis(),
        Some(&AuthorizationBasis::PersonalSponsorship(
            SponsorshipId::new("personal-sponsorship")
        ))
    );

    let organization_commission = input(
        AuthorizedAction::CommissionAudit,
        ResourceScope::Organization(OrganizationId::new("organization-record")),
        AssuranceLevel::Medium,
        Some(ORGANIZATION),
    );
    let organization_commission_record = evaluate_access(&state, &policy, &organization_commission)
        .unwrap()
        .to_access_decision(AccessDecisionId::new("organization-commission-decision"))
        .unwrap();
    assert_eq!(
        organization_commission_record.authorization_basis(),
        Some(&AuthorizationBasis::AuthorityGrant(AuthorityGrantId::new(
            "sponsor-representative"
        )))
    );

    let mut cases = vec![
        input(
            AuthorizedAction::RegisterPublicAuditSubject,
            ResourceScope::Domain(DomainInstantiationId::new("domain")),
            AssuranceLevel::Low,
            None,
        ),
        input(
            AuthorizedAction::CommissionAudit,
            ResourceScope::AuditSubject(AuditSubjectId::new("subject")),
            AssuranceLevel::Medium,
            None,
        ),
        input(
            AuthorizedAction::CommissionAudit,
            ResourceScope::Organization(OrganizationId::new("organization-record")),
            AssuranceLevel::Medium,
            Some(ORGANIZATION),
        ),
        input(
            AuthorizedAction::ViewSponsoredAudit,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::Medium,
            None,
        ),
        input(
            AuthorizedAction::ViewSponsoredAudit,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(OTHER_EPISODE)),
            AssuranceLevel::Medium,
            Some(ORGANIZATION),
        ),
        input(
            AuthorizedAction::AcceptReviewAssignment,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::Medium,
            None,
        ),
        input(
            AuthorizedAction::SubmitElementReview,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::Medium,
            None,
        ),
        input(
            AuthorizedAction::SubmitSynthesisReview,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::Medium,
            None,
        ),
        input(
            AuthorizedAction::ManageOrganizationMembers,
            ResourceScope::Organization(OrganizationId::new("organization-record")),
            AssuranceLevel::Medium,
            None,
        ),
        input(
            AuthorizedAction::PublishSynthesisReview,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::High,
            None,
        ),
        input(
            AuthorizedAction::RecordInvoice,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::High,
            None,
        ),
        input(
            AuthorizedAction::RecordPayment,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::High,
            None,
        ),
        input(
            AuthorizedAction::RecordReviewerPayout,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::High,
            None,
        ),
        input(
            AuthorizedAction::ManageAccounts,
            ResourceScope::Platform,
            AssuranceLevel::High,
            None,
        ),
        input(
            AuthorizedAction::GrantAuthority,
            ResourceScope::Platform,
            AssuranceLevel::High,
            None,
        ),
        input(
            AuthorizedAction::RevokeAuthority,
            ResourceScope::Platform,
            AssuranceLevel::High,
            None,
        ),
        input(
            AuthorizedAction::ExportPrivateAudit,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::High,
            None,
        ),
        input(
            AuthorizedAction::ViewConfidentialEvidence,
            ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
            AssuranceLevel::High,
            None,
        ),
    ];
    for case in &mut cases {
        if matches!(
            case.action,
            AuthorizedAction::GrantAuthority | AuthorizedAction::RevokeAuthority
        ) {
            case.authority_mutation_target = Some(AuthorityMutationTarget {
                actor_principal_id: IdentityPrincipalId::new(TARGET_ACTOR),
                represented_organization_principal_id: None,
                kind: AuthorityKind::PlatformOperator,
                scope: ResourceScope::Platform,
            });
        }
    }

    for case in cases {
        let decision = evaluate_access(&state, &policy, &case).unwrap();
        assert_eq!(
            decision.outcome,
            AuthorizationOutcome::Allowed,
            "unexpected decision for {:?}: {:?}",
            case.action,
            decision.reason_codes
        );
    }
}

#[test]
fn missing_authority_is_denied() {
    let ledger = Ledger::base();
    let state = project_identity_state(&ledger.events).unwrap();
    let request = input(
        AuthorizedAction::SubmitElementReview,
        ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
        AssuranceLevel::Medium,
        None,
    );

    let decision = evaluate_access(&state, &configuration(), &request).unwrap();
    assert_eq!(decision.outcome, AuthorizationOutcome::Denied);
    assert_eq!(
        decision.reason_codes,
        vec![PolicyReasonCode::AuthorityMissing]
    );
}

#[test]
fn expired_and_revoked_grants_are_denied_with_distinct_reasons() {
    let mut expired_ledger = Ledger::base();
    reviewer_grant(
        &mut expired_ledger,
        "expired-reviewer",
        EPISODE,
        Some(ValidityPeriod::new(at(0, 0), Some(at(3, 0))).unwrap()),
    );
    let request = input(
        AuthorizedAction::SubmitElementReview,
        ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
        AssuranceLevel::Medium,
        None,
    );
    let expired = evaluate_access(
        &project_identity_state(&expired_ledger.events).unwrap(),
        &configuration(),
        &request,
    )
    .unwrap();
    assert_eq!(expired.outcome, AuthorizationOutcome::Denied);
    assert_eq!(expired.reason_codes, vec![PolicyReasonCode::GrantExpired]);

    let mut revoked_ledger = Ledger::base();
    reviewer_grant(&mut revoked_ledger, "revoked-reviewer", EPISODE, None);
    revoked_ledger.push_at(
        at(3, 0),
        IdentityEventPayload::AuthorityRevoked {
            revocation: AuthorityRevocation::new(
                AuthorityRevocationId::new("revocation"),
                AuthorityGrantId::new("revoked-reviewer"),
                IdentityPrincipalId::new(ISSUER),
                at(3, 0),
                "assignment withdrawn",
            )
            .unwrap(),
        },
    );
    let revoked = evaluate_access(
        &project_identity_state(&revoked_ledger.events).unwrap(),
        &configuration(),
        &request,
    )
    .unwrap();
    assert_eq!(revoked.outcome, AuthorizationOutcome::Denied);
    assert_eq!(revoked.reason_codes, vec![PolicyReasonCode::GrantRevoked]);
}

#[test]
fn organization_and_episode_scope_mismatches_are_denied() {
    let mut organization_ledger = Ledger::base();
    organization_ledger.grant(
        "organization-grant",
        Some(ORGANIZATION),
        AuthorityKind::SponsorRepresentative,
        ResourceScope::Organization(OrganizationId::new("organization-record")),
        vec![AuthorizedAction::CommissionAudit],
        None,
    );
    let wrong_organization = input(
        AuthorizedAction::CommissionAudit,
        ResourceScope::Organization(OrganizationId::new("organization-record")),
        AssuranceLevel::Medium,
        Some(OTHER_ORGANIZATION),
    );
    let decision = evaluate_access(
        &project_identity_state(&organization_ledger.events).unwrap(),
        &configuration(),
        &wrong_organization,
    )
    .unwrap();
    assert_eq!(decision.outcome, AuthorizationOutcome::Denied);
    assert_eq!(
        decision.reason_codes,
        vec![PolicyReasonCode::WrongOrganization]
    );

    let mut episode_ledger = Ledger::base();
    reviewer_grant(&mut episode_ledger, "episode-grant", EPISODE, None);
    let wrong_episode = input(
        AuthorizedAction::SubmitElementReview,
        ResourceScope::AuditEpisode(AuditEpisodeId::new(OTHER_EPISODE)),
        AssuranceLevel::Medium,
        None,
    );
    let decision = evaluate_access(
        &project_identity_state(&episode_ledger.events).unwrap(),
        &configuration(),
        &wrong_episode,
    )
    .unwrap();
    assert_eq!(decision.outcome, AuthorizationOutcome::Denied);
    assert_eq!(
        decision.reason_codes,
        vec![PolicyReasonCode::WrongResourceScope]
    );
}

#[test]
fn assurance_and_authentication_age_produce_step_up_decisions() {
    let mut ledger = Ledger::base();
    ledger.grant(
        "operator",
        None,
        AuthorityKind::PlatformOperator,
        ResourceScope::Platform,
        vec![
            AuthorizedAction::ManageAccounts,
            AuthorizedAction::GrantAuthority,
        ],
        None,
    );
    let state = project_identity_state(&ledger.events).unwrap();

    let low_assurance = input(
        AuthorizedAction::ManageAccounts,
        ResourceScope::Platform,
        AssuranceLevel::Medium,
        None,
    );
    let decision = evaluate_access(&state, &configuration(), &low_assurance).unwrap();
    assert_eq!(decision.outcome, AuthorizationOutcome::StepUpRequired);
    assert_eq!(
        decision.reason_codes,
        vec![PolicyReasonCode::InsufficientAssurance]
    );
    assert!(matches!(
        decision.authorization_basis,
        Some(AuthorizationBasis::AuthorityGrant(_))
    ));

    let mut stale_authentication = input(
        AuthorizedAction::ManageAccounts,
        ResourceScope::Platform,
        AssuranceLevel::High,
        None,
    );
    stale_authentication.context.authenticated_at = at(3, 30);
    let decision = evaluate_access(&state, &configuration(), &stale_authentication).unwrap();
    assert_eq!(decision.outcome, AuthorizationOutcome::StepUpRequired);
    assert_eq!(
        decision.reason_codes,
        vec![PolicyReasonCode::AuthenticationTooOld]
    );

    let mut grant_operator = input(
        AuthorizedAction::GrantAuthority,
        ResourceScope::Platform,
        AssuranceLevel::Medium,
        None,
    );
    grant_operator.authority_mutation_target = Some(AuthorityMutationTarget {
        actor_principal_id: IdentityPrincipalId::new(TARGET_ACTOR),
        represented_organization_principal_id: None,
        kind: AuthorityKind::PlatformOperator,
        scope: ResourceScope::Platform,
    });
    let decision = evaluate_access(&state, &configuration(), &grant_operator).unwrap();
    assert_eq!(decision.outcome, AuthorizationOutcome::StepUpRequired);
    assert_eq!(
        decision.reason_codes,
        vec![PolicyReasonCode::InsufficientAssurance]
    );
}

#[test]
fn unresolved_review_conflict_requires_manual_review() {
    let mut ledger = Ledger::base();
    reviewer_grant(&mut ledger, "reviewer", EPISODE, None);
    let state = project_identity_state(&ledger.events).unwrap();
    let mut request = input(
        AuthorizedAction::SubmitElementReview,
        ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
        AssuranceLevel::Medium,
        None,
    );
    request.conflict_status = ConflictStatus::Unresolved;

    let decision = evaluate_access(&state, &configuration(), &request).unwrap();
    assert_eq!(decision.outcome, AuthorizationOutcome::ManualReviewRequired);
    assert_eq!(
        decision.reason_codes,
        vec![PolicyReasonCode::UnresolvedConflict]
    );
}

#[test]
fn policy_inputs_are_validated_and_decisions_convert_to_audit_records() {
    let mut ledger = Ledger::base();
    let state = project_identity_state(&ledger.events).unwrap();
    let mut request = input(
        AuthorizedAction::RegisterPublicAuditSubject,
        ResourceScope::Domain(DomainInstantiationId::new("domain")),
        AssuranceLevel::Low,
        None,
    );
    request.context.authenticated_at = at(5, 0);
    assert_eq!(
        evaluate_access(&state, &configuration(), &request),
        Err(PolicyEvaluationError::AuthenticationAfterEvaluation)
    );

    let missing_target = input(
        AuthorizedAction::GrantAuthority,
        ResourceScope::Platform,
        AssuranceLevel::High,
        None,
    );
    assert_eq!(
        evaluate_access(&state, &configuration(), &missing_target),
        Err(PolicyEvaluationError::MissingAuthorityMutationTarget)
    );

    request.context.authenticated_at = at(3, 50);
    let decision = evaluate_access(&state, &configuration(), &request).unwrap();
    let decision_id = AccessDecisionId::new("decision");
    let audit_record = decision.to_access_decision(decision_id.clone()).unwrap();
    assert_eq!(
        audit_record.reason_codes(),
        ["allowed_authenticated_principal"]
    );
    assert_eq!(
        serde_json::to_value(PolicyReasonCode::WrongResourceScope).unwrap(),
        serde_json::json!("wrong_resource_scope")
    );

    ledger.push_at(
        at(4, 0),
        IdentityEventPayload::AccessDecisionRecorded {
            decision: audit_record,
        },
    );
    assert!(project_identity_state(&ledger.events)
        .unwrap()
        .access_decision(&decision_id)
        .is_some());
}

#[test]
fn future_issued_grant_is_denied_as_not_yet_valid() {
    let mut ledger = Ledger::base();
    let grant = AuthorityGrant::new(NewAuthorityGrant {
        id: AuthorityGrantId::new("future-reviewer"),
        actor_principal_id: IdentityPrincipalId::new(ACTOR),
        represented_organization_principal_id: None,
        kind: AuthorityKind::EpisodeReviewer,
        scope: ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
        permitted_actions: vec![AuthorizedAction::SubmitElementReview],
        issued_by_principal_id: IdentityPrincipalId::new(ISSUER),
        issued_at: at(5, 0),
        validity: None,
        evidence_refs: vec![],
    })
    .unwrap();
    ledger.push_at(at(0, 0), IdentityEventPayload::AuthorityGranted { grant });
    let request = input(
        AuthorizedAction::SubmitElementReview,
        ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
        AssuranceLevel::Medium,
        None,
    );

    let decision = evaluate_access(
        &project_identity_state(&ledger.events).unwrap(),
        &configuration(),
        &request,
    )
    .unwrap();
    assert_eq!(decision.outcome, AuthorizationOutcome::Denied);
    assert_eq!(
        decision.reason_codes,
        vec![PolicyReasonCode::GrantNotYetValid]
    );
}

#[test]
fn authority_mutation_enforces_target_scope_and_prevents_self_escalation() {
    let mut ledger = Ledger::base();
    ledger.grant(
        "organization-administrator",
        Some(ORGANIZATION),
        AuthorityKind::OrganizationAdministrator,
        ResourceScope::Organization(OrganizationId::new("organization-record")),
        vec![
            AuthorizedAction::GrantAuthority,
            AuthorizedAction::RevokeAuthority,
        ],
        None,
    );
    let state = project_identity_state(&ledger.events).unwrap();

    let mut organization_grant = input(
        AuthorizedAction::GrantAuthority,
        ResourceScope::Organization(OrganizationId::new("organization-record")),
        AssuranceLevel::High,
        Some(ORGANIZATION),
    );
    organization_grant.authority_mutation_target = Some(AuthorityMutationTarget {
        actor_principal_id: IdentityPrincipalId::new(TARGET_ACTOR),
        represented_organization_principal_id: Some(IdentityPrincipalId::new(ORGANIZATION)),
        kind: AuthorityKind::OrganizationRepresentative,
        scope: ResourceScope::Organization(OrganizationId::new("organization-record")),
    });
    let allowed = evaluate_access(&state, &configuration(), &organization_grant).unwrap();
    assert_eq!(allowed.outcome, AuthorizationOutcome::Allowed);

    let mut platform_grant = organization_grant.clone();
    platform_grant.resource = ResourceScope::Platform;
    platform_grant.authority_mutation_target = Some(AuthorityMutationTarget {
        actor_principal_id: IdentityPrincipalId::new(TARGET_ACTOR),
        represented_organization_principal_id: None,
        kind: AuthorityKind::PlatformOperator,
        scope: ResourceScope::Platform,
    });
    let denied = evaluate_access(&state, &configuration(), &platform_grant).unwrap();
    assert_eq!(denied.outcome, AuthorizationOutcome::Denied);
    assert_eq!(
        denied.reason_codes,
        vec![PolicyReasonCode::AuthorityKindNotPermitted]
    );

    let mut self_grant = organization_grant;
    self_grant.authority_mutation_target = Some(AuthorityMutationTarget {
        actor_principal_id: IdentityPrincipalId::new(ACTOR),
        represented_organization_principal_id: Some(IdentityPrincipalId::new(ORGANIZATION)),
        kind: AuthorityKind::OrganizationRepresentative,
        scope: ResourceScope::Organization(OrganizationId::new("organization-record")),
    });
    let denied = evaluate_access(&state, &configuration(), &self_grant).unwrap();
    assert_eq!(denied.outcome, AuthorizationOutcome::Denied);
    assert_eq!(
        denied.reason_codes,
        vec![PolicyReasonCode::SelfEscalationNotAllowed]
    );
}

#[test]
fn every_authorized_action_fails_closed_for_an_unlinked_account() {
    let state = project_identity_state(&Ledger::base().events).unwrap();
    let actions = [
        AuthorizedAction::RegisterPublicAuditSubject,
        AuthorizedAction::CommissionAudit,
        AuthorizedAction::ManageOrganizationMembers,
        AuthorizedAction::ViewSponsoredAudit,
        AuthorizedAction::AcceptReviewAssignment,
        AuthorizedAction::SubmitElementReview,
        AuthorizedAction::SubmitSynthesisReview,
        AuthorizedAction::ViewConfidentialEvidence,
        AuthorizedAction::PublishSynthesisReview,
        AuthorizedAction::RecordInvoice,
        AuthorizedAction::RecordPayment,
        AuthorizedAction::RecordReviewerPayout,
        AuthorizedAction::ManageAccounts,
        AuthorizedAction::GrantAuthority,
        AuthorizedAction::RevokeAuthority,
        AuthorizedAction::ExportPrivateAudit,
    ];

    for action in actions {
        let resource = match action {
            AuthorizedAction::RegisterPublicAuditSubject => {
                ResourceScope::Domain(DomainInstantiationId::new("domain"))
            }
            AuthorizedAction::CommissionAudit => {
                ResourceScope::AuditSubject(AuditSubjectId::new("subject"))
            }
            AuthorizedAction::ManageOrganizationMembers => {
                ResourceScope::Organization(OrganizationId::new("organization-record"))
            }
            AuthorizedAction::ManageAccounts
            | AuthorizedAction::GrantAuthority
            | AuthorizedAction::RevokeAuthority => ResourceScope::Platform,
            _ => ResourceScope::AuditEpisode(AuditEpisodeId::new(EPISODE)),
        };
        let mut request = input(action, resource.clone(), AssuranceLevel::VeryHigh, None);
        request.context.account_id = UserId::new("unlinked-account");
        if matches!(
            action,
            AuthorizedAction::GrantAuthority | AuthorizedAction::RevokeAuthority
        ) {
            request.authority_mutation_target = Some(AuthorityMutationTarget {
                actor_principal_id: IdentityPrincipalId::new(TARGET_ACTOR),
                represented_organization_principal_id: None,
                kind: AuthorityKind::PlatformOperator,
                scope: resource,
            });
        }

        let decision = evaluate_access(&state, &configuration(), &request).unwrap();
        assert_eq!(
            decision.outcome,
            AuthorizationOutcome::Denied,
            "{action:?} did not fail closed"
        );
        assert_eq!(
            decision.reason_codes,
            vec![PolicyReasonCode::AccountPrincipalMismatch]
        );
    }
}
