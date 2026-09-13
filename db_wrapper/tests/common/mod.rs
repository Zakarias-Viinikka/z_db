#![cfg(feature = "testing")]
#![allow(dead_code)]

// Shared test helpers. Table macros stay in each test file, because
// the table shape is per-file.

use db_wrapper::testing_mascot::LiveForever;
use protocol::error::DbError;
use protocol::new_table::{
    ColumnDef, ColumnType, col_with_default_value, not_null_col, not_null_unique_col, unique_col,
};
use protocol::payload::*;

// ============================================================
// COLUMN HELPERS
// ============================================================

pub fn unique_text_col(name: &str) -> ColumnDef {
    unique_col(ColumnType::Text, name)
}

pub fn not_null_text_col(name: &str) -> ColumnDef {
    not_null_col(ColumnType::Text, name)
}

pub fn not_null_int_col(name: &str) -> ColumnDef {
    not_null_col(ColumnType::Integer, name)
}

pub fn not_null_unique_int_col(name: &str) -> ColumnDef {
    not_null_unique_col(ColumnType::Integer, name)
}

pub fn text_col_with_default(name: &str, default: &str) -> ColumnDef {
    col_with_default_value(ColumnType::Text, default.into(), name)
}

pub fn unique_text_col_with_default(name: &str, default: &str) -> ColumnDef {
    ColumnDef {
        name: name.into(),
        column_type: "TEXT".into(),
        primary_key: false,
        not_null: false,
        unique: true,
        default_value: default.into(),
        autoincrement: false,
    }
}

pub fn not_null_text_col_with_default(name: &str, default: &str) -> ColumnDef {
    ColumnDef {
        name: name.into(),
        column_type: "TEXT".into(),
        primary_key: false,
        not_null: true,
        unique: false,
        default_value: default.into(),
        autoincrement: false,
    }
}

// ============================================================
// ACTION HELPERS
// ============================================================

pub struct AddColumn<'a> {
    pub table_name: &'a str,
    pub column: ColumnDef,
    pub db: &'a LiveForever,
}

pub struct DropColumn<'a> {
    pub table_name: &'a str,
    pub column_name: &'a str,
    pub db: &'a LiveForever,
}

pub struct EditColumn<'a> {
    pub table_name: &'a str,
    pub column_name: &'a str,
    pub new_column: ColumnDef,
    pub source: ColumnSource,
    pub db: &'a LiveForever,
}

pub struct IndexColumn<'a> {
    pub table_name: &'a str,
    pub column_name: &'a str,
    pub db: &'a LiveForever,
}

pub fn setup_empty_db() -> LiveForever {
    LiveForever::new().unwrap()
}

pub fn create_table(table_name: &str, columns: Vec<ColumnDef>, db: &LiveForever) {
    db.create_table(CreateTableIn {
        table_name: table_name.to_string(),
        columns,
    })
    .unwrap();
}

pub fn add_column(params: AddColumn) -> Result<(), DbError> {
    params.db.add_column(AddColumnIn {
        table_name: params.table_name.to_string(),
        column: params.column,
    })
}

pub fn drop_column(params: DropColumn) {
    params
        .db
        .remove_column(RemoveColumnIn {
            table_name: params.table_name.to_string(),
            column_name: params.column_name.to_string(),
        })
        .unwrap();
}

pub fn edit_column(params: EditColumn) -> Result<(), DbError> {
    params
        .db
        .fundamentally_edit_existing_col(FundamentallyEditExistingColIn {
            table_name: params.table_name.to_string(),
            column_name: params.column_name.to_string(),
            new_column: params.new_column,
            source: params.source,
        })
}

pub fn index_column(params: IndexColumn) {
    params
        .db
        .create_index(CreateIndexIn {
            table_name: params.table_name.to_string(),
            column_name: params.column_name.to_string(),
        })
        .unwrap();
}

// ============================================================
// OBSERVATION HELPERS
// ============================================================

pub struct ColumnNames<'a> {
    pub table_name: &'a str,
    pub db: &'a LiveForever,
}

pub struct ColumnTypes<'a> {
    pub table_name: &'a str,
    pub db: &'a LiveForever,
}

pub struct IsIndexed<'a> {
    pub table_name: &'a str,
    pub column_name: &'a str,
    pub db: &'a LiveForever,
}

pub struct IsUnique<'a> {
    pub table_name: &'a str,
    pub column_name: &'a str,
    pub db: &'a LiveForever,
}

pub struct RowCount<'a> {
    pub table_name: &'a str,
    pub db: &'a LiveForever,
}

pub struct StringValuesInCol<'a> {
    pub table_name: &'a str,
    pub column_name: &'a str,
    pub db: &'a LiveForever,
}

pub fn column_names(params: ColumnNames) -> Vec<String> {
    params
        .db
        .check_table(CheckTableIn {
            table_name: params.table_name.to_string(),
        })
        .unwrap()
        .columns
        .iter()
        .map(|c| c.name.clone())
        .collect()
}

pub fn column_types(params: ColumnTypes) -> Vec<String> {
    params
        .db
        .check_table(CheckTableIn {
            table_name: params.table_name.to_string(),
        })
        .unwrap()
        .columns
        .iter()
        .map(|c| c.type_name.clone())
        .collect()
}

pub fn is_indexed(params: IsIndexed) -> bool {
    params
        .db
        .check_index(CheckIndexIn {
            table_name: params.table_name.to_string(),
            column_name: params.column_name.to_string(),
        })
        .unwrap()
        .is_indexed
}

pub fn is_unique(params: IsUnique) -> bool {
    db::migration::check_if_col_unique(&params.db.db_conn, params.table_name, params.column_name)
        .unwrap()
}

pub fn row_count(params: RowCount) -> u64 {
    params
        .db
        .count_all_rows(CountAllRowsIn {
            table_name: params.table_name.to_string(),
        })
        .unwrap()
        .count
}

pub fn string_values_in_col(params: StringValuesInCol) -> Vec<String> {
    params
        .db
        .get_data(GetDataIn {
            table_name: params.table_name.to_string(),
            arguments: SelectArguments::Single(SelectArgument::All),
            columns_to_read: vec![params.column_name.to_string()],
        })
        .unwrap()
        .rows
        .iter()
        .map(|r| r.cols[0].as_str().unwrap().to_string())
        .collect()
}
