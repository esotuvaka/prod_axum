use crate::context::Ctx;
use crate::model::ModelManager;
use crate::model::Result;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Types
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

#[derive(Deserialize)]
pub struct TaskCreate {
    pub title: String,
}

#[derive(Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
}
// Types

// Controllers
pub struct TaskController;

impl TaskController {
    pub async fn create(_ctx: &Ctx, mm: &ModelManager, task: TaskCreate) -> Result<i64> {
        let db = mm.db();
        let (id,) =
            sqlx::query_as::<_, (i64,)>("INSERT INTO task (title) values ($1) returning id")
                .bind(task.title)
                .fetch_one(db)
                .await?;
        Ok(id)
    }
}
// Controllers

// Tests
#[cfg(test)]
mod tests {
    use crate::_dev_utils;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        Ok(())
    }
}
// Tests