use crate::row_col;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, uniffi::Enum)]
pub enum JoinType {
    And,
    Or,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Enum)]
pub enum SelectArgument {
    XEqualY { x: String, y: String },
    XNotEqualY { x: String, y: String },
    XGreaterThanY { x: String, y: String },
    XLessThanY { x: String, y: String },
    XGreaterThanOrEqualY { x: String, y: String },
    XLessThanOrEqualY { x: String, y: String },
    XLikeY { x: String, y: String },
    XInY { x: String, y: Vec<String> },
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Enum)]
pub enum SelectArguments {
    Single(SelectArgument),
    Two {
        first: SelectArgument,
        join: JoinType,
        second: SelectArgument,
    },
    Three {
        first: SelectArgument,
        join: JoinType,
        second: SelectArgument,
        join2: JoinType,
        third: SelectArgument,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ColumnValue {
    pub column_name: String,
    pub value: row_col::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record, PartialEq)]
pub struct TableColumnInfo {
    pub cid: i64,
    pub name: String,
    pub type_name: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct TableExport {
    pub table_name: String,
    pub columns: Vec<TableColumnInfo>,
    pub rows: Vec<row_col::Row>,
}

// Where an edited column's data comes from when the table gets rebuilt.
#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Enum)]
pub enum ColumnSource {
    SameName,
    FromColumn(String),
    Expression(String),
    UseDefault,
}
