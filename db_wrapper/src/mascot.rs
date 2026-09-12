use crate::create_the_entire_universe;
use db::black_magic;
use db::black_magic_read;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::row_col;
use protocol::serialization::*;

macro_rules! unwrap_or_bail {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return e.to_payload(),
        }
    };
}
macro_rules! decode_input {
    ($data:expr) => {
        unwrap_or_bail!(<_>::un_payloadify(&$data))
    };
}
macro_rules! finish_output {
    ($value:expr) => {
        $value.to_payload()
    };
}
macro_rules! return_nothing {
    () => {
        ok_serialized()
    };
}
macro_rules! method_input_type {
    ($typed:ty) => { Vec<u8> };
}
macro_rules! method_return_type {
    ($typed:ty) => { Vec<u8> };
}

pub struct LiveForever {
    pub db_conn: rusqlite::Connection,
}

impl LiveForever {
    pub fn new(sqlitedb_path: &str) -> Result<LiveForever, String> {
        let db_conn = rusqlite::Connection::open(sqlitedb_path).map_err(|e| e.to_string())?;
        Ok(LiveForever { db_conn })
    }

    pub fn export_database(&self, _data: Vec<u8>) -> Vec<u8> {
        let bytes = unwrap_or_bail!(black_magic::export_database(&self.db_conn));
        ExportDatabaseOut { data: bytes }.to_payload()
    }

    fn get_conn(&self) -> Result<&rusqlite::Connection, DbError> {
        Ok(&self.db_conn)
    }
}

create_the_entire_universe!(LiveForever);
