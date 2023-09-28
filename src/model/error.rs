use crate::{crypt, model::store};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use sqlx::migrate::MigrateError;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as] // implmente trait Serialize for sqlx::Error
#[derive(Debug, Serialize)]
pub enum Error {
    EntityNotFound { entity: &'static str, id: i64 },

    // -- Modules
    Store(store::Error),
    Crypt(crypt::Error),

    // - Externals
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    MigrateError(#[serde_as(as = "DisplayFromStr")] MigrateError),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate

// region:    --- Froms
impl From<store::Error> for Error {
    fn from(val: store::Error) -> Self {
        Self::Store(val)
    }
}

impl From<crypt::Error> for Error {
    fn from(val: crypt::Error) -> Self {
        Self::Crypt(val)
    }
}

impl From<sqlx::Error> for Error {
    fn from(val: sqlx::Error) -> Self {
        Self::Sqlx(val)
    }
}
// endregion: --- Froms
