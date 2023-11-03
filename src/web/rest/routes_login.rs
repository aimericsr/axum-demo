use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForLogin};
use crate::model::ModelManager;
use crate::startup::SharedState;
use crate::web::{self, remove_token_cookie, Error, Result};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use hmac::digest::typenum::Mod;
use hyper::Body;
use opentelemetry::KeyValue;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::debug;
use utoipa::{IntoParams, ToSchema};
use validator::{Validate, ValidationError};

pub fn routes() -> Router<SharedState> {
    Router::new().nest("/api", sub_routes())
}

fn sub_routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(api_login))
        .route("/logoff", post(api_logoff_handler))
}

async fn check() -> &'static str {
    "check"
}

// region:    --- Structs
#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    result: LoginResponseResult,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponseResult {
    success: bool,
}
// endregion: --- Structs

// region:    --- Login
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginPayload {
    //#[validate(length(min = 1, message = "Can not be empty"))]
    pub username: String,
    //#[validate]
    pub pwd: String,
}

#[utoipa::path(
    post,
    context_path = "/api",
    path = "/login",
    tag = "account",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "Login successfully", body = LoginResponse),
        (status = 403, description = "Login Fail"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("api_key" = ["aaa","bb"])
    )
)]
async fn api_login(
    State(state): State<SharedState>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>> {
    debug!("{:<12} - api_login", "HANDLER");

    let LoginPayload {
        username,
        pwd: pwd_clear,
    } = payload;
    let root_ctx = Ctx::root_ctx();

    // -- Get the user.
    let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &state.mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound { username })?;
    let user_id = user.id;

    // -- Validate the password.
    let Some(pwd) = user.pwd else {
        return Err(Error::LoginFailUserHasNoPwd { user_id });
    };

    pwd::validate_pwd(
        &EncryptContent {
            salt: user.pwd_salt.to_string(),
            content: pwd_clear.clone(),
        },
        &pwd,
    )
    .map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

    // -- Set web token.
    web::set_token_cookie(&cookies, &user.username, &user.token_salt.to_string())?;

    let body = Json(LoginResponse {
        result: LoginResponseResult { success: true },
    });

    Ok(body)
}

// endregion:    --- Login

// region:    --- Logoff
#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct LogoffPayload {
    pub logoff: bool,
}

#[utoipa::path(
    post,
    context_path = "/api",
    path = "/logoff",
    tag = "account",
    params(
        LogoffPayload
    ),
    responses(
        (status = 200, description = "Logoff succesful",  body = LoginResponse),
        (status = 500, description = "Internal Server Error")
    )
)]
async fn api_logoff_handler(
    cookies: Cookies,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<LoginResponse>> {
    debug!("{:<12} - api_logoff_handler", "HANDLER");
    let should_logoff = payload.logoff;

    if should_logoff {
        remove_token_cookie(&cookies)?;
    }

    // Create the success body.
    let body = Json(LoginResponse {
        result: LoginResponseResult {
            success: should_logoff,
        },
    });

    Ok(body)
}
// endregion: --- Logoff
