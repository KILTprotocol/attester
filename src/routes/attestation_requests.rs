use actix_web::{
    delete, get, post, put,
    web::{self, ReqData},
    HttpResponse, Scope,
};

use subxt::ext::sp_core::H256;
use uuid::Uuid;

use crate::{
    auth::User,
    database::{
        dto::{Credential, Pagination, Query},
        querys::{
            approve_attestation_request, attestation_requests_kpis, can_approve_attestation_tx,
            can_revoke_attestation, delete_attestation_request, get_attestation_request_by_id,
            get_attestation_requests, get_attestations_count, insert_attestation_request,
            mark_attestation_approve, mark_attestation_request_in_flight,
            record_attestation_request_failed, revoke_attestation_request,
            update_attestation_request,
        },
    },
    error::AppError,
    utils::{is_user_admin, is_user_allowed_to_see_data, is_user_allowed_to_update_data},
    AppState,
};

#[get("/{attestation_request_id}")]
async fn get_attestation(
    attestation_id: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let attestation = get_attestation_request_by_id(&attestation_id, &state.db_executor).await?;
    let is_user_allowed = is_user_allowed_to_see_data(user, &vec![attestation.clone()]);
    if is_user_allowed {
        Ok(HttpResponse::Ok().json(serde_json::to_value(&attestation)?))
    } else {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allowed to see data",
        ))?
    }
}

#[get("")]
async fn get_attestations(
    state: web::Data<AppState>,
    user: ReqData<User>,
    pagination_query: web::Query<Query>,
) -> Result<HttpResponse, AppError> {
    let mut pagination: Pagination = pagination_query.into_inner().into();
    if !is_user_admin(&user) {
        pagination.filter = Some(user.id.to_string());
    }
    let content_range = get_attestations_count(&state.db_executor).await;
    let attestation_requests = get_attestation_requests(pagination, &state.db_executor).await?;
    let response = serde_json::to_value(&attestation_requests)?;
    let is_user_allowed = is_user_allowed_to_see_data(user, &attestation_requests);

    if is_user_allowed {
        Ok(HttpResponse::Ok()
            .insert_header(("Content-Range", content_range))
            .json(response))
    } else {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allowed to see data",
        ))?
    }
}

#[delete("/{attestation_request_id}")]
async fn delete_attestation(
    attestation_id: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let is_user_allowed =
        is_user_allowed_to_update_data(&user, &attestation_id, &state.db_executor).await?;

    if !is_user_allowed {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allowed to see data",
        ))?
    }

    delete_attestation_request(&attestation_id, &state.db_executor).await?;
    log::info!("Attestation with id {:?} is deleted", attestation_id);
    Ok(HttpResponse::Ok().json("ok"))
}

#[post("")]
async fn post_attestation(
    claim_request: web::Json<Credential>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let attestation = insert_attestation_request(&claim_request, &state.db_executor).await?;
    log::info!(" New attestation with id {:?} is created", attestation.id);
    Ok(HttpResponse::Ok().json(attestation))
}

#[put("/{attestation_request_id}/approve")]
async fn approve_attestation(
    attestation_id: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // check role
    if !is_user_admin(&user) {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allowed to see data",
        ))?
    }

    // start session for db
    let mut tx = state.db_executor.begin().await?;
    let attestation = can_approve_attestation_tx(&attestation_id, &mut tx).await?;
    let credential: Credential = serde_json::from_value(attestation.credential)?;
    let ctype_hash = hex::decode(credential.claim.ctype_hash.trim_start_matches("0x").trim())?;
    let claim_hash = hex::decode(credential.root_hash.trim_start_matches("0x").trim())?;
    if claim_hash.len() != 32 || ctype_hash.len() != 32 {
        Err(actix_web::error::ErrorBadRequest(
            "Claim hash or ctype hash have a wrong format",
        ))?
    }

    let payer = state.payer.clone();
    let did = state.attester_did.clone();
    let chain_client = state.chain_client.clone();
    let signer = state.signer.clone();

    log::info!(
        "Attestation with id {:?} is getting approved",
        attestation_id
    );

    // send tx async
    tokio::spawn(async move {
        let _ = mark_attestation_request_in_flight(&attestation_id, &state.db_executor).await;

        let result_create_claim = crate::kilt::create_claim(
            H256::from_slice(&claim_hash),
            H256::from_slice(&ctype_hash),
            &did,
            &chain_client,
            &payer,
            &signer,
        )
        .await;

        if let Err(err) = result_create_claim {
            log::error!("Error: Something went wrong with create_claim: {:?}", err,);
            let _ = record_attestation_request_failed(&attestation_id, &mut tx).await;
            let _ = tx.commit().await;
            return;
        }

        if let Err(err) = approve_attestation_request(&attestation_id, &mut tx).await {
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

    Ok(HttpResponse::Ok().json("ok"))
}

#[put("/{attestation_request_id}/mark_approve")]
async fn mark_approve_attestation_request(
    attestation_id: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // check role
    if !is_user_admin(&user) {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allowed to see data",
        ))?
    }

    mark_attestation_approve(&state.db_executor, &attestation_id).await?;

    Ok(HttpResponse::Ok().json("ok"))
}

#[put("/{attestation_request_id}/revoke")]
async fn revoke_attestation(
    attestation_id: web::Path<Uuid>,
    user: ReqData<User>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // is user allowed
    if !is_user_admin(&user) {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allowed to see data",
        ))?
    }

    // start db tx
    let mut tx = state.db_executor.begin().await?;
    let attestation = can_revoke_attestation(&attestation_id, &mut tx).await?;
    let credential: Credential = serde_json::from_value(attestation.credential)?;
    let claim_hash = hex::decode(credential.root_hash.trim_start_matches("0x").trim())?;
    if claim_hash.len() != 32 {
        Err(actix_web::error::ErrorBadRequest(
            "Claim hash has a wrong format",
        ))?
    }

    let payer = state.payer.clone();
    let did = state.attester_did.clone();
    let chain_client = state.chain_client.clone();
    let signer = state.signer.clone();

    log::info!(
        "Attestation with id {:?} is getting revoked",
        attestation_id
    );

    // revoke attestation async in db.
    tokio::spawn(async move {
        {
            let _ = mark_attestation_request_in_flight(&attestation_id, &state.db_executor).await;

            if let Err(err) = crate::kilt::revoke_claim(
                H256::from_slice(&claim_hash),
                &did,
                &chain_client,
                &payer,
                &signer,
            )
            .await
            {
                log::error!("Error: Something went wrong with revoke_claim: {:?}", err);
                let _ = record_attestation_request_failed(&attestation_id, &mut tx).await;
                let _ = tx.commit().await;
                return;
            }

            if let Err(err) = revoke_attestation_request(&attestation_id, &mut tx).await {
                log::error!(
                    "Something went wrong with revoke_attestation_request: {:?}",
                    err
                );
                return;
            }

            if let Err(err) = tx.commit().await {
                log::error!("Something went wrong with tx.commit: {:?}", err);
                return;
            }

            log::info!("Attestation with id {:?} is revoked", attestation_id);
        }
    });

    Ok(HttpResponse::Ok().json("ok"))
}

#[put("/{attestation_request_id}")]
async fn update_attestation(
    attestation_id: web::Path<Uuid>,
    state: web::Data<AppState>,
    user: ReqData<User>,
    credential: web::Json<Credential>,
) -> Result<HttpResponse, AppError> {
    // check role
    let is_user_allowed =
        is_user_allowed_to_update_data(&user, &attestation_id, &state.db_executor).await?;

    if !is_user_allowed {
        Err(actix_web::error::ErrorUnauthorized(
            "User is not allowed to see data",
        ))?
    }

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

pub fn get_attestation_request_scope() -> Scope {
    web::scope("/api/v1/attestation_request")
        .service(approve_attestation)
        .service(get_attestation)
        .service(get_attestations)
        .service(post_attestation)
        .service(delete_attestation)
        .service(update_attestation)
        .service(revoke_attestation)
        .service(get_attestation_kpis)
        .service(mark_approve_attestation_request)
}
