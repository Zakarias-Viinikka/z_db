use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Error, uniffi::Error)]
pub enum DbError {
    #[error(
        "This error was originally used for when failing to convert from and to js. not sure if this version of this project uses it or where: {0}"
    )]
    CureFail(String),
    #[error("connection error: {0}")]
    ConnError(String),
    #[error("illegal input: {0}")]
    IllegalInput(String),
    #[error("sql execute failed: {0}")]
    SqlExecuteFail(String),
    #[error("serialize error: {0}")]
    SerializeError(String),
    #[error("bad code: {0}")]
    BadCode(String),
}
