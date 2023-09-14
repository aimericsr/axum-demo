use crate::{crypt, model::store};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use strum_macros::Display;

pub type Result<T> = core::result::Result<T, Error>;

// implmente trait Serialize for sqlx::Error
#[serde_as]
#[derive(Debug, Display, Serialize)]
pub enum Error {
    EntityNotFound { entity: &'static str, id: i64 },

    // -- Modules
    Store(store::Error),
    Crypt(crypt::Error),
    // - Externals
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

// enable ? for this line :  let db = new_db_pool().await?;
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

impl std::error::Error for Error {}
