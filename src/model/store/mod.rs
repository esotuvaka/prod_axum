mod error;

use std::time::Duration;

pub use self::error::{Error, Result};

use crate::config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tracing::debug;

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> Result<Db> {
    // Tokio async testing framework has issues with scheduling / async connections,
    // so limit to just 1 connection when testing
    let max_connections = if cfg!(test) { 1 } else { 5 };
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&config().DB_URL)
        .await
        .map_err(|ex| Error::FailedToCreatePool(ex.to_string()))
}
