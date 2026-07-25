use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::config::DatabaseConfig;

pub async fn connect(config: &DatabaseConfig) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.url)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to connect to PostgreSQL");
            e
        })?;

    tracing::info!(
        max_connections = config.max_connections,
        "Connected to PostgreSQL database"
    );
    Ok(pool)
}
