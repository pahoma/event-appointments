use serde::{Deserialize};
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use crate::domain::{Email};
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
    pub qr_client: QRClientSettings,
    pub console_cli: ConsoleClientSettings
}

#[derive(serde::Deserialize)]
pub struct ConsoleClientSettings {
    pub web_url: String,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub timeout_milliseconds: u64,
}

#[derive(serde::Deserialize, Clone)]
pub struct QRClientSettings {
    pub api_url: String,
    pub api_key: Secret<String>,
    pub base_url: String,
    pub base_image_path: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub timeout_milliseconds: u64,
}

impl QRClientSettings {
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<Email, String> {
        Email::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

impl DatabaseSettings {
    pub fn connection_options_base(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn connection_options_with_db(&self) -> PgConnectOptions {
        self.connection_options_base().database(&self.database_name)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let conf_directory = base_path.join("../configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into()) .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(
            conf_directory.join("default.yaml"),
        ))
        .add_source(config::File::from(
            conf_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

/// The possible runtime environment for our application.
pub enum Environment {
    Development,
    Production,
    Local,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
            Environment::Local => "local",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            "local" => Ok(Self::Local),
            other => Err(format!(
                "{} is not a supported environment. Use either `development` or `production`.",
                other
            )),
        }
    }
}