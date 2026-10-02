use protocol::error::DbError;
use protocol::payload::{CountAllRowsIn, CountAllRowsOut, CountRowsIn, CountRowsOut};
use rusqlite::Connection;
use sql_builder::*;

pub fn count_rows(conn: &Connection, input: &CountRowsIn) -> Result<CountRowsOut, DbError> {
    let sql = generate_count_rows_sql(&input.table_name, &input.arguments);
    let count: i64 = conn
        .query_row(&sql, [], |row| row.get(0))
        .map_err(|e| DbError::SqlExecuteFail(format!("count_rows failed: {}, sql: {}", e, sql)))?;
    Ok(CountRowsOut {
        count: count as u64,
    })
}

pub fn count_all_rows(
    conn: &Connection,
    input: &CountAllRowsIn,
) -> Result<CountAllRowsOut, DbError> {
    let sql = generate_count_all_rows_sql(&input.table_name);
    let count: i64 = conn.query_row(&sql, [], |row| row.get(0)).map_err(|e| {
        DbError::SqlExecuteFail(format!("count_all_rows failed: {}, sql: {}", e, sql))
    })?;
    Ok(CountAllRowsOut {
        count: count as u64,
    })
}

pub fn delete_all_rows(conn: &Connection, table_name: &str) -> Result<(), DbError> {
    let sql = generate_delete_all_rows_sql(table_name);
    conn.execute(&sql, []).map_err(|e| {
        DbError::SqlExecuteFail(format!("delete_all_rows failed: {}, sql: {}", e, sql))
    })?;
    Ok(())
}
