use csqd_academic_adapter::{ArticleRetrievalResult, ArticleRetrievalSet};
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;

use super::{doi_retrieval, RepositoryError};

const CROSSREF_WORKS_URL: &str = "https://api.crossref.org/works";

pub async fn retrieve_title(
    db: &PgPool,
    query: &str,
    include_preprints: bool,
) -> Result<ArticleRetrievalSet, RepositoryError> {
    let title_query = query.trim();

    if title_query.is_empty() {
        return Err(RepositoryError::Domain(
            "article retrieval query cannot be empty".to_string(),
        ));
    }

    let dois = resolve_title_to_dois(title_query, include_preprints).await?;
    let mut results = Vec::with_capacity(dois.len());

    for doi in dois {
        let result = doi_retrieval::retrieve_doi(db, &doi).await?;

        if !results
            .iter()
            .any(|existing: &ArticleRetrievalResult| existing.doi == result.doi)
        {
            results.push(result);
        }
    }

    Ok(ArticleRetrievalSet { results })
}

async fn resolve_title_to_dois(
    query: &str,
    include_preprints: bool,
) -> Result<Vec<String>, RepositoryError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| {
            RepositoryError::Domain(format!("title search HTTP client setup failed: {error}"))
        })?;
    let body = fetch_crossref_title_search(&client, query).await?;

    let dois = select_matching_dois(&body, query, include_preprints);

    if dois.is_empty() {
        return Err(RepositoryError::NotFound {
            entity: "article_title",
            id: query.to_string(),
        });
    }

    Ok(dois)
}

async fn fetch_crossref_title_search(
    client: &reqwest::Client,
    query: &str,
) -> Result<Value, RepositoryError> {
    let contact_email = doi_retrieval::contact_email();
    let response = client
        .get(CROSSREF_WORKS_URL)
        .query(&[
            ("query.bibliographic", query),
            ("rows", "10"),
            ("select", "DOI,title,type,score"),
            ("mailto", contact_email.as_str()),
        ])
        .header(
            "User-Agent",
            format!("C-SQD local development (mailto:{contact_email})"),
        )
        .send()
        .await
        .map_err(|error| {
            RepositoryError::Domain(format!("Crossref title search failed: {error}"))
        })?;

    if !response.status().is_success() {
        return Err(RepositoryError::Domain(format!(
            "Crossref title search failed with status {}",
            response.status()
        )));
    }

    response.json::<Value>().await.map_err(|error| {
        RepositoryError::Domain(format!("Crossref title search JSON parse failed: {error}"))
    })
}

fn select_matching_dois(body: &Value, query: &str, include_preprints: bool) -> Vec<String> {
    let candidates = matching_title_candidates(body, query);
    let Some(primary) = candidates
        .iter()
        .min_by_key(|candidate| {
            (
                crossref_type_rank(candidate.item_type.as_deref()),
                candidate.index,
            )
        })
        .cloned()
    else {
        return Vec::new();
    };

    let mut dois = vec![primary.doi.clone()];

    if include_preprints {
        let primary_title = normalize_search_text(&primary.title);
        let mut preprint_dois = candidates
            .iter()
            .filter(|candidate| candidate.doi != primary.doi)
            .filter(|candidate| candidate.item_type.as_deref() == Some("posted-content"))
            .filter(|candidate| normalize_search_text(&candidate.title) == primary_title)
            .map(|candidate| candidate.doi.clone())
            .collect::<Vec<_>>();

        dois.append(&mut preprint_dois);
    }

    dois
}

#[derive(Clone, Debug)]
struct CrossrefTitleCandidate {
    doi: String,
    title: String,
    item_type: Option<String>,
    index: usize,
}

fn matching_title_candidates(body: &Value, query: &str) -> Vec<CrossrefTitleCandidate> {
    let Some(items) = body
        .get("message")
        .and_then(|message| message.get("items"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let doi = string_field(item, "DOI")?.to_ascii_lowercase();
            let title = string_array_first(item, "title")?;
            let item_type = string_field(item, "type");

            title_matches_query(query, &title).then(|| CrossrefTitleCandidate {
                doi,
                title,
                item_type,
                index,
            })
        })
        .collect()
}

fn crossref_type_rank(item_type: Option<&str>) -> usize {
    match item_type {
        Some("journal-article") => 0,
        Some("proceedings-article") => 1,
        Some("posted-content") => 2,
        Some("book-chapter") | Some("book-section") => 3,
        Some("report") | Some("report-component") => 4,
        Some("peer-review") => 5,
        _ => 6,
    }
}

fn title_matches_query(query: &str, title: &str) -> bool {
    let normalized_query = normalize_search_text(query);
    let normalized_title = normalize_search_text(title);

    if normalized_query.is_empty() || normalized_title.is_empty() {
        return false;
    }

    if normalized_query == normalized_title {
        return true;
    }

    if normalized_title.len() >= 20
        && (normalized_query.contains(&normalized_title)
            || normalized_title.contains(&normalized_query))
    {
        return true;
    }

    let query_terms = content_terms(&normalized_query);
    let title_terms = content_terms(&normalized_title);

    if query_terms.len() < 4 || title_terms.len() < 4 {
        return false;
    }

    let shared_terms = title_terms
        .iter()
        .filter(|term| query_terms.contains(term))
        .count();
    let denominator = title_terms.len().min(query_terms.len()) as f32;

    shared_terms as f32 / denominator >= 0.75
}

fn normalize_search_text(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn content_terms(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .filter(|term| term.len() > 2)
        .filter(|term| !is_noise_term(term))
        .map(ToString::to_string)
        .collect()
}

fn is_noise_term(term: &str) -> bool {
    matches!(
        term,
        "the"
            | "and"
            | "for"
            | "with"
            | "from"
            | "into"
            | "onto"
            | "that"
            | "this"
            | "these"
            | "those"
            | "using"
            | "via"
            | "study"
            | "article"
            | "paper"
    )
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{select_matching_dois, title_matches_query};

    #[test]
    fn selects_doi_for_exact_title_match() {
        let body = json!({
            "message": {
                "items": [
                    {
                        "DOI": "10.1038/s43588-021-00076-1",
                        "title": [
                            "Rapid assessment of T-cell receptor specificity of the immune repertoire"
                        ],
                        "type": "journal-article",
                        "score": 91.0
                    }
                ]
            }
        });

        assert_eq!(
            select_matching_dois(
                &body,
                "Rapid assessment of T-cell receptor specificity of the immune repertoire",
                false,
            ),
            vec!["10.1038/s43588-021-00076-1".to_string()]
        );
    }

    #[test]
    fn accepts_citation_like_query_containing_title() {
        assert!(title_matches_query(
            "Rapid assessment of T-cell receptor specificity of the immune repertoire. Nature Computational Science.",
            "Rapid assessment of T-cell receptor specificity of the immune repertoire",
        ));
    }

    #[test]
    fn prefers_journal_article_when_preprint_has_same_title() {
        let body = json!({
            "message": {
                "items": [
                    {
                        "DOI": "10.1101/2020.04.06.028415",
                        "title": [
                            "Rapid Assessment of T-Cell Receptor Specificity of the Immune Repertoire"
                        ],
                        "type": "posted-content",
                        "score": 43.0
                    },
                    {
                        "DOI": "10.1038/s43588-021-00076-1",
                        "title": [
                            "Rapid assessment of T-cell receptor specificity of the immune repertoire"
                        ],
                        "type": "journal-article",
                        "score": 38.0
                    }
                ]
            }
        });

        assert_eq!(
            select_matching_dois(
                &body,
                "Rapid assessment of T-cell receptor specificity of the immune repertoire",
                false,
            ),
            vec!["10.1038/s43588-021-00076-1".to_string()]
        );
    }

    #[test]
    fn includes_same_title_preprint_when_requested() {
        let body = json!({
            "message": {
                "items": [
                    {
                        "DOI": "10.1101/2020.04.06.028415",
                        "title": [
                            "Rapid Assessment of T-Cell Receptor Specificity of the Immune Repertoire"
                        ],
                        "type": "posted-content",
                        "score": 43.0
                    },
                    {
                        "DOI": "10.1038/s43588-021-00076-1",
                        "title": [
                            "Rapid assessment of T-cell receptor specificity of the immune repertoire"
                        ],
                        "type": "journal-article",
                        "score": 38.0
                    }
                ]
            }
        });

        assert_eq!(
            select_matching_dois(
                &body,
                "Rapid assessment of T-cell receptor specificity of the immune repertoire",
                true,
            ),
            vec![
                "10.1038/s43588-021-00076-1".to_string(),
                "10.1101/2020.04.06.028415".to_string()
            ]
        );
    }

    #[test]
    fn excludes_same_title_preprint_by_default() {
        let body = json!({
            "message": {
                "items": [
                    {
                        "DOI": "10.1101/2020.04.06.028415",
                        "title": [
                            "Rapid Assessment of T-Cell Receptor Specificity of the Immune Repertoire"
                        ],
                        "type": "posted-content",
                        "score": 43.0
                    },
                    {
                        "DOI": "10.1038/s43588-021-00076-1",
                        "title": [
                            "Rapid assessment of T-cell receptor specificity of the immune repertoire"
                        ],
                        "type": "journal-article",
                        "score": 38.0
                    }
                ]
            }
        });

        assert_eq!(
            select_matching_dois(
                &body,
                "Rapid assessment of T-cell receptor specificity of the immune repertoire",
                false,
            ),
            vec!["10.1038/s43588-021-00076-1".to_string()]
        );
    }

    #[test]
    fn rejects_unrelated_top_crossref_hit() {
        let body = json!({
            "message": {
                "items": [
                    {
                        "DOI": "10.1000/example",
                        "title": ["A completely different immunology paper"],
                        "score": 12.0
                    }
                ]
            }
        });

        assert_eq!(
            select_matching_dois(
                &body,
                "Rapid assessment of T-cell receptor specificity of the immune repertoire",
                false,
            ),
            Vec::<String>::new()
        );
    }
}
