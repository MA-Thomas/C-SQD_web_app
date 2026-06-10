use csqd_domain::{ArticleRetrievalResult, ArticleRetrievalSource, ArticleVersionKind};
use roxmltree::{Document, Node};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::time::Duration;

use super::{
    access_policy, article_access, article_versions, audit_subjects, doi_retrieval, RepositoryError,
};

const PUBMED_EFETCH_URL: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi";
const PMC_ID_CONVERTER_URL: &str = "https://www.ncbi.nlm.nih.gov/pmc/utils/idconv/v1.0/";
const PUBMED_SOURCE_CLASSIFICATION: &str = "pubmed";

#[derive(Clone, Debug)]
enum PubmedIdentifier {
    Pmcid(String),
    Pmid(String),
}

#[derive(Debug)]
struct PubmedEntry {
    pmid: String,
    pmcid: Option<String>,
    doi: Option<String>,
    title: String,
    authors: Vec<String>,
    abstract_text: Option<String>,
    source_name: String,
    publication_date: Option<String>,
    object_type: String,
    canonical_url: String,
    pubmed_url: String,
    pmc_full_text_url: Option<String>,
}

pub async fn retrieve_pubmed(
    db: &PgPool,
    query: &str,
) -> Result<ArticleRetrievalResult, RepositoryError> {
    let identifier = pubmed_identifier_from_query(query).ok_or_else(|| {
        RepositoryError::Domain(
            "article retrieval query does not contain a PMID or PMCID".to_string(),
        )
    })?;
    let requested_source = source_for_identifier(&identifier);
    let converted = match &identifier {
        PubmedIdentifier::Pmcid(pmcid) => Some(fetch_pmc_id_conversion(pmcid).await?),
        PubmedIdentifier::Pmid(_) => None,
    };
    let pmid = match (&identifier, converted.as_ref()) {
        (PubmedIdentifier::Pmid(pmid), _) => pmid.clone(),
        (PubmedIdentifier::Pmcid(_), Some(conversion)) => {
            conversion.pmid.clone().ok_or_else(|| {
                RepositoryError::Domain(
                    "PMCID did not resolve to a PubMed metadata record".to_string(),
                )
            })?
        }
        (PubmedIdentifier::Pmcid(_), None) => unreachable!("PMCID conversion should run"),
    };

    let mut entry = fetch_pubmed_entry(&pmid).await?;

    if let Some(conversion) = converted {
        entry.pmcid = entry.pmcid.or(conversion.pmcid);
        entry.doi = entry.doi.or(conversion.doi);
        entry.pmc_full_text_url = entry
            .pmcid
            .as_deref()
            .map(|pmcid| format!("https://pmc.ncbi.nlm.nih.gov/articles/{pmcid}/"));
    }

    if let Some(doi) = entry.doi.as_deref() {
        let _ = doi_retrieval::retrieve_doi(db, doi).await;
    }

    let (scholarly_object_id, was_created) = upsert_pubmed_entry(db, &entry).await?;
    let version_kind = version_kind_for_entry(&entry);
    let source_identifier = source_identifier_for_entry(&entry, &requested_source);
    let work_group = article_versions::ensure_version_group(
        db,
        &scholarly_object_id,
        &entry.title,
        version_kind.clone(),
        requested_source.as_str(),
        &source_identifier,
    )
    .await?;
    let article_access =
        article_access::find_for_scholarly_object(db, &scholarly_object_id).await?;
    let audit_subject_id =
        audit_subjects::ensure_academic_for_scholarly_object(db, &scholarly_object_id).await?;

    Ok(ArticleRetrievalResult {
        source: requested_source,
        source_identifier,
        work_group,
        version_kind,
        scholarly_object_id,
        audit_subject_id,
        title: entry.title,
        authors: entry.authors,
        abstract_text: entry.abstract_text,
        canonical_url: entry.canonical_url,
        pdf_url: None,
        doi: entry.doi,
        was_created,
        article_access,
    })
}

async fn fetch_pubmed_entry(pmid: &str) -> Result<PubmedEntry, RepositoryError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| {
            RepositoryError::Domain(format!("PubMed HTTP client setup failed: {error}"))
        })?;
    let contact_email = doi_retrieval::contact_email();
    let response = client
        .get(PUBMED_EFETCH_URL)
        .query(&[
            ("db", "pubmed"),
            ("id", pmid),
            ("retmode", "xml"),
            ("tool", "csqd"),
            ("email", contact_email.as_str()),
        ])
        .header(
            "User-Agent",
            format!("C-SQD local development (mailto:{contact_email})"),
        )
        .send()
        .await
        .map_err(|error| RepositoryError::Domain(format!("PubMed request failed: {error}")))?;

    if !response.status().is_success() {
        return Err(RepositoryError::Domain(format!(
            "PubMed request failed with status {}",
            response.status()
        )));
    }

    let body = response.text().await.map_err(|error| {
        RepositoryError::Domain(format!("PubMed response read failed: {error}"))
    })?;

    parse_first_pubmed_entry(&body, pmid).ok_or_else(|| RepositoryError::NotFound {
        entity: "pubmed_article",
        id: pmid.to_string(),
    })
}

#[derive(Debug)]
struct PmcIdConversion {
    doi: Option<String>,
    pmcid: Option<String>,
    pmid: Option<String>,
}

async fn fetch_pmc_id_conversion(pmcid: &str) -> Result<PmcIdConversion, RepositoryError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| {
            RepositoryError::Domain(format!(
                "PMC ID converter HTTP client setup failed: {error}"
            ))
        })?;
    let contact_email = doi_retrieval::contact_email();
    let response = client
        .get(PMC_ID_CONVERTER_URL)
        .query(&[
            ("ids", pmcid),
            ("format", "json"),
            ("tool", "csqd"),
            ("email", contact_email.as_str()),
        ])
        .header(
            "User-Agent",
            format!("C-SQD local development (mailto:{contact_email})"),
        )
        .send()
        .await
        .map_err(|error| {
            RepositoryError::Domain(format!("PMC ID converter request failed: {error}"))
        })?;

    if !response.status().is_success() {
        return Err(RepositoryError::Domain(format!(
            "PMC ID converter request failed with status {}",
            response.status()
        )));
    }

    let body = response.json::<Value>().await.map_err(|error| {
        RepositoryError::Domain(format!("PMC ID converter JSON parse failed: {error}"))
    })?;

    parse_pmc_id_conversion(&body).ok_or_else(|| RepositoryError::NotFound {
        entity: "pmcid",
        id: pmcid.to_string(),
    })
}

fn parse_pmc_id_conversion(body: &Value) -> Option<PmcIdConversion> {
    let record = body
        .get("records")
        .and_then(Value::as_array)
        .and_then(|records| records.first())?;

    Some(PmcIdConversion {
        doi: string_field(record, "doi").map(|doi| doi.to_ascii_lowercase()),
        pmcid: string_field(record, "pmcid").map(normalize_pmcid),
        pmid: string_field(record, "pmid"),
    })
}

async fn upsert_pubmed_entry(
    db: &PgPool,
    entry: &PubmedEntry,
) -> Result<(String, bool), RepositoryError> {
    let journal_id = ensure_source(db, entry).await?;
    let existing_id = existing_scholarly_object_id(db, entry).await?;
    let was_created = existing_id.is_none();
    let authors = json!(entry.authors);
    let metadata_provenance = json!({
        "source": "pubmed",
        "pmid": entry.pmid,
        "pmcid": entry.pmcid,
    });
    let native_display_permitted =
        access_policy::pmc_native_display_permitted(entry.pmc_full_text_url.as_deref());

    let scholarly_object_id = if let Some(existing_id) = existing_id {
        sqlx::query(
            r#"
            UPDATE scholarly_objects
            SET
                object_type = $2,
                doi = COALESCE($3, doi),
                title = $4,
                authors = $5,
                abstract = $6,
                journal_id = $7::uuid,
                publication_date = $8::date,
                canonical_url = $9,
                metadata_provenance = $10,
                native_display_permitted = $11,
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
                canonical_url,
                metadata_provenance,
                native_display_permitted
            )
            VALUES ($1, $2, $3, $4, $5, $6::uuid, $7::date, $8, $9, $10)
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
        .bind(&entry.canonical_url)
        .bind(&metadata_provenance)
        .bind(native_display_permitted)
        .fetch_one(db)
        .await?;

        row.get("id")
    };

    if let Some(doi) = &entry.doi {
        upsert_external_location(
            db,
            &scholarly_object_id,
            "publisher",
            &format!("https://doi.org/{doi}"),
            None,
            entry.pmc_full_text_url.is_none(),
            "pubmed",
        )
        .await?;
    }

    upsert_external_location(
        db,
        &scholarly_object_id,
        "landing_page",
        &entry.pubmed_url,
        None,
        entry.pmc_full_text_url.is_none() && entry.doi.is_none(),
        "pubmed",
    )
    .await?;

    if let Some(pmc_full_text_url) = &entry.pmc_full_text_url {
        upsert_external_location(
            db,
            &scholarly_object_id,
            "full_text",
            pmc_full_text_url,
            None,
            true,
            "pmc",
        )
        .await?;
    }

    upsert_search_projection(db, &scholarly_object_id, entry).await?;
    audit_subjects::ensure_academic_for_scholarly_object(db, &scholarly_object_id).await?;

    Ok((scholarly_object_id, was_created))
}

async fn ensure_source(db: &PgPool, entry: &PubmedEntry) -> Result<String, RepositoryError> {
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
        VALUES ($1, 'PubMed', $2)
        RETURNING id::text AS id
        "#,
    )
    .bind(&entry.source_name)
    .bind(PUBMED_SOURCE_CLASSIFICATION)
    .fetch_one(db)
    .await?;

    Ok(row.get("id"))
}

async fn existing_scholarly_object_id(
    db: &PgPool,
    entry: &PubmedEntry,
) -> Result<Option<String>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id
        FROM scholarly_objects
        WHERE ($1::text IS NOT NULL AND doi = $1)
           OR metadata_provenance->>'pmid' = $2
           OR ($3::text IS NOT NULL AND metadata_provenance->>'pmcid' = $3)
           OR canonical_url = $4
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&entry.doi)
    .bind(&entry.pmid)
    .bind(&entry.pmcid)
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
    entry: &PubmedEntry,
) -> Result<(), RepositoryError> {
    let search_text = format!(
        "{} {} {} {} {} {} {}",
        entry.title,
        entry.authors.join(" "),
        entry.abstract_text.clone().unwrap_or_default(),
        entry.source_name,
        entry.pmid,
        entry.pmcid.clone().unwrap_or_default(),
        entry.doi.clone().unwrap_or_default(),
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

fn source_for_identifier(identifier: &PubmedIdentifier) -> ArticleRetrievalSource {
    match identifier {
        PubmedIdentifier::Pmcid(_) => ArticleRetrievalSource::Pmc,
        PubmedIdentifier::Pmid(_) => ArticleRetrievalSource::Pubmed,
    }
}

fn source_identifier_for_entry(entry: &PubmedEntry, source: &ArticleRetrievalSource) -> String {
    match source {
        ArticleRetrievalSource::Pmc => entry.pmcid.clone().unwrap_or_else(|| entry.pmid.clone()),
        ArticleRetrievalSource::Pubmed => entry.pmid.clone(),
        _ => entry.pmid.clone(),
    }
}

fn version_kind_for_entry(entry: &PubmedEntry) -> ArticleVersionKind {
    match entry.object_type.as_str() {
        "preprint" => ArticleVersionKind::Preprint,
        "article" => ArticleVersionKind::Publisher,
        _ => ArticleVersionKind::Unknown,
    }
}

fn parse_first_pubmed_entry(body: &str, fallback_pmid: &str) -> Option<PubmedEntry> {
    let document = Document::parse(body).ok()?;
    let article_node = document
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "PubmedArticle")?;
    let medline = article_node
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "MedlineCitation")?;
    let article = medline
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "Article")?;
    let pmid = medline
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "PMID")
        .and_then(node_text)
        .unwrap_or_else(|| fallback_pmid.to_string());
    let title = article
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "ArticleTitle")
        .map(collect_text)
        .map(|value| normalize_whitespace(&value))?;
    let authors = parse_authors(article);
    let abstract_text = parse_abstract(article);
    let source_name = article
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "Journal")
        .and_then(|journal| {
            child_text(journal, "Title").or_else(|| child_text(journal, "ISOAbbreviation"))
        })
        .unwrap_or_else(|| "PubMed".to_string());
    let publication_date = parse_publication_date(article);
    let doi = article_id(article_node, "doi").map(|doi| doi.to_ascii_lowercase());
    let pmcid = article_id(article_node, "pmc").map(normalize_pmcid);
    let object_type = if has_publication_type(article, "preprint") {
        "preprint"
    } else {
        "article"
    }
    .to_string();
    let pubmed_url = format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/");
    let pmc_full_text_url = pmcid
        .as_deref()
        .map(|pmcid| format!("https://pmc.ncbi.nlm.nih.gov/articles/{pmcid}/"));
    let canonical_url = doi
        .as_ref()
        .map(|doi| format!("https://doi.org/{doi}"))
        .or_else(|| pmc_full_text_url.clone())
        .unwrap_or_else(|| pubmed_url.clone());

    Some(PubmedEntry {
        pmid,
        pmcid,
        doi,
        title,
        authors,
        abstract_text,
        source_name,
        publication_date,
        object_type,
        canonical_url,
        pubmed_url,
        pmc_full_text_url,
    })
}

fn parse_authors(article: Node<'_, '_>) -> Vec<String> {
    let Some(author_list) = article
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "AuthorList")
    else {
        return Vec::new();
    };

    author_list
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "Author")
        .filter_map(|author| {
            if let Some(collective_name) = child_text(author, "CollectiveName") {
                return Some(collective_name);
            }

            let fore_name = child_text(author, "ForeName");
            let last_name = child_text(author, "LastName");
            match (fore_name, last_name) {
                (Some(fore_name), Some(last_name)) => Some(format!("{fore_name} {last_name}")),
                (None, Some(last_name)) => Some(last_name),
                (Some(fore_name), None) => Some(fore_name),
                (None, None) => None,
            }
        })
        .collect()
}

fn parse_abstract(article: Node<'_, '_>) -> Option<String> {
    let abstract_node = article
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "Abstract")?;
    let parts = abstract_node
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "AbstractText")
        .filter_map(|node| {
            let text = normalize_whitespace(&collect_text(node));
            if text.is_empty() {
                None
            } else if let Some(label) = node.attribute("Label") {
                Some(format!("{label}: {text}"))
            } else {
                Some(text)
            }
        })
        .collect::<Vec<_>>();

    (!parts.is_empty()).then(|| parts.join(" "))
}

fn parse_publication_date(article: Node<'_, '_>) -> Option<String> {
    article
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "ArticleDate")
        .and_then(date_from_node)
        .or_else(|| {
            article
                .descendants()
                .find(|node| node.is_element() && node.tag_name().name() == "PubDate")
                .and_then(date_from_node)
        })
}

fn date_from_node(node: Node<'_, '_>) -> Option<String> {
    let medline_date = child_text(node, "MedlineDate");
    let year = child_text(node, "Year").or_else(|| {
        medline_date
            .as_deref()
            .and_then(|value| value.get(0..4))
            .map(ToString::to_string)
    })?;
    let month = child_text(node, "Month")
        .and_then(|value| month_to_number(&value))
        .unwrap_or(1);
    let day = child_text(node, "Day")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1);

    Some(format!("{year}-{month:02}-{day:02}"))
}

fn month_to_number(value: &str) -> Option<u32> {
    if let Ok(month) = value.parse::<u32>() {
        return Some(month);
    }

    match value.to_ascii_lowercase().get(0..3)? {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

fn article_id(article_node: Node<'_, '_>, id_type: &str) -> Option<String> {
    article_node
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "ArticleId")
        .find(|node| node.attribute("IdType") == Some(id_type))
        .and_then(node_text)
}

fn has_publication_type(article: Node<'_, '_>, publication_type: &str) -> bool {
    article
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "PublicationType")
        .filter_map(node_text)
        .any(|value| value.eq_ignore_ascii_case(publication_type))
}

fn pubmed_identifier_from_query(query: &str) -> Option<PubmedIdentifier> {
    let trimmed = query.trim().trim_end_matches('/');
    let lower = trimmed.to_ascii_lowercase();

    if let Some(pmid) = lower
        .split("pubmed.ncbi.nlm.nih.gov/")
        .nth(1)
        .and_then(|tail| tail.split(['/', '?', '#']).next())
        .and_then(pmid_from_candidate)
    {
        return Some(PubmedIdentifier::Pmid(pmid));
    }

    if let Some(pmcid) = pmcid_from_candidate(trimmed) {
        return Some(PubmedIdentifier::Pmcid(pmcid));
    }

    if let Some(rest) = lower.strip_prefix("pmid:") {
        return pmid_from_candidate(rest).map(PubmedIdentifier::Pmid);
    }

    pmid_from_candidate(trimmed).map(PubmedIdentifier::Pmid)
}

fn pmcid_from_candidate(candidate: &str) -> Option<String> {
    let uppercase = candidate.to_ascii_uppercase();

    uppercase.match_indices("PMC").find_map(|(start, _)| {
        let tail = uppercase.get(start..)?;
        let pmcid = tail
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric())
            .collect::<String>();

        (pmcid.len() > 3
            && pmcid[3..]
                .chars()
                .all(|character| character.is_ascii_digit()))
        .then_some(pmcid)
    })
}

fn pmid_from_candidate(candidate: &str) -> Option<String> {
    let value = candidate
        .trim()
        .split(['?', '#', '/', ' '])
        .next()
        .unwrap_or(candidate)
        .trim();

    (!value.is_empty() && value.chars().all(|character| character.is_ascii_digit()))
        .then(|| value.to_string())
}

fn normalize_pmcid(value: String) -> String {
    pmcid_from_candidate(&value).unwrap_or_else(|| value.to_ascii_uppercase())
}

fn child_text(node: Node<'_, '_>, child_name: &str) -> Option<String> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == child_name)
        .and_then(node_text)
}

fn node_text(node: Node<'_, '_>) -> Option<String> {
    node.text()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn collect_text(node: Node<'_, '_>) -> String {
    node.descendants()
        .filter(|descendant| descendant.is_text())
        .filter_map(|descendant| descendant.text())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        parse_first_pubmed_entry, parse_pmc_id_conversion, pubmed_identifier_from_query,
        PubmedIdentifier,
    };

    #[test]
    fn extracts_pubmed_identifiers_from_common_inputs() {
        assert!(matches!(
            pubmed_identifier_from_query("12345678"),
            Some(PubmedIdentifier::Pmid(value)) if value == "12345678"
        ));
        assert!(matches!(
            pubmed_identifier_from_query("pmid:12345678"),
            Some(PubmedIdentifier::Pmid(value)) if value == "12345678"
        ));
        assert!(matches!(
            pubmed_identifier_from_query("https://pubmed.ncbi.nlm.nih.gov/12345678/"),
            Some(PubmedIdentifier::Pmid(value)) if value == "12345678"
        ));
        assert!(matches!(
            pubmed_identifier_from_query("PMCID: PMC1234567"),
            Some(PubmedIdentifier::Pmcid(value)) if value == "PMC1234567"
        ));
        assert!(matches!(
            pubmed_identifier_from_query("https://pmc.ncbi.nlm.nih.gov/articles/PMC1234567/"),
            Some(PubmedIdentifier::Pmcid(value)) if value == "PMC1234567"
        ));
        assert!(pubmed_identifier_from_query("not an identifier").is_none());
    }

    #[test]
    fn parses_pubmed_xml_metadata() {
        let body = r#"
        <PubmedArticleSet>
          <PubmedArticle>
            <MedlineCitation>
              <PMID>34567890</PMID>
              <Article>
                <Journal>
                  <Title>Journal of Careful Tests</Title>
                </Journal>
                <ArticleTitle>Rapid assessment of <i>T-cell</i> specificity</ArticleTitle>
                <ArticleDate>
                  <Year>2021</Year>
                  <Month>09</Month>
                  <Day>14</Day>
                </ArticleDate>
                <Abstract>
                  <AbstractText Label="Background">Testing structured PubMed parsing.</AbstractText>
                  <AbstractText>Second sentence.</AbstractText>
                </Abstract>
                <AuthorList>
                  <Author>
                    <ForeName>Ada</ForeName>
                    <LastName>Lovelace</LastName>
                  </Author>
                  <Author>
                    <CollectiveName>Example Research Group</CollectiveName>
                  </Author>
                </AuthorList>
                <PublicationTypeList>
                  <PublicationType>Journal Article</PublicationType>
                </PublicationTypeList>
              </Article>
            </MedlineCitation>
            <PubmedData>
              <ArticleIdList>
                <ArticleId IdType="doi">10.1038/example</ArticleId>
                <ArticleId IdType="pmc">PMC1234567</ArticleId>
              </ArticleIdList>
            </PubmedData>
          </PubmedArticle>
        </PubmedArticleSet>
        "#;

        let entry = parse_first_pubmed_entry(body, "34567890").expect("entry should parse");

        assert_eq!(entry.pmid, "34567890");
        assert_eq!(entry.pmcid, Some("PMC1234567".to_string()));
        assert_eq!(entry.doi, Some("10.1038/example".to_string()));
        assert_eq!(entry.title, "Rapid assessment of T-cell specificity");
        assert_eq!(
            entry.authors,
            vec!["Ada Lovelace", "Example Research Group"]
        );
        assert_eq!(entry.source_name, "Journal of Careful Tests");
        assert_eq!(entry.publication_date, Some("2021-09-14".to_string()));
        assert_eq!(
            entry.abstract_text,
            Some("Background: Testing structured PubMed parsing. Second sentence.".to_string())
        );
        assert_eq!(
            entry.pmc_full_text_url,
            Some("https://pmc.ncbi.nlm.nih.gov/articles/PMC1234567/".to_string())
        );
    }

    #[test]
    fn parses_pmc_id_converter_response() {
        let body = json!({
            "records": [
                {
                    "pmcid": "PMC1234567",
                    "pmid": "34567890",
                    "doi": "10.1038/example"
                }
            ]
        });

        let conversion = parse_pmc_id_conversion(&body).expect("conversion should parse");

        assert_eq!(conversion.pmid, Some("34567890".to_string()));
        assert_eq!(conversion.pmcid, Some("PMC1234567".to_string()));
        assert_eq!(conversion.doi, Some("10.1038/example".to_string()));
    }
}
