use std::fs;

use crate::config::DB_CONFIG;
use sqlx::{AssertSqlSafe, Pool, Postgres, postgres::PgPoolOptions};
use tracing::{error, info};

type Db = Pool<Postgres>;

// NOTE: Hardcode to prevent deployed system db update.
const PG_DEV_SUPERUSER: &str = "postgres:welcome";
// sql files
const SQL_RECREATE_DB_FILE_NAME: &str = "00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - inti_dev_db()", "FOR-DEV-ONLY");
    // -- Create the golion_db and golion_user with postgres user
    {
        let root_url = format!(
            "postgres://{PG_DEV_SUPERUSER}@{}:{}/postgres",
            DB_CONFIG.postgres_host, DB_CONFIG.postgres_port
        );
        let root_db = new_db_pool(&root_url).await?;
        pexec(&root_db, &format!("{SQL_DIR}/{SQL_RECREATE_DB_FILE_NAME}")).await?;
    }
    // -- Get sql files and sort the mas they must be executed in order and are number 00 ,01 ..
    let mut paths: Vec<std::path::PathBuf> =
        fs::read_dir(SQL_DIR)?.filter_map(|entry| entry.ok().map(|e| e.path())).collect();
    paths.sort();

    // -- Execute sql files with golion_user and golion_db
    let app_db = new_db_pool(&DB_CONFIG.database_url()).await?;
    for path in paths {
        let path_str = path.to_string_lossy();

        if path_str.ends_with(".sql") && !path_str.ends_with(SQL_RECREATE_DB_FILE_NAME) {
            pexec(&app_db, &path_str).await?;
        }
    }
    Ok(())
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file:?}", "FOR-DEV-ONLY");

    let content = fs::read_to_string(file)?;
    // NOTE: Dev-only SQL from our own files, no user input.
    for sql in content.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        sqlx::query(AssertSqlSafe(sql)).execute(db).await.map_err(|e| {
            error!("pexec error while running {file}");
            error!("cause:\n{e}");
            e
        })?;
    }

    Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_millis(500))
        .connect(db_con_url)
        .await
}
