//! Academic Publishing adapter over the FEN substrate.
//!
//! Everything scholarly lives here: scholarly objects, article access and
//! retrieval, version groups, library and CRWE-browse view types. This crate
//! depends on `csqd-domain` (the substrate) and never the reverse — domains
//! are lenses over one substrate, and a future clinical-trials adapter would
//! sit beside this crate, not inside the domain.

pub mod article_access;
pub mod article_retrieval;
pub mod scholarly_object;

pub use article_access::{ArticleAccessSummary, ArticleDisplayStrategy, ArticleRightsStatus};
pub use article_retrieval::{
    ArticleRetrievalResult, ArticleRetrievalSet, ArticleRetrievalSource,
    ArticleVersionGroupSummary, ArticleVersionKind,
};
pub use scholarly_object::{
    ArticleVersionSummary, AuditWorkStatus, ExternalArticleLocationSummary,
    ExternalArticleLocationType, LibraryAddedReason, LibraryItemSummary, ProblemAreaRelevance,
    ProblemAreaWorkSummary, ScholarlyObjectDetail, ScholarlyObjectSummary, ScholarlyObjectType,
};
