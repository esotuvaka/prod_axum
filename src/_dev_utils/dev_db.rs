use std::{fs, path::PathBuf, time::Duration};

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::info;

use crate::{
    context::Ctx,
    model::{
        user::{User, UserController},
        ModelManager,
    },
};

type Db = Pool<Postgres>;

// NOTE: Hardcode these vars to prevent deployed system db updates
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@localhost/app_db";

// SQL files
const SQL_RECREATE_DB: &str = "sql/dev_initial/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

const DEMO_PWD: &str = "welcome";

pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");
    {
        // Create app_db + app_user with the postgres user; Drop root_db at end of scope to prevent misuse
        let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
        pg_exec(&root_db, SQL_RECREATE_DB).await?;
    }

    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort(); // Sort SQL files by order prefix [00-recreate-db.sql, 01-create-schema.sql, ...]

    let app_db = new_db_pool(PG_DEV_APP_URL).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            let path = path.replace('\\', "/"); // For windows compatability
            if path.ends_with(".sql") && path != SQL_RECREATE_DB {
                pg_exec(&app_db, &path).await?;
            }
        }
    }

    let mm = ModelManager::new().await?;
    let ctx = Ctx::root_ctx();

    let demo1_user: User = UserController::first_by_username(&ctx, &mm, "demo1")
        .await?
        .unwrap();
    UserController::update_pwd(&ctx, &mm, demo1_user.id, DEMO_PWD).await?;
    info!("{:<12} - init_dev_db = set demo1 pwd", "FOR-DEV-ONLY");

    Ok(())
}

async fn pg_exec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    info!("{:<12} - pg_exec: {file}", "FOR-DEV-ONLY");
    let content = fs::read_to_string(file)?;
    let sqls: Vec<&str> = content.split(';').collect();
    for sql in sqls {
        sqlx::query(sql).execute(db).await?;
    }
    Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_con_url)
        .await
}
