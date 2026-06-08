use serde::{Deserialize, Serialize};

use crate::article_access::ArticleAccessSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleRetrievalResult {
    pub source: ArticleRetrievalSource,
    pub source_identifier: String,
    pub work_group: ArticleVersionGroupSummary,
    pub version_kind: ArticleVersionKind,
    pub scholarly_object_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: Option<String>,
    pub canonical_url: String,
    pub pdf_url: Option<String>,
    pub doi: Option<String>,
    pub was_created: bool,
    pub article_access: ArticleAccessSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleRetrievalSet {
    pub results: Vec<ArticleRetrievalResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleRetrievalSource {
    Arxiv,
    Doi,
    Pmc,
    Pubmed,
}

impl ArticleRetrievalSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Arxiv => "arxiv",
            Self::Doi => "doi",
            Self::Pmc => "pmc",
            Self::Pubmed => "pubmed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleVersionGroupSummary {
    pub id: String,
    pub title: String,
    pub primary_scholarly_object_id: Option<String>,
    pub version_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleVersionKind {
    Publisher,
    Preprint,
    Repository,
    Unknown,
}

impl ArticleVersionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Publisher => "publisher",
            Self::Preprint => "preprint",
            Self::Repository => "repository",
            Self::Unknown => "unknown",
        }
    }
}

impl TryFrom<&str> for ArticleVersionKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "publisher" => Ok(Self::Publisher),
            "preprint" => Ok(Self::Preprint),
            "repository" => Ok(Self::Repository),
            "unknown" => Ok(Self::Unknown),
            other => Err(format!("unknown article version kind: {other}")),
        }
    }
}
