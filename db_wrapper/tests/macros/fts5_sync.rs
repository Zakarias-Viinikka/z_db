macro_rules! setup {
    () => {
        db_wrapper::testing_mascot::LiveForever::new().unwrap()
    };
}

macro_rules! create_text_table {
    ($db:expr, [$($col:expr),* $(,)?] $(,)?) => {{
        let mut columns = Vec::new();
        $(columns.push($col);)*
        $db.create_table(protocol::payload::CreateTableIn {
            table_name: "keyword_lookup".to_string(),
            columns,
        })
        .unwrap();
    }};
}

macro_rules! create_text_fts5 {
    ($db:expr, [$($fts_col:expr),* $(,)?] $(,)?) => {{
        let mut columns = Vec::new();
        $(columns.push($fts_col.to_string());)*
        $db.create_fts5_table(protocol::payload::CreateFts5TableIn {
            source_table_name: "keyword_lookup".to_string(),
            columns,
        })
        .unwrap();
    }};
}
