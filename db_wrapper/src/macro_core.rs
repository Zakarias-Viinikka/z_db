#[macro_export]
macro_rules! create_the_entire_universe {
    ($type:ty $(, $attr:meta)*) => {
        $(#[$attr])*
        impl $type {
            pub fn create_table(
                &self,
                data: method_input_type!(CreateTableIn),
            ) -> method_return_type!(()) {
                let input: CreateTableIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::create_table(
                    &*conn,
                    &input.table_name,
                    input.columns,
                ));
                return_nothing!()
            }

            pub fn list_tables(&self) -> method_return_type!(ListTablesOut) {
                let conn = unwrap_or_bail!(self.get_conn());
                let table_names = unwrap_or_bail!(::db::black_magic::list_tables(&*conn));
                finish_output!(ListTablesOut { table_names })
            }

            pub fn get_data(
                &self,
                data: method_input_type!(GetDataIn),
            ) -> method_return_type!(GetDataOut) {
                let input: GetDataIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                let rows = unwrap_or_bail!(::db::black_magic_read::read_from_db(&*conn, &input));
                finish_output!(GetDataOut { rows })
            }

            pub fn get_data_ordered(
                &self,
                data: method_input_type!(GetDataOrderedIn),
            ) -> method_return_type!(GetDataOut) {
                let input: GetDataOrderedIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                let rows = unwrap_or_bail!(::db::black_magic_read::read_from_db_ordered(&*conn, &input));
                finish_output!(GetDataOut { rows })
            }

            pub fn insert_data(
                &self,
                data: method_input_type!(InsertDataIn),
            ) -> method_return_type!(()) {
                let input: InsertDataIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::insert_into_table(
                    &*conn,
                    &input.table_name,
                    input.values,
                ));
                return_nothing!()
            }

            pub fn drop_table(
                &self,
                data: method_input_type!(DropTableIn),
            ) -> method_return_type!(()) {
                let input: DropTableIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::drop_table(&*conn, &input.table_name));
                return_nothing!()
            }

            pub fn delete_row(
                &self,
                data: method_input_type!(DeleteRowIn),
            ) -> method_return_type!(()) {
                let input: DeleteRowIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::delete_row(
                    &*conn,
                    &input.table_name,
                    &input.row_id,
                ));
                return_nothing!()
            }

            pub fn edit_col_in_row(
                &self,
                data: method_input_type!(EditColInRowIn),
            ) -> method_return_type!(()) {
                let input: EditColInRowIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::edit_col_in_row(
                    &*conn,
                    &input.table_name,
                    &input.row_id,
                    &input.column,
                    &input.new_value,
                ));
                return_nothing!()
            }

            pub fn check_table(
                &self,
                data: method_input_type!(CheckTableIn),
            ) -> method_return_type!(CheckTableOut) {
                let input: CheckTableIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                let columns = unwrap_or_bail!(::db::black_magic::table_shape(&*conn, &input.table_name));
                finish_output!(CheckTableOut { columns })
            }

            pub fn swap_columns(
                &self,
                data: method_input_type!(SwapColumnsIn),
            ) -> method_return_type!(()) {
                let input: SwapColumnsIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());

                let get_value = |row_id: &str| -> Result<::protocol::row_col::Col, ::protocol::error::DbError> {
                    let get_data_in = GetDataIn {
                        table_name: input.table_name.clone(),
                        arguments: vec![SelectArgument::XEqualY {
                            x: "id".to_string(),
                            y: row_id.to_string(),
                            join: None,
                        }],
                        columns_to_read: vec![input.column.clone()],
                    };

                    let rows = ::db::black_magic_read::read_from_db(&*conn, &get_data_in)?;
                    let row = rows
                        .into_iter()
                        .next()
                        .ok_or_else(|| ::protocol::error::DbError::IllegalInput(format!("No row found for id {}", row_id)))?;

                    let col = row.cols.into_iter().next().ok_or_else(|| {
                        ::protocol::error::DbError::IllegalInput(format!("No column value for id {}", row_id))
                    })?;

                    Ok(col)
                };

                let value1 = unwrap_or_bail!(get_value(&input.row_id_1));
                let value2 = unwrap_or_bail!(get_value(&input.row_id_2));

                unwrap_or_bail!(::db::black_magic::edit_col_in_row(
                    &*conn,
                    &input.table_name,
                    &input.row_id_1,
                    &input.column,
                    &value2,
                ));

                unwrap_or_bail!(::db::black_magic::edit_col_in_row(
                    &*conn,
                    &input.table_name,
                    &input.row_id_2,
                    &input.column,
                    &value1,
                ));

                return_nothing!()
            }

            pub fn create_index(
                &self,
                data: method_input_type!(CreateIndexIn),
            ) -> method_return_type!(()) {
                let input: CreateIndexIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::create_index(
                    &*conn,
                    &input.table_name,
                    &input.column_name,
                ));
                return_nothing!()
            }

            pub fn check_index(
                &self,
                data: method_input_type!(CheckIndexIn),
            ) -> method_return_type!(CheckIndexOut) {
                let input: CheckIndexIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                let is_indexed = unwrap_or_bail!(::db::black_magic::check_index(
                    &*conn,
                    &input.table_name,
                    &input.column_name,
                ));
                finish_output!(CheckIndexOut { is_indexed })
            }

            pub fn add_column(
                &self,
                data: method_input_type!(AddColumnIn),
            ) -> method_return_type!(()) {
                let input: AddColumnIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::add_column(
                    &*conn,
                    &input.table_name,
                    input.column,
                ));
                return_nothing!()
            }

            pub fn remove_column(
                &self,
                data: method_input_type!(RemoveColumnIn),
            ) -> method_return_type!(()) {
                let input: RemoveColumnIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::remove_column(
                    &*conn,
                    &input.table_name,
                    &input.column_name,
                ));
                return_nothing!()
            }

            pub fn export_tables(
                &self,
                data: method_input_type!(ExportTablesIn),
            ) -> method_return_type!(ExportTablesOut) {
                let input: ExportTablesIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());

                let mut tables = Vec::new();

                for table_name in input.table_names {
                    let columns = unwrap_or_bail!(::db::black_magic::table_shape(&*conn, &table_name));

                    let get_in = GetDataIn {
                        table_name: table_name.clone(),
                        arguments: vec![SelectArgument::All],
                        columns_to_read: Vec::new(),
                    };

                    let rows = unwrap_or_bail!(::db::black_magic_read::read_from_db(&*conn, &get_in));

                    tables.push(TableExport { table_name, columns, rows });
                }

                finish_output!(ExportTablesOut { tables })
            }

            pub fn create_table_from_export(
                &self,
                data: method_input_type!(CreateTableFromExportIn),
            ) -> method_return_type!(()) {
                let input: CreateTableFromExportIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::create_table_from_export(
                    &*conn,
                    &input.table_name,
                    &input.table,
                ));
                return_nothing!()
            }

            pub fn copy_table(
                &self,
                data: method_input_type!(CopyTableIn),
            ) -> method_return_type!(()) {
                let input: CopyTableIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::copy_table(
                    &*conn,
                    &input.source_table_name,
                    &input.new_table_name,
                ));
                return_nothing!()
            }

            pub fn begin_all_or_nothing(&self) -> method_return_type!(()) {
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::safety_first::begin_all_or_nothing(&*conn));
                return_nothing!()
            }

            pub fn everything_went_perfectly(&self) -> method_return_type!(()) {
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::safety_first::everything_went_perfectly(&*conn));
                return_nothing!()
            }

            pub fn regret_everything(&self) -> method_return_type!(()) {
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::safety_first::regret_everything(&*conn));
                return_nothing!()
            }

            pub fn count_all_rows(
                &self,
                data: method_input_type!(CountAllRowsIn),
            ) -> method_return_type!(CountAllRowsOut) {
                let input: CountAllRowsIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                let out = unwrap_or_bail!(::db::black_magic_extension::count_all_rows(&*conn, &input));
                finish_output!(out)
            }

            pub fn create_foreign_table(
                &self,
                data: method_input_type!(CreateForeignTableIn),
            ) -> method_return_type!(()) {
                let input: CreateForeignTableIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::create_foreign_table(&*conn, &input));
                return_nothing!()
            }

            pub fn create_fts5_table(
                &self,
                data: method_input_type!(CreateFts5TableIn),
            ) -> method_return_type!(()) {
                let input: CreateFts5TableIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::fts5_magic::create_fts5_table(
                    &*conn,
                    &input.source_table_name,
                    input.columns,
                ));
                return_nothing!()
            }

            pub fn search_fts5(
                &self,
                data: method_input_type!(SearchFts5In),
            ) -> method_return_type!(GetDataOut) {
                let input: SearchFts5In = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                let rows = unwrap_or_bail!(::db::fts5_magic::search_fts5(
                    &*conn,
                    &input.table_name,
                    &input.text_to_lookup,
                ));
                finish_output!(GetDataOut { rows })
            }

            pub fn rebuild_fts5_index(
                &self,
                data: method_input_type!(RebuildFts5In),
            ) -> method_return_type!(()) {
                let input: RebuildFts5In = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::fts5_magic::rebuild_fts5_index(&*conn, &input.table_name));
                return_nothing!()
            }

            pub fn force_drop_table(
                &self,
                data: method_input_type!(DropTableIn),
            ) -> method_return_type!(()) {
                let input: DropTableIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                unwrap_or_bail!(::db::black_magic::force_drop_table(&*conn, &input.table_name));
                return_nothing!()
            }
        }
    };
}
