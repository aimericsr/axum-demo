use axum::Router;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(description = "Simple API to demonstrate axum framework capabilites"),
    paths(
        // Hello
        crate::web::rest::routes_hello::hello,
        crate::web::rest::routes_hello::hello_name,

        // Login
        crate::web::rest::routes_login::login,
        crate::web::rest::routes_login::logoff,

        // Health Check
        crate::web::rest::routes_health::health,
        crate::web::rest::routes_health::health_ready,
        crate::web::rest::routes_health::health_live,
    ),
    components(
        schemas(
            // Error 
            crate::web::ClientError,
            crate::web::mw_res_map::HttpApiProblemCustom,

            // Login
            crate::web::rest::routes_login::LoginResponse,
            crate::web::rest::routes_login::LoginResponseResult,
            crate::web::rest::routes_login::LogoffPayload,
            crate::web::rest::routes_login::LoginPayload)
    ),
    tags(
        (name = "Account", description = "All related user endpoints"),
        (name = "Health", description = "Retreive the current status of the service"),
        (name = "Hello", description = "Basic routes for testing"),
    )
)]
struct ApiDoc;

pub fn routes() -> Router {
    Router::new()
    // .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
    // .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
    // .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
}
