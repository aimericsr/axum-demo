use crate::{crypt, model, web};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use http::{header, HeaderMap};
use serde::Serialize;
use std::sync::Arc;
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

/// Represents a problem details object as defined by RFC 7807.
/// This structure provides a standardized format for returning error details
/// in HTTP responses, including additional custom fields.
#[derive(Debug, Serialize, ToSchema)]
pub struct ProblemDetails {
    /// A URI reference that identifies the problem type.
    /// This should point to a human-readable document about the error type.
    #[schema(example = "http://localhost:8080/swagger-ui/#/Account/account_login")]
    type_url: String,

    /// A short, human-readable summary of the problem type.
    #[schema(example = "JSON_VALIDATION")]
    title: String,

    /// The HTTP status code applicable to the problem, as an integer.
    #[schema(example = 404)]
    status: u16,

    /// An optional detailed, human-readable explanation specific to the occurrence of the problem.
    #[schema(example = "JSON_VALIDATION_DETAIL")]
    detail: Option<String>,

    /// An optional URI reference that identifies the specific occurrence of the problem.
    #[schema(example = "/account/login")]
    instance: Option<String>,

    /// A map for including custom fields not defined in RFC 7807.
    #[serde(flatten)]
    extensions: std::collections::HashMap<String, serde_json::Value>,

    /// A custom field representing the trace ID for correlating logs or tracking issues.
    #[schema(example = "afb61afc9b97368003e84351d3eb7586")]
    trace_id: String,
}

impl IntoResponse for ProblemDetails {
    /// Converts the `ProblemDetails` object into an HTTP response with a
    /// `Content-Type` of `application/problem+json`.
    ///
    /// # Returns
    /// - An HTTP response with the serialized problem details as the body.
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

        (
            StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            headers,
            body,
        )
            .into_response()
    }
}

/// A builder for constructing `ProblemDetails` objects.
/// This follows the builder pattern for optional configuration and ease of use.
#[derive(Debug)]
pub struct ProblemDetailsBuilder {
    type_url: Option<String>,
    title: Option<String>,
    status: Option<StatusCode>,
    detail: Option<String>,
    instance: Option<String>,
    extensions: std::collections::HashMap<String, serde_json::Value>,
    trace_id: Option<String>,
}

impl Default for ProblemDetailsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProblemDetailsBuilder {
    /// Creates a new `ProblemDetailsBuilder` instance with no fields set.
    pub fn new() -> Self {
        Self {
            type_url: None,
            title: None,
            status: None,
            detail: None,
            instance: None,
            extensions: std::collections::HashMap::new(),
            trace_id: None,
        }
    }

    /// Sets the `type_url` field.
    pub fn type_url(mut self, type_url: impl Into<String>) -> Self {
        self.type_url = Some(type_url.into());
        self
    }

    /// Sets the `title` field.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the `status` field.
    pub fn status(mut self, status: impl Into<StatusCode>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Sets the `detail` field.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Sets the `instance` field.
    pub fn instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    /// Adds a custom extension field.
    pub fn extension(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.extensions.insert(key.into(), value.into());
        self
    }

    /// Sets the `trace_id` field.
    pub fn trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Builds the `ProblemDetails` object.
    ///
    /// # Panics
    /// Panics if any of the required fields (`type_url`, `title`, `status`, or `trace_id`) are missing.
    pub fn build(self) -> ProblemDetails {
        if let (Some(type_url), Some(title), Some(status), Some(trace_id)) =
            (self.type_url, self.title, self.status, self.trace_id)
        {
            ProblemDetails {
                type_url,
                title,
                status: status.as_u16(),
                detail: self.detail,
                instance: self.instance,
                extensions: self.extensions,
                trace_id,
            }
        } else {
            panic!("Missing required fields: type_url, title, status, and trace_id")
        }
    }
}
