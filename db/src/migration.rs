use protocol::error::DbError;
use protocol::new_table::ColumnDef;
use protocol::payload::{
    AddColumnIn, ColumnSource, FundamentallyEditExistingColIn, RemoveColumnIn,
};
use rusqlite::Connection;
use sql_builder::*;

use rusqlite::OptionalExtension;

use crate::black_magic::table_shape;

macro_rules! start_transaction_if_not_in_one {
    ($conn:expr) => {{
        let started_here = !crate::safety_first::are_we_in_middle_of_transaction($conn);
        if started_here {
            crate::safety_first::begin_all_or_nothing($conn)?;
        }
        started_here
    }};
}

macro_rules! end_transaction_if_in_one {
    ($conn:expr, $started_here:expr, $result:expr) => {
        match $result {
            Ok(v) => {
                if $started_here {
                    crate::safety_first::everything_went_perfectly($conn)?;
                }
                Ok(v)
            }
            Err(e) => {
                if $started_here {
                    let _ = crate::safety_first::regret_everything($conn);
                }
                Err(e)
            }
        }
    };
}

pub fn add_column(conn: &Connection, input: AddColumnIn) -> Result<(), DbError> {
    let table_name = &input.table_name;
    let mut column = input.column;

    if table_name.is_empty() {
        return Err(DbError::IllegalInput("table_name is empty".to_string()));
    }
    if column.primary_key {
        return Err(DbError::IllegalInput(
            "ADD COLUMN does not support PRIMARY KEY".to_string(),
        ));
    }
    if column.autoincrement {
        return Err(DbError::IllegalInput(
            "ADD COLUMN does not support AUTOINCREMENT".to_string(),
        ));
    }
    if column.not_null && column.default_value.is_empty() {
        return Err(DbError::IllegalInput(
            "NOT NULL column requires a DEFAULT value when adding to an existing table".to_string(),
        ));
    }

    let wants_unique = column.unique;
    column.unique = false;

    let started_here = start_transaction_if_not_in_one!(conn);

    let result = (|| -> Result<(), DbError> {
        let sql = generate_add_column_sql(table_name, &column);
        conn.execute(&sql, []).map_err(|e| {
            DbError::SqlExecuteFail(format!("add_column failed: {}, sql: {}", e, sql))
        })?;

        if wants_unique {
            let index_name = quote_ident(&format!("uidx_{}_{}", table_name, column.name));
            let idx_sql = format!(
                "CREATE UNIQUE INDEX IF NOT EXISTS {} ON {} ({});",
                index_name,
                quote_ident(table_name),
                quote_ident(&column.name)
            );
            conn.execute(&idx_sql, []).map_err(|e| {
                DbError::SqlExecuteFail(format!(
                    "add_column: unique index failed: {}, sql: {}",
                    e, idx_sql
                ))
            })?;
        }

        Ok(())
    })();

    end_transaction_if_in_one!(conn, started_here, result)
}

pub fn make_col_disappear(conn: &Connection, input: RemoveColumnIn) -> Result<(), DbError> {
    let table_name = &input.table_name;
    let column_name = &input.column_name;

    if table_name.is_empty() {
        return Err(DbError::IllegalInput("table_name is empty".to_string()));
    }

    let columns = table_shape(conn, table_name)?;
    let column = columns
        .iter()
        .find(|c| c.name == *column_name)
        .ok_or_else(|| {
            DbError::IllegalInput(format!(
                "column '{}' does not exist in table '{}'",
                column_name, table_name
            ))
        })?;

    let needs_rebuild = column.primary_key
        || check_if_this_col_is_indexed(conn, table_name, column_name)?
        || this_is_a_foreign_key_to_some_other_col(conn, table_name, column_name)?;

    if needs_rebuild {
        return remake_entire_table_from_scratch(conn, table_name, column_name);
    }

    let sql = generate_drop_column_sql(table_name, column_name);
    conn.execute(&sql, []).map_err(|e| {
        DbError::SqlExecuteFail(format!("make_col_disappear failed: {}, sql: {}", e, sql))
    })?;

    Ok(())
}

pub fn fundamentally_edit_existing_col(
    conn: &Connection,
    input: FundamentallyEditExistingColIn,
) -> Result<(), DbError> {
    let table_name = &input.table_name;
    let column_name = &input.column_name;

    if table_name.is_empty() {
        return Err(DbError::IllegalInput("table_name is empty".to_string()));
    }

    let columns = table_shape(conn, table_name)?;
    columns
        .iter()
        .find(|c| c.name == *column_name)
        .ok_or_else(|| {
            DbError::IllegalInput(format!(
                "column '{}' does not exist in table '{}'",
                column_name, table_name
            ))
        })?;

    if input.new_column.name != *column_name
        && columns.iter().any(|c| c.name == input.new_column.name)
    {
        return Err(DbError::IllegalInput(format!(
            "column '{}' already exists in table '{}'",
            input.new_column.name, table_name
        )));
    }

    let temp_table_name = format!("{}__rebuild_tmp", table_name);

    let unique_groups = unique_column_groups_in_table(conn, table_name)?;
    let index_state = get_state_of_indexed_for_entire_table(conn, table_name)?;
    let autoincrement_col = col_is_autoincrement(conn, table_name)?;

    // Anywhere the edited column is referenced by name (unique groups, indexes),
    // point it at its new name instead.
    let unique_groups: Vec<Vec<String>> = unique_groups
        .into_iter()
        .map(|group| {
            group
                .into_iter()
                .map(|c| rename_col(c, column_name, &input.new_column.name))
                .collect()
        })
        .collect();

    let index_state: Vec<(String, Vec<String>, bool)> = index_state
        .into_iter()
        .map(|(name, cols, is_unique)| {
            let cols = cols
                .into_iter()
                .map(|c| rename_col(c, column_name, &input.new_column.name))
                .collect();
            (name, cols, is_unique)
        })
        .collect();

    let (single_col_groups, composite_unique_groups): (Vec<Vec<String>>, Vec<Vec<String>>) =
        unique_groups.into_iter().partition(|g| g.len() == 1);
    let single_col_unique: Vec<String> = single_col_groups.into_iter().flatten().collect();

    let pk_col_names: Vec<String> = columns
        .iter()
        .filter(|c| c.primary_key)
        .map(|c| rename_col(c.name.clone(), column_name, &input.new_column.name))
        .collect();
    let composite_primary_key: Vec<String> = if pk_col_names.len() > 1 {
        pk_col_names
    } else {
        Vec::new()
    };

    let mut new_columns: Vec<ColumnDef> = Vec::new();
    let mut column_mappings: Vec<(String, String)> = Vec::new();

    for col in &columns {
        if col.name == *column_name {
            new_columns.push(ColumnDef {
                unique: input.new_column.unique
                    || single_col_unique
                        .iter()
                        .any(|c| c.eq_ignore_ascii_case(&input.new_column.name)),
                primary_key: input.new_column.primary_key && composite_primary_key.is_empty(),
                ..input.new_column.clone()
            });

            if let Some(expr) = resolve_source_expr(column_name, &input.source) {
                column_mappings.push((input.new_column.name.clone(), expr));
            }
            continue;
        }

        new_columns.push(ColumnDef {
            name: col.name.clone(),
            column_type: col.type_name.clone(),
            primary_key: col.primary_key && composite_primary_key.is_empty(),
            not_null: col.not_null,
            unique: single_col_unique
                .iter()
                .any(|c| c.eq_ignore_ascii_case(&col.name)),
            default_value: col.default_value.clone().unwrap_or_default(),
            autoincrement: autoincrement_col.as_deref() == Some(col.name.as_str()),
        });
        column_mappings.push((col.name.clone(), quote_ident(&col.name)));
    }

    execute_delete_old_table_and_create_from_scratch(
        conn,
        table_name,
        &temp_table_name,
        &new_columns,
        &index_state,
        &composite_unique_groups,
        &composite_primary_key,
        &column_mappings,
    )
}

fn resolve_source_expr(old_column_name: &str, source: &ColumnSource) -> Option<String> {
    match source {
        ColumnSource::SameName => Some(quote_ident(old_column_name)),
        ColumnSource::FromColumn(old_name) => Some(quote_ident(old_name)),
        ColumnSource::Expression(expr) => Some(expr.clone()),
        ColumnSource::UseDefault => None,
    }
}

fn rename_col(name: String, old_name: &str, new_name: &str) -> String {
    if name.eq_ignore_ascii_case(old_name) {
        new_name.to_string()
    } else {
        name
    }
}

fn check_if_this_col_is_indexed(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, DbError> {
    let sql = format!("PRAGMA index_list({})", quote_sql_string(table_name));
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let index_names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    for index_name in index_names {
        let info_sql = format!("PRAGMA index_info({})", quote_sql_string(&index_name));
        let mut info_stmt = conn
            .prepare(&info_sql)
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

        let cols: Vec<String> = info_stmt
            .query_map([], |row| row.get::<_, String>(2))
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

        if cols.iter().any(|c| c.eq_ignore_ascii_case(column_name)) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn this_is_a_foreign_key_to_some_other_col(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, DbError> {
    let sql = format!("PRAGMA foreign_key_list({})", quote_sql_string(table_name));
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let mut rows = stmt
        .query([])
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    while let Some(row) = rows
        .next()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
    {
        let from_col: String = row
            .get(3)
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;
        if from_col.eq_ignore_ascii_case(column_name) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn remake_entire_table_from_scratch(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<(), DbError> {
    let temp_table_name = format!("{}__rebuild_tmp", table_name);

    let existing = table_shape(conn, table_name)?;
    let unique_groups = unique_column_groups_in_table(conn, table_name)?;
    let autoincrement_col = col_is_autoincrement(conn, table_name)?;
    let index_state = get_state_of_indexed_for_entire_table(conn, table_name)?;

    // Single-column unique groups become a per-column UNIQUE flag. Multi-column
    // groups can't be expressed that way and get passed through as table-level
    // UNIQUE(...) constraints instead.
    let (single_col_groups, composite_unique_groups): (Vec<Vec<String>>, Vec<Vec<String>>) =
        unique_groups.into_iter().partition(|g| g.len() == 1);
    let single_col_unique: Vec<String> = single_col_groups.into_iter().flatten().collect();

    // Same idea for primary key: if more than one surviving column is part of the
    // PK, it's a composite key and can't be expressed as a per-column flag either.
    let pk_col_names: Vec<String> = existing
        .iter()
        .filter(|c| c.primary_key && c.name != *column_name)
        .map(|c| c.name.clone())
        .collect();
    let composite_primary_key: Vec<String> = if pk_col_names.len() > 1 {
        pk_col_names
    } else {
        Vec::new()
    };

    let mut new_columns: Vec<ColumnDef> = Vec::new();
    for col in &existing {
        if col.name == column_name {
            continue;
        }

        new_columns.push(ColumnDef {
            name: col.name.clone(),
            column_type: col.type_name.clone(),
            primary_key: col.primary_key && composite_primary_key.is_empty(),
            not_null: col.not_null,
            unique: single_col_unique
                .iter()
                .any(|c| c.eq_ignore_ascii_case(&col.name)),
            default_value: col.default_value.clone().unwrap_or_default(),
            autoincrement: autoincrement_col.as_deref() == Some(col.name.as_str()),
        });
    }

    let column_mappings: Vec<(String, String)> = new_columns
        .iter()
        .map(|c| (c.name.clone(), quote_ident(&c.name)))
        .collect();

    execute_delete_old_table_and_create_from_scratch(
        conn,
        table_name,
        &temp_table_name,
        &new_columns,
        &index_state,
        &composite_unique_groups,
        &composite_primary_key,
        &column_mappings,
    )
}

fn execute_delete_old_table_and_create_from_scratch(
    conn: &Connection,
    table_name: &str,
    temp_table_name: &str,
    new_columns: &[ColumnDef],
    index_state: &[(String, Vec<String>, bool)],
    composite_unique_groups: &[Vec<String>],
    composite_primary_key: &[String],
    // (target column name in the new table, SQL expression to select from the old table).
    // A column left out of this list is omitted from the copy — its default/NULL fills it.
    column_mappings: &[(String, String)],
) -> Result<(), DbError> {
    let started_here = start_transaction_if_not_in_one!(conn);

    let result = (|| -> Result<(), DbError> {
        let create_sql = generate_create_table_sql_with_unique_groups(
            temp_table_name,
            new_columns,
            composite_unique_groups,
            composite_primary_key,
        );
        conn.execute(&create_sql, []).map_err(|e| {
            DbError::SqlExecuteFail(format!(
                "execute_delete_old_table_and_create_from_scratch: create temp table failed: {}, sql: {}",
                e, create_sql
            ))
        })?;

        let insert_cols: Vec<String> = column_mappings
            .iter()
            .map(|(target, _)| quote_ident(target))
            .collect();
        let select_exprs: Vec<String> = column_mappings
            .iter()
            .map(|(_, expr)| expr.clone())
            .collect();

        let copy_sql = format!(
            "INSERT INTO {} ({}) SELECT {} FROM {};",
            quote_ident(temp_table_name),
            insert_cols.join(", "),
            select_exprs.join(", "),
            quote_ident(table_name)
        );
        conn.execute(&copy_sql, []).map_err(|e| {
            DbError::SqlExecuteFail(format!(
                "execute_delete_old_table_and_create_from_scratch: copy rows failed: {}, sql: {}",
                e, copy_sql
            ))
        })?;

        let drop_sql = format!("DROP TABLE {};", quote_ident(table_name));
        conn.execute(&drop_sql, []).map_err(|e| {
            DbError::SqlExecuteFail(format!(
                "execute_delete_old_table_and_create_from_scratch: drop old table failed: {}, sql: {}",
                e, drop_sql
            ))
        })?;

        let rename_sql = format!(
            "ALTER TABLE {} RENAME TO {};",
            quote_ident(temp_table_name),
            quote_ident(table_name)
        );
        conn.execute(&rename_sql, []).map_err(|e| {
            DbError::SqlExecuteFail(format!(
                "execute_delete_old_table_and_create_from_scratch: rename failed: {}, sql: {}",
                e, rename_sql
            ))
        })?;

        for (index_name, cols, is_unique) in index_state {
            let cols_csv = cols
                .iter()
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ");
            let unique_kw = if *is_unique { "UNIQUE " } else { "" };
            let idx_sql = format!(
                "CREATE {}INDEX {} ON {} ({});",
                unique_kw,
                quote_ident(index_name),
                quote_ident(table_name),
                cols_csv
            );
            conn.execute(&idx_sql, []).map_err(|e| {
                DbError::SqlExecuteFail(format!(
                    "execute_delete_old_table_and_create_from_scratch: recreate index failed: {}, sql: {}",
                    e, idx_sql
                ))
            })?;
        }

        Ok(())
    })();

    end_transaction_if_in_one!(conn, started_here, result)
}

fn get_state_of_indexed_for_entire_table(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<(String, Vec<String>, bool)>, DbError> {
    let sql = format!("PRAGMA index_list({})", quote_sql_string(table_name));
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let rows: Vec<(String, i64, String)> = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let unique: i64 = row.get(2)?;
            let origin: String = row.get(3)?;
            Ok((name, unique, origin))
        })
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let mut result = Vec::new();
    for (name, unique, origin) in rows {
        if origin != "c" {
            continue;
        }

        let info_sql = format!("PRAGMA index_info({})", quote_sql_string(&name));
        let mut info_stmt = conn
            .prepare(&info_sql)
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

        let cols: Vec<String> = info_stmt
            .query_map([], |row| row.get::<_, String>(2))
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

        result.push((name, cols, unique != 0));
    }

    Ok(result)
}

pub fn unique_column_groups_in_table(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<Vec<String>>, DbError> {
    let index_list_sql = format!("PRAGMA index_list({})", quote_sql_string(table_name));
    let mut stmt = conn
        .prepare(&index_list_sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let indexes: Vec<(String, i64, String)> = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let is_unique: i64 = row.get(2)?;
            let origin: String = row.get(3)?;
            Ok((name, is_unique, origin))
        })
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let mut groups: Vec<Vec<String>> = Vec::new();

    for (index_name, is_unique, origin) in indexes {
        // Only inline `UNIQUE` constraints (origin "u"). Named `CREATE UNIQUE
        // INDEX` indexes (origin "c") are already handled by
        // get_state_of_indexed_for_entire_table, and "pk" is the primary key's
        // own auto-index, handled separately.
        if is_unique == 0 || origin != "u" {
            continue;
        }

        let info_sql = format!("PRAGMA index_info({})", quote_sql_string(&index_name));
        let mut info_stmt = conn
            .prepare(&info_sql)
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

        let cols: Vec<String> = info_stmt
            .query_map([], |row| row.get::<_, String>(2))
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

        groups.push(cols);
    }

    Ok(groups)
}

pub fn unique_columns_in_table(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<String>, DbError> {
    Ok(unique_column_groups_in_table(conn, table_name)?
        .into_iter()
        .flatten()
        .collect())
}

pub fn check_if_col_unique(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, DbError> {
    // inline UNIQUE constraints (origin "u")
    let inline_unique = unique_columns_in_table(conn, table_name)?;
    if inline_unique
        .iter()
        .any(|c| c.eq_ignore_ascii_case(column_name))
    {
        return Ok(true);
    }

    // explicit CREATE UNIQUE INDEX (origin "c")
    let explicit = get_state_of_indexed_for_entire_table(conn, table_name)?;
    for (_, cols, is_unique) in explicit {
        if is_unique && cols.iter().any(|c| c.eq_ignore_ascii_case(column_name)) {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn col_is_autoincrement(
    conn: &Connection,
    table_name: &str,
) -> Result<Option<String>, DbError> {
    let mut stmt = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name=?")
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let create_sql: Option<String> = stmt
        .query_row([table_name], |row| row.get(0))
        .optional()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let Some(create_sql) = create_sql else {
        return Ok(None);
    };

    if !create_sql.to_uppercase().contains("AUTOINCREMENT") {
        return Ok(None);
    }

    let columns = table_shape(conn, table_name)?;
    Ok(columns
        .iter()
        .find(|c| c.primary_key)
        .map(|c| c.name.clone()))
}
