use crate::crypt::token::{validate_web_token, Token};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForAuth};
use crate::model::ModelManager;
use crate::web::AUTH_TOKEN;
use crate::web::{Error, Result};
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;
use thiserror::Error;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

use super::set_token_cookie;

#[allow(dead_code)]
pub async fn mw_ctx_require<B>(
    ctx: Result<Ctx>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

// Save info in the request extensions
pub async fn mw_ctx_resolve(
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    let ctx_ext_result = _ctx_resolve(mm, &cookies).await;

    // Remove the cookie if something went wrong other than NoAuthTokenCookie.
    if ctx_ext_result.is_err() && !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }

    // Store the ctx_result in the request extension.
    req.extensions_mut().insert(ctx_ext_result);

    Ok(next.run(req).await)
}

async fn _ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
    // -- Get Token String
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;

    // -- Parse Token
    let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

    // -- Get UserForAuth
    let user: UserForAuth = UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
        .await
        .map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    // -- Validate Token
    validate_web_token(&token, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::FailValidate)?;

    // -- Update Token
    set_token_cookie(cookies, &user.username, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::CannotSetTokenCookie)?;

    // -- Create CtxExtResult
    Ctx::new(user.id).map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// Retreive info from the request extensions
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!("{:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt)
    }
}

type CtxExtResult = core::result::Result<Ctx, CtxExtError>;

#[derive(Clone, Error, Serialize, Debug)]
pub enum CtxExtError {
    #[error("Token not found in cookie")]
    TokenNotInCookie,
    #[error("Context not found in request extension")]
    CtxNotInRequestExt,
    #[error("Failed to create context: {0}")]
    CtxCreateFail(String),
    #[error("Failed to set token cookie")]
    CannotSetTokenCookie,
    #[error("Validation failed")]
    FailValidate,
    #[error("User not found")]
    UserNotFound,
    #[error("Token has the wrong format")]
    TokenWrongFormat,
    #[error("Model access error: {0}")]
    ModelAccessError(String),
}
