use csqd_domain::{
    ArticleAccessSummary, ArticleDisplayStrategy, ArticleRightsStatus,
    ExternalArticleLocationSummary, ExternalArticleLocationType,
};
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

pub async fn find_for_scholarly_object(
    db: &PgPool,
    object_id: &str,
) -> Result<ArticleAccessSummary, RepositoryError> {
    let object_row = sqlx::query(
        r#"
        SELECT
            so.id::text AS scholarly_object_id,
            so.doi,
            COALESCE(j.name, 'Unknown source') AS source_name,
            so.publication_date::text AS publication_date,
            so.license,
            so.canonical_url,
            so.native_display_permitted
        FROM scholarly_objects so
        LEFT JOIN journals j ON j.id = so.journal_id
        WHERE so.id::text = $1
        "#,
    )
    .bind(object_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "scholarly_object",
        id: object_id.to_string(),
    })?;

    let location_rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            location_type,
            url,
            license,
            is_canonical
        FROM external_article_locations
        WHERE scholarly_object_id::text = $1
        ORDER BY is_canonical DESC, created_at ASC
        "#,
    )
    .bind(object_id)
    .fetch_all(db)
    .await?;

    row_to_access_summary(object_row, location_rows)
}

fn row_to_access_summary(
    row: PgRow,
    location_rows: Vec<PgRow>,
) -> Result<ArticleAccessSummary, RepositoryError> {
    let native_display_permitted = row.get("native_display_permitted");
    let canonical_url: String = row.get("canonical_url");
    let external_locations = location_rows
        .into_iter()
        .map(row_to_external_location)
        .collect::<Result<Vec<_>, _>>()?;
    let canonical_location = external_locations
        .iter()
        .find(|location| location.is_canonical)
        .cloned();
    let preferred_source = canonical_location
        .clone()
        .or_else(|| preferred_external_location(&external_locations));
    let display_strategy = display_strategy(
        native_display_permitted,
        preferred_source.as_ref(),
        canonical_url.as_str(),
    );
    let rights_status = rights_status(
        native_display_permitted,
        preferred_source.as_ref(),
        canonical_url.as_str(),
    );

    Ok(ArticleAccessSummary {
        scholarly_object_id: row.get("scholarly_object_id"),
        doi: row.get("doi"),
        source_name: row.get("source_name"),
        publication_date: row.get("publication_date"),
        license: row.get("license"),
        canonical_url,
        display_strategy,
        rights_status,
        native_display_permitted,
        canonical_location,
        preferred_source,
        external_locations,
    })
}

fn row_to_external_location(row: PgRow) -> Result<ExternalArticleLocationSummary, RepositoryError> {
    let location_type: String = row.get("location_type");

    Ok(ExternalArticleLocationSummary {
        id: row.get("id"),
        location_type: ExternalArticleLocationType::try_from(location_type.as_str())
            .map_err(RepositoryError::Domain)?,
        url: row.get("url"),
        license: row.get("license"),
        is_canonical: row.get("is_canonical"),
    })
}

fn preferred_external_location(
    locations: &[ExternalArticleLocationSummary],
) -> Option<ExternalArticleLocationSummary> {
    locations
        .iter()
        .min_by_key(|location| location_priority(location))
        .cloned()
}

fn location_priority(location: &ExternalArticleLocationSummary) -> u8 {
    match &location.location_type {
        ExternalArticleLocationType::Publisher => 0,
        ExternalArticleLocationType::Repository => 1,
        ExternalArticleLocationType::LandingPage => 2,
        ExternalArticleLocationType::FullText => 3,
        ExternalArticleLocationType::Pdf => 4,
    }
}

fn display_strategy(
    native_display_permitted: bool,
    preferred_source: Option<&ExternalArticleLocationSummary>,
    canonical_url: &str,
) -> ArticleDisplayStrategy {
    if native_display_permitted {
        return ArticleDisplayStrategy::PermittedNativeDisplay;
    }

    match preferred_source.map(|location| &location.location_type) {
        Some(ExternalArticleLocationType::Publisher) => {
            ArticleDisplayStrategy::ExternalPublisherPage
        }
        Some(ExternalArticleLocationType::Repository) => {
            ArticleDisplayStrategy::ExternalRepositoryPage
        }
        Some(ExternalArticleLocationType::LandingPage) => {
            ArticleDisplayStrategy::ExternalLandingPage
        }
        Some(ExternalArticleLocationType::FullText) => ArticleDisplayStrategy::ExternalFullText,
        Some(ExternalArticleLocationType::Pdf) if native_display_permitted => {
            ArticleDisplayStrategy::ExternalPdf
        }
        Some(ExternalArticleLocationType::Pdf) => ArticleDisplayStrategy::ExternalSource,
        None if canonical_url.trim().is_empty() => ArticleDisplayStrategy::Unavailable,
        None => ArticleDisplayStrategy::Unknown,
    }
}

fn rights_status(
    native_display_permitted: bool,
    preferred_source: Option<&ExternalArticleLocationSummary>,
    canonical_url: &str,
) -> ArticleRightsStatus {
    if native_display_permitted {
        ArticleRightsStatus::NativeDisplayPermitted
    } else if preferred_source.is_some() || !canonical_url.trim().is_empty() {
        ArticleRightsStatus::ExternalSourceOnly
    } else if canonical_url.trim().is_empty() {
        ArticleRightsStatus::SourceUnavailable
    } else {
        ArticleRightsStatus::Unknown
    }
}
