use serde::Serialize;
use strum_macros::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Display, Serialize)]
pub enum Error {}

impl std::error::Error for Error {}

