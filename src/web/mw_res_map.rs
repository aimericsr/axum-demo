use crate::web;
use axum::extract::Host;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use http_api_problem::HttpApiProblem;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;
use utoipa::ToSchema;
use web::Error;

/// Map all web:Error to web:ClientError
pub async fn mw_res_map(host: Host, uri: Uri, res: Response) -> impl IntoResponse {
    // Get the eventual response error.
    // TODO: handle if the value is None
    let web_error = res.extensions().get::<Error>();
    let client_status_error = web_error.map(|se| se.client_status_and_error());

    // TODO: Create a custom error response for 405 Method not allowd instaed of empty body

    // If client error, build the new reponse.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            // Retreive the current opentelemetry trace id
            let trace_id = find_current_trace_id().unwrap_or("unknown".to_string());

            // Set the message(enum name) and detail(Display implementation)
            let client_error_message = client_error.as_ref();
            let client_error_detail = client_error.to_string();

            let serveur_err = web_error.expect("Failed to retreive error");
            let server_error_message = serveur_err.as_ref();
            let server_error_detail = serveur_err.to_string();

            // TODO fix deeplinking
            let uri_api_doc = to_open_api_deeplink(&uri.to_string());
            // TODO add env variable to know if we are running over HTTP or HTTPS
            let type_url = format!("http://{}/swagger-ui/#{}", host.0, uri_api_doc);

            if status_code.is_server_error() || status_code.as_u16() == 408 {
                error!(
                    server_error = "true",
                    server_error_message,
                    server_error_detail,
                    client_error_message,
                    client_error_detail,
                );
            } else {
                info!(
                    server_error = "false",
                    server_error_message,
                    server_error_detail,
                    client_error_message,
                    client_error_detail,
                );
            }

            match client_error {
                web::ClientError::JSON_VALDIDATION { errors } => HttpApiProblem::new(status_code)
                    .title(client_error_message)
                    .detail(client_error_detail)
                    .type_url(type_url)
                    .instance(uri.to_string())
                    .value("trace_id", &trace_id)
                    .value("detail_validation", &errors)
                    .to_axum_response(),
                _ => HttpApiProblem::new(status_code)
                    .title(client_error_message)
                    .detail(client_error_detail)
                    .type_url(type_url)
                    .instance(uri.to_string())
                    .value("trace_id", &trace_id)
                    .to_axum_response(),
            }
        });

    error_response.unwrap_or(res)
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HttpApiProblemCustom {
    #[schema(example = "http://localhost:8080/swagger-ui/#/Account/account_login")]
    r#type: String,
    #[schema(example = 404)]
    status: i32,
    #[schema(example = "JSON_VALDIDATION")]
    title: String,
    #[schema(example = "JSON_VALDIDATION_DETAIL")]
    detail: String,
    #[schema(example = "/account/login")]
    instance: String,
    #[schema(example = "afb61afc9b97368003e84351d3eb7586")]
    trace_id: String,
}

fn to_open_api_deeplink(input: &str) -> String {
    let mut transformed_string = String::new();
    let mut slash_count = 0;

    // Capture the firest word without the /
    let re = Regex::new(r"^/(\w+)").unwrap();
    let Some(first_word) = re.captures(input) else {
        return "".to_string();
    };

    for c in input.chars() {
        if c == '/' {
            slash_count += 1;
            if slash_count == 2 {
                transformed_string.push('_');
            } else {
                transformed_string.push(c);
            }
        } else {
            transformed_string.push(c);
        }
    }

    let tag = &first_word[1];
    let tag = &capitalize_first(tag);
    transformed_string.insert_str(0, tag);
    transformed_string.insert(0, '/');
    transformed_string
}

fn capitalize_first(s: &str) -> String {
    s.chars()
        .take(1)
        .flat_map(|f| f.to_uppercase())
        .chain(s.chars().skip(1))
        .collect()
}
