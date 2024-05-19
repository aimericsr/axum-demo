use crate::helpers::spawn_app;
use serde::Serialize;
use serde_json::to_value;

// Struct
#[derive(Serialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}

#[derive(Serialize)]
struct LogoffPayload {
    logoff: bool,
}

// #[tokio::test]
// async fn login_works() {
//     // Arrange
//     let app = spawn_app().await;
//     let username = app.seed_user().await;
//     let body = LoginPayload {
//         username: username,
//         pwd: String::from("test"),
//     };
//     let json = to_value(body).expect("Failed to serialize body");

//     // Act
//     let response = app.post_login(json).await;

//     // Assert
//     assert_eq!(response.status(), 200, "Status code should be 200");
// }

#[tokio::test]
async fn login_fails() {
    // Arrange
    let app = spawn_app().await;
    let body = LoginPayload {
        username: String::from("test"),
        pwd: String::from("test"),
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
