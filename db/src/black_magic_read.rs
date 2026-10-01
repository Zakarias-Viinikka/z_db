use protocol::error::DbError;
use protocol::payload;
use protocol::row_col::{Col, Row, StructRepresentingNull};
use rusqlite::Connection;
use sql_builder::*;

pub fn read_from_db(conn: &Connection, ctx: &payload::GetDataIn) -> Result<Vec<Row>, DbError> {
    let table_name = &ctx.table_name;
    let columns_to_read = &ctx.columns_to_read;
    let where_clause = to_sql_condition(&ctx.arguments);
    let sql = generate_read_from_table_sql(table_name, &where_clause, columns_to_read);
    query_rows(conn, &sql)
        .map_err(|e| DbError::SqlExecuteFail(format!("read_from_db failed: {:?}, sql: {}", e, sql)))
}

pub fn read_from_db_ordered(
    conn: &Connection,
    ctx: &payload::GetDataOrderedIn,
) -> Result<Vec<Row>, DbError> {
    let table_name = &ctx.table_name;
    let columns_to_read = &ctx.columns_to_read;
    let order_by = &ctx.order_by;
    let where_clause = to_sql_condition(&ctx.arguments);
    let sql = generate_get_data_by_order_sql(table_name, &where_clause, columns_to_read, order_by);
    query_rows(conn, &sql)
}

pub fn query_rows(conn: &Connection, sql: &str) -> Result<Vec<Row>, DbError> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;
    let col_count = stmt.column_count();
    let mut rows = Vec::new();
    let mut query = stmt
        .query([])
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    while let Some(row) = query
        .next()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
    {
        let mut cols = Vec::with_capacity(col_count);
        for i in 0..col_count {
            let col = match row
                .get_ref(i)
                .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
            {
                rusqlite::types::ValueRef::Null => Col::Null(StructRepresentingNull {}),
                rusqlite::types::ValueRef::Integer(n) => Col::Integer(n),
                rusqlite::types::ValueRef::Real(f) => Col::Real(f),
                rusqlite::types::ValueRef::Text(t) => {
                    Col::Text(String::from_utf8_lossy(t).into_owned())
                }
                rusqlite::types::ValueRef::Blob(b) => Col::Blob(b.to_vec()),
            };
            cols.push(col);
        }
        rows.push(Row { cols });
    }
    Ok(rows)
}

pub fn get_single_col(conn: &Connection, ctx: &payload::GetSingleColIn) -> Result<Col, DbError> {
    let get_data_in = payload::GetDataIn {
        table_name: ctx.table_name.clone(),
        arguments: ctx.arguments.clone(),
        columns_to_read: vec![ctx.column_to_read.clone()],
    };

    let mut rows = read_from_db(conn, &get_data_in)?;

    let row = rows
        .drain(..)
        .next()
        .ok_or_else(|| DbError::IllegalInput("get_single_col: no rows".to_string()))?;

    let col =
        row.cols.into_iter().next().ok_or_else(|| {
            DbError::IllegalInput("get_single_col: row had no columns".to_string())
        })?;

    Ok(col)
}
