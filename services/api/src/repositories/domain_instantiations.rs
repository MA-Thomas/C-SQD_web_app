use csqd_domain::{
    CWENode, CWESource, DomainConfig, DomainInstantiationDetail, DomainInstantiationSummary,
    DomainType, Principal,
};
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::RepositoryError;

pub async fn list_summaries(
    db: &PgPool,
) -> Result<Vec<DomainInstantiationSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            domain_type,
            domain_type_detail,
            name,
            created_at,
            governed_by
        FROM domain_instantiations
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_to_summary).collect()
}

pub async fn find_detail(
    db: &PgPool,
    domain_id: &str,
) -> Result<DomainInstantiationDetail, RepositoryError> {
    let domain_row = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            domain_type,
            domain_type_detail,
            name,
            config,
            created_at,
            governed_by
        FROM domain_instantiations
        WHERE id::text = $1
        "#,
    )
    .bind(domain_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| RepositoryError::NotFound {
        entity: "domain_instantiation",
        id: domain_id.to_string(),
    })?;

    let cwe_rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            domain_instantiation_id::text AS domain_instantiation_id,
            parent_id::text AS parent,
            label,
            description,
            source,
            community_id::text AS community_id
        FROM cwe_nodes
        WHERE domain_instantiation_id::text = $1
        ORDER BY parent_id NULLS FIRST, label ASC
        "#,
    )
    .bind(domain_id)
    .fetch_all(db)
    .await?;

    row_to_detail(domain_row, cwe_rows)
}

fn row_to_summary(row: PgRow) -> Result<DomainInstantiationSummary, RepositoryError> {
    let domain_type: String = row.get("domain_type");
    let domain_type_detail: Option<String> = row.get("domain_type_detail");
    let governed_by: Value = row.get("governed_by");

    Ok(DomainInstantiationSummary {
        id: row.get("id"),
        domain_type: DomainType::from_db(domain_type.as_str(), domain_type_detail.as_deref())
            .map_err(RepositoryError::Domain)?,
        name: row.get("name"),
        created_at: row.get("created_at"),
        governed_by: serde_json::from_value::<Principal>(governed_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
    })
}

fn row_to_detail(
    row: PgRow,
    cwe_rows: Vec<PgRow>,
) -> Result<DomainInstantiationDetail, RepositoryError> {
    let domain_type: String = row.get("domain_type");
    let domain_type_detail: Option<String> = row.get("domain_type_detail");
    let config: Value = row.get("config");
    let governed_by: Value = row.get("governed_by");
    let cwe_nodes = cwe_rows
        .into_iter()
        .map(row_to_cwe_node)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DomainInstantiationDetail {
        id: row.get("id"),
        domain_type: DomainType::from_db(domain_type.as_str(), domain_type_detail.as_deref())
            .map_err(RepositoryError::Domain)?,
        name: row.get("name"),
        config: serde_json::from_value::<DomainConfig>(config)
            .map_err(|error| RepositoryError::Domain(format!("invalid domain config: {error}")))?,
        cwe_nodes,
        created_at: row.get("created_at"),
        governed_by: serde_json::from_value::<Principal>(governed_by)
            .map_err(|error| RepositoryError::Domain(format!("invalid principal: {error}")))?,
    })
}

fn row_to_cwe_node(row: PgRow) -> Result<CWENode, RepositoryError> {
    let source: String = row.get("source");
    let community_id: Option<String> = row.get("community_id");

    Ok(CWENode {
        id: row.get("id"),
        domain_instantiation_id: row.get("domain_instantiation_id"),
        parent: row.get("parent"),
        label: row.get("label"),
        description: row.get("description"),
        source: CWESource::from_db(source.as_str(), community_id.as_deref())
            .map_err(RepositoryError::Domain)?,
    })
}
