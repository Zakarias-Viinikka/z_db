use protocol::error::DbError;
use rusqlite::Connection;

pub fn begin_all_or_nothing(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| DbError::SqlExecuteFail(format!("begin_all_or_nothing failed: {}", e)))?;
    Ok(())
}

pub fn everything_went_perfectly(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch("COMMIT")
        .map_err(|e| DbError::SqlExecuteFail(format!("everything_went_perfectly failed: {}", e)))?;
    Ok(())
}

pub fn regret_everything(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch("ROLLBACK")
        .map_err(|e| DbError::SqlExecuteFail(format!("regret_everything failed: {}", e)))?;
    Ok(())
}

pub fn are_we_in_middle_of_transaction(conn: &Connection) -> bool {
    !conn.is_autocommit()
}
