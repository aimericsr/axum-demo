use crate::{crypt, model, web};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

/// Result type to not have to specified the web::Error type each time for other modules
pub type Result<T> = core::result::Result<T, Error>;

/// Error type that can be return from handlers, services or custom extractors. See this impl IntoResponse for this type
#[derive(Debug, Error, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // -- Login
    #[error("Username not found : {username:?}")]
    LoginFailUsernameNotFound { username: String },
    #[error("Hash no password : {user_id:?}")]
    LoginFailUserHasNoPwd { user_id: i64 },
    #[error("Password not matching : {user_id:?}")]
    LoginFailPwdNotMatching { user_id: i64 },

    // -- Json
    #[error("Wrong json schema provided")]
    JsonSchema,
    #[error("Json validatin failed")]
    JsonValidation(#[from] validator::ValidationErrors),

    // -- Rate Limit
    #[error("The Rate Limit has been reached")]
    RateLimitExceeded,

    // -- Timeout
    #[error("The request took too long to complete")]
    Timeout,

    // -- FallBack
    #[error("No route matche the requested Uri")]
    FallBack,

    /// -- Ctx Error
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

/// This implementations give us the ability to return this error directly from handlers, services or custom extractors.
/// When the response will be build, this function will be called and web::Error will be added to the extensions of the request via this method.
/// Then all responses will pass through the mw_res_map function to be converted to web::ClientError before sending it to the clients.
/// This architecture centralize the error handling of all the errors and limit the potential leak of sensitive data
/// because there is a final type([ClientError]) to hide some informations to the client.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        response.extensions_mut().insert(self);
        response
    }
}

impl Error {
    /// From the root error to the http status code and [ClientError].
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use web::Error::*;

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

            // -- Json
            JsonValidation(validation_errors) => (
                StatusCode::BAD_REQUEST,
                ClientError::JSON_VALDIDATION {
                    errors: validation_errors.to_owned(),
                },
            ),
            JsonSchema => (StatusCode::BAD_REQUEST, ClientError::JSON_SCHEMA),

            // -- Rate Limit
            RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                ClientError::RATE_LIMIT_EXCEEDED,
            ),

            // -- Timeout
            Timeout => (StatusCode::REQUEST_TIMEOUT, ClientError::TIMEOUT),

            // -- FallBack
            FallBack => (StatusCode::NOT_FOUND, ClientError::ROUTE_NOT_FOUND),

            // -- Fallback.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

/// Error type that will be sent to the client.
/// Used to hide some informations that we don't want our client to have.
#[derive(Debug, Serialize, Error, strum_macros::AsRefStr, ToSchema)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
    #[error("LOGIN_FAIL")]
    LOGIN_FAIL,
    #[error("NO_AUTH")]
    NO_AUTH,
    #[error("RATE_LIMIT_EXCEEDED")]
    RATE_LIMIT_EXCEEDED,
    #[error("TIMEOUT")]
    TIMEOUT,
    #[error("ROUTE_NOT_FOUND")]
    ROUTE_NOT_FOUND,
    #[error("JSON_VALDIDATIONdmokfjsfjk")]
    JSON_VALDIDATION { errors: validator::ValidationErrors },
    #[error("JSON_SCHEMA")]
    JSON_SCHEMA,
    #[error("ENTITY_NOT_FOUND")]
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
    #[error("SERVICE_ERROR")]
    SERVICE_ERROR,
}
