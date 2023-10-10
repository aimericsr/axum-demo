use crate::ctx::Ctx;
use crate::web;
use crate::web::rpc::RpcInfo;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, to_value};
use uuid::Uuid;
use web::Error;

pub async fn mw_res_map(ctx: Option<Ctx>, uri: Uri, req_method: Method, res: Response) -> Response {
    let uuid = Uuid::new_v4();
    let rpc_info = res.extensions().get::<RpcInfo>();

    // -- Get the eventual response error.
    let web_error = res.extensions().get::<Error>();
    let client_status_error = web_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new reponse.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error = to_value(client_error).ok();
            let message = client_error.as_ref().and_then(|v| v.get("message"));
            let detail = client_error.as_ref().and_then(|v| v.get("detail"));

            let client_error_body = json!({
                "id": rpc_info.as_ref().map(|rpc| rpc.id.clone()),
                "error": {
                    "message": message,
                    "data" : {
                        "req_uuid" : uuid.to_string(),
                        "details" : detail,
                    }

                }
            });

            // Build the new response from the client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    error_response.unwrap_or(res)
}
