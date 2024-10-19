use axum_demo::config::Postgres as PostgresConfig;
use axum_demo::{
    config::{get_configuration, Otel},
    observability::tracing::init_subscriber,
    startup::Application,
};
use secrecy::{ExposeSecret, SecretBox};
use sqlx::{postgres::PgConnectOptions, Connection, Executor, PgConnection, PgPool};
use std::sync::OnceLock;
use uuid::Uuid;
pub fn tracing(otel: &Otel) -> &'static () {
    static INSTANCE: OnceLock<()> = OnceLock::new();

    INSTANCE.get_or_init(|| init_subscriber(otel))
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    #[warn(dead_code)]
    pub async fn seed_user(&self) -> String {
        let username = String::from("demo2");
        let _ = String::from("demo2");

        self.db_pool
            .execute(format!(r#"INSERT INTO "user" (username) VALUES ('{}');"#, username).as_str())
            .await
            .expect("Failed to create database.");

        // note that bound parameters are added to the query macro
        // let user = sqlx::query_as!(
        //     UserForLogin,
        //     "SELECT * FROM user WHERE username = ?",
        //     username
        // )
        // .execute(&self.db_pool)
        // .await;

        // let pwd = encrypt_pwd(&EncryptContent {
        //     content: password_clear,
        //     salt: user.pwd_salt.to_string(),
        // })?;

        username
    }

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
        c.postgres.db_name = Box::new(Uuid::new_v4().to_string()).into();
        c.application.port = 0;
        c.otel.otel_enabled = true;
        c.otel.stdout_enabled = false;
        c
    };

    tracing(&configuration.otel);

    //Create and migrate the database
    let db_pool = configure_database(&configuration.postgres).await;
    // Launch the application as a background task

    let application = Application::build(configuration)
        .await
        .expect("Failed to build the app");
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());
    TestApp { address, db_pool }
}

async fn configure_database(config: &PostgresConfig) -> PgPool {
    let mut connection_info = PgConnectOptions::new()
        .host(config.db_host.expose_secret())
        .username(config.db_user.expose_secret())
        .password(config.db_password.expose_secret())
        .port(config.db_port);

    // Create database
    let mut connection = PgConnection::connect_with(&connection_info)
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name.expose_secret()).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    connection_info = connection_info.database(config.db_name.expose_secret());
    let connection_pool = PgPool::connect_with(connection_info)
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
