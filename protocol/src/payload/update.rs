use crate::new_table::ColumnDef;
use crate::payload::types::{ColumnSource, SelectArguments};
use crate::row_col;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct EditColInRowIn {
    pub table_name: String,
    pub row_id: String,
    pub column: String,
    pub new_value: row_col::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct EditColInRowWhereIn {
    pub table_name: String,
    pub where_clause: SelectArguments,
    pub column: String,
    pub new_value: row_col::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct AddColumnIn {
    pub table_name: String,
    pub column: ColumnDef,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct RemoveColumnIn {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct FundamentallyEditExistingColIn {
    pub table_name: String,
    pub column_name: String,
    pub new_column: ColumnDef,
    pub source: ColumnSource,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct SwapColumnsIn {
    pub table_name: String,
    pub row_id_1: String,
    pub row_id_2: String,
    pub column: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct RebuildFts5In {
    pub table_name: String,
}
