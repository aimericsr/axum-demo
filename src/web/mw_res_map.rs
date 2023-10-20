use crate::web::rpc::RpcInfo;
use crate::{error, web};
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use http_api_problem::*;
use serde_json::to_value;
use tracing::error;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;
use web::Error;

pub async fn mw_res_map(uri: Uri, res: Response) -> impl IntoResponse {
    let trace_id = find_current_trace_id().unwrap_or("unknown".to_string());
    let _ = res.extensions().get::<RpcInfo>();

    // Get the eventual response error.
    let web_error = res.extensions().get::<Error>();
    let client_status_error = web_error.map(|se| se.client_status_and_error());

    // If client error, build the new reponse.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
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

            // let client_error_body = json!({
            //     "id": rpc_info.as_ref().map(|rpc| rpc.id.clone()),
            //     "error": {
            //         "message": message,
            //         "data" : {
            //             "trace_id" : trace_id,
            //             "details" : detail,
            //         }

            //     }
            // });

            // Build the new response from the client_error_body
            //(*status_code, Json(client_error_body)).into_response()
        });

    error_response.unwrap_or(res)
}
