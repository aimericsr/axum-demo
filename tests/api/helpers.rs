use axum_demo::{
    config::{get_configuration, Otel},
    observability::tracing::init_subscriber,
    startup::Application,
};
use std::sync::OnceLock;
use uuid::Uuid;
pub fn tracing(otel: &Otel) -> &'static () {
    static INSTANCE: OnceLock<()> = OnceLock::new();

    INSTANCE.get_or_init(|| init_subscriber(otel))
}

pub struct TestApp {
    pub address: String,
}

impl TestApp {
    pub async fn post_login(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/login", &self.address))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logoff(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/logoff", &self.address))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        //c.postgres.db_name = Uuid::new_v4().to_string().into();
        c.application.port = 0;
        c.otel.enabled = true;
        c.otel.stdout_enabled = true;
        c
    };

    tracing(&configuration.otel);

    // Create and migrate the database
    //configure_database(&configuration.database).await;
    // Launch the application as a background task

    let application = Application::build(configuration)
        .await
        .expect("Failed to build the app");
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());
    TestApp { address }
}

// async fn configure_database(config: &DatabaseSettings) -> PgPool {
//     // Create database
//     let mut connection = PgConnection::connect_with(&config.without_db())
//         .await
//         .expect("Failed to connect to Postgres");
//     connection
//         .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
//         .await
//         .expect("Failed to create database.");
//     // Migrate database
//     let connection_pool = PgPool::connect_with(config.with_db())
//         .await
//         .expect("Failed to connect to Postgres.");
//     sqlx::migrate!("./migrations")
//         .run(&connection_pool)
//         .await
//         .expect("Failed to migrate the database");
//     connection_pool
// }
