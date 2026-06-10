use csqd_domain::{ArticleRetrievalResult, ArticleRetrievalSource, ArticleVersionKind};
use roxmltree::{Document, Node};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::time::Duration;

use super::{access_policy, article_access, article_versions, audit_subjects, RepositoryError};

const ARXIV_API_URL: &str = "http://export.arxiv.org/api/query";
const ARXIV_JOURNAL_NAME: &str = "arXiv";

#[derive(Debug)]
struct ArxivEntry {
    arxiv_id: String,
    title: String,
    authors: Vec<String>,
    abstract_text: Option<String>,
    published_date: Option<String>,
    canonical_url: String,
    pdf_url: Option<String>,
    doi: Option<String>,
    license: Option<String>,
    categories: Vec<String>,
}

pub async fn retrieve_arxiv(
    db: &PgPool,
    query: &str,
) -> Result<ArticleRetrievalResult, RepositoryError> {
    let cleaned_query = query.trim();

    if cleaned_query.is_empty() {
        return Err(RepositoryError::Domain(
            "article retrieval query cannot be empty".to_string(),
        ));
    }

    let entry = fetch_arxiv_entry(cleaned_query).await?;
    let (scholarly_object_id, was_created) = upsert_arxiv_entry(db, &entry).await?;
    let version_kind = ArticleVersionKind::Preprint;
    let work_group = article_versions::ensure_version_group(
        db,
        &scholarly_object_id,
        &entry.title,
        version_kind.clone(),
        "arxiv",
        &entry.arxiv_id,
    )
    .await?;
    let article_access =
        article_access::find_for_scholarly_object(db, &scholarly_object_id).await?;
    let audit_subject_id =
        audit_subjects::ensure_academic_for_scholarly_object(db, &scholarly_object_id).await?;

    Ok(ArticleRetrievalResult {
        source: ArticleRetrievalSource::Arxiv,
        source_identifier: entry.arxiv_id,
        work_group,
        version_kind,
        scholarly_object_id,
        audit_subject_id,
        title: entry.title,
        authors: entry.authors,
        abstract_text: entry.abstract_text,
        canonical_url: entry.canonical_url,
        pdf_url: entry.pdf_url,
        doi: entry.doi,
        was_created,
        article_access,
    })
}

async fn fetch_arxiv_entry(query: &str) -> Result<ArxivEntry, RepositoryError> {
    let exact_arxiv_id = arxiv_identifier_from_query(query);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| {
            RepositoryError::Domain(format!("arXiv HTTP client setup failed: {error}"))
        })?;

    if let Some(arxiv_id) = exact_arxiv_id.as_deref() {
        if let Ok(entry) = fetch_arxiv_abs_entry(&client, arxiv_id).await {
            return Ok(entry);
        }
    }

    let mut request = client.get(ARXIV_API_URL).header(
        "User-Agent",
        "C-SQD local development (mailto:admin@csqd.local)",
    );

    if let Some(arxiv_id) = exact_arxiv_id.as_deref() {
        request = request.query(&[("id_list", arxiv_id), ("max_results", "1")]);
    } else {
        let search_query = format!("all:{query}");
        request = request.query(&[
            ("search_query", search_query.as_str()),
            ("start", "0"),
            ("max_results", "1"),
        ]);
    }

    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            if let Some(arxiv_id) = exact_arxiv_id.as_deref() {
                return fetch_arxiv_abs_entry(&client, arxiv_id).await;
            }

            return Err(RepositoryError::Domain(format!(
                "arXiv request failed: {error}"
            )));
        }
    };

    if !response.status().is_success() {
        if let Some(arxiv_id) = exact_arxiv_id.as_deref() {
            return fetch_arxiv_abs_entry(&client, arxiv_id).await;
        }

        return Err(RepositoryError::Domain(format!(
            "arXiv request failed with status {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|error| RepositoryError::Domain(format!("arXiv response read failed: {error}")))?;

    parse_first_arxiv_entry(&body).ok_or_else(|| RepositoryError::NotFound {
        entity: "arxiv_article",
        id: query.to_string(),
    })
}

async fn fetch_arxiv_abs_entry(
    client: &reqwest::Client,
    arxiv_id: &str,
) -> Result<ArxivEntry, RepositoryError> {
    let canonical_id = arxiv_id.trim_end_matches(".pdf");
    let url = format!("https://arxiv.org/abs/{canonical_id}");
    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "C-SQD local development (mailto:admin@csqd.local)",
        )
        .send()
        .await
        .map_err(|error| {
            RepositoryError::Domain(format!("arXiv abstract page request failed: {error}"))
        })?;

    if !response.status().is_success() {
        return Err(RepositoryError::Domain(format!(
            "arXiv abstract page request failed with status {}",
            response.status()
        )));
    }

    let body = response.text().await.map_err(|error| {
        RepositoryError::Domain(format!("arXiv abstract page read failed: {error}"))
    })?;

    parse_arxiv_abs_page(&body, canonical_id).ok_or_else(|| RepositoryError::NotFound {
        entity: "arxiv_article",
        id: canonical_id.to_string(),
    })
}

async fn upsert_arxiv_entry(
    db: &PgPool,
    entry: &ArxivEntry,
) -> Result<(String, bool), RepositoryError> {
    let journal_id = ensure_arxiv_journal(db).await?;
    let existing_id = existing_scholarly_object_id(db, entry).await?;
    let was_created = existing_id.is_none();
    let metadata_provenance = json!({
        "source": "arxiv",
        "arxiv_id": entry.arxiv_id,
        "categories": entry.categories,
    });
    let authors = json!(entry.authors);
    let license = entry.license.clone().or_else(|| Some("arXiv".to_string()));
    let native_display_permitted = access_policy::arxiv_native_display_permitted(
        &entry.canonical_url,
        entry.pdf_url.as_deref(),
    );

    let scholarly_object_id = if let Some(existing_id) = existing_id {
        sqlx::query(
            r#"
            UPDATE scholarly_objects
            SET
                object_type = 'preprint',
                doi = COALESCE($2, doi),
                title = $3,
                authors = $4,
                abstract = $5,
                journal_id = $6::uuid,
                publication_date = $7::date,
                license = $8,
                canonical_url = $9,
                metadata_provenance = $10,
                native_display_permitted = $11,
                updated_at = now()
            WHERE id::text = $1
            "#,
        )
        .bind(&existing_id)
        .bind(&entry.doi)
        .bind(&entry.title)
        .bind(&authors)
        .bind(&entry.abstract_text)
        .bind(&journal_id)
        .bind(&entry.published_date)
        .bind(&license)
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
            VALUES (
                'preprint',
                $1,
                $2,
                $3,
                $4,
                $5::uuid,
                $6::date,
                $7,
                $8,
                $9,
                $10
            )
            RETURNING id::text AS id
            "#,
        )
        .bind(&entry.doi)
        .bind(&entry.title)
        .bind(&authors)
        .bind(&entry.abstract_text)
        .bind(&journal_id)
        .bind(&entry.published_date)
        .bind(&license)
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
        "repository",
        &entry.canonical_url,
        license.as_deref(),
        true,
    )
    .await?;

    if let Some(pdf_url) = &entry.pdf_url {
        upsert_external_location(
            db,
            &scholarly_object_id,
            "pdf",
            pdf_url,
            license.as_deref(),
            false,
        )
        .await?;
    }

    upsert_search_projection(db, &scholarly_object_id, entry).await?;
    audit_subjects::ensure_academic_for_scholarly_object(db, &scholarly_object_id).await?;

    Ok((scholarly_object_id, was_created))
}

async fn ensure_arxiv_journal(db: &PgPool) -> Result<String, RepositoryError> {
    if let Some(row) = sqlx::query(
        r#"
        SELECT id::text AS id
        FROM journals
        WHERE name = $1
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(ARXIV_JOURNAL_NAME)
    .fetch_optional(db)
    .await?
    {
        return Ok(row.get("id"));
    }

    let row = sqlx::query(
        r#"
        INSERT INTO journals (name, publisher, source_classification)
        VALUES ($1, 'Cornell University', 'repository')
        RETURNING id::text AS id
        "#,
    )
    .bind(ARXIV_JOURNAL_NAME)
    .fetch_one(db)
    .await?;

    Ok(row.get("id"))
}

async fn existing_scholarly_object_id(
    db: &PgPool,
    entry: &ArxivEntry,
) -> Result<Option<String>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id
        FROM scholarly_objects
        WHERE canonical_url = $1
           OR ($2::text IS NOT NULL AND doi = $2)
           OR metadata_provenance->>'arxiv_id' = $3
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&entry.canonical_url)
    .bind(&entry.doi)
    .bind(&entry.arxiv_id)
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
        .bind(json!({ "source": "arxiv" }))
        .execute(db)
        .await?;
    }

    Ok(())
}

async fn upsert_search_projection(
    db: &PgPool,
    scholarly_object_id: &str,
    entry: &ArxivEntry,
) -> Result<(), RepositoryError> {
    let search_text = format!(
        "{} {} {} {} {}",
        entry.title,
        entry.authors.join(" "),
        entry.abstract_text.clone().unwrap_or_default(),
        entry.categories.join(" "),
        entry.arxiv_id,
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

fn parse_first_arxiv_entry(body: &str) -> Option<ArxivEntry> {
    let document = Document::parse(body).ok()?;
    let entry = document
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "entry")?;

    let canonical_url = child_text(entry, "id")?;
    let arxiv_id = arxiv_identifier_from_query(&canonical_url)?;
    let title = normalize_whitespace(child_text(entry, "title")?.as_str());
    let summary = child_text(entry, "summary").map(|value| normalize_whitespace(value.as_str()));
    let published_date =
        child_text(entry, "published").and_then(|value| value.get(0..10).map(str::to_string));
    let authors = entry
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "author")
        .filter_map(|author| child_text(author, "name"))
        .map(|name| normalize_whitespace(name.as_str()))
        .collect::<Vec<_>>();
    let doi = child_text(entry, "doi");
    let categories = entry
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "category")
        .filter_map(|node| node.attribute("term").map(ToString::to_string))
        .collect::<Vec<_>>();
    let pdf_url = entry
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "link")
        .find(|node| {
            node.attribute("title") == Some("pdf")
                || node.attribute("type") == Some("application/pdf")
        })
        .and_then(|node| node.attribute("href"))
        .map(ToString::to_string);
    let license = entry
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "link")
        .find(|node| node.attribute("rel") == Some("license"))
        .and_then(|node| node.attribute("href"))
        .map(ToString::to_string);

    Some(ArxivEntry {
        arxiv_id,
        title,
        authors,
        abstract_text: summary,
        published_date,
        canonical_url,
        pdf_url,
        doi,
        license,
        categories,
    })
}

fn parse_arxiv_abs_page(body: &str, fallback_arxiv_id: &str) -> Option<ArxivEntry> {
    let arxiv_id =
        meta_content(body, "citation_arxiv_id").unwrap_or_else(|| fallback_arxiv_id.to_string());
    let title = normalize_whitespace(meta_content(body, "citation_title")?.as_str());
    let authors = meta_contents(body, "citation_author")
        .into_iter()
        .map(|author| normalize_whitespace(author.as_str()))
        .collect::<Vec<_>>();
    let abstract_text =
        meta_content(body, "citation_abstract").map(|value| normalize_whitespace(value.as_str()));
    let published_date = meta_content(body, "citation_date").and_then(|value| {
        let normalized = value.replace('/', "-");
        normalized.get(0..10).map(str::to_string)
    });
    let canonical_url = format!("https://arxiv.org/abs/{arxiv_id}");
    let pdf_url = meta_content(body, "citation_pdf_url")
        .or_else(|| Some(format!("https://arxiv.org/pdf/{arxiv_id}")));
    let doi =
        meta_content(body, "citation_doi").or_else(|| Some(format!("10.48550/arXiv.{arxiv_id}")));
    let license = license_href(body);

    Some(ArxivEntry {
        arxiv_id,
        title,
        authors,
        abstract_text,
        published_date,
        canonical_url,
        pdf_url,
        doi,
        license,
        categories: Vec::new(),
    })
}

fn meta_content(body: &str, name: &str) -> Option<String> {
    meta_contents(body, name).into_iter().next()
}

fn meta_contents(body: &str, name: &str) -> Vec<String> {
    let marker = format!("name=\"{name}\"");
    body.match_indices(&marker)
        .filter_map(|(index, _)| body.get(index..))
        .filter_map(|tail| {
            let content_start = tail.find("content=\"")? + "content=\"".len();
            let content_tail = tail.get(content_start..)?;
            let content_end = content_tail.find('"')?;
            Some(decode_html_entities(&content_tail[..content_end]))
        })
        .collect()
}

fn license_href(body: &str) -> Option<String> {
    body.match_indices("rel=\"license\"")
        .filter_map(|(index, _)| {
            let end = (index + 13).min(body.len());
            body.get(index.saturating_sub(240)..end)
        })
        .find_map(|fragment| {
            let href_start = fragment.find("href=\"")? + "href=\"".len();
            let href_tail = fragment.get(href_start..)?;
            let href_end = href_tail.find('"')?;
            Some(decode_html_entities(&href_tail[..href_end]))
        })
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn child_text(node: Node<'_, '_>, child_name: &str) -> Option<String> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == child_name)
        .and_then(|child| child.text())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn arxiv_identifier_from_query(query: &str) -> Option<String> {
    let trimmed = query.trim().trim_end_matches('/');
    let candidate = if let Some((_, id)) = trimmed.split_once("arxiv.org/abs/") {
        id
    } else if let Some((_, id)) = trimmed.split_once("arxiv.org/pdf/") {
        id
    } else {
        trimmed
    };
    let candidate = candidate
        .trim_start_matches("abs/")
        .trim_start_matches("pdf/")
        .trim_end_matches(".pdf")
        .split(['?', '#'])
        .next()
        .unwrap_or(candidate)
        .trim();

    if candidate.contains(' ') || candidate.is_empty() {
        return None;
    }

    let has_new_style_id = candidate.len() >= 9
        && candidate
            .chars()
            .take(4)
            .all(|character| character.is_ascii_digit())
        && candidate.chars().nth(4) == Some('.');
    let has_old_style_id = candidate.contains('/')
        && candidate
            .chars()
            .any(|character| character.is_ascii_digit());

    if has_new_style_id || has_old_style_id {
        Some(candidate.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{arxiv_identifier_from_query, parse_arxiv_abs_page, parse_first_arxiv_entry};

    #[test]
    fn extracts_arxiv_identifier_from_common_inputs() {
        assert_eq!(
            arxiv_identifier_from_query("1706.03762"),
            Some("1706.03762".to_string())
        );
        assert_eq!(
            arxiv_identifier_from_query("https://arxiv.org/abs/1706.03762v7"),
            Some("1706.03762v7".to_string())
        );
        assert_eq!(
            arxiv_identifier_from_query("https://arxiv.org/pdf/1706.03762.pdf"),
            Some("1706.03762".to_string())
        );
        assert_eq!(
            arxiv_identifier_from_query("attention is all you need"),
            None
        );
    }

    #[test]
    fn parses_first_arxiv_atom_entry() {
        let body = r#"
        <feed xmlns="http://www.w3.org/2005/Atom" xmlns:arxiv="http://arxiv.org/schemas/atom">
          <entry>
            <id>http://arxiv.org/abs/1706.03762v7</id>
            <updated>2023-08-02T00:00:00Z</updated>
            <published>2017-06-12T17:57:34Z</published>
            <title>Attention Is All You Need</title>
            <summary>
              The dominant sequence transduction models are based on complex
              recurrent or convolutional neural networks.
            </summary>
            <author><name>Ashish Vaswani</name></author>
            <author><name>Noam Shazeer</name></author>
            <arxiv:doi>10.48550/arXiv.1706.03762</arxiv:doi>
            <category term="cs.CL" />
            <link href="http://arxiv.org/abs/1706.03762v7" rel="alternate" type="text/html" />
            <link title="pdf" href="http://arxiv.org/pdf/1706.03762v7" rel="related" type="application/pdf" />
            <link href="http://creativecommons.org/licenses/by/4.0/" rel="license" />
          </entry>
        </feed>
        "#;

        let entry = parse_first_arxiv_entry(body).expect("entry should parse");

        assert_eq!(entry.arxiv_id, "1706.03762v7");
        assert_eq!(entry.title, "Attention Is All You Need");
        assert_eq!(entry.authors, vec!["Ashish Vaswani", "Noam Shazeer"]);
        assert_eq!(entry.published_date, Some("2017-06-12".to_string()));
        assert_eq!(
            entry.pdf_url,
            Some("http://arxiv.org/pdf/1706.03762v7".to_string())
        );
        assert_eq!(entry.categories, vec!["cs.CL"]);
    }

    #[test]
    fn parses_arxiv_abs_page_citation_metadata() {
        let body = r#"
        <html>
          <head>
            <meta name="citation_title" content="Towards Error-Centric Intelligence II: Energy-Structured Causal Models" />
            <meta name="citation_author" content="Thomas, Marcus" />
            <meta name="citation_date" content="2025/10/24" />
            <meta name="citation_pdf_url" content="https://arxiv.org/pdf/2510.22050" />
            <meta name="citation_arxiv_id" content="2510.22050" />
            <meta name="citation_abstract" content="Building on Part I&#39;s principles &amp; definitions." />
          </head>
          <body></body>
        </html>
        "#;

        let entry = parse_arxiv_abs_page(body, "2510.22050").expect("entry should parse");

        assert_eq!(entry.arxiv_id, "2510.22050");
        assert_eq!(
            entry.title,
            "Towards Error-Centric Intelligence II: Energy-Structured Causal Models"
        );
        assert_eq!(entry.authors, vec!["Thomas, Marcus"]);
        assert_eq!(entry.published_date, Some("2025-10-24".to_string()));
        assert_eq!(
            entry.abstract_text,
            Some("Building on Part I's principles & definitions.".to_string())
        );
        assert_eq!(
            entry.pdf_url,
            Some("https://arxiv.org/pdf/2510.22050".to_string())
        );
        assert_eq!(entry.doi, Some("10.48550/arXiv.2510.22050".to_string()));
    }
}
