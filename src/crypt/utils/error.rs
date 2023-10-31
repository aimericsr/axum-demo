use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    // -- Time
    #[error("Failed to parse the date `{0}`")]
    DateFailParse(String),

    // -- Base64
    #[error("Failed to decode base64")]
    FailToB64uDecode,
}
