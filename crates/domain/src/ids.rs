//! Entity id newtypes.
//!
//! The FEN schema defines distinct id types per entity (`FactId`,
//! `AuditEpisodeId`, ...). These are string-backed wrappers over the UUID
//! values stored in Postgres: serde-transparent on the wire, distinct to the
//! compiler. Backing by `String` rather than `Uuid` keeps the database
//! boundary (queries cast `id::text`) and seeded data trivially compatible;
//! upgrading the backing type later is a contained change inside this module.

use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

define_id!(
    /// Identifies a configured audit domain (`DomainInstantiation`).
    DomainInstantiationId
);
define_id!(
    /// Identifies a node in a domain's CWE taxonomy.
    CWENodeId
);
define_id!(
    /// Identifies a reviewer community proposing CWE extensions (deferred post-MVP).
    CommunityId
);
define_id!(
    /// Identifies the artifact under evaluation (`AuditSubject`).
    AuditSubjectId
);
define_id!(
    /// Identifies an atomic epistemic or administrative act (`Fact`).
    FactId
);
define_id!(
    /// Identifies a coherent audit question over time (`AuditEpisode`).
    AuditEpisodeId
);
define_id!(
    /// Identifies a provenance-bearing fact-to-episode link (`EpisodeMembership`).
    MembershipId
);
define_id!(
    /// Identifies an asserted relation between episodes or synthesis reviews.
    RelationId
);
define_id!(
    /// Identifies an authored integrative interpretation (`SynthesisReview`).
    SynthesisReviewId
);
define_id!(
    /// Identifies a section within a `SynthesisReview`.
    SectionId
);
define_id!(
    /// Identifies a platform participant (`User`).
    UserId
);
define_id!(
    /// Identifies an institutional sponsor (`Organization`).
    OrganizationId
);
define_id!(
    /// Identifies a reviewer expertise tag.
    TagId
);
define_id!(
    /// Identifies an evidence-artifact attachment to an audit episode
    /// (`EpisodeEvidenceArtifact`).
    EvidenceArtifactId
);
define_id!(
    /// Identifies a durable human, organization, or system identity principal.
    IdentityPrincipalId
);
define_id!(
    /// Identifies a link between an authenticated account and an identity principal.
    AccountPrincipalLinkId
);
define_id!(
    /// Identifies an external authentication identity linked to an account.
    AuthenticationIdentityId
);
define_id!(
    /// Identifies an evidence-backed assertion about an identity principal.
    IdentityAssertionId
);
define_id!(
    /// Identifies an append-only identity-domain event.
    IdentityEventId
);
define_id!(
    /// Identifies a human's membership in an organization.
    OrganizationMembershipId
);
define_id!(
    /// Identifies a human or organization sponsorship of an audit episode.
    SponsorshipId
);
define_id!(
    /// Identifies a scoped grant of authority.
    AuthorityGrantId
);
define_id!(
    /// Identifies an append-only revocation of an authority grant.
    AuthorityRevocationId
);
define_id!(
    /// Identifies an auditable authorization decision.
    AccessDecisionId
);
define_id!(
    /// Identifies a versioned authorization policy.
    PolicyId
);

#[cfg(feature = "sqlx")]
mod sqlx_impls {
    use sqlx::encode::IsNull;
    use sqlx::error::BoxDynError;
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
    use sqlx::{Decode, Encode, Postgres, Type};

    use super::*;

    macro_rules! impl_sqlx_id {
        ($name:ident) => {
            impl Type<Postgres> for $name {
                fn type_info() -> PgTypeInfo {
                    <String as Type<Postgres>>::type_info()
                }

                fn compatible(ty: &PgTypeInfo) -> bool {
                    <String as Type<Postgres>>::compatible(ty)
                }
            }

            impl<'r> Decode<'r, Postgres> for $name {
                fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
                    Ok(Self(<String as Decode<'r, Postgres>>::decode(value)?))
                }
            }

            impl<'q> Encode<'q, Postgres> for $name {
                fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
                    <String as Encode<'q, Postgres>>::encode_by_ref(&self.0, buf)
                }
            }
        };
    }

    impl_sqlx_id!(DomainInstantiationId);
    impl_sqlx_id!(CWENodeId);
    impl_sqlx_id!(CommunityId);
    impl_sqlx_id!(AuditSubjectId);
    impl_sqlx_id!(FactId);
    impl_sqlx_id!(AuditEpisodeId);
    impl_sqlx_id!(MembershipId);
    impl_sqlx_id!(RelationId);
    impl_sqlx_id!(SynthesisReviewId);
    impl_sqlx_id!(SectionId);
    impl_sqlx_id!(UserId);
    impl_sqlx_id!(OrganizationId);
    impl_sqlx_id!(TagId);
    impl_sqlx_id!(EvidenceArtifactId);
    impl_sqlx_id!(IdentityPrincipalId);
    impl_sqlx_id!(AccountPrincipalLinkId);
    impl_sqlx_id!(AuthenticationIdentityId);
    impl_sqlx_id!(IdentityAssertionId);
    impl_sqlx_id!(IdentityEventId);
    impl_sqlx_id!(OrganizationMembershipId);
    impl_sqlx_id!(SponsorshipId);
    impl_sqlx_id!(AuthorityGrantId);
    impl_sqlx_id!(AuthorityRevocationId);
    impl_sqlx_id!(AccessDecisionId);
    impl_sqlx_id!(PolicyId);
}

#[cfg(test)]
mod tests {
    use super::FactId;

    #[test]
    fn id_newtype_is_serde_transparent() {
        let id = FactId::new("fact-1");
        let json = serde_json::to_value(&id).unwrap();

        assert_eq!(json, serde_json::json!("fact-1"));

        let back: FactId = serde_json::from_value(json).unwrap();

        assert_eq!(back, id);
    }
}
