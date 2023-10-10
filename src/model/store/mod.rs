mod error;
pub use self::error::{Error, Result};
use crate::config::config;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> Result<Db> {
    let user = config().postgres.db_user.expose_secret();
    let password = config().postgres.db_password.expose_secret();
    let host = config().postgres.db_host.expose_secret();
    let db = config().postgres.db_name.expose_secret();
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!("postgres://{user}:{password}@{host}/{db}"))
        .await
        .map_err(|ex| Error::FailToCreatePool(ex.to_string()))
}

// pub async fn new_db_pool_without_db() -> PgConnectOptions {
//     let user = config().postgres.db_user.expose_secret();
//     let password = config().postgres.db_password.expose_secret();
//     let host = config().postgres.db_host.expose_secret();

//     PgConnectOptions::new()
//         .host(host)
//         .username(user)
//         .password(password)
// }
