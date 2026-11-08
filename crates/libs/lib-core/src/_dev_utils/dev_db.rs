use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::config::DB_CONFIG;
use sqlx::{
    AssertSqlSafe, Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use tracing::{error, info};

type Db = Pool<Postgres>;
// NOTE: Hardcode to prevent deployed system db update.
const PG_DEV_SUPERUSER_USERNAME: &str = "postgres";
const PG_DEV_SUPERUSER_PASSWORD: &str = "welcome";
const PG_DEV_SUPERUSER_DB: &str = "postgres";
// sql files
const SQL_RECREATE_DB_FILE_NAME: &str = "00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - inti_dev_db()", "FOR-DEV-ONLY");
    // --Get the sql dir
    // Note: this is because cargo test and cargo  run wound't gitve the
    // same current_dir.
    let current_dir = std::env::current_dir().unwrap();
    let v: Vec<_> = current_dir.components().collect();
    let path_comp = v.get(v.len().wrapping_sub(3));
    let base_dir = if Some(true) == path_comp.map(|c| c.as_os_str() == "crates") {
        v[..v.len() - 3].iter().collect::<PathBuf>()
    } else {
        current_dir.clone()
    };
    println!("{base_dir:?}");
    let sql_dir = base_dir.join(SQL_DIR);
    // -- Create the golion_db and golion_user with postgres user
    {
        let root_db = new_db_pool(
            DB_CONFIG
                .connect_options()
                .username(PG_DEV_SUPERUSER_USERNAME)
                .password(PG_DEV_SUPERUSER_PASSWORD)
                .database(PG_DEV_SUPERUSER_DB),
        )
        .await?;
        pexec(&root_db, &sql_dir.join(SQL_RECREATE_DB_FILE_NAME)).await?;
    }
    // -- Get sql files and sort the mas they must be executed in order and are number 00 ,01 ..
    let mut paths: Vec<std::path::PathBuf> =
        fs::read_dir(sql_dir)?.filter_map(|entry| entry.ok().map(|e| e.path())).collect();
    paths.sort();

    // -- Execute sql files with golion_user and golion_db
    let app_db = new_db_pool(DB_CONFIG.connect_options()).await?;
    for path in paths {
        let path_str = path.to_string_lossy();

        if path_str.ends_with(".sql") && !path_str.ends_with(SQL_RECREATE_DB_FILE_NAME) {
            pexec(&app_db, &path).await?;
        }
    }
    Ok(())
}

async fn pexec(db: &Db, file: &Path) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file:?}", "FOR-DEV-ONLY");

    let content = fs::read_to_string(file)?;
    // NOTE: Dev-only SQL from our own files, no user input.
    for sql in content.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        sqlx::query(AssertSqlSafe(sql)).execute(db).await.map_err(|e| {
            error!("pexec error while running {sql}");
            error!("cause:\n{e}");
            e
        })?;
    }

    Ok(())
}

async fn new_db_pool(options: PgConnectOptions) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_millis(500))
        .connect_with(options)
        .await
}
