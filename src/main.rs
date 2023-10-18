use axum_demo::config::{config, get_configuration};
use axum_demo::observability::tracing::init_subscriber;
use axum_demo::startup::Application;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration.");

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
