use crate::payload::types::SelectArguments;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct DropTableIn {
    pub table_name: String,
}

pub type ForceDropTableIn = DropTableIn;

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct DeleteRowIn {
    pub table_name: String,
    pub row_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct DeleteRowWhereIn {
    pub table_name: String,
    pub arguments: SelectArguments,
}

#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct DeleteAllRowsIn {
    pub table_name: String,
}
