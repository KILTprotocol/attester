use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn init(database_url: &str) -> Pool<Postgres> {
    let pool = connect(database_url)
        .await
        .expect("Connection to database should not fail");

    log::info!("Connection to database established");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migration should not fail");

    log::info!("Database migration succeded");

    return pool;
}

pub async fn connect(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    Ok(pool)
}
