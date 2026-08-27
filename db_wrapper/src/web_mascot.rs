// db_wrapper/src/web_db_mascot.rs

use db::black_magic;
use db::black_magic_read;
use db::fts5_magic;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::row_col;
use protocol::serialization::*;

use sqlite_wasm_rs as ffi;
use sqlite_wasm_vfs::sahpool::{OpfsSAHPoolCfg, OpfsSAHPoolUtil, install as install_opfs_sahpool};

pub struct LiveForever {
    db_conn: Option<rusqlite::Connection>,
    sahpool_util: Option<OpfsSAHPoolUtil>,
    conn_name: String,
}

macro_rules! unwrap_or_bail {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return e.to_payload(),
        }
    };
}

impl LiveForever {
    pub async fn new(conn_name: String) -> Result<LiveForever, DbError> {
        let sahpool_util =
            install_opfs_sahpool::<ffi::WasmOsCallback>(&OpfsSAHPoolCfg::default(), true)
                .await
                .map_err(|e| {
                    DbError::ConnError(format!("Failed to install OPFS SAH pool: {}", e))
                })?;

        let db_conn = rusqlite::Connection::open_with_flags(
            &conn_name,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
        )
        .map_err(|e| DbError::ConnError(format!("Failed to open database: {}", e)))?;

        Ok(LiveForever {
            db_conn: Some(db_conn),
            sahpool_util: Some(sahpool_util),
            conn_name,
        })
    }

    pub fn create_table(&self, data: Vec<u8>) -> Vec<u8> {
        let data = unwrap_or_bail!(CreateTableIn::un_payloadify(&data));
        let (table_name, columns) = (data.table_name, data.columns);
        let conn = unwrap_or_bail!(self.conn());
        let result = black_magic::create_table(conn, &table_name, columns);
        result.to_payload()
    }

    pub fn list_tables(&self) -> Vec<u8> {
        let conn = unwrap_or_bail!(self.conn());
        let list_of_table_names = unwrap_or_bail!(black_magic::list_tables(conn));
        ListTablesOut {
            table_names: list_of_table_names,
        }
        .to_payload()
    }

    pub fn get_data(&self, data: Vec<u8>) -> Vec<u8> {
        let get_data_in = unwrap_or_bail!(GetDataIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        let result = black_magic_read::read_from_db(conn, &get_data_in);
        match result {
            Ok(result) => GetDataOut { rows: result }.to_payload(),
            Err(e) => e.to_payload(),
        }
    }

    pub fn get_data_ordered(&self, data: Vec<u8>) -> Vec<u8> {
        let get_data_ordered_in = unwrap_or_bail!(GetDataOrderedIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        let result = black_magic_read::read_from_db_ordered(conn, &get_data_ordered_in);
        match result {
            Ok(rows) => GetDataOut { rows }.to_payload(),
            Err(e) => e.to_payload(),
        }
    }

    pub fn insert_data(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(InsertDataIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        let result = black_magic::insert_into_table(conn, &input.table_name, input.values);
        InsertDataOut {
            result: if let Err(e) = result { Some(e) } else { None },
        }
        .to_payload()
    }

    pub fn drop_table(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(DropTableIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        match black_magic::drop_table(conn, &input.table_name) {
            Ok(()) => ok_serialized(),
            Err(e) => e.to_payload(),
        }
    }

    pub fn edit_col_in_row(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(EditColInRowIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        unwrap_or_bail!(black_magic::edit_col_in_row(
            conn,
            &input.table_name,
            &input.row_id,
            &input.column,
            &input.new_value
        ));
        ok_serialized()
    }

    pub fn check_table(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CheckTableIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        let result = black_magic::table_shape(conn, &input.table_name);
        let columns = unwrap_or_bail!(result);
        CheckTableOut { columns }.to_payload()
    }

    pub fn delete_row(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(DeleteRowIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        unwrap_or_bail!(black_magic::delete_row(
            conn,
            &input.table_name,
            &input.row_id
        ));
        ok_serialized()
    }

    pub fn swap_columns(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(SwapColumnsIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());

        let get_value = |row_id: &str| -> Result<row_col::Col, DbError> {
            let get_data_in = GetDataIn {
                table_name: input.table_name.clone(),
                arguments: vec![SelectArgument::XEqualY {
                    x: "id".to_string(),
                    y: row_id.to_string(),
                }],
                columns_to_read: vec![input.column.clone()],
            };

            let rows = black_magic_read::read_from_db(conn, &get_data_in)?;
            let row = rows
                .into_iter()
                .next()
                .ok_or_else(|| DbError::IllegalInput(format!("No row found for id {}", row_id)))?;

            let col = row.cols.into_iter().next().ok_or_else(|| {
                DbError::IllegalInput(format!("No column value for id {}", row_id))
            })?;

            Ok(col)
        };

        let value1 = unwrap_or_bail!(get_value(&input.row_id_1));
        let value2 = unwrap_or_bail!(get_value(&input.row_id_2));

        unwrap_or_bail!(black_magic::edit_col_in_row(
            conn,
            &input.table_name,
            &input.row_id_1,
            &input.column,
            &value2
        ));

        unwrap_or_bail!(black_magic::edit_col_in_row(
            conn,
            &input.table_name,
            &input.row_id_2,
            &input.column,
            &value1
        ));

        ok_serialized()
    }

    pub fn create_index(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CreateIndexIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        unwrap_or_bail!(black_magic::create_index(
            conn,
            &input.table_name,
            &input.column_name
        ));
        ok_serialized()
    }

    pub fn check_index(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CheckIndexIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        let is_indexed = unwrap_or_bail!(black_magic::check_index(
            conn,
            &input.table_name,
            &input.column_name
        ));
        CheckIndexOut { is_indexed }.to_payload()
    }

    pub fn add_column(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(AddColumnIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        unwrap_or_bail!(black_magic::add_column(
            conn,
            &input.table_name,
            input.column
        ));
        ok_serialized()
    }

    pub fn remove_column(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(RemoveColumnIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        unwrap_or_bail!(black_magic::remove_column(
            conn,
            &input.table_name,
            &input.column_name
        ));
        ok_serialized()
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

    pub fn export_tables(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(ExportTablesIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());

        let mut tables = Vec::new();

        for table_name in input.table_names {
            let columns = unwrap_or_bail!(black_magic::table_shape(conn, &table_name));

            let get_in = GetDataIn {
                table_name: table_name.clone(),
                arguments: vec![SelectArgument::All],
                columns_to_read: Vec::new(),
            };

            let rows = unwrap_or_bail!(black_magic_read::read_from_db(conn, &get_in));

            tables.push(TableExport {
                table_name,
                columns,
                rows,
            });
        }

        ExportTablesOut { tables }.to_payload()
    }

    pub fn create_table_from_export(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CreateTableFromExportIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        unwrap_or_bail!(black_magic::create_table_from_export(
            conn,
            &input.table_name,
            &input.table
        ));
        ok_serialized()
    }

    pub fn copy_table(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CopyTableIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        unwrap_or_bail!(black_magic::copy_table(
            conn,
            &input.source_table_name,
            &input.new_table_name
        ));
        ok_serialized()
    }

    pub fn close_conn(&mut self) -> Vec<u8> {
        if let Some(conn) = self.db_conn.take() {
            if let Err((_, err)) = conn.close() {
                return DbError::ConnError(format!("Failed to close connection: {}", err))
                    .to_payload();
            }
        }

        if let Some(util) = self.sahpool_util.take() {
            if let Err(e) = util.pause_vfs() {
                return DbError::ConnError(format!("Failed to pause VFS: {}", e)).to_payload();
            }
        }

        ok_serialized()
    }

    fn conn(&self) -> Result<&rusqlite::Connection, DbError> {
        self.db_conn
            .as_ref()
            .ok_or_else(|| DbError::ConnError("Database not connected".to_string()))
    }

    pub fn create_fts5_table(&self, data: Vec<u8>) -> Vec<u8> {
        let payload_in = unwrap_or_bail!(CreateFts5TableIn::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        let source_table_name = &payload_in.source_table_name;
        let columns = payload_in.columns;
        match fts5_magic::create_fts5_table(conn, source_table_name, columns) {
            Ok(_) => ok_serialized(),
            Err(e) => e.to_payload(),
        }
    }

    pub fn search_fts5(&self, data: Vec<u8>) -> Vec<u8> {
        // pub fn search_fts5(conn: &Connection, table_name: &str, query: &str) -> Result<Vec<Row>, DbError>
        let payload_in = unwrap_or_bail!(SearchFts5In::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        let query = &payload_in.text_to_lookup;
        let table_name = &payload_in.table_name;
        let result = unwrap_or_bail!(fts5_magic::search_fts5(conn, table_name, query));

        return SearchFts5Out { rows: result }.to_payload();
    }

    pub fn rebuild_fts5_index(&self, data: Vec<u8>) -> Vec<u8> {
        let payload_in = unwrap_or_bail!(RebuildFts5In::un_payloadify(&data));
        let conn = unwrap_or_bail!(self.conn());
        let table_name = &payload_in.table_name;
        match fts5_magic::rebuild_fts5_index(conn, table_name) {
            Ok(_) => ok_serialized(),
            Err(e) => e.to_payload(),
        }
    }
}
