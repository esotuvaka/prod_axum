use crate::model::store;
use core::result::Result as CoreResult;
use serde::{Serialize, Serializer};
use serde_json::Error as SerdeJsonError;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    /// Entity is the table name, id is the unique id of the item
    EntityNotFound {
        entity: &'static str,
        id: i64,
    },
    Store(store::Error),
    // -- External
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    #[serde(serialize_with = "serialize_serde_json_error")]
    Serde(SerdeJsonError),
}

fn serialize_serde_json_error<S>(
    error: &serde_json::Error,
    serializer: S,
) -> CoreResult<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&error.to_string())
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serde(error)
    }
}

impl From<sqlx::Error> for Error {
    fn from(val: sqlx::Error) -> Self {
        Self::Sqlx(val)
    }
}

impl From<store::Error> for Error {
    fn from(val: store::Error) -> Self {
        Self::Store(val)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
