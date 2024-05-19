use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForLogin};
use crate::startup::SharedState;
use crate::web::mw_validate_json::ValidatedJson;
use crate::web::{self, remove_token_cookie, Error, Result};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use redact::Secret;
use serde::{Deserialize, Serialize};
use std::result::Result as Resultstd;
use tower_cookies::Cookies;
use tracing::debug;
use utoipa::{IntoParams, ToSchema};
use validator::HasLen;
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
    pub pwd: StringWrapper,
}

#[derive(Debug, Deserialize)]
pub struct StringWrapper(pub Secret<String>);

impl HasLen for &StringWrapper {
    fn length(&self) -> u64 {
        self.0.expose_secret().length()
    }
}

impl Serialize for StringWrapper {
    fn serialize<S>(&self, serializer: S) -> Resultstd<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.expose_secret().serialize(serializer)
    }
}

// #[derive(Debug, Deserialize)]
// struct SecretValidate(Secret<String>);

// impl HasLen for SecretValidate {
//     fn length(&self) -> u64 {
//         self.0.expose_secret().length()
//     }
// }

#[utoipa::path(
    post,
    context_path = "/account",
    path = "/login",
    tag = "Account",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "Login successfully", body = LoginResponse),
        (status = 403, description = "Login Fail", body = HttpApiProblemCustom),
        (status = 500, description = "Internal Server Error", body = HttpApiProblemCustom)
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
            content: pwd_clear.0.expose_secret().clone(),
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
