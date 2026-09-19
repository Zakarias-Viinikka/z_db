use crate::black_magic_read::query_rows;
use protocol::error::DbError;
use protocol::row_col::Row;
use rusqlite::Connection;
use sql_builder::*;

pub fn create_fts5_table(
    conn: &Connection,
    source_table_name: &str,
    columns: Vec<String>,
) -> Result<(), DbError> {
    let sql = create_fts5_sql_builder(source_table_name, &columns);
    conn.execute(&sql, []).map_err(|e| {
        DbError::SqlExecuteFail(format!("create_fts5_table failed: {}, sql: {}", e, sql))
    })?;

    for column in &columns {
        let trigger_sql = create_fts5_triggers_sql_builder(
            source_table_name,
            &format!("fts5_{}", source_table_name),
            column,
        );
        conn.execute_batch(&trigger_sql).map_err(|e| {
            DbError::SqlExecuteFail(format!(
                "create_fts5_triggers failed: {}, sql: {}",
                e, trigger_sql
            ))
        })?;
    }

    Ok(())
}

pub fn search_fts5(conn: &Connection, table_name: &str, query: &str) -> Result<Vec<Row>, DbError> {
    let sql = search_fts5_sql_builder(table_name, query);
    query_rows(conn, &sql)
        .map_err(|e| DbError::SqlExecuteFail(format!("search_fts5 failed: {:?}, sql: {}", e, sql)))
}

pub fn rebuild_fts5_index(conn: &Connection, source_table_name: &str) -> Result<(), DbError> {
    let fts_table_name = quote_ident(&format!("fts5_{}", source_table_name));

    let sql = format!(
        "INSERT INTO {table}({table}) VALUES('rebuild');",
        table = fts_table_name
    );

    conn.execute(&sql, []).map_err(|e| {
        DbError::SqlExecuteFail(format!("rebuild_fts5_index failed: {}, sql: {}", e, sql))
    })?;

    Ok(())
}
