use protocol::new_table::ColumnDef;
use protocol::payload::SelectArgument;
use protocol::row_col;

// Builds CREATE TABLE SQL from a caller-supplied column list (replaces the old Table/Column version).
pub fn generate_create_table_sql(table_name: &str, columns: &[ColumnDef]) -> String {
    let mut col_defs = Vec::new();
    for col in columns {
        let mut def = format!("{} {}", quote_ident(&col.name), col.column_type);
        if col.primary_key {
            def.push_str(" PRIMARY KEY");
        }
        if col.autoincrement {
            def.push_str(" AUTOINCREMENT");
        }
        if col.not_null {
            def.push_str(" NOT NULL");
        }
        if col.unique {
            def.push_str(" UNIQUE");
        }
        if !col.default_value.is_empty() {
            def.push_str(&format!(" DEFAULT {}", col.default_value));
        }
        col_defs.push(def);
    }
    format!(
        "CREATE TABLE IF NOT EXISTS {} ({});",
        quote_ident(table_name),
        col_defs.join(", ")
    )
}

pub fn generate_add_column_sql(table_name: &str, column: &ColumnDef) -> String {
    let mut def = format!("{} {}", quote_ident(&column.name), column.column_type);
    if column.primary_key {
        def.push_str(" PRIMARY KEY");
    }
    if column.autoincrement {
        def.push_str(" AUTOINCREMENT");
    }
    if column.not_null {
        def.push_str(" NOT NULL");
    }
    if column.unique {
        def.push_str(" UNIQUE");
    }
    if !column.default_value.is_empty() {
        def.push_str(&format!(" DEFAULT {}", column.default_value));
    }

    format!(
        "ALTER TABLE {} ADD COLUMN {};",
        quote_ident(table_name),
        def
    )
}

pub fn generate_drop_column_sql(table_name: &str, column_name: &str) -> String {
    format!(
        "ALTER TABLE {} DROP COLUMN {};",
        quote_ident(table_name),
        quote_ident(column_name)
    )
}

// Builds INSERT SQL from (column, value) pairs - replaces the old positional
// Table/Vec<String> version. Order comes from the pairs themselves, not two
// separately-ordered lists, so columns and values can't drift apart.
pub fn generate_insert_sql(table_name: &str, values: Vec<(String, row_col::Col)>) -> String {
    let columns: Vec<String> = values.iter().map(|(col, _)| quote_ident(col)).collect();

    let quoted_values: Vec<String> = values
        .iter()
        .map(|(_, value)| col_to_sql_literal(value))
        .collect();

    format!(
        "INSERT INTO {} ({}) VALUES ({});",
        quote_ident(table_name),
        columns.join(", "),
        quoted_values.join(", ")
    )
}

pub fn generate_delete_sql(table_name: &str, id: &str) -> String {
    format!("DELETE FROM {} WHERE id = {};", quote_ident(table_name), id)
}

pub fn generate_update_sql_typed(
    table_name: &str,
    id: usize,
    column: &str,
    new_value: &row_col::Col,
) -> String {
    let quoted_table = quote_ident(table_name);
    let quoted_column = quote_ident(column);
    let value_literal = col_to_sql_literal(new_value);

    let col_and_val = format!("{} = {}", quoted_column, value_literal);

    format!(
        "UPDATE {table} SET {col_and_val} WHERE id = {id};",
        table = quoted_table,
        col_and_val = col_and_val,
        id = id
    )
}

pub fn generate_read_from_table_sql(
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
) -> String {
    let valid_columns: Vec<&str> = columns_to_read
        .iter()
        .filter(|c| !c.as_ref().is_empty())
        .map(|c| c.as_ref())
        .collect();

    let columns = if valid_columns.is_empty() {
        "*".to_string()
    } else {
        valid_columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let valid_conditions: Vec<&str> = arguments
        .iter()
        .filter(|a| !a.as_ref().is_empty())
        .map(|a| a.as_ref())
        .collect();

    let where_clause = if valid_conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", valid_conditions.join(" AND "))
    };

    format!(
        "SELECT {} FROM {}{};",
        columns,
        quote_ident(table_name.as_ref()),
        where_clause
    )
}

pub fn generate_get_data_by_order_sql(
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
    order_by: &str,
) -> String {
    let valid_columns: Vec<&str> = columns_to_read
        .iter()
        .filter(|c| !c.as_ref().is_empty())
        .map(|c| c.as_ref())
        .collect();

    let columns = if valid_columns.is_empty() {
        "*".to_string()
    } else {
        valid_columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let valid_conditions: Vec<&str> = arguments
        .iter()
        .filter(|a| !a.as_ref().is_empty())
        .map(|a| a.as_ref())
        .collect();

    let where_clause = if valid_conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", valid_conditions.join(" AND "))
    };

    format!(
        "SELECT {} FROM {}{} ORDER BY {};",
        columns,
        quote_ident(table_name.as_ref()),
        where_clause,
        order_by
    )
}

fn col_to_sql_literal(value: &row_col::Col) -> String {
    match value {
        row_col::Col::Null => "NULL".to_string(),
        row_col::Col::Integer(i) => i.to_string(),
        row_col::Col::Real(f) => f.to_string(),
        row_col::Col::Text(s) => format!("'{}'", sanitize(s)),
        row_col::Col::Blob(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            format!("X'{}'", hex)
        }
    }
}

fn sanitize(input: &str) -> String {
    input.replace("'", "''")
}

pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

pub fn quote_sql_string(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

pub trait ToSqlCondition {
    fn to_sql_condition(&self) -> String;
}

impl ToSqlCondition for SelectArgument {
    fn to_sql_condition(&self) -> String {
        match self {
            SelectArgument::XEqualY { x, y } => {
                format!("{} = {}", quote_ident(x), quote_sql_string(y))
            }
            SelectArgument::XNotEqualY { x, y } => {
                format!("{} != {}", quote_ident(x), quote_sql_string(y))
            }
            SelectArgument::XGreaterThanY { x, y } => {
                format!("{} > {}", quote_ident(x), quote_sql_string(y))
            }
            SelectArgument::XLessThanY { x, y } => {
                format!("{} < {}", quote_ident(x), quote_sql_string(y))
            }
            SelectArgument::XGreaterThanOrEqualY { x, y } => {
                format!("{} >= {}", quote_ident(x), quote_sql_string(y))
            }
            SelectArgument::XLessThanOrEqualY { x, y } => {
                format!("{} <= {}", quote_ident(x), quote_sql_string(y))
            }
            SelectArgument::XLikeY { x, y } => {
                format!("{} LIKE {}", quote_ident(x), quote_sql_string(y))
            }
            SelectArgument::XInY { x, y } => {
                let quoted_values: Vec<String> = y.iter().map(|v| quote_sql_string(v)).collect();
                format!("{} IN ({})", quote_ident(x), quoted_values.join(", "))
            }
            SelectArgument::All => String::new(),
        }
    }
}

//---
// fts5
//---
//both are untested for now
pub fn create_fts5_sql_builder(source_table_name: &str, columns: &[String]) -> String {
    let fts_table_name = format!("fts5_{}", source_table_name);
    let quoted_table = quote_ident(&fts_table_name);
    let quoted_source = quote_ident(source_table_name);

    let quoted_columns: Vec<String> = columns.iter().map(|c| quote_ident(c)).collect();
    let columns_joined = quoted_columns.join(", ");

    format!(
        "CREATE VIRTUAL TABLE {table} USING fts5({columns}, content={source}, content_rowid='id');",
        table = quoted_table,
        columns = columns_joined,
        source = quoted_source
    )
}

pub fn search_fts5_sql_builder(source_table_name: &str, text_to_lookup: &str) -> String {
    let fts_table_name = quote_ident(&format!("fts5_{}", source_table_name));
    let prefix_query = format!("{}*", text_to_lookup);
    let quoted_query = quote_sql_string(&prefix_query);

    format!(
        "SELECT rowid, * FROM {table} WHERE {table} MATCH {query};",
        table = fts_table_name,
        query = quoted_query
    )
}
