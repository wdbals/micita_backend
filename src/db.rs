use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub async fn connect_to_db() -> Result<Pool<Postgres>, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(std::time::Duration::from_secs(30)) // Cierra conexiones inactivas después de 30s
        .connect(&database_url)
        .await
}
