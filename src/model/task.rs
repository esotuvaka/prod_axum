use crate::context::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::base::{self, DbController};

// Types
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct TaskCreate {
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
}
// Types

// Controllers
pub struct TaskController;

impl DbController for TaskController {
    const TABLE: &'static str = "task";
}

impl TaskController {
    pub async fn create(ctx: &Ctx, mm: &ModelManager, task: TaskCreate) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, task).await
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
        base::list::<Self, _>(ctx, mm).await
    }

    pub async fn update(ctx: &Ctx, mm: &ModelManager, id: i64, data: TaskUpdate) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, data).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}
// Controllers

// Tests
#[cfg(test)]
mod tests {
    use crate::_dev_utils;

    use super::*;
    use anyhow::Result;
    // Ensure that tests execute serially, preventing DB state mishaps during init_test() across tests
    use serial_test::serial;
    use tracing::debug;

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
    async fn test_update_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let t_title = "test_update_ok - task 01";
        let t_title_new = "test_updated_ok - task 01 - new";
        let t_task = _dev_utils::seed_tasks(&ctx, &mm, &[t_title])
            .await?
            .remove(0);

        TaskController::update(
            &ctx,
            &mm,
            t_task.id,
            TaskUpdate {
                title: Some(t_title_new.to_string()),
            },
        )
        .await?;

        // Check
        let task = TaskController::get(&ctx, &mm, t_task.id).await?;
        assert_eq!(task.title, t_title_new);

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
