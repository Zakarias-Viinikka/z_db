use crate::error::DbError;
use crate::new_table::{ColumnDef, ForeignKeyDef};
use crate::payload::types::{ColumnValue, TableExport};
use crate::row_col;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, uniffi::Record)]
pub struct CreateTableIn {
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Serialize, Deserialize, Debug, uniffi::Record)]
pub struct CreateTableOut {
    pub result: Option<DbError>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct InsertDataIn {
    pub table_name: String,
    pub values: Vec<ColumnValue>,
}

#[derive(Serialize, Deserialize, Debug, uniffi::Record)]
pub struct InsertDataOut {
    pub result: Option<DbError>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct InsertDataAndGetColIn {
    pub table_name: String,
    pub values: Vec<ColumnValue>,
    pub column_to_return: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct InsertDataAndGetColOut {
    pub value: row_col::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CreateIndexIn {
    pub table_name: String,
    pub column_name: String,
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
pub struct CreateForeignTableIn {
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
    pub foreign_keys: Vec<ForeignKeyDef>,
}
