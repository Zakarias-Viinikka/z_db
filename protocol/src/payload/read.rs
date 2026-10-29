use crate::payload::types::{SelectArguments, TableColumnInfo, TableExport};
use crate::row_col;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ListTablesOut {
    pub table_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetDataIn {
    pub table_name: String,
    pub arguments: SelectArguments,
    pub columns_to_read: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetDataOut {
    pub rows: Vec<row_col::Row>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetSingleColIn {
    pub table_name: String,
    pub arguments: SelectArguments,
    pub column_to_read: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetSingleColOut {
    pub value: row_col::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetDataOrderedIn {
    pub table_name: String,
    pub arguments: SelectArguments,
    pub columns_to_read: Vec<String>,
    pub order_by: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct GetDataOrderedAndSkipFirstXIn {
    pub table_name: String,
    pub arguments: SelectArguments,
    pub columns_to_read: Vec<String>,
    pub order_by: String,
    pub skip_first_x: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CheckTableIn {
    pub table_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record, PartialEq)]
pub struct CheckTableOut {
    pub columns: Vec<TableColumnInfo>,
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
pub struct SearchFts5In {
    pub table_name: String,
    pub text_to_lookup: String,
}

pub type SearchFts5Out = GetDataOut;

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct CountRowsIn {
    pub table_name: String,
    pub arguments: SelectArguments,
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

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ExportDatabaseIn {}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ExportDatabaseOut {
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ExportTablesIn {
    pub table_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct ExportTablesOut {
    pub tables: Vec<TableExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct AreWeInMiddleOfTransactionOut {
    pub value: bool,
}
