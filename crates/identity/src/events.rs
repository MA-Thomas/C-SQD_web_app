use std::fmt;

use csqd_domain::{
    AccountPrincipalLinkId, AuthenticationIdentityId, IdentityAssertionId, IdentityEventId,
    IdentityPrincipalId, OrganizationMembershipId, Principal, Timestamp,
};
use serde::{Deserialize, Serialize};

use crate::{
    AccessDecision, AccountPrincipalLink, AssertionStatus, AuthenticationIdentity,
    AuthenticationIdentityStatus, AuthorityGrant, AuthorityRevocation, IdentityAssertion,
    IdentityModelError, IdentityPrincipal, IdentityPrincipalStatus, LinkStatus,
    OrganizationMembership, OrganizationMembershipStatus, OrganizationPrincipalLink, Sponsorship,
};

/// An immutable, explicitly ordered identity-domain event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityEvent {
    pub id: IdentityEventId,
    pub append_sequence: u64,
    pub recorded_at: Timestamp,
    pub recorded_by: Principal,
    pub payload: IdentityEventPayload,
}

impl IdentityEvent {
    pub fn new(
        id: IdentityEventId,
        append_sequence: u64,
        recorded_at: Timestamp,
        recorded_by: Principal,
        payload: IdentityEventPayload,
    ) -> Result<Self, IdentityEventValidationError> {
        payload.validate()?;

        Ok(Self {
            id,
            append_sequence,
            recorded_at,
            recorded_by,
            payload,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), IdentityEventValidationError> {
        self.payload.validate()
    }
}

/// Closed event vocabulary for the initial identity model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum IdentityEventPayload {
    PrincipalCreated {
        principal: IdentityPrincipal,
    },
    PrincipalStatusChanged {
        principal_id: IdentityPrincipalId,
        status: IdentityPrincipalStatus,
    },
    AccountPrincipalLinked {
        link: AccountPrincipalLink,
    },
    AccountPrincipalLinkStatusChanged {
        link_id: AccountPrincipalLinkId,
        status: LinkStatus,
    },
    OrganizationPrincipalLinked {
        link: OrganizationPrincipalLink,
    },
    AuthenticationIdentityAdded {
        identity: AuthenticationIdentity,
    },
    AuthenticationIdentityStatusChanged {
        identity_id: AuthenticationIdentityId,
        status: AuthenticationIdentityStatus,
    },
    AssertionRecorded {
        assertion: IdentityAssertion,
    },
    AssertionStatusChanged {
        assertion_id: IdentityAssertionId,
        status: AssertionStatus,
    },
    OrganizationMembershipRecorded {
        membership: OrganizationMembership,
    },
    OrganizationMembershipStatusChanged {
        membership_id: OrganizationMembershipId,
        status: OrganizationMembershipStatus,
    },
    SponsorshipRecorded {
        sponsorship: Sponsorship,
    },
    AuthorityGranted {
        grant: AuthorityGrant,
    },
    AuthorityRevoked {
        revocation: AuthorityRevocation,
    },
    AccessDecisionRecorded {
        decision: AccessDecision,
    },
}

impl IdentityEventPayload {
    pub(crate) fn validate(&self) -> Result<(), IdentityEventValidationError> {
        match self {
            Self::PrincipalCreated { principal } => {
                principal.validate()?;
                require_initial_status(
                    matches!(principal.status, IdentityPrincipalStatus::Active),
                    "principal",
                )
            }
            Self::PrincipalStatusChanged { status, .. } => {
                if let IdentityPrincipalStatus::Superseded { reason, .. } = status {
                    require_non_empty(reason, "principal supersession reason")?;
                }
                Ok(())
            }
            Self::AccountPrincipalLinked { link } => require_initial_status(
                matches!(link.status, LinkStatus::Active),
                "account principal link",
            ),
            Self::AccountPrincipalLinkStatusChanged { .. } => Ok(()),
            Self::OrganizationPrincipalLinked { .. } => Ok(()),
            Self::AuthenticationIdentityAdded { identity } => {
                identity.validate()?;
                require_initial_status(
                    matches!(identity.status, AuthenticationIdentityStatus::Active),
                    "authentication identity",
                )
            }
            Self::AuthenticationIdentityStatusChanged { .. } => Ok(()),
            Self::AssertionRecorded { assertion } => {
                assertion.validate()?;
                require_initial_status(
                    matches!(assertion.status, AssertionStatus::Active),
                    "identity assertion",
                )
            }
            Self::AssertionStatusChanged { .. } => Ok(()),
            Self::OrganizationMembershipRecorded { membership } => {
                membership.validate()?;
                require_initial_status(
                    matches!(
                        membership.status,
                        OrganizationMembershipStatus::Invited
                            | OrganizationMembershipStatus::Active
                    ),
                    "organization membership",
                )
            }
            Self::OrganizationMembershipStatusChanged { .. } => Ok(()),
            Self::SponsorshipRecorded { sponsorship } => {
                sponsorship.validate()?;
                Ok(())
            }
            Self::AuthorityGranted { grant } => {
                grant.validate()?;
                Ok(())
            }
            Self::AuthorityRevoked { revocation } => {
                revocation.validate()?;
                Ok(())
            }
            Self::AccessDecisionRecorded { decision } => {
                decision.validate()?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityEventValidationError {
    Model(IdentityModelError),
    InvalidInitialStatus(&'static str),
    EmptyValue(&'static str),
}

impl From<IdentityModelError> for IdentityEventValidationError {
    fn from(error: IdentityModelError) -> Self {
        Self::Model(error)
    }
}

impl fmt::Display for IdentityEventValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(error) => error.fmt(formatter),
            Self::InvalidInitialStatus(entity) => {
                write!(formatter, "{entity} event must begin in an initial status")
            }
            Self::EmptyValue(field) => write!(formatter, "{field} must not be empty"),
        }
    }
}

impl std::error::Error for IdentityEventValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Model(error) => Some(error),
            Self::InvalidInitialStatus(_) | Self::EmptyValue(_) => None,
        }
    }
}

fn require_initial_status(
    condition: bool,
    entity: &'static str,
) -> Result<(), IdentityEventValidationError> {
    if condition {
        Ok(())
    } else {
        Err(IdentityEventValidationError::InvalidInitialStatus(entity))
    }
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), IdentityEventValidationError> {
    if value.trim().is_empty() {
        Err(IdentityEventValidationError::EmptyValue(field))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use csqd_domain::{IdentityEventId, IdentityPrincipalId, Principal};
    use serde_json::json;

    use super::*;
    use crate::{IdentityPrincipalKind, SponsorVisibility};

    fn timestamp() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 7, 26, 12, 0, 0)
            .single()
            .unwrap()
    }

    #[test]
    fn event_type_has_stable_tagged_wire_shape() {
        let principal = IdentityPrincipal::new(
            IdentityPrincipalId::new("principal-1"),
            IdentityPrincipalKind::Human,
            "Ada Reviewer",
            timestamp(),
            Principal::Platform,
        )
        .unwrap();
        let event = IdentityEvent::new(
            IdentityEventId::new("event-1"),
            1,
            timestamp(),
            Principal::Platform,
            IdentityEventPayload::PrincipalCreated { principal },
        )
        .unwrap();
        let value = serde_json::to_value(event).unwrap();

        assert_eq!(value["payload"]["event_type"], json!("principal_created"));
        assert_eq!(value["append_sequence"], json!(1));
    }

    #[test]
    fn event_rejects_non_initial_embedded_status() {
        let mut principal = IdentityPrincipal::new(
            IdentityPrincipalId::new("principal-1"),
            IdentityPrincipalKind::Human,
            "Ada Reviewer",
            timestamp(),
            Principal::Platform,
        )
        .unwrap();
        principal.status = IdentityPrincipalStatus::Deactivated;

        let result = IdentityEvent::new(
            IdentityEventId::new("event-1"),
            1,
            timestamp(),
            Principal::Platform,
            IdentityEventPayload::PrincipalCreated { principal },
        );

        assert_eq!(
            result,
            Err(IdentityEventValidationError::InvalidInitialStatus(
                "principal"
            ))
        );
    }

    #[test]
    fn event_revalidates_sponsorship_built_outside_constructor() {
        let sponsorship = Sponsorship {
            id: csqd_domain::SponsorshipId::new("sponsorship-1"),
            episode_id: csqd_domain::AuditEpisodeId::new("episode-1"),
            sponsor: crate::SponsoringParty::Individual(IdentityPrincipalId::new("human-1")),
            actor_principal_id: IdentityPrincipalId::new("different-human"),
            represented_organization_principal_id: None,
            authority_grant_id: None,
            visibility: SponsorVisibility::Named,
            created_at: timestamp(),
        };

        let result = IdentityEvent::new(
            IdentityEventId::new("event-1"),
            1,
            timestamp(),
            Principal::Platform,
            IdentityEventPayload::SponsorshipRecorded { sponsorship },
        );

        assert!(matches!(
            result,
            Err(IdentityEventValidationError::Model(
                IdentityModelError::InconsistentSponsorship(_)
            ))
        ));
    }
}
