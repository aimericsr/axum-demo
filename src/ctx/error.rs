use serde::Serialize;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error, Serialize)]
pub enum Error {
    #[error("Can't create the root ctx")]
    CtxCannotNewRootCtx,
}
