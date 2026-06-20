use serde::{Deserialize, Serialize};

use crate::scholarly_object::ExternalArticleLocationSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleAccessSummary {
    pub scholarly_object_id: String,
    pub doi: Option<String>,
    pub source_name: String,
    pub publication_date: Option<String>,
    pub license: Option<String>,
    pub canonical_url: String,
    pub display_strategy: ArticleDisplayStrategy,
    pub rights_status: ArticleRightsStatus,
    pub native_display_permitted: bool,
    pub canonical_location: Option<ExternalArticleLocationSummary>,
    pub preferred_source: Option<ExternalArticleLocationSummary>,
    pub external_locations: Vec<ExternalArticleLocationSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleDisplayStrategy {
    PermittedNativeDisplay,
    ExternalPublisherPage,
    ExternalRepositoryPage,
    ExternalLandingPage,
    ExternalFullText,
    ExternalPdf,
    ExternalSource,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleRightsStatus {
    NativeDisplayPermitted,
    ExternalSourceOnly,
    SourceUnavailable,
    Unknown,
}
