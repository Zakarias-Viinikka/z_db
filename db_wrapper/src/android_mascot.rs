use db::black_magic;
use db::black_magic_read;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::row_col;
use protocol::serialization::*;

pub struct LiveForever {
    pub db_conn: rusqlite::Connection,
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
    pub fn new(sqlitedb_path: &str) -> Result<LiveForever, String> {
        let db_conn = rusqlite::Connection::open(sqlitedb_path).map_err(|e| e.to_string())?;
        Ok(LiveForever { db_conn })
    }

    pub fn create_table(&self, data: Vec<u8>) -> Vec<u8> {
        let data = unwrap_or_bail!(CreateTableIn::un_payloadify(&data));
        let (table_name, columns) = (data.table_name, data.columns);
        let result = black_magic::create_table(&self.db_conn, &table_name, columns);
        result.to_payload()
    }

    pub fn list_tables(&self) -> Vec<u8> {
        let conn = self.conn();
        let list_of_table_names = unwrap_or_bail!(black_magic::list_tables(conn));
        ListTablesOut {
            table_names: list_of_table_names,
        }
        .to_payload()
    }

    pub fn get_data(&self, data: Vec<u8>) -> Vec<u8> {
        let get_data_in = unwrap_or_bail!(GetDataIn::un_payloadify(&data));
        let conn = self.conn();
        let result = black_magic_read::read_from_db(conn, &get_data_in);
        match result {
            Ok(result) => GetDataOut { rows: result }.to_payload(),
            Err(e) => e.to_payload(),
        }
    }

    pub fn get_data_ordered(&self, data: Vec<u8>) -> Vec<u8> {
        let get_data_ordered_in = unwrap_or_bail!(GetDataOrderedIn::un_payloadify(&data));
        let conn = self.conn();
        let result = black_magic_read::read_from_db_ordered(conn, &get_data_ordered_in);
        match result {
            Ok(rows) => GetDataOut { rows }.to_payload(),
            Err(e) => e.to_payload(),
        }
    }

    pub fn insert_data(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(InsertDataIn::un_payloadify(&data));
        let conn = self.conn();
        let result = black_magic::insert_into_table(conn, &input.table_name, input.values);
        InsertDataOut {
            result: if let Err(e) = result { Some(e) } else { None },
        }
        .to_payload()
    }

    pub fn drop_table(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(DropTableIn::un_payloadify(&data));
        let conn = self.conn();
        match black_magic::drop_table(conn, &input.table_name) {
            Ok(()) => ok_serialized(),
            Err(e) => e.to_payload(),
        }
    }

    pub fn edit_col_in_row(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(EditColInRowIn::un_payloadify(&data));
        let conn = self.conn();
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
        let conn = self.conn();
        let result = black_magic::table_shape(conn, &input.table_name);
        let columns = unwrap_or_bail!(result);
        CheckTableOut { columns }.to_payload()
    }

    pub fn delete_row(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(DeleteRowIn::un_payloadify(&data));
        let conn = self.conn();
        unwrap_or_bail!(black_magic::delete_row(
            conn,
            &input.table_name,
            &input.row_id
        ));
        ok_serialized()
    }

    pub fn swap_columns(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(SwapColumnsIn::un_payloadify(&data));
        let conn = self.conn();

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
        let conn = self.conn();
        unwrap_or_bail!(black_magic::create_index(
            conn,
            &input.table_name,
            &input.column_name
        ));
        ok_serialized()
    }

    pub fn check_index(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CheckIndexIn::un_payloadify(&data));
        let conn = self.conn();
        let is_indexed = unwrap_or_bail!(black_magic::check_index(
            conn,
            &input.table_name,
            &input.column_name
        ));
        CheckIndexOut { is_indexed }.to_payload()
    }

    pub fn add_column(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(AddColumnIn::un_payloadify(&data));
        let conn = self.conn();
        unwrap_or_bail!(black_magic::add_column(
            conn,
            &input.table_name,
            input.column
        ));
        ok_serialized()
    }

    pub fn remove_column(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(RemoveColumnIn::un_payloadify(&data));
        let conn = self.conn();
        unwrap_or_bail!(black_magic::remove_column(
            conn,
            &input.table_name,
            &input.column_name
        ));
        ok_serialized()
    }

    pub fn export_database(&self, _data: Vec<u8>) -> Vec<u8> {
        let bytes = unwrap_or_bail!(black_magic::export_database(&self.db_conn));
        ExportDatabaseOut { data: bytes }.to_payload()
    }

    pub fn export_tables(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(ExportTablesIn::un_payloadify(&data));
        let conn = self.conn();

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
        let conn = self.conn();
        unwrap_or_bail!(black_magic::create_table_from_export(
            conn,
            &input.table_name,
            &input.table
        ));
        ok_serialized()
    }

    pub fn copy_table(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CopyTableIn::un_payloadify(&data));
        let conn = self.conn();
        unwrap_or_bail!(black_magic::copy_table(
            conn,
            &input.source_table_name,
            &input.new_table_name
        ));
        ok_serialized()
    }

    fn conn(&self) -> &rusqlite::Connection {
        &self.db_conn
    }
}
