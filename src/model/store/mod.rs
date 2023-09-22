mod error;
pub use self::error::{Error, Result};
use crate::config::config;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> Result<Db> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config().postgres.db_url.expose_secret())
        .await
        .map_err(|ex| Error::FailToCreatePool(ex.to_string()))
}
