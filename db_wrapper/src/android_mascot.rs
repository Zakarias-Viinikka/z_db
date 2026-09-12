use crate::create_the_entire_universe;
use protocol::error::DbError;
use protocol::payload::*;
use std::sync::Mutex;

macro_rules! unwrap_or_bail {
    ($result:expr) => {
        $result?
    };
}
macro_rules! decode_input {
    ($data:expr) => {
        $data
    };
}
macro_rules! finish_output {
    ($value:expr) => {
        Ok($value)
    };
}
macro_rules! return_nothing {
    () => {
        Ok(())
    };
}
macro_rules! method_input_type {
    ($typed:ty) => {
        $typed
    };
}
macro_rules! method_return_type { ($typed:ty) => { Result<$typed, DbError> }; }

#[derive(uniffi::Object)]
pub struct LiveForever {
    db_conn: Mutex<rusqlite::Connection>,
}

#[uniffi::export]
impl LiveForever {
    #[uniffi::constructor]
    pub fn new(sqlitedb_path: &str) -> Result<LiveForever, DbError> {
        let db_conn = rusqlite::Connection::open(sqlitedb_path)
            .map_err(|e| DbError::ConnError(e.to_string()))?;
        Ok(LiveForever {
            db_conn: Mutex::new(db_conn),
        })
    }
}

impl LiveForever {
    fn get_conn(&self) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, DbError> {
        self.db_conn
            .lock()
            .map_err(|e| DbError::ConnError(format!("failed to lock db connection: {e}")))
    }
}

create_the_entire_universe!(LiveForever, uniffi::export);
