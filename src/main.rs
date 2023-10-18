use axum_demo::config::config;
use axum_demo::observability::tracing::init_subscriber;
use axum_demo::startup::Application;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = config();

    init_subscriber();

    let application = Application::build(config)
        .await
        .expect("Failed to build the app");
    application
        .run_until_stopped()
        .await
        .expect("Failed to lunch the app");

    Ok(())
}
