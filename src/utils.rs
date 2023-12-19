use actix_web::web::ReqData;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::User,
    database::{dto::AttestationResponse, querys::get_attestation_request_by_id},
    error::AppError,
};

pub fn is_user_allowed_to_see_data(
    user: ReqData<User>,
    attestatations: &Vec<AttestationResponse>,
) -> bool {
    let user_ids = attestatations
        .iter()
        .map(|a| &a.claimer)
        .all(|claimer| claimer == &user.id);

    if user_ids || user.is_admin {
        true
    } else {
        false
    }
}

pub async fn is_user_allowed_to_update_data(
    user: &User,
    attestation_id: &Uuid,
    db_executor: &PgPool,
) -> Result<bool, AppError> {
    let attestation = get_attestation_request_by_id(attestation_id, db_executor).await?;
    if attestation.claimer == user.id || user.is_admin {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn is_user_admin(user: &User) -> bool {
    user.is_admin
}
