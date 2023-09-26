use axum_demo::startup::build;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    build().await.expect("Failed to build the app");
    Ok(())
}
