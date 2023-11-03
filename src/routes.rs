use actix_web::{
    delete, get, post, put,
    web::{self, ReqData},
    HttpResponse, Scope,
};
use subxt::ext::sp_core::H256;
use uuid::Uuid;

use crate::{
    database::{
        dto::{Credential, Pagination, Query},
        querys::{
            approve_attestation_request_tx, attestation_requests_kpis, can_approve_attestation_tx,
            can_revoke_attestation, delete_attestation_request, get_attestation_request_by_id,
            get_attestation_requests, get_attestations_count, insert_attestation_request,
            mark_attestation_request_in_flight, record_attestation_request_failed,
            revoke_attestation_request, update_attestation_request,
        },
    },
    error::AppError,
    utils::{is_user_admin, is_user_allowed_to_see_data, is_user_allowed_to_update_data},
    AppState, User,
};

/// Get attestation information by ID.
/// This endpoint allows users to retrieve attestation information by its unique ID.
///
/// # Parameters
/// - `attestation_request_id`: A unique UUID identifier for the attestation request.
/// - `user`: Information about the requesting user.
/// - `state`: Application state data.
///
/// # Returns
/// - If the user is authorized to view the attestation, the endpoint responds with a JSON representation of the attestation.
/// - If the user is not authorized, it returns an error response.
#[get("/{attestation_request_id}")]
async fn get_attestation(
    path: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let attestation_id = path.into_inner();
    let attestation = get_attestation_request_by_id(&attestation_id, &state.db_executor).await?;
    is_user_allowed_to_see_data(user, &vec![attestation.clone()])?;
    Ok(HttpResponse::Ok().json(serde_json::to_value(&attestation)?))
}

/// Get a list of attestation requests.
/// This endpoint allows users to retrieve a list of attestation requests based on pagination parameters.
///
/// # Parameters
/// - `state`: Application state data.
/// - `user`: Information about the requesting user.
/// - `pagination_query`: Query parameters for pagination.
///
/// # Returns
/// - If the user is authorized to view the attestation requests, the endpoint responds with a JSON list of attestation requests and includes a `Content-Range` header.
/// - If the user is not authorized, it returns an error response.
#[get("")]
async fn get_attestations(
    state: web::Data<AppState>,
    user: ReqData<User>,
    pagination_query: web::Query<Query>,
) -> Result<HttpResponse, AppError> {
    let mut pagination: Pagination = pagination_query.into_inner().into();
    if !user.is_admin {
        pagination.filter = Some(user.id.to_string());
    }
    let content_range = get_attestations_count(&state.db_executor).await;
    let attestation_requests = get_attestation_requests(pagination, &state.db_executor).await?;
    let response = serde_json::to_value(&attestation_requests)?;
    is_user_allowed_to_see_data(user, &attestation_requests)?;
    Ok(HttpResponse::Ok()
        .insert_header(("Content-Range", content_range))
        .json(response))
}

/// Delete an attestation request by ID.
/// This endpoint allows users to delete an attestation request by its unique ID.
///
/// # Parameters
/// - `attestation_request_id`: A unique UUID identifier for the attestation request.
/// - `user`: Information about the requesting user.
/// - `state`: Application state data.
///
/// # Returns
/// - If the user is authorized to delete the attestation, the endpoint responds with a success message.
/// - If the user is not authorized, it returns an error response 401.
#[delete("/{attestation_request_id}")]
async fn delete_attestation(
    path: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let attestation_id = path.into_inner();
    is_user_allowed_to_update_data(user, &attestation_id, &state.db_executor).await?;
    delete_attestation_request(&attestation_id, &state.db_executor).await?;
    log::info!("Attestation with id {:?} is deleted", attestation_id);
    Ok(HttpResponse::Ok().json("ok"))
}

/// Create a new attestation request.
/// This endpoint allows users to create a new attestation request.
///
/// # Parameters
/// - `body`: JSON data representing the credential for the attestation request.
/// - `state`: Application state data.
///
/// # Returns
/// - If the attestation request is successfully created, the endpoint responds with a JSON representation of the attestation request.
#[post("")]
async fn post_attestation(
    body: web::Json<Credential>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let claim_request = body.into_inner();
    let attestation = insert_attestation_request(&claim_request, &state.db_executor).await?;
    log::info!(" New attestation with id {:?} is created", attestation.id);
    Ok(HttpResponse::Ok().json(attestation))
}

/// Approve an attestation request.
/// This endpoint allows administrators to approve an attestation request, which triggers the creation of a new claim.
///
/// # Parameters
/// - `attestation_request_id`: A unique UUID identifier for the attestation request.
/// - `user`: Information about the requesting user.
/// - `state`: Application state data.
///
/// # Returns
/// - If the user is an administrator and the approval process is successful, the endpoint responds with a success message.
/// - If the user is not authorized or if there are errors in the approval process, it returns an error response.
#[put("/{attestation_request_id}/approve")]
async fn approve_attestation(
    path: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // get data
    let attestation_id = path.into_inner();
    // check role
    is_user_admin(user)?;

    // start session for db
    let mut tx = state.db_executor.begin().await?;
    let attestation = can_approve_attestation_tx(&attestation_id, &mut tx).await?;
    let credential: Credential = serde_json::from_value(attestation.credential)?;
    let ctype_hash = hex::decode(credential.claim.ctype_hash.trim_start_matches("0x").trim())?;
    let claim_hash = hex::decode(credential.root_hash.trim_start_matches("0x").trim())?;
    if claim_hash.len() != 32 || ctype_hash.len() != 32 {
        return Ok(HttpResponse::BadRequest().json("Claim hash or ctype hash have a wrong format"));
    }

    // send tx async
    tokio::spawn(async move {
        let _ = mark_attestation_request_in_flight(&attestation_id, &mut tx).await;

        let result_create_claim = crate::tx::create_claim(
            H256::from_slice(&claim_hash),
            H256::from_slice(&ctype_hash),
            state.config.clone(),
        )
        .await;

        if let Err(err) = result_create_claim {
            log::error!("Error: Something went wrong with create_claim: {:?}", err,);
            let _ = record_attestation_request_failed(&attestation_id, &mut tx).await;
            let _ = tx.commit().await;
            return;
        }

        if let Err(err) = approve_attestation_request_tx(&attestation_id, &mut tx).await {
            log::error!(
                "Error: Something went wrong with approve_attestation_request_tx: {:?}",
                err
            );
            return;
        }

        if let Err(err) = tx.commit().await {
            log::error!("Error: Something went wrong with tx.commit: {:?}", err);
            return;
        }

        log::info!("Attestation with id {:?} is approved", attestation_id);
    });

    log::info!(
        "Attestation with id {:?} is getting approved",
        attestation_id
    );
    Ok(HttpResponse::Ok().json("ok"))
}

/// Revoke an attestation request.
/// This endpoint allows users to revoke a previously approved attestation request.
///
/// # Parameters
/// - `attestation_request_id`: A unique UUID identifier for the attestation request.
/// - `user`: Information about the requesting user.
/// - `state`: Application state data.
///
/// # Returns
/// - If the user is authorized and the revocation process is successful, the endpoint responds with a success message.
/// - If the user is not authorized or if there are errors in the revocation process, it returns an error response.
#[put("/{attestation_request_id}/revoke")]
async fn revoke_attestation(
    path: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // get data
    let attestation_id = path.into_inner();
    // is user allowed
    is_user_allowed_to_update_data(user, &attestation_id, &state.db_executor).await?;

    // start db tx
    let mut tx = state.db_executor.begin().await?;
    let attestation = can_revoke_attestation(&attestation_id, &mut tx).await?;
    let credential: Credential = serde_json::from_value(attestation.credential)?;
    let claim_hash = hex::decode(credential.root_hash.trim_start_matches("0x").trim())?;
    if claim_hash.len() != 32 {
        return Ok(HttpResponse::BadRequest().json("Claim hash has a wrong format"));
    }

    // revoke attestation async in db.
    tokio::spawn(async move {
        {
            if let Err(err) =
                crate::tx::revoke_claim(H256::from_slice(&claim_hash), state.config.clone()).await
            {
                log::info!("Error: Something went wrong with revoke_claim: {:?}", err);
                return;
            }

            if let Err(err) = revoke_attestation_request(&attestation_id, &mut tx).await {
                log::info!(
                    "Error: Something went wrong with revoke_attestation_request: {:?}",
                    err
                );
                return;
            }

            if let Err(err) = tx.commit().await {
                log::info!("Error: Something went wrong with tx.commit: {:?}", err);
                return;
            }

            log::info!("Attestation with id {:?} is revoked", attestation_id);
        }
    });

    log::info!(
        "Attestation with id {:?} is getting revoked",
        attestation_id
    );
    Ok(HttpResponse::Ok().json("ok"))
}

/// Update an attestation request.
/// This endpoint allows users to update an existing attestation request.
///
/// # Parameters
/// - `attestation_request_id`: A unique UUID identifier for the attestation request to be updated.
/// - `state`: Application state data.
/// - `user`: Information about the requesting user.
/// - `body`: JSON data representing the updated credential for the attestation request.
///
/// # Returns
/// - If the user is authorized to update the attestation and the update process is successful, the endpoint responds with a JSON representation of the updated attestation request.
/// - If the user is not authorized or if there are errors in the update process, it returns an error response.
#[put("/{attestation_request_id}")]
async fn update_attestation(
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
    user: ReqData<User>,
    body: web::Json<Credential>,
) -> Result<HttpResponse, AppError> {
    // get data
    let attestation_id = path.into_inner();
    let credential = body.into_inner();
    // check role
    is_user_allowed_to_update_data(user, &attestation_id, &state.db_executor).await?;
    // update
    let attestation =
        update_attestation_request(&attestation_id, &credential, &state.db_executor).await?;
    log::info!("Attestation with id {:?} is updated", attestation_id);
    Ok(HttpResponse::Ok().json(serde_json::to_value(&attestation)?))
}

/// Get attestation key performance indicators (KPIs).
/// This endpoint allows users to retrieve key performance indicators related to attestation requests.
///
/// # Parameters
/// - `state`: Application state data.
///
/// # Returns
/// - If the request is successful, the endpoint responds with a JSON representation of attestation-related KPIs.
/// - If there are errors in the retrieval process, it returns an error response.
#[get("/metric/kpis")]
async fn get_attestation_kpis(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let kpis = attestation_requests_kpis(&state.db_executor).await?;
    Ok(HttpResponse::Ok().json(serde_json::to_value(kpis)?))
}

// Create a scope for attestation request-related endpoints.
/// This function sets up a scope for handling attestation request-related endpoints, such as approving, retrieving, creating,
/// deleting, updating, revoking attestation requests, and fetching key performance indicators (KPIs).
///
/// # Returns
/// A scope that contains various attestation request-related endpoints for handling attestation requests and related actions.
pub(crate) fn get_attestation_request_scope() -> Scope {
    web::scope("/api/v1/attestation_request")
        .service(approve_attestation)
        .service(get_attestation)
        .service(get_attestations)
        .service(post_attestation)
        .service(delete_attestation)
        .service(update_attestation)
        .service(revoke_attestation)
        .service(get_attestation_kpis)
}
