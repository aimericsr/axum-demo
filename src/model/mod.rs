mod error;
mod store;
pub mod task;

pub use self::error::{Error, Result};
use self::store::{new_db_pool, Db};

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager { db: db })
    }

    /// Returns the sqlx db pool reference.
    /// (Only accesible for the module below the model layer)
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
