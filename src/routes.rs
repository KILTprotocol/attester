use actix_web::{
    delete, get, post, put,
    web::{self, ReqData},
    HttpResponse, Scope,
};
use subxt::ext::sp_core::H256;
use uuid::Uuid;

use crate::{
    database::{
        dto::{ChallengeData, Credential, Pagination, Query},
        querys::{
            approve_attestation_request_tx, attestation_requests_kpis, can_approve_attestation_tx,
            can_revoke_attestation, delete_attestation_request, generate_new_session,
            get_attestation_request_by_id, get_attestation_requests, get_attestations_count,
            insert_attestation_request, mark_attestation_request_in_flight,
            record_attestation_request_failed, revoke_attestation_request,
            update_attestation_request,
        },
    },
    error::AppError,
    utils::{is_user_admin, is_user_allowed_to_see_data, is_user_allowed_to_update_data},
    AppState, User,
};

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

#[get("/metric/kpis")]
async fn get_attestation_kpis(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let kpis = attestation_requests_kpis(&state.db_executor).await?;
    Ok(HttpResponse::Ok().json(serde_json::to_value(kpis)?))
}

#[get("/api/v1/challenge")]
async fn challenge_handler(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let app_name = state.config.app_name.clone();
    let challenge = generate_new_session(&state.db_executor).await?;

    let encryption_key_uri = "key".to_string();
    let challenge_data = ChallengeData {
        challenge: challenge.as_bytes().to_vec(),
        app_name,
        encryption_key_uri,
    };
    Ok(HttpResponse::Ok().json(challenge_data))
}

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
