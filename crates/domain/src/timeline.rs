//! Heterogeneous audit timeline, the consumer of the `Temporal` trait.

use serde::{Deserialize, Serialize};

use crate::audit_episode::{EpisodeMembership, EpisodeRelation};
use crate::common::{Temporal, Timestamp};
use crate::fact::Fact;
use crate::synthesis_review::{SynthesisReview, SynthesisReviewRelation};

/// One entry in an interleaved audit timeline. The variants deliberately span
/// entity types; `Temporal` supplies the single sort key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "entry_type")]
pub enum TimelineEntry {
    Fact { fact: Fact },
    Membership { membership: EpisodeMembership },
    EpisodeRelation { relation: EpisodeRelation },
    SynthesisReview { review: SynthesisReview },
    SynthesisRelation { relation: SynthesisReviewRelation },
}

impl Temporal for TimelineEntry {
    fn temporal_anchor(&self) -> Timestamp {
        match self {
            Self::Fact { fact } => fact.temporal_anchor(),
            Self::Membership { membership } => membership.temporal_anchor(),
            Self::EpisodeRelation { relation } => relation.temporal_anchor(),
            Self::SynthesisReview { review } => review.temporal_anchor(),
            Self::SynthesisRelation { relation } => relation.temporal_anchor(),
        }
    }
}

/// Sorts entries ascending by temporal anchor (oldest first).
pub fn sort_timeline(entries: &mut [TimelineEntry]) {
    entries.sort_by_key(|entry| entry.temporal_anchor());
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::audit_episode::{EpisodeMembership, EpisodeMembershipStatus, FactRole};
    use crate::common::{Principal, Temporal};
    use crate::ids::{AuditEpisodeId, FactId, MembershipId};

    use super::{sort_timeline, TimelineEntry};

    #[test]
    fn sorts_entries_by_temporal_anchor() {
        let make_membership = |id: &str, seconds: i64| EpisodeMembership {
            id: MembershipId::new(id),
            fact_id: FactId::new("fact-1"),
            episode_id: AuditEpisodeId::new("episode-1"),
            role: FactRole::Administrative,
            asserted_by: Principal::Platform,
            asserted_at: Utc.timestamp_opt(seconds, 0).unwrap(),
            status: EpisodeMembershipStatus::Active,
        };
        let mut entries = vec![
            TimelineEntry::Membership {
                membership: make_membership("m2", 200),
            },
            TimelineEntry::Membership {
                membership: make_membership("m1", 100),
            },
        ];

        sort_timeline(&mut entries);

        let anchors: Vec<i64> = entries
            .iter()
            .map(|entry| entry.temporal_anchor().timestamp())
            .collect();

        assert_eq!(anchors, vec![100, 200]);
    }
}
