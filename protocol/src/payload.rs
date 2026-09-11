use crate::error::DbError;
use crate::new_table;
use crate::new_table::ForeignKeyDef;
use crate::row_col;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, uniffi::Record)]
pub struct CreateTableIn {
    pub table_name: String,
    pub columns: Vec<new_table::ColumnDef>,
}

#[derive(Serialize, Deserialize, Debug, uniffi::Record)]
pub struct CreateTableOut {
    //this used to be Result<(), DbError>
    // but uniffi didn't like that so i changed it to option
    pub result: Option<DbError>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ListTablesOut {
    pub table_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, uniffi::Enum)]
pub enum JoinType {
    And,
    Or,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Enum)]
pub enum SelectArgument {
    XEqualY {
        x: String,
        y: String,
        join: Option<JoinType>,
    },
    XNotEqualY {
        x: String,
        y: String,
        join: Option<JoinType>,
    },
    XGreaterThanY {
        x: String,
        y: String,
        join: Option<JoinType>,
    },
    XLessThanY {
        x: String,
        y: String,
        join: Option<JoinType>,
    },
    XGreaterThanOrEqualY {
        x: String,
        y: String,
        join: Option<JoinType>,
    },
    XLessThanOrEqualY {
        x: String,
        y: String,
        join: Option<JoinType>,
    },
    XLikeY {
        x: String,
        y: String,
        join: Option<JoinType>,
    },
    XInY {
        x: String,
        y: Vec<String>,
        join: Option<JoinType>,
    },
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetDataIn {
    pub table_name: String,
    pub arguments: Vec<SelectArgument>,
    pub columns_to_read: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetDataOut {
    pub rows: Vec<row_col::Row>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetDataOrderedIn {
    pub table_name: String,
    pub arguments: Vec<SelectArgument>,
    pub columns_to_read: Vec<String>,
    pub order_by: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ColumnValue {
    pub column_name: String,
    pub value: row_col::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct InsertDataIn {
    pub table_name: String,
    pub values: Vec<ColumnValue>,
}

#[derive(Serialize, Deserialize, Debug, uniffi::Record)]
pub struct InsertDataOut {
    //this used to be Result<(), DbError>
    // but uniffi didn't like that so i changed it to option
    pub result: Option<DbError>,
}

// public_data_shapes.rs
#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct DropTableIn {
    pub table_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct EditColInRowIn {
    pub table_name: String,
    pub row_id: String,
    pub column: String,
    pub new_value: row_col::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CheckTableIn {
    pub table_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct TableColumnInfo {
    pub cid: i64,
    pub name: String,
    pub type_name: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CheckTableOut {
    pub columns: Vec<TableColumnInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct DeleteRowIn {
    pub table_name: String,
    pub row_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct SwapColumnsIn {
    pub table_name: String,
    pub row_id_1: String,
    pub row_id_2: String,
    pub column: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CreateIndexIn {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CheckIndexIn {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CheckIndexOut {
    pub is_indexed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct AddColumnIn {
    pub table_name: String,
    pub column: new_table::ColumnDef,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ExportDatabaseIn {}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ExportDatabaseOut {
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct RemoveColumnIn {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ExportTablesIn {
    pub table_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct TableExport {
    pub table_name: String,
    pub columns: Vec<TableColumnInfo>,
    pub rows: Vec<row_col::Row>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ExportTablesOut {
    pub tables: Vec<TableExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CreateTableFromExportIn {
    pub table_name: String,
    pub table: TableExport,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CopyTableIn {
    pub source_table_name: String,
    pub new_table_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CreateFts5TableIn {
    pub source_table_name: String,
    pub columns: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct SearchFts5In {
    pub table_name: String,
    pub text_to_lookup: String,
}

pub type SearchFts5Out = GetDataOut;

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct RebuildFts5In {
    pub table_name: String,
}

pub type ForceDropTableIn = DropTableIn;

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CreateForeignTableIn {
    pub table_name: String,
    pub columns: Vec<new_table::ColumnDef>,
    pub foreign_keys: Vec<ForeignKeyDef>,
}

// Describes the filters for one column, used when counting rows.
// To count rows where "name" is "x":
// ```
// ColumnFilter {
//     col_name: "name".into(),
//     arguments: vec![SelectArgument::XEqualY {
//         x: "name".into(),
//         y: "x".into(),
//         join: None,
//     }],
//     join: None,
// }
// ```
#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ColumnFilter {
    pub col_name: String,
    pub arguments: Vec<SelectArgument>,
    pub join: Option<JoinType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CountRowsIn {
    pub table_name: String,
    pub filters: Vec<ColumnFilter>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CountRowsOut {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CountAllRowsIn {
    pub table_name: String,
}

pub type CountAllRowsOut = CountRowsOut;
