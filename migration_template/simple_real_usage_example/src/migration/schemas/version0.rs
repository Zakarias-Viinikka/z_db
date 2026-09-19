use crate::migration::helpers::ADbTable;
use protocol::new_table::{ColumnDef, ColumnType, ForeignKeyDef, id_column, not_null_col};

pub fn entire_table() -> Vec<ADbTable> {
    let mut entire_table: Vec<ADbTable> = Vec::new();

    entire_table.push(ADbTable {
        table_name: "texts".to_string(),
        columns: texts_columns(),
    });

    entire_table.push(ADbTable {
        table_name: "keyword_lookup".to_string(),
        columns: keyword_lookup_columns(),
    });

    entire_table
}

pub fn texts_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "title"),
        not_null_col(ColumnType::Text, "body"),
    ]
}

pub fn keyword_lookup_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Integer, "text_id"),
        not_null_col(ColumnType::Text, "keyword"),
    ]
}

pub fn get_foreign_def_keyword_lookup() -> Vec<ForeignKeyDef> {
    vec![ForeignKeyDef {
        column: "text_id".to_string(),
        referenced_table: "texts".to_string(),
        referenced_column: "id".to_string(),
    }]
}
