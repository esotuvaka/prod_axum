use std::collections::HashMap;

use serde::{ser::Error as SerdeError, Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgRow, query::Query, FromRow, Postgres};
use tracing::debug;

use crate::{context::Ctx, model};

use super::{Error, ModelManager, Result};

/// Initialize a `Controller` to manage all CRUD operations for a given `TABLE`
///
/// The `TABLE`'s entities are `INSERT`ed as `<D>` where `<D>` must `impl serde::Serialize` trait
///
/// The `TABLE`'s entities are `SELECT`ed as `<E>` where `<E>` must `impl sqlx::FromRow` trait, and the `Unpin`
/// and `Send` traits (for async use)
pub trait DbController {
    const TABLE: &'static str;
}

fn bind_value_to_query<'a>(
    mut query: Query<'a, Postgres, sqlx::postgres::PgArguments>,
    value: &'a Value,
) -> Result<Query<'a, Postgres, sqlx::postgres::PgArguments>> {
    match value {
        Value::String(s) => query = query.bind(s.as_str()), // Bind as &str
        Value::Number(n) => {
            if let Some(num) = n.as_i64() {
                query = query.bind(num); // Bind as i64
            } else if let Some(num) = n.as_f64() {
                query = query.bind(num); // Bind as f64
            } else {
                return Err(Error::Serde(serde_json::Error::custom(
                    "Unsupported number type",
                )));
            }
        }
        Value::Bool(b) => query = query.bind(*b), // Bind as bool
        Value::Array(arr) => {
            // Handle arrays - convert to a JSON string or another format as needed
            let array_str = serde_json::to_string(arr)?;
            query = query.bind(array_str);
        }
        Value::Object(obj) => {
            // Handle objects if needed (may be specific to your use case)
            let obj_str = serde_json::to_string(obj)?;
            query = query.bind(obj_str);
        }
        Value::Null => query = query.bind::<Option<&str>>(None), // Bind null as SQL NULL
    }
    Ok(query)
}

/// `C` must implement the DbController trait.
///
/// `D` must implement the serde::Serialize trait; It is the generic struct that we parameterize and INSERT into the database
///
/// Returns the `Id` of the newly created item
pub async fn create<C, D>(_ctx: &Ctx, mm: &ModelManager, data: D) -> Result<i64>
where
    C: DbController,
    D: Serialize,
{
    let data_json: HashMap<String, Value> = match serde_json::to_value(data)? {
        serde_json::Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        _ => {
            return Err(model::error::Error::Serde(serde_json::Error::custom(
                "expected json object",
            )))
        }
    };

    let fields: Vec<String> = data_json.keys().cloned().collect();
    let placeholders: Vec<String> = (1..=data_json.len()).map(|i| format!("${}", i)).collect();

    let db = mm.db();
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
        C::TABLE,
        fields.join(", "),
        placeholders.join(", ")
    );

    debug!("{sql}");

    // Build the query
    let mut query = sqlx::query_as::<_, (i64,)>(&sql);

    // Bind each value to the query with type extraction
    // TODO: Abstract to json value mapping function to provide consistency between Create / Update
    for value in data_json.values() {
        match value {
            Value::String(s) => query = query.bind(s.as_str()), // Bind as &str
            Value::Number(n) => {
                if let Some(num) = n.as_i64() {
                    query = query.bind(num); // Bind as i64
                } else if let Some(num) = n.as_f64() {
                    query = query.bind(num); // Bind as f64
                } else {
                    return Err(model::error::Error::Serde(serde_json::Error::custom(
                        "Unsupported number type",
                    )));
                }
            }
            Value::Bool(b) => query = query.bind(*b), // Bind as bool
            Value::Array(arr) => {
                // Handle arrays - convert to a JSON string or another format as needed
                let array_str = serde_json::to_string(arr)?;
                query = query.bind(array_str);
            }
            Value::Object(obj) => {
                // Handle objects if needed (may be specific to your use case)
                let obj_str = serde_json::to_string(obj)?;
                query = query.bind(obj_str);
            }
            Value::Null => query = query.bind::<Option<&str>>(None), // Bind null as SQL NULL
        }
    }

    // Execute the query and return the id
    let (id,) = query.fetch_one(db).await?;
    Ok(id)
}

/// `C` must implement the DbController trait.
///
/// `E` must implement the sqlx::FromRow trait, and the Unpin and Send traits (for async use); It is the Entity we are querying for
///
/// `'r` lifetime guarantees the result of the query will live long enough to be returned by the function
///
/// Returns `E`, the entity the query looks for
pub async fn get<C, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    C: DbController,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = mm.db();
    let sql = format!("SELECT * FROM {} WHERE id = $1", C::TABLE);
    debug!("{sql}");
    let entity: E = sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: C::TABLE,
            id,
        })?;
    Ok(entity)
}

/// `C` must implement the DbController trait.
///
/// `E` must implement the sqlx::FromRow trait, and the Unpin and Send traits (for async use); It is the Entity we are querying for
///
/// `'r` lifetime guarantees the result of the query will live long enough to be returned by the function
///
/// Returns `Vec<E>`, the entities the query looks for
pub async fn list<C, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    C: DbController,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = mm.db();
    let sql = format!("SELECT * FROM {} ORDER BY id LIMIT 100", C::TABLE);
    debug!("{sql}");
    let entities: Vec<E> = sqlx::query_as(&sql).fetch_all(db).await?;
    Ok(entities)
}

/// `C` must implement the DbController trait.
///
/// `D` must implement the serde::Serialize trait; It is the generic struct that we parameterize and INSERT into the database
///
/// Returns the `Id` of the newly created item
pub async fn update<C, D>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: D) -> Result<()>
where
    C: DbController,
    D: Serialize,
{
    let data_json: HashMap<String, Value> = match serde_json::to_value(data)? {
        serde_json::Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        _ => {
            return Err(model::error::Error::Serde(serde_json::Error::custom(
                "expected json object",
            )))
        }
    };
    // Using 1-based indexing for SQL placeholders
    // Start at + 2 to avoid clash with parameterized .bind(id) which maps to $1
    let assignments: Vec<String> = data_json
        .keys()
        .enumerate()
        .map(|(i, k)| format!("{} = ${}", k, i + 2))
        .collect();

    let db = mm.db();
    let sql = format!(
        "UPDATE {} SET {} WHERE id = $1 RETURNING id",
        C::TABLE,
        assignments.join(", "),
    );
    debug!("{sql}");
    let mut query = sqlx::query(&sql).bind(id); // Bind the id first

    // Bind each value to the query with type extraction
    // TODO: Abstract to json value mapping function to provide consistency between Create / Update
    for value in data_json.values() {
        query = bind_value_to_query(query, value)?
    }
    let count = query.execute(db).await?.rows_affected();

    match count {
        0 => Err(Error::EntityNotFound {
            entity: C::TABLE,
            id,
        }),
        _ => Ok(()),
    }
}

pub async fn delete<C>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
    C: DbController,
{
    let db = mm.db();
    let sql = format!("DELETE FROM {} WHERE id = $1", C::TABLE,);
    debug!("{sql}");
    let count = sqlx::query(&sql)
        .bind(id) // Bind the id first
        .execute(db)
        .await?
        .rows_affected();

    match count {
        0 => Err(Error::EntityNotFound {
            entity: C::TABLE,
            id,
        }),
        _ => Ok(()),
    }
}
