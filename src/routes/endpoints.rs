use actix_web::{get, web, HttpResponse, Scope};

use crate::{error::AppError, AppState};

#[get("")]
async fn get_endpoints(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let auth_endpoint = state.auth_url.clone();
    let wss_endpoint = state.kilt_endpoint.clone();
    Ok(HttpResponse::Ok().json(vec![auth_endpoint, wss_endpoint]))
}

pub fn get_endpoint_scope() -> Scope {
    web::scope("/api/v1/endpoints").service(get_endpoints)
}
