//i wanted to figure this stuff out later. i asked ai for an example temporarily so i don't have to start from scratch with figuring this stuff out when i wanna do it

/*
// THEORETICAL PARAMETERIZED QUERY EXAMPLE (for future DML refactor)

use rusqlite::Connection;
use protocol::error::DbError;
use protocol::row_col::Col;

// 1. Define a payload for the operation
struct InsertPayload {
    table_name: String,
    columns: Vec<String>,
    values: Vec<Col>,
}

// 2. Generate SQL with placeholders (no user values embedded)
fn generate_insert_sql(payload: &InsertPayload) -> String {
    let placeholders: Vec<String> = (1..=payload.columns.len())
        .map(|i| format!("?{}", i))
        .collect();

    format!(
        "INSERT INTO {} ({}) VALUES ({});",
        quote_ident(&payload.table_name),
        payload.columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", "),
        placeholders.join(", ")
    )
}

// 3. Execute with bound values (mock do_sql)
fn do_sql(conn: &Connection, sql: &str, values: &[Col]) -> Result<(), DbError> {
    // This is where rusqlite would bind the values as parameters
    // For example:
    // let params = values.iter().map(|v| match v {
    //     Col::Null => rusqlite::types::Value::Null,
    //     Col::Integer(i) => rusqlite::types::Value::Integer(*i),
    //     Col::Real(f) => rusqlite::types::Value::Real(*f),
    //     Col::Text(s) => rusqlite::types::Value::Text(s.clone()),
    //     Col::Blob(b) => rusqlite::types::Value::Blob(b.clone()),
    // }).collect::<Vec<_>>();

    // conn.execute(sql, &params)
    //     .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    // For this mock, we just pretend it succeeded:
    let _ = (conn, sql, values);
    Ok(())
}

// 4. Example usage
fn insert_into_table_parameterized(
    conn: &Connection,
    table_name: &str,
    columns: Vec<String>,
    values: Vec<Col>,
) -> Result<(), DbError> {
    let payload = InsertPayload {
        table_name: table_name.to_string(),
        columns,
        values,
    };

    let sql = generate_insert_sql(&payload);
    do_sql(conn, &sql, &payload.values)
}
*/
