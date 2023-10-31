use serde::Serialize;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error, Serialize, PartialEq)]
pub enum Error {
    // -- Key
    #[error("Key fail")]
    KeyFailHmac,

    // -- Pwd
    #[error("The passwords didn't match")]
    PwdNotMatching,

    // -- Token
    #[error("The token has an invalid format")]
    TokenInvalidFormat,
    #[error("The ident part of the token could not be decoded")]
    TokenCannotDecodeIdent,
    #[error("The exp part of the token could not be decoded")]
    TokenCannotDecodeExp,
    #[error("The signatures token are not matching")]
    TokenSignatureNotMatching,
    #[error("The token format of the Exp is not ISO compliante")]
    TokenExpNotIso,
    #[error("The token has expired")]
    TokenExpired,
}
