pub mod configuration;
pub mod database;
pub mod error;
pub mod routes;
pub mod tx;
pub mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use configuration::Configuration;
use sqlx::{Pool, Postgres};

use crate::routes::get_attestation_request_scope;

#[derive(Clone)]
pub struct AppState {
    pub config: Configuration,
    pub db_executor: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let config = configuration::init();

    let host_name = config.host_name.clone();
    let port = config.port.clone();
    let db_executor = database::utils::init(&config.database_url).await;

    let app_state = AppState {
        config,
        db_executor,
    };

    log::info!("Starting Server");
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .service(get_attestation_request_scope())
    })
    .bind((host_name, port))?
    .run()
    .await
}
