//! Claim-scoped audit view types (CLAIM_SCOPED_AUDITS_MEMO.md).
//!
//! The substrate stores evidence attachments with an opaque scholarly-object
//! id; this module enriches them with Academic Publishing metadata for
//! display, and gives paper pages the full list of audits a work is involved
//! in — as the subject of a single-paper audit or as attached evidence in a
//! claim-scoped one. Papers stay first-class discovery surfaces without
//! being the audit's epistemic target.

use serde::{Deserialize, Serialize};

use csqd_domain::{
    ArtifactBearing, EpisodeEvidenceArtifact, EvalTuple, ScopeCondition, SynthesisReview,
};

/// One attached evidence artifact, enriched for display: the substrate link,
/// the scholarly metadata, and the derived (never stored) audit bearing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifactSummary {
    pub artifact: EpisodeEvidenceArtifact,
    pub scholarly_object_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub source_name: String,
    pub publication_year: Option<i32>,
    pub canonical_url: String,
    /// Derived from warrant assertions and the element reviews that
    /// scrutinize them. Attachment never confers support.
    pub bearing: ArtifactBearing,
    /// Active warrant assertions running through this artifact.
    pub warrant_count: i64,
    /// Active element reviews targeting this artifact or its warrants.
    pub review_count: i64,
}

/// What the audit episode is actually about. This is separate from the
/// scholarly work's role because the same work can be a direct target in one
/// episode and evidence in another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTargetSummary {
    pub subject_id: String,
    pub subject_type: String,
    pub title: Option<String>,
    pub claim_statement: Option<String>,
    pub scope_conditions: Vec<ScopeCondition>,
}

/// The episode containing this work involvement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEpisodeInvolvementSummary {
    pub id: String,
    pub label: String,
    pub status: String,
}

/// This work's role within one audit episode. Role is episode-scoped, never a
/// global property of the scholarly work.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkRoleInAudit {
    DirectSubject,
    Evidence {
        artifact_id: String,
        bearing: ArtifactBearing,
        warrant_count: i64,
        review_count: i64,
    },
    Background {
        artifact_id: String,
        bearing: ArtifactBearing,
        warrant_count: i64,
        review_count: i64,
    },
}

/// Public audit state for one episode involvement. Kept with the involvement
/// so cards can answer "what is being audited here?" without another round of
/// frontend inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvolvementAuditState {
    pub status_label: String,
    pub tuple: Option<EvalTuple>,
    pub latest_synthesis: Option<SynthesisReview>,
    pub element_review_count: i64,
    pub synthesis_review_count: i64,
    pub challenge_count: i64,
}

/// One audit episode a scholarly work is involved in, for the paper page's
/// "every audit that involves this paper" listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAuditInvolvement {
    pub episode: AuditEpisodeInvolvementSummary,
    pub audit_target: AuditTargetSummary,
    pub work_role: WorkRoleInAudit,
    pub audit_state: InvolvementAuditState,
}

/// How an audit target is functioning in the public "claim audits" index.
/// A paper can be claim-bearing when it is the direct audit target, while a
/// scoped claim is claim-bearing because the claim is explicit in the subject.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClaimAuditRole {
    ExplicitScopedClaim,
    WorkAsClaim,
}

/// The scholarly work behind a direct paper-as-claim entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimAuditScholarlyObjectSummary {
    pub id: String,
    pub object_type: String,
    pub title: String,
    pub authors: Vec<String>,
    pub source_name: String,
    pub publication_year: Option<i32>,
    pub canonical_url: String,
}

/// Unified public index entry for claim audits. This intentionally includes
/// both explicit scoped-claim subjects and scholarly works that are directly
/// serving as the auditable claim object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimAuditIndexEntry {
    pub subject: AuditTargetSummary,
    pub claim_role: ClaimAuditRole,
    pub primary_episode: AuditEpisodeInvolvementSummary,
    pub audit_state: InvolvementAuditState,
    pub scholarly_object: Option<ClaimAuditScholarlyObjectSummary>,
    pub evidence_artifact_count: i64,
}
