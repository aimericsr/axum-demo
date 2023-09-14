use crate::model::store;
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
    // - Externals
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

// enable ? for this line :  let db = new_db_pool().await?;
impl From<store::Error> for Error {
    fn from(val: store::Error) -> Self {
        Self::Store(val)
    }
}

impl From<sqlx::Error> for Error {
    fn from(val: sqlx::Error) -> Self {
        Self::Sqlx(val)
    }
}

impl std::error::Error for Error {}
