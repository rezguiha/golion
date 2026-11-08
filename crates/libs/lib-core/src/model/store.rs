use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::config::DB_CONFIG;

#[derive(Debug)]
pub enum Error {
    FailToCreatePool(String),
}

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> crate::Result<Db> {
    PgPoolOptions::new()
        .max_connections(DB_CONFIG.max_connections)
        .connect_with(DB_CONFIG.connect_options())
        .await
        .map_err(|err| Error::FailToCreatePool(err.to_string()).into())
}
