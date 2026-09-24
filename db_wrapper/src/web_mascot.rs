use crate::create_the_entire_universe;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::serialization::*;
use wasm_bindgen::prelude::*;

use sqlite_wasm_rs as ffi;
use sqlite_wasm_vfs::sahpool::{OpfsSAHPoolCfg, OpfsSAHPoolUtil, install as install_opfs_sahpool};

macro_rules! unwrap_or_bail {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return Err::<(), DbError>(e).to_payload(),
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

#[wasm_bindgen]
pub struct LiveForever {
    db_conn: Option<rusqlite::Connection>,
    sahpool_util: Option<OpfsSAHPoolUtil>,
    conn_name: String,
}

#[wasm_bindgen]
impl LiveForever {
    pub async fn new(conn_name: String) -> Result<LiveForever, JsValue> {
        let sahpool_util =
            install_opfs_sahpool::<ffi::WasmOsCallback>(&OpfsSAHPoolCfg::default(), true)
                .await
                .map_err(|e| {
                    JsValue::from_str(&format!("Failed to install OPFS SAH pool: {}", e))
                })?;

        let db_conn = rusqlite::Connection::open_with_flags(
            &conn_name,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
        )
        .map_err(|e| JsValue::from_str(&format!("Failed to open database: {}", e)))?;

        Ok(LiveForever {
            db_conn: Some(db_conn),
            sahpool_util: Some(sahpool_util),
            conn_name,
        })
    }

    pub fn export_database(&self, _data: Vec<u8>) -> Vec<u8> {
        let util = self
            .sahpool_util
            .as_ref()
            .ok_or_else(|| DbError::ConnError("OPFS SAH pool is not initialised".to_string()));
        let util = unwrap_or_bail!(util);
        let bytes = unwrap_or_bail!(
            util.export_db(&self.conn_name)
                .map_err(|e| DbError::SqlExecuteFail(e.to_string()))
        );
        ExportDatabaseOut { data: bytes }.to_payload()
    }

    pub fn close_conn_js(&mut self) -> Result<(), JsValue> {
        if let Some(conn) = self.db_conn.take() {
            if let Err((_, err)) = conn.close() {
                return Err(JsValue::from_str(&format!(
                    "Failed to close connection: {}",
                    err
                )));
            }
        }

        if let Some(util) = self.sahpool_util.take() {
            if let Err(e) = util.pause_vfs() {
                return Err(JsValue::from_str(&format!("Failed to pause VFS: {}", e)));
            }
        }

        Ok(())
    }
}

impl LiveForever {
    fn get_conn(&self) -> Result<&rusqlite::Connection, DbError> {
        self.db_conn
            .as_ref()
            .ok_or_else(|| DbError::ConnError("Database not connected".to_string()))
    }
}

create_the_entire_universe!(LiveForever, wasm_bindgen);
