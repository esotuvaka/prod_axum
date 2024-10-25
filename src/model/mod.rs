//! Model Layer
//!
//! Design:
//!
//! - The model layer normalizes the application's data type
//!   structures and access.
//! - All application code data access must go through the Model layer
//! - The `ModelManager` holds the internal states/resources needed by
//!   ModelControllers to access Data
//! - ModelControllers implement CRUD operations on entities (Task, User, etc.)
//! - A ModelManager is typically used as the app's state, e.g: 1 Manager, many Controllers
//! - ModelManager is passed as an argument to all Model Controllers

mod error;
mod store;
pub mod task;

use store::{new_db_pool, Db};

pub use self::error::{Error, Result};

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    /// Constructor
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager { db })
    }

    /// Restricted to only be available to files under the 'model/' module.
    /// Returns a sqlx db pool reference that can only be used in the model layer
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
