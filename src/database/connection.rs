use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn init(database_url: &str) -> anyhow::Result<Pool<Postgres>> {
    let pool = connect(database_url)
        .await
        .context("Connection to database should not fail")?;

    log::info!("Connection to database established");

    sqlx::migrate!()
        .run(&pool)
        .await
        .context("Migration should not fail")?;

    log::info!("Database migration succeded");

    Ok(pool)
}

pub async fn connect(database_url: &str) -> anyhow::Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
        .context("Creating database connection failed")?;
    Ok(pool)
}
