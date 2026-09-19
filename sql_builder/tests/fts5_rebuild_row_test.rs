#[cfg(test)]
mod tests {
    use sql_builder::create_fts5_triggers_sql_builder;

    #[test]
    fn creates_insert_update_and_delete_triggers() {
        let sql = create_fts5_triggers_sql_builder("users", "fts5_users", "name");
        let expected = concat!(
            "CREATE TRIGGER \"fts5_users_name_insert\" AFTER INSERT ON \"users\" BEGIN ",
            "INSERT INTO \"fts5_users\"(rowid, \"name\") VALUES (new.rowid, new.\"name\"); END;\n",
            "CREATE TRIGGER \"fts5_users_name_update\" AFTER UPDATE ON \"users\" BEGIN ",
            "INSERT INTO \"fts5_users\"(\"fts5_users\", rowid, \"name\") VALUES ('delete', old.rowid, old.\"name\"); ",
            "INSERT INTO \"fts5_users\"(rowid, \"name\") VALUES (new.rowid, new.\"name\"); END;\n",
            "CREATE TRIGGER \"fts5_users_name_delete\" AFTER DELETE ON \"users\" BEGIN ",
            "INSERT INTO \"fts5_users\"(\"fts5_users\", rowid, \"name\") VALUES ('delete', old.rowid, old.\"name\"); END;"
        );
        assert_eq!(sql, expected);
    }

    #[test]
    fn trigger_identifiers_are_escaped() {
        let sql = create_fts5_triggers_sql_builder(r#"a"b"#, r#"fts5_a"b"#, r#"c"d"#);
        assert!(sql.contains(r#"CREATE TRIGGER \"fts5_a\"\"b_c\"\"d_insert\""#));
        assert!(sql.contains(r#"AFTER INSERT ON \"a\"\"b\""#));
        assert!(sql.contains(r#"INSERT INTO \"fts5_a\"\"b\"(rowid, \"c\"\"d\")"#));
    }
}
