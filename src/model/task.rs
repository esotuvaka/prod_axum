use crate::context::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
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

    pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        let db = mm.db();
        let task: Task = sqlx::query_as("SELECT * FROM task WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::EntityNotFound { entity: "task", id })?;
        Ok(task)
    }

    pub async fn list(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
        let db = mm.db();
        let tasks: Vec<Task> = sqlx::query_as("SELECT * FROM task ORDER BY id")
            .fetch_all(db)
            .await?;
        Ok(tasks)
    }

    pub async fn delete(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        let db = mm.db();
        let count = sqlx::query("DELETE FROM task where id = $1")
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();
        if count == 0 {
            return Err(Error::EntityNotFound { entity: "task", id });
        }
        Ok(())
    }
}
// Controllers

// Tests
#[cfg(test)]
mod tests {
    use crate::_dev_utils;

    use super::*;
    use anyhow::Result;
    use serial_test::serial;
    use tracing::debug; // Ensure that tests execute serially, preventing DB state mishaps during init_test() across tests

    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let t_title = "test_create_ok title";

        // Create
        let task = TaskCreate {
            title: t_title.to_string(),
        };
        let id = TaskController::create(&ctx, &mm, task).await?;

        // Check
        let task = TaskController::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, t_title);

        // Delete
        TaskController::delete(&ctx, &mm, id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let t_titles = &["test_list_ok-task 01", "test_list_ok-task 02"];
        _dev_utils::seed_tasks(&ctx, &mm, t_titles).await?;

        let tasks = TaskController::list(&ctx, &mm).await?;

        // Check
        let tasks: Vec<Task> = tasks
            .into_iter()
            .filter(|t| t.title.starts_with("test_list_ok-task")) // Workaround until filtering on list operation is implemented
            .collect();
        assert_eq!(tasks.len(), 2, "number of seeded tasks");

        // Clean
        for task in tasks.iter() {
            TaskController::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_get_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let t_id = 100;

        let res = TaskController::get(&ctx, &mm, t_id).await;

        // Check
        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100
                })
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_delete_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let t_id = 100;

        let res = TaskController::delete(&ctx, &mm, t_id).await;

        // Check
        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100
                })
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }
}
// Tests
