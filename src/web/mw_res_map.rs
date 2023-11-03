use crate::web;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use http_api_problem::*;
use serde_json::to_value;
use tracing::error;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;
use web::Error;

pub async fn mw_res_map(uri: Uri, res: Response) -> impl IntoResponse {
    // Get the eventual response error.
    let web_error = res.extensions().get::<Error>();
    let client_status_error = web_error.map(|se| se.client_status_and_error());

    // If client error, build the new reponse.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let trace_id = find_current_trace_id().unwrap_or("unknown".to_string());
            let client_error = to_value(client_error).expect("Failed to transform to value");

            let message = client_error["message"].as_str().unwrap_or("unknown");
            let detail = client_error["detail"].as_str().unwrap_or("unknown");
            let serveur_err = web_error.expect("Failed to retreive error");

            if status_code.is_server_error() {
                error!(
                    server_error = "true",
                    server_error_message = serveur_err.to_string(),
                    server_error_detail = serveur_err.to_string(),
                    client_error_message = message,
                    client_error_detail = detail,
                );
            }

            HttpApiProblem::new(status_code)
                .title(message)
                .detail(detail)
                .instance(uri.to_string())
                .value("trace_id", &trace_id)
                .to_axum_response()
        });

    error_response.unwrap_or(res)
}
