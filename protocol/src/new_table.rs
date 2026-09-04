#![warn(unused)]
#![allow(dead_code)]

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, uniffi::Record)]
pub struct ColumnDef {
    pub name: String,          // name
    pub column_type: String,   // column type
    pub primary_key: bool,     // primary key
    pub not_null: bool,        // not null
    pub unique: bool,          // unique
    pub default_value: String, // default value
    pub autoincrement: bool,   // autoincrement
}

pub struct ColumnDefBuilder(
    pub String,     // name
    pub ColumnType, // column type
    pub bool,       // primary key
    pub bool,       // not null
    pub bool,       // unique
    pub String,     // default value
    pub bool,       // autoincrement
);

#[derive(uniffi::Enum)]
pub enum ColumnType {
    Integer,
    Text,
    Real,
    Blob,
    UniFfiDontRenameMyEnums(bool),
}

pub fn builder_to_column_def(builder: ColumnDefBuilder) -> ColumnDef {
    let column_type = match builder.1 {
        ColumnType::Integer => "INTEGER".to_string(),
        ColumnType::Text => "TEXT".to_string(),
        ColumnType::Real => "REAL".to_string(),
        ColumnType::Blob => "BLOB".to_string(),
        ColumnType::UniFfiDontRenameMyEnums(_) => "ILLEGAL".to_string(),
    };

    ColumnDef {
        name: builder.0,
        column_type,
        primary_key: builder.2,
        not_null: builder.3,
        unique: builder.4,
        default_value: builder.5,
        autoincrement: builder.6,
    }
}

#[uniffi::export]
pub fn id_column() -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        "id".to_string(),
        ColumnType::Integer,
        true,
        true,
        false,
        "".to_string(),
        true,
    ))
}

#[uniffi::export]
pub fn default_col(column_type: ColumnType, column_name: &str) -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        column_name.to_string(),
        column_type,
        false,
        false,
        false,
        "".to_string(),
        false,
    ))
}

#[uniffi::export]
pub fn col_with_default_value(
    column_type: ColumnType,
    default_value: String,
    column_name: &str,
) -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        column_name.to_string(),
        column_type,
        false,
        false,
        false,
        default_value,
        false,
    ))
}

#[uniffi::export]
pub fn not_null_col(column_type: ColumnType, column_name: &str) -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        column_name.to_string(),
        column_type,
        false,
        true,
        false,
        "".to_string(),
        false,
    ))
}

#[uniffi::export]
pub fn unique_col(column_type: ColumnType, column_name: &str) -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        column_name.to_string(),
        column_type,
        false,
        false,
        true,
        "".to_string(),
        false,
    ))
}

#[uniffi::export]
pub fn not_null_unique_col(column_type: ColumnType, column_name: &str) -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        column_name.to_string(),
        column_type,
        false,
        true,
        true,
        "".to_string(),
        false,
    ))
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, uniffi::Record)]
pub struct ForeignKeyDef {
    pub column: String,            // which column in this table
    pub referenced_table: String,  // other table name
    pub referenced_column: String, // column in other table
}
