use serde::{Deserialize, Serialize};

use crate::article_retrieval::{ArticleVersionGroupSummary, ArticleVersionKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScholarlyObjectSummary {
    pub id: String,
    pub object_type: ScholarlyObjectType,
    pub work_group: Option<ArticleVersionGroupSummary>,
    pub version_kind: ArticleVersionKind,
    pub title: String,
    pub authors: Vec<String>,
    pub source_name: String,
    pub publication_year: Option<i32>,
    pub canonical_url: String,
    pub license: Option<String>,
    pub review_status: ReviewStatus,
    pub evaluation_fact_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemSummary {
    pub id: String,
    pub user_id: String,
    pub audit_object_id: String,
    pub added_reason: LibraryAddedReason,
    pub added_at: String,
    pub scholarly_object: ScholarlyObjectSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryAddedReason {
    Manual,
    ReviewCreated,
    AssignmentAccepted,
    Imported,
    AdminSeeded,
}

impl TryFrom<&str> for LibraryAddedReason {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "manual" => Ok(Self::Manual),
            "review_created" => Ok(Self::ReviewCreated),
            "assignment_accepted" => Ok(Self::AssignmentAccepted),
            "imported" => Ok(Self::Imported),
            "admin_seeded" => Ok(Self::AdminSeeded),
            other => Err(format!("unknown library added reason: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScholarlyObjectDetail {
    pub id: String,
    pub object_type: ScholarlyObjectType,
    pub work_group: Option<ArticleVersionGroupSummary>,
    pub version_kind: ArticleVersionKind,
    pub versions: Vec<ArticleVersionSummary>,
    pub doi: Option<String>,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: Option<String>,
    pub source_name: String,
    pub publication_date: Option<String>,
    pub publication_year: Option<i32>,
    pub canonical_url: String,
    pub license: Option<String>,
    pub native_display_permitted: bool,
    pub review_status: ReviewStatus,
    pub evaluation_fact_count: i64,
    pub external_locations: Vec<ExternalArticleLocationSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleVersionSummary {
    pub scholarly_object_id: String,
    pub title: String,
    pub version_kind: ArticleVersionKind,
    pub doi: Option<String>,
    pub source_name: String,
    pub canonical_url: String,
    pub native_display_permitted: bool,
    pub is_current: bool,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalArticleLocationSummary {
    pub id: String,
    pub location_type: ExternalArticleLocationType,
    pub url: String,
    pub license: Option<String>,
    pub is_canonical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalArticleLocationType {
    Publisher,
    LandingPage,
    FullText,
    Pdf,
    Repository,
}

impl TryFrom<&str> for ExternalArticleLocationType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "publisher" => Ok(Self::Publisher),
            "landing_page" => Ok(Self::LandingPage),
            "full_text" => Ok(Self::FullText),
            "pdf" => Ok(Self::Pdf),
            "repository" => Ok(Self::Repository),
            other => Err(format!("unknown external article location type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScholarlyObjectType {
    Article,
    Preprint,
    Dataset,
    Software,
    Protocol,
    Report,
}

impl TryFrom<&str> for ScholarlyObjectType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "article" => Ok(Self::Article),
            "preprint" => Ok(Self::Preprint),
            "dataset" => Ok(Self::Dataset),
            "software" => Ok(Self::Software),
            "protocol" => Ok(Self::Protocol),
            "report" => Ok(Self::Report),
            other => Err(format!("unknown scholarly object type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    NotAssigned,
    Assigned,
    InReview,
    Submitted,
    Published,
}

impl TryFrom<&str> for ReviewStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not_assigned" => Ok(Self::NotAssigned),
            "assigned" => Ok(Self::Assigned),
            "in_review" => Ok(Self::InReview),
            "submitted" => Ok(Self::Submitted),
            "published" => Ok(Self::Published),
            other => Err(format!("unknown review status: {other}")),
        }
    }
}
