use csqd_academic_adapter::{ArticleRetrievalResult, ArticleRetrievalSource, ArticleVersionKind};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::{env, time::Duration};

use super::{access_policy, article_access, article_versions, audit_subjects, RepositoryError};

const CROSSREF_WORKS_URL: &str = "https://api.crossref.org/works";
const UNPAYWALL_URL: &str = "https://api.unpaywall.org/v2";

#[derive(Debug)]
struct DoiEntry {
    doi: String,
    title: String,
    authors: Vec<String>,
    abstract_text: Option<String>,
    source_name: String,
    publisher: Option<String>,
    publication_date: Option<String>,
    object_type: String,
    canonical_url: String,
    license: Option<String>,
    publisher_url: Option<String>,
    oa_landing_url: Option<String>,
    oa_pdf_url: Option<String>,
    is_oa: bool,
}

pub async fn retrieve_doi(
    db: &PgPool,
    query: &str,
) -> Result<ArticleRetrievalResult, RepositoryError> {
    let doi = doi_from_query(query).ok_or_else(|| {
        RepositoryError::Domain("article retrieval query does not contain a DOI".to_string())
    })?;

    let entry = fetch_doi_entry(&doi).await?;
    let (scholarly_object_id, was_created) = upsert_doi_entry(db, &entry).await?;
    let version_kind = version_kind_for_entry(&entry);
    let work_group = article_versions::ensure_version_group(
        db,
        &scholarly_object_id,
        &entry.title,
        version_kind.clone(),
        "doi",
        &entry.doi,
    )
    .await?;
    let article_access =
        article_access::find_for_scholarly_object(db, &scholarly_object_id).await?;
    let audit_subject_id =
        audit_subjects::ensure_academic_for_scholarly_object(db, &scholarly_object_id).await?;

    Ok(ArticleRetrievalResult {
        source: ArticleRetrievalSource::Doi,
        source_identifier: entry.doi.clone(),
        work_group,
        version_kind,
        scholarly_object_id,
        audit_subject_id,
        title: entry.title,
        authors: entry.authors,
        abstract_text: entry.abstract_text,
        canonical_url: entry.canonical_url,
        pdf_url: entry.oa_pdf_url,
        doi: Some(doi),
        was_created,
        article_access,
    })
}

fn version_kind_for_entry(entry: &DoiEntry) -> ArticleVersionKind {
    match entry.object_type.as_str() {
        "article" => ArticleVersionKind::Publisher,
        "preprint" => ArticleVersionKind::Preprint,
        _ => ArticleVersionKind::Unknown,
    }
}

async fn fetch_doi_entry(doi: &str) -> Result<DoiEntry, RepositoryError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| {
            RepositoryError::Domain(format!("DOI HTTP client setup failed: {error}"))
        })?;
    let crossref = fetch_crossref_work(&client, doi).await?;
    let unpaywall = fetch_unpaywall_work(&client, doi).await.ok();

    crossref_and_unpaywall_to_entry(doi, crossref, unpaywall)
}

async fn fetch_crossref_work(
    client: &reqwest::Client,
    doi: &str,
) -> Result<Value, RepositoryError> {
    let url = format!("{CROSSREF_WORKS_URL}/{doi}");
    let response = client
        .get(url)
        .query(&[("mailto", contact_email())])
        .header(
            "User-Agent",
            format!("C-SQD local development (mailto:{})", contact_email()),
        )
        .send()
        .await
        .map_err(|error| RepositoryError::Domain(format!("Crossref request failed: {error}")))?;

    if !response.status().is_success() {
        return Err(RepositoryError::Domain(format!(
            "Crossref request failed with status {}",
            response.status()
        )));
    }

    let body = response
        .json::<Value>()
        .await
        .map_err(|error| RepositoryError::Domain(format!("Crossref JSON parse failed: {error}")))?;

    body.get("message").cloned().ok_or_else(|| {
        RepositoryError::Domain("Crossref response did not include a message".to_string())
    })
}

async fn fetch_unpaywall_work(
    client: &reqwest::Client,
    doi: &str,
) -> Result<Value, RepositoryError> {
    let url = format!("{UNPAYWALL_URL}/{doi}");
    let response = client
        .get(url)
        .query(&[("email", contact_email())])
        .header(
            "User-Agent",
            format!("C-SQD local development (mailto:{})", contact_email()),
        )
        .send()
        .await
        .map_err(|error| RepositoryError::Domain(format!("Unpaywall request failed: {error}")))?;

    if !response.status().is_success() {
        return Err(RepositoryError::Domain(format!(
            "Unpaywall request failed with status {}",
            response.status()
        )));
    }

    response
        .json::<Value>()
        .await
        .map_err(|error| RepositoryError::Domain(format!("Unpaywall JSON parse failed: {error}")))
}

fn crossref_and_unpaywall_to_entry(
    requested_doi: &str,
    crossref: Value,
    unpaywall: Option<Value>,
) -> Result<DoiEntry, RepositoryError> {
    let doi = string_field(&crossref, "DOI").unwrap_or_else(|| requested_doi.to_string());
    let title = string_array_first(&crossref, "title").ok_or_else(|| {
        RepositoryError::Domain("Crossref response did not include a title".to_string())
    })?;
    let authors = crossref_authors(&crossref);
    let abstract_text = string_field(&crossref, "abstract").map(strip_simple_html);
    let source_name = string_array_first(&crossref, "container-title")
        .or_else(|| string_field(&crossref, "publisher"))
        .unwrap_or_else(|| "Unknown DOI source".to_string());
    let publisher = string_field(&crossref, "publisher");
    let publication_date = crossref_date(&crossref);
    let object_type = object_type_from_crossref(&crossref);
    let publisher_url = string_field(&crossref, "URL");
    let canonical_url = publisher_url
        .clone()
        .unwrap_or_else(|| format!("https://doi.org/{doi}"));
    let crossref_license = crossref_license(&crossref);
    let unpaywall_license = unpaywall
        .as_ref()
        .and_then(|value| best_oa_location(value))
        .and_then(|location| string_field(location, "license"));
    let license = unpaywall_license.or(crossref_license);
    let oa_landing_url = unpaywall
        .as_ref()
        .and_then(|value| best_oa_location(value))
        .and_then(|location| {
            string_field(location, "url_for_landing_page").or_else(|| string_field(location, "url"))
        });
    let oa_pdf_url = unpaywall
        .as_ref()
        .and_then(|value| best_oa_location(value))
        .and_then(|location| string_field(location, "url_for_pdf"));
    let is_oa = unpaywall
        .as_ref()
        .and_then(|value| value.get("is_oa"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(DoiEntry {
        doi,
        title,
        authors,
        abstract_text,
        source_name,
        publisher,
        publication_date,
        object_type,
        canonical_url,
        license,
        publisher_url,
        oa_landing_url,
        oa_pdf_url,
        is_oa,
    })
}

async fn upsert_doi_entry(
    db: &PgPool,
    entry: &DoiEntry,
) -> Result<(String, bool), RepositoryError> {
    let journal_id = ensure_source(db, entry).await?;
    let existing_id = existing_scholarly_object_id(db, entry).await?;
    let was_created = existing_id.is_none();
    let authors = json!(entry.authors);
    let metadata_provenance = json!({
        "source": "doi",
        "doi": entry.doi,
        "is_oa": entry.is_oa,
    });
    let native_display_permitted = access_policy::doi_native_display_permitted(
        entry.is_oa,
        entry.license.as_deref(),
        entry.oa_pdf_url.as_deref(),
    );

    let scholarly_object_id = if let Some(existing_id) = existing_id {
        sqlx::query(
            r#"
            UPDATE scholarly_objects
            SET
                object_type = $2,
                doi = $3,
                title = $4,
                authors = $5,
                abstract = $6,
                journal_id = $7::uuid,
                publication_date = $8::date,
                license = $9,
                canonical_url = $10,
                metadata_provenance = $11,
                native_display_permitted = $12,
                updated_at = now()
            WHERE id::text = $1
            "#,
        )
        .bind(&existing_id)
        .bind(&entry.object_type)
        .bind(&entry.doi)
        .bind(&entry.title)
        .bind(&authors)
        .bind(&entry.abstract_text)
        .bind(&journal_id)
        .bind(&entry.publication_date)
        .bind(&entry.license)
        .bind(&entry.canonical_url)
        .bind(&metadata_provenance)
        .bind(native_display_permitted)
        .execute(db)
        .await?;

        existing_id
    } else {
        let row = sqlx::query(
            r#"
            INSERT INTO scholarly_objects (
                object_type,
                doi,
                title,
                authors,
                abstract,
                journal_id,
                publication_date,
                license,
                canonical_url,
                metadata_provenance,
                native_display_permitted
            )
            VALUES ($1, $2, $3, $4, $5, $6::uuid, $7::date, $8, $9, $10, $11)
            RETURNING id::text AS id
            "#,
        )
        .bind(&entry.object_type)
        .bind(&entry.doi)
        .bind(&entry.title)
        .bind(&authors)
        .bind(&entry.abstract_text)
        .bind(&journal_id)
        .bind(&entry.publication_date)
        .bind(&entry.license)
        .bind(&entry.canonical_url)
        .bind(&metadata_provenance)
        .bind(native_display_permitted)
        .fetch_one(db)
        .await?;

        row.get("id")
    };

    upsert_external_location(
        db,
        &scholarly_object_id,
        "publisher",
        entry
            .publisher_url
            .as_deref()
            .unwrap_or(&entry.canonical_url),
        entry.license.as_deref(),
        true,
        "doi",
    )
    .await?;

    if let Some(oa_landing_url) = &entry.oa_landing_url {
        upsert_external_location(
            db,
            &scholarly_object_id,
            "repository",
            oa_landing_url,
            entry.license.as_deref(),
            false,
            "unpaywall",
        )
        .await?;
    }

    if let Some(oa_pdf_url) = &entry.oa_pdf_url {
        upsert_external_location(
            db,
            &scholarly_object_id,
            "pdf",
            oa_pdf_url,
            entry.license.as_deref(),
            false,
            "unpaywall",
        )
        .await?;
    }

    upsert_search_projection(db, &scholarly_object_id, entry).await?;
    audit_subjects::ensure_academic_for_scholarly_object(db, &scholarly_object_id).await?;

    Ok((scholarly_object_id, was_created))
}

async fn ensure_source(db: &PgPool, entry: &DoiEntry) -> Result<String, RepositoryError> {
    if let Some(row) = sqlx::query(
        r#"
        SELECT id::text AS id
        FROM journals
        WHERE name = $1
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(&entry.source_name)
    .fetch_optional(db)
    .await?
    {
        return Ok(row.get("id"));
    }

    let row = sqlx::query(
        r#"
        INSERT INTO journals (name, publisher, source_classification)
        VALUES ($1, $2, 'doi')
        RETURNING id::text AS id
        "#,
    )
    .bind(&entry.source_name)
    .bind(&entry.publisher)
    .fetch_one(db)
    .await?;

    Ok(row.get("id"))
}

async fn existing_scholarly_object_id(
    db: &PgPool,
    entry: &DoiEntry,
) -> Result<Option<String>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id
        FROM scholarly_objects
        WHERE doi = $1
           OR canonical_url = $2
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&entry.doi)
    .bind(&entry.canonical_url)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|row| row.get("id")))
}

async fn upsert_external_location(
    db: &PgPool,
    scholarly_object_id: &str,
    location_type: &str,
    url: &str,
    license: Option<&str>,
    is_canonical: bool,
    source: &str,
) -> Result<(), RepositoryError> {
    let existing = sqlx::query(
        r#"
        SELECT id
        FROM external_article_locations
        WHERE scholarly_object_id::text = $1
          AND url = $2
        LIMIT 1
        "#,
    )
    .bind(scholarly_object_id)
    .bind(url)
    .fetch_optional(db)
    .await?;

    if existing.is_some() {
        sqlx::query(
            r#"
            UPDATE external_article_locations
            SET
                location_type = $3,
                license = $4,
                is_canonical = $5
            WHERE scholarly_object_id::text = $1
              AND url = $2
            "#,
        )
        .bind(scholarly_object_id)
        .bind(url)
        .bind(location_type)
        .bind(license)
        .bind(is_canonical)
        .execute(db)
        .await?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO external_article_locations (
                scholarly_object_id,
                location_type,
                url,
                license,
                is_canonical,
                provenance
            )
            VALUES ($1::uuid, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(scholarly_object_id)
        .bind(location_type)
        .bind(url)
        .bind(license)
        .bind(is_canonical)
        .bind(json!({ "source": source }))
        .execute(db)
        .await?;
    }

    Ok(())
}

async fn upsert_search_projection(
    db: &PgPool,
    scholarly_object_id: &str,
    entry: &DoiEntry,
) -> Result<(), RepositoryError> {
    let search_text = format!(
        "{} {} {} {} {}",
        entry.title,
        entry.authors.join(" "),
        entry.abstract_text.clone().unwrap_or_default(),
        entry.source_name,
        entry.doi,
    );

    sqlx::query(
        r#"
        INSERT INTO scholarly_object_search (scholarly_object_id, search_text)
        VALUES ($1::uuid, $2)
        ON CONFLICT (scholarly_object_id) DO UPDATE SET
            search_text = EXCLUDED.search_text,
            updated_at = now()
        "#,
    )
    .bind(scholarly_object_id)
    .bind(search_text)
    .execute(db)
    .await?;

    Ok(())
}

fn doi_from_query(query: &str) -> Option<String> {
    let trimmed = query.trim().trim_end_matches('/');
    let candidate = if let Some((_, rest)) = trimmed.split_once("doi.org/") {
        rest
    } else if let Some((_, rest)) = trimmed.split_once("dx.doi.org/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("doi:") {
        rest
    } else {
        trimmed
    };
    let candidate = candidate
        .split(['?', '#', ' '])
        .next()
        .unwrap_or(candidate)
        .trim()
        .trim_matches(|character: char| matches!(character, '.' | ',' | ';' | ')' | ']'));

    if candidate.starts_with("10.") && candidate.contains('/') {
        Some(candidate.to_ascii_lowercase())
    } else {
        None
    }
}

pub(crate) fn contact_email() -> String {
    env::var("CSQD_CONTACT_EMAIL")
        .or_else(|_| env::var("CSQD_UNPAYWALL_EMAIL"))
        .unwrap_or_else(|_| "admin@csqd.local".to_string())
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn string_array_first(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn crossref_authors(value: &Value) -> Vec<String> {
    value
        .get("author")
        .and_then(Value::as_array)
        .map(|authors| {
            authors
                .iter()
                .filter_map(|author| {
                    let given = string_field(author, "given");
                    let family = string_field(author, "family");
                    match (given, family) {
                        (Some(given), Some(family)) => Some(format!("{given} {family}")),
                        (None, Some(family)) => Some(family),
                        (Some(given), None) => Some(given),
                        (None, None) => string_field(author, "name"),
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn crossref_date(value: &Value) -> Option<String> {
    ["published-print", "published-online", "published", "issued"]
        .iter()
        .find_map(|key| {
            let parts = value
                .get(key)
                .and_then(|date| date.get("date-parts"))
                .and_then(Value::as_array)
                .and_then(|rows| rows.first())
                .and_then(Value::as_array)?;
            let year = parts.first()?.as_i64()?;
            let month = parts.get(1).and_then(Value::as_i64).unwrap_or(1);
            let day = parts.get(2).and_then(Value::as_i64).unwrap_or(1);

            Some(format!("{year:04}-{month:02}-{day:02}"))
        })
}

fn crossref_license(value: &Value) -> Option<String> {
    value
        .get("license")
        .and_then(Value::as_array)
        .and_then(|licenses| licenses.first())
        .and_then(|license| string_field(license, "URL"))
}

fn object_type_from_crossref(value: &Value) -> String {
    match string_field(value, "type").as_deref() {
        Some("posted-content") => "preprint",
        Some("dataset") => "dataset",
        Some("report") | Some("report-component") => "report",
        _ => "article",
    }
    .to_string()
}

fn best_oa_location(value: &Value) -> Option<&Value> {
    value
        .get("best_oa_location")
        .filter(|location| !location.is_null())
}

fn strip_simple_html(value: String) -> String {
    let mut output = String::new();
    let mut in_tag = false;

    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }

    output
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{crossref_and_unpaywall_to_entry, doi_from_query};

    #[test]
    fn extracts_doi_from_common_inputs() {
        assert_eq!(
            doi_from_query("10.1038/nature12373"),
            Some("10.1038/nature12373".to_string())
        );
        assert_eq!(
            doi_from_query("https://doi.org/10.1038/nature12373"),
            Some("10.1038/nature12373".to_string())
        );
        assert_eq!(
            doi_from_query("doi:10.1126/science.169.3946.635"),
            Some("10.1126/science.169.3946.635".to_string())
        );
        assert_eq!(doi_from_query("not a doi"), None);
    }

    #[test]
    fn combines_crossref_and_unpaywall_metadata() {
        let crossref = json!({
            "DOI": "10.1038/nature12373",
            "title": ["A demo article"],
            "author": [
                { "given": "Ada", "family": "Lovelace" },
                { "name": "Research Group" }
            ],
            "container-title": ["Nature"],
            "publisher": "Springer Nature",
            "published-online": { "date-parts": [[2013, 9, 12]] },
            "type": "journal-article",
            "URL": "https://doi.org/10.1038/nature12373",
            "abstract": "<jats:p>Structured &amp; useful.</jats:p>"
        });
        let unpaywall = json!({
            "is_oa": true,
            "best_oa_location": {
                "url_for_landing_page": "https://example.org/article",
                "url_for_pdf": "https://example.org/article.pdf",
                "license": "cc-by"
            }
        });

        let entry =
            crossref_and_unpaywall_to_entry("10.1038/nature12373", crossref, Some(unpaywall))
                .expect("entry should combine");

        assert_eq!(entry.title, "A demo article");
        assert_eq!(entry.authors, vec!["Ada Lovelace", "Research Group"]);
        assert_eq!(entry.publication_date, Some("2013-09-12".to_string()));
        assert_eq!(entry.object_type, "article");
        assert_eq!(
            entry.oa_pdf_url,
            Some("https://example.org/article.pdf".to_string())
        );
        assert!(entry.is_oa);
    }
}
