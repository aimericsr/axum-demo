use crate::model;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    // -- Config
    #[error("Failed to load the environnement variable file, not found : `{0}`")]
    ConfigMissingEnv(&'static str),
    #[error("Failed to load the environnement variable file, wrong format : `{0}`")]
    ConfigWrongFormat(&'static str),

    // -- Modules
    #[error("Model layer error")]
    Model(#[from] model::Error),
}
