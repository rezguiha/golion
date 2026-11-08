use config::{Config, Environment};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use std::sync::LazyLock;
use tracing::error;

pub(crate) static DB_CONFIG: LazyLock<DBConfig> = LazyLock::new(|| {
    let env_source = Environment::default();
    let config = Config::builder()
        .set_default("postgres_host", "localhost")
        .and_then(|builder| builder.set_default("postgres_port", 5432))
        .and_then(|builder| builder.set_default("max_connections", 5))
        .and_then(|builder| builder.add_source(env_source).build());
    if let Err(e) = config {
        error!("{:<12} - {:?}\n", "CONFIGURATION", e);
        panic!("{e}");
    };
    let config = config.unwrap().try_deserialize();
    if let Err(e) = config {
        error!("{:<12} - {:?}\n", "CONFIGURATION", e);
        panic!("{e}");
    };
    config.unwrap()
});
// No `Debug`: it would print the password.
#[derive(Deserialize)]
pub struct DBConfig {
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub max_connections: u32,
}

impl DBConfig {
    pub fn connect_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.postgres_host)
            .port(self.postgres_port)
            .username(&self.postgres_user)
            .password(&self.postgres_password)
            .database(&self.postgres_db)
    }
}
