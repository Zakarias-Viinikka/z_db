use db::black_magic;
use db::black_magic_read;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::row_col;
use std::sync::Mutex;

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

    pub fn create_table(&self, data: CreateTableIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        match black_magic::create_table(&conn, &data.table_name, data.columns) {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    pub fn create_foreign_table(&self, payload: CreateForeignTableIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        match black_magic::create_foreign_table(&conn, &payload) {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    pub fn list_tables(&self) -> Result<ListTablesOut, DbError> {
        let conn = self.conn()?;
        let table_names = black_magic::list_tables(&conn)?;
        Ok(ListTablesOut { table_names })
    }

    pub fn get_data(&self, data: GetDataIn) -> Result<GetDataOut, DbError> {
        let conn = self.conn()?;
        let rows = black_magic_read::read_from_db(&conn, &data)?;
        Ok(GetDataOut { rows })
    }

    pub fn get_data_ordered(&self, data: GetDataOrderedIn) -> Result<GetDataOut, DbError> {
        let conn = self.conn()?;
        let rows = black_magic_read::read_from_db_ordered(&conn, &data)?;
        Ok(GetDataOut { rows })
    }

    pub fn insert_data(&self, data: InsertDataIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::insert_into_table(&conn, &data.table_name, data.values)
    }

    pub fn drop_table(&self, data: DropTableIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::drop_table(&conn, &data.table_name)
    }

    pub fn edit_col_in_row(&self, data: EditColInRowIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::edit_col_in_row(
            &conn,
            &data.table_name,
            &data.row_id,
            &data.column,
            &data.new_value,
        )
    }

    pub fn check_table(&self, data: CheckTableIn) -> Result<CheckTableOut, DbError> {
        let conn = self.conn()?;
        let columns = black_magic::table_shape(&conn, &data.table_name)?;
        Ok(CheckTableOut { columns })
    }

    pub fn delete_row(&self, data: DeleteRowIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::delete_row(&conn, &data.table_name, &data.row_id)
    }

    pub fn swap_columns(&self, data: SwapColumnsIn) -> Result<(), DbError> {
        let conn = self.conn()?;

        let get_value = |row_id: &str| -> Result<row_col::Col, DbError> {
            let get_data_in = GetDataIn {
                table_name: data.table_name.clone(),
                arguments: vec![SelectArgument::XEqualY {
                    x: "id".to_string(),
                    y: row_id.to_string(),
                }],
                columns_to_read: vec![data.column.clone()],
            };

            let rows = black_magic_read::read_from_db(&conn, &get_data_in)?;
            let row = rows
                .into_iter()
                .next()
                .ok_or_else(|| DbError::IllegalInput(format!("No row found for id {}", row_id)))?;

            let col = row.cols.into_iter().next().ok_or_else(|| {
                DbError::IllegalInput(format!("No column value for id {}", row_id))
            })?;

            Ok(col)
        };

        let value1 = get_value(&data.row_id_1)?;
        let value2 = get_value(&data.row_id_2)?;

        black_magic::edit_col_in_row(
            &conn,
            &data.table_name,
            &data.row_id_1,
            &data.column,
            &value2,
        )?;

        black_magic::edit_col_in_row(
            &conn,
            &data.table_name,
            &data.row_id_2,
            &data.column,
            &value1,
        )?;

        Ok(())
    }

    pub fn create_index(&self, data: CreateIndexIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::create_index(&conn, &data.table_name, &data.column_name)
    }

    pub fn check_index(&self, data: CheckIndexIn) -> Result<CheckIndexOut, DbError> {
        let conn = self.conn()?;
        let is_indexed = black_magic::check_index(&conn, &data.table_name, &data.column_name)?;
        Ok(CheckIndexOut { is_indexed })
    }

    pub fn add_column(&self, data: AddColumnIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::add_column(&conn, &data.table_name, data.column)
    }

    pub fn remove_column(&self, data: RemoveColumnIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::remove_column(&conn, &data.table_name, &data.column_name)
    }

    pub fn export_database(&self) -> Result<ExportDatabaseOut, DbError> {
        let conn = self.conn()?;
        let bytes = black_magic::export_database(&conn)?;
        Ok(ExportDatabaseOut { data: bytes })
    }

    pub fn export_tables(&self, data: ExportTablesIn) -> Result<ExportTablesOut, DbError> {
        let conn = self.conn()?;
        let mut tables = Vec::new();

        for table_name in data.table_names {
            let columns = black_magic::table_shape(&conn, &table_name)?;

            let get_in = GetDataIn {
                table_name: table_name.clone(),
                arguments: vec![SelectArgument::All],
                columns_to_read: Vec::new(),
            };

            let rows = black_magic_read::read_from_db(&conn, &get_in)?;

            tables.push(TableExport {
                table_name,
                columns,
                rows,
            });
        }

        Ok(ExportTablesOut { tables })
    }

    pub fn create_table_from_export(&self, data: CreateTableFromExportIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::create_table_from_export(&conn, &data.table_name, &data.table)
    }

    pub fn copy_table(&self, data: CopyTableIn) -> Result<(), DbError> {
        let conn = self.conn()?;
        black_magic::copy_table(&conn, &data.source_table_name, &data.new_table_name)
    }
}

impl LiveForever {
    fn conn(&self) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, DbError> {
        self.db_conn
            .lock()
            .map_err(|e| DbError::ConnError(format!("failed to lock db connection: {e}")))
    }
}
