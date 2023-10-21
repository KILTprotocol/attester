use crate::{
    database::dto::{AttestationResponse, Pagination, TxState},
    error::AppError,
};

use sqlx::{postgres::PgQueryResult, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use super::dto::{AttestationRequest, Credential};

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
    attestation_request: &AttestationRequest,
    credential: &Credential,
    db_executor: &PgPool,
) -> Result<AttestationResponse, AppError> {
    let result = sqlx::query_as!(
        AttestationResponse,
        r#"INSERT INTO attestation_requests (ctype_hash, claimer, credential) VALUES ($1, $2, $3) 
        RETURNING  id, approved, revoked, created_at, deleted_at, updated_at, approved_at, revoked_at, ctype_hash, credential, claimer, tx_state as "tx_state: TxState""#,
        attestation_request.ctype_hash,
        attestation_request.claimer,
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

pub async fn approve_attestation_request_tx(
    attestation_request_id: &Uuid,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<PgQueryResult, AppError> {
    sqlx::query!(
        "UPDATE attestation_requests SET approved = true WHERE id = $1",
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
        "UPDATE attestation_requests SET revoked = true WHERE id = $1",
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
