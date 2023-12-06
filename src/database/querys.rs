use crate::{
    database::dto::{
        AttestationCreatedOverTime, AttestationKPIs, AttestationResponse, Pagination, TxState,
    },
    error::AppError,
};

use sqlx::{postgres::PgQueryResult, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use super::dto::Credential;

pub async fn get_attestation_request_by_id(
    attestation_request_id: &Uuid,
    db_executor: &PgPool,
) -> Result<AttestationResponse, AppError> {
    sqlx::query_as!(
        AttestationResponse,
        r#"SELECT id, approved, revoked, created_at, deleted_at, updated_at, approved_at, revoked_at, ctype_hash, credential, claimer, tx_state as "tx_state: TxState"
        FROM attestation_requests WHERE id = $1"#,
        attestation_request_id,
    )
    .fetch_one(db_executor)
    .await
    .map_err(AppError::from)
}

pub async fn get_attestations_count(db_executor: &PgPool) -> i64 {
    sqlx::query_scalar!("SELECT COUNT (*) FROM attestation_requests")
        .fetch_one(db_executor)
        .await
        .map_or(0, |count| count.unwrap())
}

pub async fn get_attestation_requests(
    pagination: Pagination,
    db_executor: &PgPool,
) -> Result<Vec<AttestationResponse>, AppError> {
    let (query, bind_values) = construct_query(&pagination);
    get_attestations(&query, bind_values, db_executor)
        .await
        .map_err(AppError::from)
}

pub fn construct_query(pagination: &Pagination) -> (String, Vec<String>) {
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM attestation_requests");
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(filter) = &pagination.filter {
        query.push(" WHERE claimer =");
        query.push_bind(filter.clone());
        bind_values.push(filter.clone());
    }

    if let Some(sort) = &pagination.sort {
        query.push(" ORDER BY ");
        query.push_bind(sort[0].clone());
        query.push(if sort[1] == "ASC" { " ASC" } else { " DESC" });
        bind_values.push(sort[0].clone());
    }

    if let Some(offset) = &pagination.offset {
        query.push(" OFFSET ");
        query.push_bind(offset[0].to_string());
        query.push(" LIMIT ");
        query.push_bind(offset[1].to_string());
        bind_values.push(offset[0].to_string());
        bind_values.push(offset[1].to_string());
    }

    (query.into_sql(), bind_values)
}

async fn get_attestations(
    query_string: &str,
    bind_values: Vec<String>,
    db_executor: &PgPool,
) -> Result<Vec<AttestationResponse>, sqlx::Error> {
    let mut query = sqlx::query_as::<_, AttestationResponse>(&query_string);

    for value in bind_values {
        query = query.bind(value);
    }

    query.fetch_all(db_executor).await
}

pub async fn delete_attestation_request(
    attestation_id: &Uuid,
    db_executor: &PgPool,
) -> Result<PgQueryResult, AppError> {
    sqlx::query!(
        "UPDATE attestation_requests SET deleted_at = NOW() WHERE id = $1",
        attestation_id
    )
    .execute(db_executor)
    .await
    .map_err(AppError::from)
}

pub async fn insert_attestation_request(
    credential: &Credential,
    db_executor: &PgPool,
) -> Result<AttestationResponse, AppError> {
    let ctype_hash = credential.claim.ctype_hash.clone();
    let claimer = credential.claim.owner.clone();
    let result = sqlx::query_as!(
        AttestationResponse,
        r#"INSERT INTO attestation_requests (ctype_hash, claimer, credential) VALUES ($1, $2, $3) 
        RETURNING  id, approved, revoked, created_at, deleted_at, updated_at, approved_at, revoked_at, ctype_hash, credential, claimer, tx_state as "tx_state: TxState""#,
        ctype_hash,
        claimer,
        serde_json::json!(credential)
    )
    .fetch_one(db_executor)
    .await;

    result.map_err(AppError::from)
}

pub async fn can_approve_attestation_tx(
    attestation_request_id: &Uuid,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<AttestationResponse, AppError> {
    sqlx::query_as!(
        AttestationResponse,
        r#"SELECT id, approved, revoked, created_at, deleted_at, updated_at, approved_at, revoked_at, ctype_hash, credential, claimer, tx_state as "tx_state: TxState" 
        FROM attestation_requests WHERE id = $1 AND approved = false AND revoked = false"#,
        attestation_request_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)
}

pub async fn mark_attestation_request_in_flight(
    attestation_request_id: &Uuid,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<PgQueryResult, AppError> {
    sqlx::query!(
        "UPDATE attestation_requests SET tx_state = 'InFlight' WHERE id = $1",
        attestation_request_id
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)
}

pub async fn record_attestation_request_failed(
    attestation_request_id: &Uuid,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<PgQueryResult, AppError> {
    sqlx::query!(
        "UPDATE attestation_requests SET tx_state = 'Failed' WHERE id = $1",
        attestation_request_id
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)
}

pub async fn approve_attestation_request_tx(
    attestation_request_id: &Uuid,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<PgQueryResult, AppError> {
    sqlx::query!(
        "UPDATE attestation_requests SET approved = true, tx_state = 'Succeeded', approved_at = NOW() WHERE id = $1",
        attestation_request_id
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)
}

pub async fn can_revoke_attestation(
    attestation_request_id: &Uuid,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<AttestationResponse, AppError> {
    sqlx::query_as!(
        AttestationResponse,
        r#"SELECT id, approved, revoked, created_at, deleted_at, updated_at, approved_at, revoked_at, ctype_hash, credential, claimer, tx_state as "tx_state: TxState" 
        FROM attestation_requests WHERE id = $1 AND approved = true AND revoked = false"#,
        attestation_request_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)
}

pub async fn revoke_attestation_request(
    attestation_request_id: &Uuid,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<PgQueryResult, AppError> {
    sqlx::query!(
        "UPDATE attestation_requests SET revoked = true, revoked_at = NOW() WHERE id = $1",
        attestation_request_id
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)
}

pub async fn update_attestation_request(
    attestation_request_id: &Uuid,
    credential: &Credential,
    db_executor: &PgPool,
) -> Result<AttestationResponse, AppError> {
    sqlx::query_as!(
        AttestationResponse,
        r#"UPDATE attestation_requests SET credential = $1 WHERE id = $2 AND approved = false 
        RETURNING id, approved, revoked, created_at, deleted_at, updated_at, approved_at, revoked_at, ctype_hash, credential, claimer, tx_state as "tx_state: TxState""#,
        serde_json::json!(credential),
        attestation_request_id
    )
    .fetch_one(db_executor)
    .await
    .map_err(AppError::from)
}

pub async fn attestation_requests_kpis(pool: &PgPool) -> Result<AttestationKPIs, sqlx::Error> {
    let attestations_created_over_time = sqlx::query_as!(
        AttestationCreatedOverTime,
        "SELECT date_trunc('day', created_at) AS date, COUNT(*) AS total_attestations_created
         FROM attestation_requests
         GROUP BY date
         ORDER BY date;",
    )
    .fetch_all(pool)
    .await?;

    let attestations_not_approved =
        sqlx::query_scalar!("SELECT COUNT(*) FROM attestation_requests WHERE approved = FALSE;")
            .fetch_one(pool)
            .await
            .map_or(0, |count| count.unwrap());

    let attestations_revoked =
        sqlx::query_scalar!("SELECT COUNT(*) FROM attestation_requests WHERE revoked = TRUE;")
            .fetch_one(pool)
            .await
            .map_or(0, |count| count.unwrap());

    let total_claimers =
        sqlx::query_scalar!("SELECT COUNT(DISTINCT claimer) FROM attestation_requests;")
            .fetch_one(pool)
            .await
            .map_or(0, |count| count.unwrap());

    Ok(AttestationKPIs {
        attestations_created_over_time,
        attestations_not_approved,
        attestations_revoked,
        total_claimers,
    })
}

pub async fn generate_new_session(pool: &PgPool) -> Result<Uuid, AppError> {
    let result = sqlx::query!("INSERT INTO session_request DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(result.id)
}
