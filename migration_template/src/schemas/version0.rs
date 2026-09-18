use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};

use crate::helpers::ADbTable;

pub fn entire_table() -> Vec<ADbTable> {
    let mut entire_table: Vec<ADbTable> = Vec::new();

    entire_table.push(ADbTable {
        table_name: "table1".to_string(),
        columns: table1_columns(),
    });

    entire_table
}

pub fn table1_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "text"),
        not_null_col(ColumnType::Integer, "number"),
    ]
}
