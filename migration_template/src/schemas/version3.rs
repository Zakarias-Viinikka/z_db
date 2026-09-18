use db_wrapper::testing_mascot::LiveForever;
use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};
use protocol::payload::RemoveColumnIn;

use crate::helpers::ADbTable;

pub fn update_from_old_version(db_wrapper: &LiveForever) {
    db_wrapper
        .remove_column(RemoveColumnIn {
            table_name: "table2".to_string(),
            column_name: "extra".to_string(),
        })
        .unwrap();
}

pub fn entire_table() -> Vec<ADbTable> {
    let mut entire_table: Vec<ADbTable> = Vec::new();

    entire_table.push(ADbTable {
        table_name: "table1".to_string(),
        columns: table1_columns(),
    });

    entire_table.push(ADbTable {
        table_name: "table2".to_string(),
        columns: table2_columns(),
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

pub fn table2_columns() -> Vec<ColumnDef> {
    vec![id_column(), not_null_col(ColumnType::Text, "label")]
}
