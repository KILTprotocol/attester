mod cli;
mod configuration;
mod database;
mod error;
mod routes;
mod tx;
mod utils;
mod well_known_did_configuration;

use actix_cors::Cors;
use actix_web::{dev::ServiceRequest, middleware::Logger, web, App, HttpMessage, HttpServer};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use clap::Parser;
use cli::Cli;
use configuration::{Configuration, SessionConfig};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use routes::{get_attestation_request_scope, get_challenge_scope, well_known_did_config_handler};
use sha2::Sha256;
use sodiumoxide::crypto::box_::SecretKey;
use sqlx::{Pool, Postgres};
use subxt::{ext::sp_core::sr25519::Pair, tx::PairSigner, utils::AccountId32, OnlineClient};
use well_known_did_configuration::{create_well_known_did_config, WellKnownDidConfig};

use crate::tx::KiltConfig;

/// App State of the application. No need of read/write locks since we read only from the state.
#[derive(Clone)]
pub struct AppState {
    pub payer: PairSigner<KiltConfig, Pair>,
    pub signer: PairSigner<KiltConfig, Pair>,
    pub app_name: String,
    pub jwt_secret: String,
    pub api: OnlineClient<KiltConfig>,
    pub db_executor: Pool<Postgres>,
    pub attester_did: AccountId32,
    pub well_known_did_config: WellKnownDidConfig,
    pub session: SessionConfig,
    pub encryption_key: SecretKey,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cli = Cli::parse();

    let config = cli.get_config();

    let attester_did = config.get_did().expect("Did should be set");

    log::info!("Did: {}", attester_did);

    let port = config.port.clone();
    let front_end_path = config.front_end_path.clone();

    let db_executor = database::connection::init(&config.database_url).await;

    #[cfg(feature = "spiritnet")]
    log::info!(
        "Spiritnet features are enabled. WSS adress is set to: {}",
        &config.kilt_endpoint
    );

    #[cfg(not(feature = "spiritnet"))]
    log::info!(
        "Peregrine features are enabled. WSS adress is set to: {}",
        &config.kilt_endpoint
    );

    let signer = config
        .get_credential_signer()
        .expect("Creating payer should not fail.");

    let payer = config
        .get_payer_signer()
        .expect("Creating signer should not fail.");

    let api = config
        .get_client()
        .await
        .expect("Creating blockchain client should not fail.");

    let encryption_key = config.get_nacl_secret_key();

    let well_known_did_config = create_well_known_did_config(&config.well_known_did_config)
        .expect("Creating well known did config should not fail.");

    let app_state = AppState {
        session: config.session,
        jwt_secret: config.jwt_secret,
        app_name: config.app_name,
        well_known_did_config,
        db_executor,
        payer,
        signer,
        api,
        attester_did,
        encryption_key,
    };

    log::info!("started server at port: {}", port);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        let logger = Logger::default();
        let auth = HttpAuthentication::bearer(jwt_validator);

        App::new()
            .wrap(logger)
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .service(get_attestation_request_scope().wrap(auth))
            .service(get_challenge_scope())
            .service(well_known_did_config_handler)
            .service(actix_files::Files::new("/", &front_end_path).index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
