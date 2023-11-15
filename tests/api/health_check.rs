use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_general_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// #[tokio::test]
// async fn health_check_live_works() {
//     // Arrange
//     let app = spawn_app().await;
//     let client = reqwest::Client::new();

//     // Act
//     let response = client
//         .get(&format!("{}/health/live", &app.address))
//         .send()
//         .await
//         .expect("Failed to execute request.");

//     // Assert
//     assert!(response.status().is_success());
//     assert_eq!(Some(16), response.content_length());
// }

#[tokio::test]
async fn health_check_ready_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health/ready", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(16), response.content_length());
}
