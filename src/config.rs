use crate::error::{Error, Result};
use dotenv::dotenv;
use secrecy::Secret;
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

// be sure to be available during the whole execution of the program
// be sure the init it only do once
pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
    })
}

pub struct Config {
    pub application: ApplicationSettings,
    pub postgres: Postgres,
    pub otel: Otel,
    pub crypt: Crypt,
}

pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
    pub web_folder: String,
}

pub struct Postgres {
    pub db_user: Secret<String>,
    pub db_password: Secret<String>,
    pub db_host: Secret<String>,
    pub db_name: Secret<String>,
    pub db_port: i64,
}

pub struct Crypt {
    pub pwd_key: Vec<u8>,
    pub token_key: Vec<u8>,
    pub token_duration_sec: f64,
}

pub struct Otel {
    pub endpoint: String,
    pub service_name: String,
    pub service_version: String,
    pub service_namespace: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        dotenv().expect("Failed to read .env file");
        Ok(Config {
            application: ApplicationSettings {
                host: get_env("APP_HOST")?,
                port: get_env_parse("APP_PORT")?,
                web_folder: get_env("APP_WEB_FOLDER")?,
            },
            postgres: Postgres {
                db_user: get_env("SERVICE_DB_USER")?.into(),
                db_password: get_env("SERVICE_DB_PASSWORD")?.into(),
                db_host: get_env("SERVICE_DB_HOST")?.into(),
                db_name: get_env("SERVICE_DB_NAME")?.into(),
                db_port: get_env_parse("SERVICE_DB_PORT")?,
            },
            otel: Otel {
                endpoint: get_env("OTEL_EXPORTER_OTLP_ENDPOINT")?,
                service_name: get_env("OTEL_SERVICE_NAME")?,
                service_version: get_env("OTEL_SERVICE_VERSION")?,
                service_namespace: get_env("OTEL_SERVICE_NAMESPACE")?,
            },
            crypt: Crypt {
                pwd_key: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
                token_key: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
                token_duration_sec: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
            },
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    base64_url::decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}
