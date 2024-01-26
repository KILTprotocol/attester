mod auth;
mod cli;
mod configuration;
mod database;
mod error;
mod kilt;
mod routes;
mod utils;

// external imports
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use anyhow::Context;
use clap::Parser;
use sodiumoxide::crypto::box_::SecretKey;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use subxt::{ext::sp_core::sr25519::Pair, tx::PairSigner, utils::AccountId32, OnlineClient};

// internal imports
use auth::jwt_validator;
use cli::Cli;
use configuration::{Configuration, SessionConfig};
use kilt::{create_well_known_did_config, KiltConfig, WellKnownDidConfig};
use routes::{
    get_attestation_request_scope, get_challenge_scope, get_credential_scope,
    well_known_did_config_handler,
};

/// App State of the application. No need of read/write locks since we read only from the state.
#[derive(Clone)]
pub struct AppState {
    pub payer: Arc<PairSigner<KiltConfig, Pair>>,
    pub signer: Arc<PairSigner<KiltConfig, Pair>>,
    pub app_name: String,
    pub jwt_secret: String,
    pub chain_client: Arc<OnlineClient<KiltConfig>>,
    pub db_executor: Arc<Pool<Postgres>>,
    pub attester_did: AccountId32,
    pub well_known_did_config: WellKnownDidConfig,
    pub session: SessionConfig,
    pub encryption_key: SecretKey,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cli = Cli::parse();

    let config = cli.get_config()?;

    let attester_did = config.get_did().context("Did should be set")?;

    log::info!("Did: {}", attester_did);

    let port = config.port.clone();
    let front_end_path = config.front_end_path.clone();

    let db_executor = database::connection::init(&config.database_url).await?;

    #[cfg(feature = "spiritnet")]
    log::info!(
        "Spiritnet features are enabled. WSS address is set to: {}",
        &config.kilt_endpoint
    );

    #[cfg(not(feature = "spiritnet"))]
    log::info!(
        "Peregrine features are enabled. WSS address is set to: {}",
        &config.kilt_endpoint
    );

    let signer = config
        .get_credential_signer()
        .context("Creating payer should not fail.")?;

    let payer = config
        .get_payer_signer()
        .context("Creating signer should not fail.")?;

    let chain_client = config
        .get_client()
        .await
        .context("Creation of online client failed")?;

    let encryption_key = config
        .get_nacl_secret_key()
        .context("Creating of encryption key failed.")?;

    let well_known_did_config = create_well_known_did_config(&config.well_known_did_config)
        .context("Creating well known did config should not fail.")?;

    let app_state = AppState {
        session: config.session,
        jwt_secret: config.jwt_secret,
        app_name: config.app_name,
        well_known_did_config,
        db_executor: Arc::new(db_executor),
        payer: Arc::new(payer),
        signer: Arc::new(signer),
        chain_client: Arc::new(chain_client),
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
            .service(get_attestation_request_scope().wrap(auth.clone()))
            .service(get_challenge_scope().wrap(auth.clone()))
            .service(get_credential_scope().wrap(auth.clone()))
            .service(well_known_did_config_handler)
            .service(actix_files::Files::new("/", &front_end_path).index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
