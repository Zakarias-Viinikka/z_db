#[cfg(test)]
mod tests {
    use sql_builder::{fts5_delete_sql_builder, fts5_insert_sql_builder};

    #[test]
    fn value_with_single_quote_is_escaped() {
        let insert_result = fts5_insert_sql_builder("users", "1", "name", "alice's");
        let insert_expected = r#"INSERT INTO "fts5_users"("rowid", "name") VALUES(1, 'alice''s');"#;
        assert_eq!(insert_result, insert_expected);

        let delete_result = fts5_delete_sql_builder("users", "1", "name", "alice's");
        let delete_expected = r#"INSERT INTO "fts5_users"("fts5_users", "rowid", "name") VALUES('delete', 1, 'alice''s');"#;
        assert_eq!(delete_result, delete_expected);
    }

    #[test]
    fn column_name_with_double_quote_is_escaped() {
        let insert_result = fts5_insert_sql_builder("users", "1", r#"weird"col"#, "x");
        let insert_expected = r#"INSERT INTO "fts5_users"("rowid", "weird""col") VALUES(1, 'x');"#;
        assert_eq!(insert_result, insert_expected);

        let delete_result = fts5_delete_sql_builder("users", "1", r#"weird"col"#, "x");
        let delete_expected = r#"INSERT INTO "fts5_users"("fts5_users", "rowid", "weird""col") VALUES('delete', 1, 'x');"#;
        assert_eq!(delete_result, delete_expected);
    }

    #[test]
    fn source_table_name_with_double_quote_is_escaped() {
        let insert_result = fts5_insert_sql_builder(r#"weird"table"#, "1", "name", "x");
        let insert_expected = r#"INSERT INTO "fts5_weird""table"("rowid", "name") VALUES(1, 'x');"#;
        assert_eq!(insert_result, insert_expected);

        let delete_result = fts5_delete_sql_builder(r#"weird"table"#, "1", "name", "x");
        let delete_expected = r#"INSERT INTO "fts5_weird""table"("fts5_weird""table", "rowid", "name") VALUES('delete', 1, 'x');"#;
        assert_eq!(delete_result, delete_expected);
    }

    #[test]
    fn everything_escapes_at_once() {
        let source_table = r#"a"b"#;
        let column = r#"c"d"#;
        let value = "e'f";

        let insert_result = fts5_insert_sql_builder(source_table, "1", column, value);
        let insert_expected = r#"INSERT INTO "fts5_a""b"("rowid", "c""d") VALUES(1, 'e''f');"#;
        assert_eq!(insert_result, insert_expected);

        let delete_result = fts5_delete_sql_builder(source_table, "1", column, value);
        let delete_expected =
            r#"INSERT INTO "fts5_a""b"("fts5_a""b", "rowid", "c""d") VALUES('delete', 1, 'e''f');"#;
        assert_eq!(delete_result, delete_expected);
    }
}

