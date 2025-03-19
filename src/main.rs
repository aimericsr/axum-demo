use axum_demo::config::get_configuration;
use axum_demo::observability::init_observability;
use axum_demo::startup::Application;
use tracing::info;

fn main() -> std::io::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = get_configuration().expect("Failed to read configuration");

            let observability_guard = init_observability(&config);

            let application = Application::build(config, observability_guard.meter.clone())
                .await
                .expect("Failed to build the app");

            application
                .run_until_stopped()
                .await
                .expect("Failed to lunch the app");

                info!("Graceful shutdown started successfully with a timeout of 5 seconds");
                tokio::select! {
                    _  = observability_guard.shutdown() => {
                        info!("Graceful shutdown has been completed successfully");
                    },
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                        info!("Timeout of 5 seconds has been reached without the shutdown to complete, some traces/metrics may have been lost, exiting the appliction");
                    },
                }
            Ok(())
        })
}
