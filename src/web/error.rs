use crate::{crypt, model, web};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // -- RPC
    #[error("Rpc unknown methode : `{0}`")]
    RpcMethodUnknown(String),
    #[error("Rpc missing params : {rpc_method:?}")]
    RpcMissingParams { rpc_method: String },
    #[error("Rpc fail json params : {rpc_method:?}")]
    RpcFailJsonParams { rpc_method: String },

    // -- Login
    #[error("Username not found : {username:?}")]
    LoginFailUsernameNotFound { username: String },
    #[error("Hash no password : {user_id:?}")]
    LoginFailUserHasNoPwd { user_id: i64 },
    #[error("Password not matching : {user_id:?}")]
    LoginFailPwdNotMatching { user_id: i64 },

    // -- Ctx Error
    #[error("Can't create the context")]
    CtxExt(web::mw_auth::CtxExtError),

    // -- Modules
    #[error("Model layer error")]
    Model(#[from] model::Error),
    #[error("Crypt layer error")]
    Crypt(#[from] crypt::Error),

    // -- External Modules
    #[error("Serde json error : `{0}`")]
    SerdeJson(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Create a placeholder Axum reponse.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the reponse.
        response.extensions_mut().insert(self);

        response
    }
}

/// From the root error to the http status code and ClientError
impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use web::Error::*;
        #[allow(unreachable_patterns)]
        match self {
            // -- Login
            LoginFailUsernameNotFound { .. }
            | LoginFailUserHasNoPwd { .. }
            | LoginFailPwdNotMatching { .. } => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            // -- Auth
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // -- Model
            Model(model::Error::EntityNotFound { entity, id }) => (
                StatusCode::BAD_REQUEST,
                ClientError::ENTITY_NOT_FOUND { entity, id: *id },
            ),

            // -- Fallback.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, Serialize, strum_macros::AsRefStr, ToSchema)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
    SERVICE_ERROR,
}
