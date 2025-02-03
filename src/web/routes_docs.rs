use axum::Router;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

pub const ACCOUNT_TAG: &str = "Account";
pub const HEALTH_TAG: &str = "Health";
pub const HELLO_TAG: &str = "Hello";

//#[openapi(paths(test_collect_schemas))]
#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    info(description = "Simple API to demonstrate axum framework capabilites"),
    tags(
        (name = ACCOUNT_TAG, description = "All related user endpoints"),
        (name = HEALTH_TAG, description = "Retreive the current status of the service"),
        (name = HELLO_TAG, description = "Basic routes for testing"),
    ),
    security(
        (),
        ("my_auth" = ["read:items", "edit:items"]),
        ("token_jwt" = [])
    ),
)]
struct ApiDoc;

/// Serving multiples format of API documentation :
/// - /swagger-ui
/// - /rapidoc
/// - /redoc
/// - /scalar
pub fn routes() -> Router {
    let api_doc = ApiDoc::openapi();

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_doc.clone()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .merge(Redoc::with_url("/redoc", api_doc.clone()))
        .merge(Scalar::with_url("/scalar", api_doc))
}
