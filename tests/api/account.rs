use crate::helpers::spawn_app;
use axum_demo::web::rest::routes_login::LoginPayload;
use axum_demo::web::rest::routes_login::LogoffPayload;
use axum_demo::web::rest::routes_login::StringWrapper;
use redact::Secret;
use serde_json::to_value;

// Models

#[tokio::test]
async fn login_works() {
    // Arrange
    let app = spawn_app().await;
    let username = app.seed_user().await;
    let body = LoginPayload {
        username: username,
        pwd: StringWrapper(Secret::from("test")),
    };
    let json = to_value(body).expect("Failed to serialize body");

    // Act
    let response = app.post_login(json).await;

    // Assert
    assert_eq!(response.status(), 404, "Status code should be 404");
}

#[tokio::test]
async fn login_fails() {
    // Arrange
    let app = spawn_app().await;
    let body = LoginPayload {
        username: "test".to_string(),
        pwd: StringWrapper(Secret::from("test")),
    };
    let json = to_value(body).expect("Failed to serialize body");

    // Act
    let response = app.post_login(json).await;

    // Assert
    assert_eq!(response.status(), 404, "Status code should be 404");
}

#[tokio::test]
async fn logoff_fails() {
    // Arrange
    let app = spawn_app().await;
    let body = LogoffPayload { logoff: true };
    let json = to_value(body).expect("Failed to serialize body");

    // Act
    let response = app.post_logoff(json).await;

    // Assert
    assert_eq!(response.status(), 404, "Status code should be 404");
}
