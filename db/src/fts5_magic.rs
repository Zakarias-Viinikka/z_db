use crate::black_magic_read::{self, query_rows};
use protocol::error::DbError;
use protocol::payload::{GetDataIn, SelectArgument, SelectArguments, SyncFts5RowIn};
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
    Ok(())
}

pub fn search_fts5(conn: &Connection, table_name: &str, query: &str) -> Result<Vec<Row>, DbError> {
    let sql = search_fts5_sql_builder(table_name, query);
    query_rows(conn, &sql)
        .map_err(|e| DbError::SqlExecuteFail(format!("search_fts5 failed: {:?}, sql: {}", e, sql)))
}

pub fn sync_fts5_row(conn: &Connection, input: &SyncFts5RowIn) -> Result<(), DbError> {
    if fts5_has_row(conn, &input.source_table_name, &input.row_id)? {
        if let Some(old) = read_current_value(conn, input)? {
            let sql = fts5_delete_sql_builder(
                &input.source_table_name,
                &input.row_id,
                &input.column_name,
                &old,
            );
            conn.execute(&sql, []).map_err(|e| {
                DbError::SqlExecuteFail(format!("fts5 delete failed: {}, sql: {}", e, sql))
            })?;
        }
    }

    if let Some(new) = &input.new_value {
        let sql = fts5_insert_sql_builder(
            &input.source_table_name,
            &input.row_id,
            &input.column_name,
            new,
        );
        conn.execute(&sql, []).map_err(|e| {
            DbError::SqlExecuteFail(format!("fts5 insert failed: {}, sql: {}", e, sql))
        })?;
    }

    Ok(())
}

fn fts5_has_row(conn: &Connection, source_table_name: &str, row_id: &str) -> Result<bool, DbError> {
    let fts_table = quote_ident(&format!("fts5_{}", source_table_name));
    let sql = format!(
        "SELECT 1 FROM {} WHERE rowid = {} LIMIT 1;",
        fts_table, row_id
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;
    stmt.exists([])
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))
}

fn read_current_value(conn: &Connection, input: &SyncFts5RowIn) -> Result<Option<String>, DbError> {
    let rows = black_magic_read::read_from_db(
        conn,
        &GetDataIn {
            table_name: input.source_table_name.clone(),
            arguments: SelectArguments::Single(SelectArgument::XEqualY {
                x: "id".to_string(),
                y: input.row_id.clone(),
            }),
            columns_to_read: vec![input.column_name.clone()],
        },
    )?;

    let value = rows
        .first()
        .and_then(|r| r.cols.first())
        .and_then(|c| c.as_str().ok())
        .map(|s| s.to_string());

    Ok(value)
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
