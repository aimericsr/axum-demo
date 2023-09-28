pub mod task;
pub mod user;
pub use self::error::{Error, Result};

mod base;
mod error;
mod store;

use self::store::{new_db_pool, new_db_pool_without_db, Db};

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    /// Setup the connection to the db
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager { db })
    }

    pub async fn migrate(self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.db)
            .await
            .map_err(|ex| Error::MigrateError(ex))
    }

    /// Returns the sqlx db pool reference.
    /// (Only accesible for the module below the model layer)
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
