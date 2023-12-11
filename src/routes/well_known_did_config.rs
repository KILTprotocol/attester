use actix_web::{get, web, HttpResponse};

use crate::{error::AppError, AppState};

#[get("/.well-known/did-configuration.json")]
async fn well_known_did_config_handler(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(&app_state.well_known_did_config))
}
