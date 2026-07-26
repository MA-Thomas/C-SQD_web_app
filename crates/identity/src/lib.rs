//! Identity and authorization rules for C-SQD.
//!
//! This crate models durable identity principals, evidence-backed assertions,
//! sponsorship, organization membership, scoped authority, and authorization
//! outcomes. It deliberately contains no HTTP, database, cookie, or UI code.
//! Those boundaries remain in `services/api` and `apps/web`.
//!
//! Identity identifiers are distinct at compile time:
//!
//! ```compile_fail
//! use csqd_domain::{AuthorityGrantId, IdentityPrincipalId};
//!
//! fn load_principal(_id: IdentityPrincipalId) {}
//!
//! let grant_id = AuthorityGrantId::new("grant-1");
//! load_principal(grant_id);
//! ```

mod error;
mod events;
mod model;
mod policy;
mod projection;

pub use error::IdentityModelError;
pub use events::{IdentityEvent, IdentityEventPayload, IdentityEventValidationError};
pub use model::{
    AccessDecision, AccountPrincipalLink, AssertionStatus, AssuranceLevel, AuthenticationIdentity,
    AuthenticationIdentityKind, AuthenticationIdentityStatus, AuthenticationMethod, AuthorityGrant,
    AuthorityKind, AuthorityRevocation, AuthorizationBasis, AuthorizationOutcome, AuthorizedAction,
    IdentityAssertion, IdentityAssertionKind, IdentityPrincipal, IdentityPrincipalKind,
    IdentityPrincipalStatus, LinkStatus, NewAccessDecision, NewAuthorityGrant,
    NewIdentityAssertion, NewOrganizationMembership, NewOrganizationSponsorship,
    OrganizationMembership, OrganizationMembershipRole, OrganizationMembershipStatus,
    OrganizationPrincipalLink, ResourceScope, SponsorVisibility, SponsoringParty, Sponsorship,
    ValidityPeriod,
};
pub use policy::{
    evaluate_access, AuthorityMutationTarget, AuthorizationContext, ConflictStatus,
    InitialPolicyConfiguration, PolicyDecision, PolicyEvaluationError, PolicyInput,
    PolicyReasonCode,
};
pub use projection::{
    project_identity_state, project_identity_state_at, IdentityProjectionError, IdentityState,
};
