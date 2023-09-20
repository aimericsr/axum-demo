use axum::Router;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(description = "My Api description"),
    paths(
        // Hello
        crate::web::routes_hello::handler_hello,
        crate::web::routes_hello::handler_hello_2,

        // Login
        crate::web::routes_login::api_login,
        crate::web::routes_login::api_logoff_handler,
    ),
    components(
        schemas(crate::web::ClientError, 
            crate::web::rest::routes_login::LoginResponse, 
            crate::web::rest::routes_login::LoginResponseResult, 
            crate::web::rest::routes_login::LogoffPayload, 
            crate::web::rest::routes_login::LoginPayload)
    ),
    tags(
        (name = "test", description = "Test")
    )
)]
struct ApiDoc;

pub fn routes() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
}
