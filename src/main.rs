use axum_demo::config::get_configuration;
use axum_demo::observability::metrics::{init_metrics, init_tokio_metrics};
use axum_demo::observability::traces::init_traces;
use axum_demo::startup::Application;

fn main() -> std::io::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = get_configuration().expect("Failed to read configuration");

            let meter = init_metrics(&config.otel);
            init_tokio_metrics(&meter).await;

            init_traces(&config.otel);

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

// async fn my_async_function() {
//     let span = tracing::info_span!("my_async_function");

//     // WARNING: This span will remain entered until this
//     // guard is dropped...
//     let _enter = span.enter();
//     // ...but the `await` keyword may yield, causing the
//     // runtime to switch to another task, while remaining in
//     // this span!
//     test_func().await

//     // ...
// }

// async fn test_func() {}
