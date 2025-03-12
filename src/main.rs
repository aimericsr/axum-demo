use axum_demo::config::get_configuration;
use axum_demo::observability::init_observability;
use axum_demo::startup::Application;

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
                .run_until_stopped(observability_guard)
                .await
                .expect("Failed to lunch the app");
            Ok(())
        })
}
