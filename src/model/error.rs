use crate::{crypt, model::store};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use sqlx::migrate::MigrateError;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as] // implmente trait Serialize for sqlx::Error
#[derive(Debug, Error, Serialize)]
pub enum Error {
    #[error("The entity {entity:?} with the id {id:?} has not been found")]
    EntityNotFound { entity: &'static str, id: i64 },

    // -- Modules
    #[error("Error at the store level")]
    Store(#[from] store::Error),
    #[error("Error at the crypt level")]
    Crypt(#[from] crypt::Error),

    // - Externals
    #[error("Error at the sqlx level")]
    Sqlx(
        #[from]
        #[serde_as(as = "DisplayFromStr")]
        sqlx::Error,
    ),
    #[error("Error at the migration level")]
    Migrate(
        #[from]
        #[serde_as(as = "DisplayFromStr")]
        MigrateError,
    ),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_not_found_error() {
        let error = Error::EntityNotFound {
            entity: "User",
            id: 123,
        };

        assert_eq!(
            "EntityNotFound { entity: \"User\", id: 123 }",
            format!("{error:?}")
        );
    }
}
