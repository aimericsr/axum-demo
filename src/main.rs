use axum_demo::config::get_configuration;
use axum_demo::observability::metrics::init_metrics;
use axum_demo::observability::traces::init_traces;
use axum_demo::startup::Application;

fn main() -> std::io::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = get_configuration().expect("Failed to read configuration");

            let meter = init_metrics(&config.tracing);

            init_traces(&config.tracing, &config.env);

            let application = Application::build(config, meter)
                .await
                .expect("Failed to build the app");

            application
                .run_until_stopped()
                .await
                .expect("Failed to lunch the app");
            Ok(())
        })
}
