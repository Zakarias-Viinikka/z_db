#[cfg(test)]
mod tests {
    use protocol::new_table::{ColumnDef, ForeignKeyDef};
    use sql_builder::generate_create_foreign_table_sql;

    fn col(name: &str, column_type: &str) -> ColumnDef {
        ColumnDef {
            name: name.to_string(),
            column_type: column_type.to_string(),
            primary_key: false,
            not_null: false,
            unique: false,
            default_value: String::new(),
            autoincrement: false,
        }
    }

    fn fk(column: &str, referenced_table: &str, referenced_column: &str) -> ForeignKeyDef {
        ForeignKeyDef {
            column: column.to_string(),
            referenced_table: referenced_table.to_string(),
            referenced_column: referenced_column.to_string(),
        }
    }

    #[test]
    fn basic_table_no_foreign_keys() {
        let columns = vec![col("id", "INTEGER"), col("name", "TEXT")];
        let sql = generate_create_foreign_table_sql("users", &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "users" ("id" INTEGER, "name" TEXT);"#
        );
    }

    #[test]
    fn column_with_primary_key_and_autoincrement() {
        let columns = vec![ColumnDef {
            primary_key: true,
            autoincrement: true,
            ..col("id", "INTEGER")
        }];
        let sql = generate_create_foreign_table_sql("users", &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "users" ("id" INTEGER PRIMARY KEY AUTOINCREMENT);"#
        );
    }

    #[test]
    fn column_with_not_null_and_unique() {
        let columns = vec![ColumnDef {
            not_null: true,
            unique: true,
            ..col("email", "TEXT")
        }];
        let sql = generate_create_foreign_table_sql("users", &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "users" ("email" TEXT NOT NULL UNIQUE);"#
        );
    }

    #[test]
    fn column_with_default_value() {
        let columns = vec![ColumnDef {
            default_value: "0".to_string(),
            ..col("score", "INTEGER")
        }];
        let sql = generate_create_foreign_table_sql("users", &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "users" ("score" INTEGER DEFAULT 0);"#
        );
    }

    #[test]
    fn column_with_all_optional_attributes_in_order() {
        let columns = vec![ColumnDef {
            primary_key: true,
            autoincrement: true,
            not_null: true,
            unique: true,
            default_value: "1".to_string(),
            ..col("id", "INTEGER")
        }];
        let sql = generate_create_foreign_table_sql("items", &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "items" ("id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE DEFAULT 1);"#
        );
    }

    #[test]
    fn empty_default_value_omits_default_clause() {
        let columns = vec![col("name", "TEXT")];
        let sql = generate_create_foreign_table_sql("users", &columns, &[]);
        assert_eq!(sql, r#"CREATE TABLE IF NOT EXISTS "users" ("name" TEXT);"#);
    }

    #[test]
    fn single_foreign_key() {
        let columns = vec![
            ColumnDef {
                primary_key: true,
                ..col("id", "INTEGER")
            },
            ColumnDef {
                not_null: true,
                ..col("user_id", "INTEGER")
            },
        ];
        let fks = vec![fk("user_id", "users", "id")];
        let sql = generate_create_foreign_table_sql("orders", &columns, &fks);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "orders" ("id" INTEGER PRIMARY KEY, "user_id" INTEGER NOT NULL, FOREIGN KEY ("user_id") REFERENCES "users"("id"));"#
        );
    }

    #[test]
    fn multiple_foreign_keys() {
        let columns = vec![
            ColumnDef {
                primary_key: true,
                ..col("id", "INTEGER")
            },
            col("user_id", "INTEGER"),
            col("product_id", "INTEGER"),
        ];
        let fks = vec![
            fk("user_id", "users", "id"),
            fk("product_id", "products", "id"),
        ];
        let sql = generate_create_foreign_table_sql("order_items", &columns, &fks);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "order_items" ("id" INTEGER PRIMARY KEY, "user_id" INTEGER, "product_id" INTEGER, FOREIGN KEY ("user_id") REFERENCES "users"("id"), FOREIGN KEY ("product_id") REFERENCES "products"("id"));"#
        );
    }

    #[test]
    fn quotes_table_name_with_space() {
        let columns = vec![col("id", "INTEGER")];
        let sql = generate_create_foreign_table_sql("order items", &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "order items" ("id" INTEGER);"#
        );
    }

    #[test]
    fn quotes_column_name_that_is_reserved_word() {
        let columns = vec![col("order", "TEXT")];
        let sql = generate_create_foreign_table_sql("events", &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "events" ("order" TEXT);"#
        );
    }

    #[test]
    fn escapes_double_quotes_in_table_name() {
        let columns = vec![col("id", "INTEGER")];
        let sql = generate_create_foreign_table_sql(r#"my"table"#, &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "my""table" ("id" INTEGER);"#
        );
    }

    #[test]
    fn escapes_double_quotes_in_column_name() {
        let columns = vec![col(r#"na"me"#, "TEXT")];
        let sql = generate_create_foreign_table_sql("users", &columns, &[]);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "users" ("na""me" TEXT);"#
        );
    }

    #[test]
    fn escapes_double_quotes_in_foreign_key_identifiers() {
        let columns = vec![col("user_id", "INTEGER")];
        let fks = vec![fk("user_id", r#"us"ers"#, r#"i"d"#)];
        let sql = generate_create_foreign_table_sql("orders", &columns, &fks);
        assert_eq!(
            sql,
            r#"CREATE TABLE IF NOT EXISTS "orders" ("user_id" INTEGER, FOREIGN KEY ("user_id") REFERENCES "us""ers"("i""d"));"#
        );
    }

    #[test]
    fn empty_foreign_keys_slice_matches_no_fk_output() {
        let columns = vec![col("id", "INTEGER")];
        let with_empty_fks = generate_create_foreign_table_sql("t", &columns, &[]);
        assert_eq!(
            with_empty_fks,
            r#"CREATE TABLE IF NOT EXISTS "t" ("id" INTEGER);"#
        );
    }
}
