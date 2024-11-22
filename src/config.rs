use std::{env::current_dir, time::Duration};

use anyhow::anyhow;
use config::{Config, File};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

pub fn get_config() -> Settings {
    let config_dir = current_dir()
        .expect("Couldn't get current directory")
        .join("config");

    let app_env: AppEnv = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "dev".into())
        .as_str()
        .try_into()
        .expect("The APP_ENV environment variable should be a valid running mode");

    let config = Config::builder()
        .add_source(File::from(config_dir.join("base.yaml")))
        .add_source(File::from(config_dir.join(app_env.config_file())))
        .build()
        .expect("Source collection of configuration data should pass");

    config
        .try_deserialize()
        .expect("Configuration data should be deserializable into Settings")
}

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub application: Application,
    pub postgres: Postgres,
}

#[derive(Debug, serde::Deserialize)]
pub struct Application {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Postgres {
    pub user: String,
    pub password: SecretString,
    pub db_name: String,
    pub host: String,
    pub port: u16,
    pub acquire_timeout: u64,
    pub max_connections: u32,
}

impl Postgres {
    pub async fn get_pool(&self) -> Result<PgPool, sqlx::Error> {
        let opts = PgConnectOptions::new()
            .host(&self.host)
            .password(self.password.expose_secret())
            .port(self.port)
            .database(&self.db_name)
            .username(&self.user);

        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .acquire_timeout(Duration::from_secs(self.acquire_timeout))
            .connect_with(opts)
            .await
    }
}

#[derive(Debug)]
pub enum AppEnv {
    Dev,
    Prod,
}

impl AppEnv {
    pub fn config_file(&self) -> &'static str {
        match self {
            Self::Dev => "dev.yaml",
            Self::Prod => "prod.yaml",
        }
    }
}

impl TryFrom<&str> for AppEnv {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "dev" => Ok(Self::Dev),
            "prod" => Ok(Self::Prod),
            other => Err(anyhow!("{} is not a valid APP_ENV, use dev/prod", other)),
        }
    }
}
