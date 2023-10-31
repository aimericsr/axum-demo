use serde::Serialize;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error, Serialize, PartialEq)]
pub enum Error {
    #[error("Failed to create the pool to the db : `{0}`")]
    FailToCreatePool(String),
}
