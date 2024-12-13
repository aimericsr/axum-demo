use axum::Router;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
//#[openapi(paths(test_collect_schemas))]
#[openapi(
    info(description = "Simple API to demonstrate axum framework capabilites"),
    paths(
        // Account
        crate::web::rest::routes_login::login,
        crate::web::rest::routes_login::logoff,

        // Health
        crate::web::rest::routes_health::health,
        crate::web::rest::routes_health::health_ready,
        crate::web::rest::routes_health::health_live,

        // Hello
        crate::web::rest::routes_hello::hello,
        crate::web::rest::routes_hello::hello_name,
    ),
    tags(
        (name = "Account", description = "All related user endpoints"),
        (name = "Health", description = "Retreive the current status of the service"),
        (name = "Hello", description = "Basic routes for testing"),
    ),
    security(
        (),
        ("my_auth" = ["read:items", "edit:items"]),
        ("token_jwt" = [])
    ),
)]
struct ApiDoc;

pub fn routes() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
}
