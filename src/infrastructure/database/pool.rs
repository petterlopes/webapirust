use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::DatabaseConfig;
use crate::shared::error::{AppError, AppResult};

pub async fn init_pool(config: &DatabaseConfig) -> AppResult<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.uri)
        .await
        .map_err(AppError::from)
}
