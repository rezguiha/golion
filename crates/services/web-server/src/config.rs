use config::{Config, Environment};
use serde::Deserialize;
use std::sync::LazyLock;
use tracing::error;

pub(crate) static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    let env_source = Environment::default();
    let config = Config::builder()
        .set_default("postgres_host", "localhost")
        .and_then(|builder| builder.set_default("postgres_port", 5432))
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
pub struct AppConfig {
    pub server_port: u16,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
}

impl AppConfig {
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.postgres_user,
            self.postgres_password,
            self.postgres_host,
            self.postgres_port,
            self.postgres_db
        )
    }
}
