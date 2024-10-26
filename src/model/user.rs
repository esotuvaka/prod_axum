use crate::context::Ctx;
use crate::model::base::{self, DbController};
use crate::model::ModelManager;
use crate::model::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use tracing::debug;
use uuid::Uuid;

// Types
#[derive(Clone, FromRow, Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize)]
pub struct UserCreate {
    pub username: String,
    pub pwd_clear: String,
}

struct UserInsert {
    username: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct UserLogin {
    pub id: i64,
    pub username: String,

    pub pwd: Option<String>, // ENCRYPTED, #_scheme_id_#......
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Debug)]
pub struct UserAuth {
    pub id: i64,
    pub username: String,

    pub token_salt: Uuid,
}

// Group the 3 structs via a shared trait
pub trait UserBy: for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserLogin {}
impl UserBy for UserAuth {}
// Types

pub struct UserController;

impl DbController for UserController {
    const TABLE: &'static str = "user";
}

impl UserController {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy, // Ensure that only our User structs can be returned
    {
        base::get::<Self, E>(ctx, mm, id).await
    }

    pub async fn first_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        let db = mm.db();
        let sql = format!("SELECT * FROM {} WHERE username = $1", Self::TABLE);
        debug!("{sql}");
        let entity = sqlx::query_as(&sql)
            .bind(username)
            .fetch_optional(db)
            .await?;

        Ok(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils;
    use anyhow::{Context, Result};
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_first_ok_demo1() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let t_username = "demo1";

        let user: User = UserController::first_by_username(&ctx, &mm, t_username)
            .await?
            .context("Should have user 'demo1'")?;

        // Check
        assert_eq!(user.username, t_username);

        Ok(())
    }
}
