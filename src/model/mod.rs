pub mod task;
pub mod user;
pub use self::error::{Error, Result};
use self::store::{new_db_pool, Db};
use axum_macros::FromRef;

mod base;
mod error;
mod store;

#[derive(Clone, FromRef)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    /// Setup the connection to the db
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager { db })
    }

    /// Create the db and setup the connection to the db
    /// Only for tests
    // pub async fn new_for_test(db_name: String) -> Result<Self> {
    //     let db = new_db_pool_without_db().await;
    //     let mut connection = PgConnection::connect_with(&db)
    //         .await
    //         .expect("Failed to connect to Postgres");
    //     connection
    //         .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
    //         .await
    //         .expect("Failed to create database.");
    //     // Migrate database
    //     // let connection_pool = PgPool::connect_with(config.with_db())
    //     //     .await
    //     //     .expect("Failed to connect to Postgres.");
    //     connection
    //     sqlx::migrate!("./migrations")
    //         .run(&mut connection)
    //         .await
    //         .expect("Failed to migrate the database");

    //     Ok(ModelManager { db })
    // }

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
