use crate::error::{Error, Result};
use dotenvy::dotenv;
use secrecy::SecretBox;
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

/// Be sure to be available during the whole execution of the program
/// and to init it only once.
pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
    })
}

pub fn get_configuration() -> Result<Config> {
    Config::load_from_env()
}

/// Struct holding all the variables needed to start the application.
pub struct Config {
    pub application: ApplicationSettings,
    pub env: Env,
    pub postgres: Postgres,
    pub tracing: Tracing,
    pub crypt: Crypt,
}

pub enum Env {
    Dev,
    Staging,
    Prod,
}

impl From<String> for Env {
    fn from(value: String) -> Self {
        match value.to_lowercase().trim() {
            "dev" => Env::Dev,
            "staging" => Env::Staging,
            "prod" => Env::Prod,
            _ => Env::Dev,
        }
    }
}

pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
    pub web_folder: String,
}

pub struct Postgres {
    pub db_user: SecretBox<String>,
    pub db_password: SecretBox<String>,
    pub db_host: SecretBox<String>,
    pub db_name: SecretBox<String>,
    pub db_port: u16,
}

pub struct Crypt {
    pub pwd_key: Vec<u8>,
    pub token_key: Vec<u8>,
    pub token_duration_sec: f64,
}

pub struct Tracing {
    pub otel_enabled: bool,
    pub stdout_enabled: bool,
    pub file_enabled: bool,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        dotenv().expect("Failed to read .env file");
        Ok(Config {
            env: get_env("APP_ENV")?.into(),
            application: ApplicationSettings {
                host: get_env("APP_HOST")?,
                port: get_env_parse("APP_PORT")?,
                web_folder: get_env("APP_WEB_FOLDER")?,
            },
            postgres: Postgres {
                db_user: Box::new(get_env("SERVICE_DB_USER")?).into(),
                db_password: Box::new(get_env("SERVICE_DB_PASSWORD")?).into(),
                db_host: Box::new(get_env("SERVICE_DB_HOST")?).into(),
                db_name: Box::new(get_env("SERVICE_DB_NAME")?).into(),
                db_port: get_env_parse("SERVICE_DB_PORT")?,
            },
            tracing: Tracing {
                stdout_enabled: get_env_parse("STDOUT_ENABLED")?,
                file_enabled: get_env_parse("FILE_ENABLED")?,
                otel_enabled: get_env_parse("OTEL_ENABLED")?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env() {
        unsafe {
            env::set_var("APP_HOST", "localhost");
        }
        let result = get_env("APP_HOST").expect("Failed to get APP_HOST from env");
        assert_eq!(result, "localhost".to_string());
        unsafe {
            env::remove_var("APP_HOST");
        }
    }

    #[test]
    fn test_get_env_parse() {
        unsafe {
            env::set_var("APP_PORT", "8080");
        }
        let result = get_env_parse::<u16>("APP_PORT").expect("Failed to get APP_PORT from env");
        assert_eq!(result, 8080);
        unsafe {
            env::remove_var("APP_PORT");
        }
    }

    #[test]
    fn test_get_env_b64u_as_u8s() {
        unsafe {
            env::set_var("SERVICE_TOKEN_KEY", "SGVsbG8gV29ybGQ");
        }
        let result = get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")
            .expect("Failed to get SERVICE_TOKEN_KEY from env");
        assert_eq!(
            result,
            vec![72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]
        );
        unsafe {
            env::remove_var("SERVICE_TOKEN_KEY");
        }
    }
}
