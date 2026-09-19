use crate::create_the_entire_universe;
use db::black_magic;
use protocol::error::DbError;
use protocol::payload::*;

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

pub struct LiveForever {
    pub db_conn: rusqlite::Connection,
}

impl LiveForever {
    pub fn new(sqlitedb_path: &str) -> Result<LiveForever, DbError> {
        let db_conn = rusqlite::Connection::open(sqlitedb_path)
            .map_err(|e| DbError::ConnError(e.to_string()))?;
        Ok(LiveForever { db_conn })
    }

    pub fn export_database(&self, _data: ExportDatabaseIn) -> Result<ExportDatabaseOut, DbError> {
        let bytes = black_magic::export_database(&self.db_conn)?;
        Ok(ExportDatabaseOut { data: bytes })
    }

    fn get_conn(&self) -> Result<&rusqlite::Connection, DbError> {
        Ok(&self.db_conn)
    }
}

create_the_entire_universe!(LiveForever);
