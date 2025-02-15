use reqwest::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CACHE_CONTROL};

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
    let headers = response.headers();
    let rate_limit_range = 0..50;
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    assert_eq!(
        "no-cache",
        headers.get(CACHE_CONTROL).unwrap().to_str().unwrap()
    );
    assert_eq!(
        "http://localhost:3000",
        headers
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert!(rate_limit_range.contains(
        &headers
            .get("x-ratelimit-limit")
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<u8>()
            .unwrap()
    ));
    assert!(rate_limit_range.contains(
        &headers
            .get("x-ratelimit-remaining")
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<u8>()
            .unwrap()
    ));
    // assert_eq!(
    //     55,
    //     headers.get("traceparent").unwrap().to_str().unwrap().len()
    // );
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
