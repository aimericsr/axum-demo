use axum_demo::config::config;
use axum_demo::observability::tracing::init_subscriber;
use axum_demo::startup::Application;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Setup environnement variables
    let config = config();

    // Setup tracing
    init_subscriber();

    // Lunch the application
    let _ = Application::new(config);
    Application::run_until_stopped(config).await;

    Ok(())
}
