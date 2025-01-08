use std::sync::Arc;

use crate::{crypt, model, web};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use http::{header, HeaderMap};
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

/// Result type to not have to specified the web::Error type each time for other modules
pub type Result<T> = core::result::Result<T, Error>;

/// Error type that can be return from handlers, services or custom extractors. See the IntoResponse impl for this type
#[derive(Debug, Error, strum_macros::AsRefStr)]
//#[serde(tag = "type", content = "data")]
// need Serialize Trait ?
pub enum Error {
    // -- Login
    #[error("Username not found : {username:?}")]
    LoginFailUsernameNotFound { username: String },
    #[error("Hash no password : {user_id:?}")]
    LoginFailUserHasNoPwd { user_id: i64 },
    #[error("Password not matching : {user_id:?}")]
    LoginFailPwdNotMatching { user_id: i64 },

    // -- RPC
    #[error("RpcMethodUnknown")]
    RpcMethodUnknown(String),
    #[error("RpcMissingParams")]
    RpcMissingParams { rpc_method: String },
    #[error("RpcFailJsonParams")]
    RpcFailJsonParams { rpc_method: String },

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
    #[error("SerdeJsonError")]
    SerdeJson(#[from] serde_json::Error),
}

/// This implementations give us the ability to return this error directly from handlers, services or custom extractors.
/// When this type will be return from handlers, the response will be build and this function will be called,
/// web::Error will be added to the extensions of the request via this method.
/// Then all responses will pass through the mw_res_map function to be converted to web::ClientError before sending it to the clients.
/// This architecture centralize the error handling of all the errors and limit the potential leak of sensitive data
/// because there is a final type([ClientError]) to hide some informations to the client.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        response.extensions_mut().insert(Arc::new(self));
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
                    errors: validation_errors.to_string(),
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

            // -- FallBack Routing
            FallBack => (StatusCode::NOT_FOUND, ClientError::ROUTE_NOT_FOUND),

            // -- Other Errors
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
#[allow(non_camel_case_types)]
pub enum ClientError {
    #[error("The login failed")]
    LOGIN_FAIL,
    #[error("The authentication failed")]
    NO_AUTH,
    #[error("The rate limit has been reached")]
    RATE_LIMIT_EXCEEDED,
    #[error("The request took too long to complete")]
    TIMEOUT,
    #[error("The route does not exist")]
    ROUTE_NOT_FOUND,
    #[error("The json is not valid, please make sure that the json is valid")]
    JSON_VALDIDATION { errors: String },
    //JSON_VALDIDATION { errors: validator::ValidationErrors },
    #[error("The json schema is not valid, please make sure to use the right schema")]
    JSON_SCHEMA,
    #[error("The entity {entity} with id {id} does not exist")]
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
    #[error("Service error, please contact the administrator")]
    SERVICE_ERROR,
}

// RFC 7807
#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    pub type_url: String,
    pub title: String,
    pub status: u16,
    pub detail: Option<String>,
    pub instance: Option<String>,
    #[serde(flatten)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let body = match serde_json::to_string(&self) {
            Ok(json) => json,
            Err(_) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                    .into_response();
            }
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/problem+json"),
        );

        // Set the response status and body
        (
            StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            headers,
            body,
        )
            .into_response()
    }
}

pub struct ProblemDetailsBuilder {
    type_url: Option<String>,
    title: Option<String>,
    status: Option<StatusCode>,
    detail: Option<String>,
    instance: Option<String>,
    extensions: std::collections::HashMap<String, serde_json::Value>,
}

impl ProblemDetailsBuilder {
    pub fn new() -> Self {
        Self {
            type_url: None,
            title: None,
            status: None,
            detail: None,
            instance: None,
            extensions: std::collections::HashMap::new(),
        }
    }

    pub fn type_url(mut self, type_url: impl Into<String>) -> Self {
        self.type_url = Some(type_url.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn status(mut self, status: impl Into<StatusCode>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    pub fn extension(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.extensions.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> core::result::Result<ProblemDetails, &'static str> {
        if let (Some(type_url), Some(title), Some(status)) =
            (self.type_url, self.title, self.status)
        {
            Ok(ProblemDetails {
                type_url,
                title,
                status: status.as_u16(),
                detail: self.detail,
                instance: self.instance,
                extensions: self.extensions,
            })
        } else {
            Err("Missing required fields: type_url, title, and status")
        }
    }
}
