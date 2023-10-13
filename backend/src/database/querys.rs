use crate::{
    database::dto::{AttestationResponse, Pagination},
    error::AppError,
};

use sqlx::{postgres::PgQueryResult, Execute, FromRow, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use super::dto::{AttestationRequest, Credential};

pub async fn get_attestation_request_by_id(
    attestation_request_id: &Uuid,
    db_executor: &PgPool,
) -> Result<AttestationResponse, AppError> {
    sqlx::query_as!(
        AttestationResponse,
        "SELECT * FROM attestation_requests WHERE id = $1 AND deleted_at IS NULL",
        attestation_request_id
    )
    .fetch_one(db_executor)
    .await
    .map_err(AppError::from)
}

pub async fn get_attestations_count(db_executor: &PgPool) -> i64 {
    sqlx::query_scalar!("SELECT COUNT (*) FROM attestation_requests WHERE deleted_at IS NULL")
        .fetch_one(db_executor)
        .await
        .map_or(0, |count| count.unwrap())
}

pub async fn get_attestation_requests(
    pagination: &Pagination,
    db_executor: &PgPool,
) -> Result<Vec<AttestationResponse>, AppError> {
    let query = build_pagination_query(pagination);
    get_attestations(&query, db_executor)
        .await
        .map_err(AppError::from)
}

pub fn build_pagination_query(pagination: &Pagination) -> String {
    let mut query: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT * FROM attestation_requests WHERE deleted_at IS NULL ");

    if let Some(sorting) = &pagination.sort {
        query.push(format!("ORDER BY {} {} ", sorting[0], sorting[1]));
    }

    if let Some(offset) = pagination.offset {
        query.push(format!("LIMIT {:?} OFFSET {:?}", offset[0], offset[1]));
    }

    query.build().sql().into()
}

async fn get_attestations(
    query: &str,
    db_executor: &PgPool,
) -> Result<Vec<AttestationResponse>, sqlx::Error> {
    let query_result = sqlx::query(query).fetch_all(db_executor).await;

    let attestation_rows = match query_result {
        Ok(rows) => Ok(rows),
        Err(e) => Err(e),
    }?;

    attestation_rows
        .into_iter()
        .map(|attestation| AttestationResponse::from_row(&attestation))
        .collect()
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
        "WITH CreatedRow AS (
            INSERT INTO attestation_requests (ctype_hash, claimer, credential) 
            VALUES ($1, $2, $3) RETURNING *
        ) 
        SELECT * FROM CreatedRow",
        attestation_request.ctype_hash,
        attestation_request.claimer,
        serde_json::json!(credential)
    )
    .fetch_one(db_executor)
    .await;

    result.map_err(AppError::from)
}

pub async fn can_approve_attestation(
    attestation_request_id: &Uuid,
    db_executor: &PgPool,
) -> Result<AttestationResponse, AppError> {
    sqlx::query_as!(
        AttestationResponse,
        "SELECT * FROM attestation_requests WHERE id = $1 AND approved = false AND revoked = false",
        attestation_request_id
    )
    .fetch_one(db_executor)
    .await
    .map_err(AppError::from)
}

pub async fn approve_attestation_request(
    attestation_request_id: &Uuid,
    db_executor: &PgPool,
) -> Result<PgQueryResult, AppError> {
    sqlx::query!(
        "UPDATE attestation_requests SET approved = true WHERE id = $1",
        attestation_request_id
    )
    .execute(db_executor)
    .await
    .map_err(AppError::from)
}

pub async fn can_revoke_attestation(
    attestation_request_id: &Uuid,
    db_executor: &PgPool,
) -> Result<AttestationResponse, AppError> {
    sqlx::query_as!(
        AttestationResponse,
        "SELECT * FROM attestation_requests WHERE id = $1 AND approved = true AND revoked = false",
        attestation_request_id
    )
    .fetch_one(db_executor)
    .await
    .map_err(AppError::from)
}

pub async fn revoke_attestation_request(
    attestation_request_id: &Uuid,
    db_executor: &PgPool,
) -> Result<PgQueryResult, AppError> {
    sqlx::query!(
        "UPDATE attestation_requests SET revoked = true WHERE id = $1",
        attestation_request_id
    )
    .execute(db_executor)
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
        "WITH UpdateRow AS (
            UPDATE attestation_requests SET credential = $1 
            WHERE id = $2 AND approved = false AND deleted_at IS NULL 
            RETURNING *
        ) 
        SELECT * FROM UpdateRow",
        serde_json::json!(credential),
        attestation_request_id
    )
    .fetch_one(db_executor)
    .await
    .map_err(AppError::from)
}
