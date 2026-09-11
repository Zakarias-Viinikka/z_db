use db::black_magic;
use db::black_magic_extension;
use db::black_magic_read;
use db::safety_first;
use protocol::error::DbError;
use protocol::payload::*;

pub struct LiveForever {
    pub db_conn: rusqlite::Connection,
}

impl LiveForever {
    pub fn new() -> Result<LiveForever, DbError> {
        let db_conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| DbError::ConnError(e.to_string()))?;
        Ok(LiveForever { db_conn })
    }

    fn conn(&self) -> &rusqlite::Connection {
        &self.db_conn
    }

    pub fn create_table(&self, data: CreateTableIn) -> Result<(), DbError> {
        match black_magic::create_table(self.conn(), &data.table_name, data.columns) {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    pub fn list_tables(&self) -> Result<ListTablesOut, DbError> {
        let table_names = black_magic::list_tables(self.conn())?;
        Ok(ListTablesOut { table_names })
    }

    pub fn get_data(&self, data: GetDataIn) -> Result<GetDataOut, DbError> {
        let rows = black_magic_read::read_from_db(self.conn(), &data)?;
        Ok(GetDataOut { rows })
    }

    pub fn insert_data(&self, data: InsertDataIn) -> Result<(), DbError> {
        black_magic::insert_into_table(self.conn(), &data.table_name, data.values)
    }

    pub fn delete_row(&self, data: DeleteRowIn) -> Result<(), DbError> {
        black_magic::delete_row(self.conn(), &data.table_name, &data.row_id)
    }

    pub fn edit_col_in_row(&self, data: EditColInRowIn) -> Result<(), DbError> {
        black_magic::edit_col_in_row(
            self.conn(),
            &data.table_name,
            &data.row_id,
            &data.column,
            &data.new_value,
        )
    }

    // transaction API
    pub fn begin_all_or_nothing(&self) -> Result<(), DbError> {
        safety_first::begin_all_or_nothing(self.conn())
    }

    pub fn everything_went_perfectly(&self) -> Result<(), DbError> {
        safety_first::everything_went_perfectly(self.conn())
    }

    pub fn regret_everything(&self) -> Result<(), DbError> {
        safety_first::regret_everything(self.conn())
    }

    pub fn count_all_rows(&self, data: CountAllRowsIn) -> Result<CountAllRowsOut, DbError> {
        black_magic_extension::count_all_rows(self.conn(), &data)
    }
}
