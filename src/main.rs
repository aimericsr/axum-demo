use axum_demo::config::get_configuration;
use axum_demo::observability::metrics::create_prometheus_recorder;
use axum_demo::observability::tracing::init_subscriber;
use axum_demo::startup::Application;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration.");

    init_subscriber(&config.otel);
    let prom = create_prometheus_recorder();

    let application = Application::build(config, prom)
        .await
        .expect("Failed to build the app");
    application
        .run_until_stopped()
        .await
        .expect("Failed to lunch the app");
    Ok(())
}
