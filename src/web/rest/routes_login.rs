use crate::crypt::{EncryptContent, pwd};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForLogin};
use crate::startup::SharedState;
use crate::web::error::ProblemDetails;
use crate::web::mw_validate_json::ValidatedJson;
use crate::web::{self, Error, Result, remove_token_cookie};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use redact::Secret;
use serde::{Deserialize, Serialize};
use std::result::Result as Resultstd;
use tower_cookies::Cookies;
use tracing::debug;
use utoipa::{IntoParams, ToSchema};
use validator::{Validate, ValidateLength};
use validator_derive::Validate;

pub fn routes() -> Router<SharedState> {
    Router::new().nest("/account", sub_routes())
}

fn sub_routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login))
        .route("/logoff", post(logoff))
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
// TODO find a way to store the password with the type Seceret<String> because it need to impl HasLen
// this is not possible to implemente myself because the type Secret is of an external crate
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginPayload {
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub username: String,
    #[validate(length(min = 1, message = "Can not be empty",))]
    pub pwd: String,
    //pub pwd: SecretStringWrapper,
}

// Move this part of the code in it's own file
#[derive(Debug, Deserialize)]
pub struct SecretStringWrapper(pub Secret<String>);

impl Serialize for SecretStringWrapper {
    fn serialize<S>(&self, serializer: S) -> Resultstd<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.expose_secret().as_bytes())
    }
}

impl Validate for SecretStringWrapper {
    fn validate(&self) -> Resultstd<(), validator::ValidationErrors> {
        Ok(())
    }
}

impl ValidateLength<usize> for SecretStringWrapper {
    fn length(&self) -> Option<usize> {
        Some(self.0.expose_secret().len())
    }
}

#[utoipa::path(
    post,
    context_path = "/account",
    path = "/login",
    tag = "Account",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "Login successfully", body = LoginResponse),
        (status = 403, description = "Login Fail", body = ProblemDetails),
        (status = 500, description = "Internal Server Error", body = ProblemDetails)
    ),
    security(
        ("api_key" = ["aaa","bb"])
    )
)]

async fn login(
    State(state): State<SharedState>,
    cookies: Cookies,
    ValidatedJson(payload): ValidatedJson<LoginPayload>,
) -> Result<Json<LoginResponse>> {
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
            //content: pwd_clear.0.expose_secret().clone(),
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
    context_path = "/account",
    path = "/logoff",
    tag = "Account",
    params(
        LogoffPayload
    ),
    responses(
        (status = 200, description = "Logoff succesful",  body = LoginResponse),
        (status = 500, description = "Internal Server Error")
    )
)]
async fn logoff(
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
