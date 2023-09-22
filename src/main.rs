use axum_demo::startup::build;
use tracing_log::LogTracer;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    build().await.expect("Failed to build the app");
    Ok(())
}
