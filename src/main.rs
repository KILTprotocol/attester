pub mod configuration;
pub mod database;
pub mod error;
pub mod routes;
pub mod tx;
pub mod utils;

use actix_cors::Cors;
use actix_web::{dev::ServiceRequest, middleware::Logger, web, App, HttpMessage, HttpServer};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use clap::Parser;
use configuration::Configuration;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use sqlx::{Pool, Postgres};

use crate::routes::get_attestation_request_scope;

#[derive(Clone)]
pub struct AppState {
    pub config: Configuration,
    pub db_executor: Pool<Postgres>,
}

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

async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let (http_req, payload) = req.into_parts();

    let app_data = http_req.app_data::<web::Data<AppState>>().ok_or((
        actix_web::error::ErrorInternalServerError("App data are not set"),
        ServiceRequest::from_parts(http_req.clone(), actix_web::dev::Payload::None),
    ))?;

    let token = credentials.token();

    let jwt_secret = &app_data.config.jwt_secret;

    let secret: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).map_err(|_| {
        (
            actix_web::error::ErrorInternalServerError("Secret is in wrong format"),
            ServiceRequest::from_parts(http_req.clone(), actix_web::dev::Payload::None),
        )
    })?;

    let jwt_payload: JWTPayload = token.verify_with_key(&secret).map_err(|_| {
        (
            actix_web::error::ErrorUnauthorized("JWT Verification did not succeed"),
            ServiceRequest::from_parts(http_req.clone(), actix_web::dev::Payload::None),
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

    let request = ServiceRequest::from_parts(http_req, payload);
    request.extensions_mut().insert(user);

    Ok(request)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = Configuration::parse();

    let host_name = config.host_name.clone();
    let port = config.port.clone();
    let front_end_path = config.front_end_path.clone();
    let db_executor = database::connection::init(&config.database_url).await;

    let app_state = AppState {
        config,
        db_executor,
    };

    log::info!("started server at {}:{}", host_name, port);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        let logger = Logger::default();
        let auth = HttpAuthentication::bearer(jwt_validator);

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(logger)
            .wrap(cors)
            .service(get_attestation_request_scope().wrap(auth))
            .service(actix_files::Files::new("/", &front_end_path).index_file("index.html"))
    })
    .bind((host_name, port))?
    .run()
    .await
}
