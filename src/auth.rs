use actix_web::{dev::ServiceRequest, web, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;

use crate::AppState;

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct JWTPayload {
    sub: String,
    w3n: String,
    exp: i64,
    iat: i64,
    iss: String,
    aud: String,
    pro: serde_json::Map<String, serde_json::Value>,
    nonce: String,
}

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub is_admin: bool,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let http_req = req.request();

    let app_data = http_req.app_data::<web::Data<AppState>>().ok_or((
        actix_web::error::ErrorInternalServerError("App data are not set"),
        ServiceRequest::from_request(http_req.to_owned()),
    ))?;

    let token = credentials.token();

    let jwt_secret = &app_data.jwt_secret;

    let secret: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).map_err(|_| {
        (
            actix_web::error::ErrorInternalServerError("Secret is in wrong format"),
            ServiceRequest::from_request(http_req.to_owned()),
        )
    })?;

    let jwt_payload: JWTPayload = token.verify_with_key(&secret).map_err(|_| {
        (
            actix_web::error::ErrorUnauthorized("JWT Verification did not succeed"),
            ServiceRequest::from_request(http_req.to_owned()),
        )
    })?;

    let mut id = jwt_payload.sub;

    if !id.starts_with("did:kilt") {
        id = format!("did:kilt:{}", id);
    }

    let user = User {
        id,
        is_admin: !jwt_payload.pro.is_empty(),
    };

    req.extensions_mut().insert(user);
    Ok(req)
}
