use crate::web;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use strum_macros::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Display, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
	LoginFail,

	CtxExt(web::mw_auth::CtxExtError),
}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		println!("->> {:<12} - model::Error {self:?}", "INTO_RES");

		// Create a placeholder Axum reponse.
		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

		// Insert the Error into the reponse.
		response.extensions_mut().insert(self);

		response
	}
}

impl std::error::Error for Error {}

/// From the root error to the http status code and ClientError
impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		use web::Error::*;

		#[allow(unreachable_patterns)]
		match self {
			// -- Login/Auth
			CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

			// -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
		}
	}
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
	SERVICE_ERROR,
}
