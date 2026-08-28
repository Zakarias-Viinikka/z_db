#[cfg(test)]
mod tests {
    use sql_builder::search_fts5_sql_builder;

    #[test]
    fn search_fts5_single_word() {
        let sql = search_fts5_sql_builder("users", "alice");
        assert_eq!(
            sql,
            r#"SELECT rowid, * FROM "fts5_users" WHERE "fts5_users" MATCH 'alice' ORDER BY rank;"#
        );
    }

    #[test]
    fn search_fts5_multiple_words() {
        let sql = search_fts5_sql_builder("users", "alice smith");
        assert_eq!(
            sql,
            r#"SELECT rowid, * FROM "fts5_users" WHERE "fts5_users" MATCH 'alice smith' ORDER BY rank;"#
        );
    }

    #[test]
    fn search_fts5_extra_whitespace() {
        let sql = search_fts5_sql_builder("users", "  alice   smith  ");
        assert_eq!(
            sql,
            r#"SELECT rowid, * FROM "fts5_users" WHERE "fts5_users" MATCH '  alice   smith  ' ORDER BY rank;"#
        );
    }

    #[test]
    fn search_fts5_escapes_double_quotes() {
        let sql = search_fts5_sql_builder("users", r#"alice "smith""#);
        assert_eq!(
            sql,
            r#"SELECT rowid, * FROM "fts5_users" WHERE "fts5_users" MATCH 'alice "smith"' ORDER BY rank;"#
        );
    }

    #[test]
    fn search_fts5_escapes_sql_single_quotes() {
        let sql = search_fts5_sql_builder("users", "alice's");
        assert_eq!(
            sql,
            r#"SELECT rowid, * FROM "fts5_users" WHERE "fts5_users" MATCH 'alice''s' ORDER BY rank;"#
        );
    }

    #[test]
    fn search_fts5_empty_query() {
        let sql = search_fts5_sql_builder("users", "");
        assert_eq!(
            sql,
            r#"SELECT rowid, * FROM "fts5_users" WHERE "fts5_users" MATCH '' ORDER BY rank;"#
        );
    }
}
