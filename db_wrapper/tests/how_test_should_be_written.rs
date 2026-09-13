// Test style reference.

// --- Goals ---
//
// The shape of the data a test works with should be easy to read.
// That's why table setup is a macro: expand it and you see a clean
// column list, without it bloating every test in source.
//
// Macros also mean that when something needs to change, you change
// it in one place, not in every test.
//
// Helper methods should make tests read like English. If you want
// to verify what a helper does, you can go look — but if you don't
// want to, it doesn't bloat the test.

// --- Style rules ---
//
// 1. Table setup uses a macro named for the table shape.
// 2. Column helpers hide the type when type isn't the point.
// 3. Action helpers take a params struct, so the call site reads
//    as English: DropColumn { table_name, column_name, db }.
// 4. Assertions: let result / let expected_result / assert_eq!.
// 5. Only assert what the test is about. No "just in case" checks.
// 6. Comments explain why, not what.

use db_wrapper::testing_mascot::LiveForever;
use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};
use protocol::payload::*;

macro_rules! create_users_table {
    ($db:expr) => {
        $db.create_table(CreateTableIn {
            table_name: "users".to_string(),
            columns: vec![
                id_column(),
                not_null_text_col("name"),
                not_null_int_col("age"),
            ],
        })
        .unwrap();
    };
}

fn not_null_text_col(name: &str) -> ColumnDef {
    not_null_col(ColumnType::Text, name)
}

fn not_null_int_col(name: &str) -> ColumnDef {
    not_null_col(ColumnType::Integer, name)
}

struct DropColumn<'a> {
    table_name: &'a str,
    column_name: &'a str,
    db: &'a LiveForever,
}

fn drop_column(params: DropColumn) {
    params
        .db
        .remove_column(RemoveColumnIn {
            table_name: params.table_name.to_string(),
            column_name: params.column_name.to_string(),
        })
        .unwrap();
}

struct ColumnNames<'a> {
    table_name: &'a str,
    db: &'a LiveForever,
}

fn column_names(params: ColumnNames) -> Vec<String> {
    params
        .db
        .check_table(CheckTableIn {
            table_name: params.table_name.to_string(),
        })
        .unwrap()
        .columns
        .iter()
        .map(|c| c.name.clone())
        .collect()
}

// --- Source form ---

#[test]
fn drop_plain_column() {
    let db = LiveForever::new().unwrap();
    create_users_table!(&db);

    drop_column(DropColumn {
        table_name: "users",
        column_name: "age",
        db: &db,
    });

    let result = column_names(ColumnNames {
        table_name: "users",
        db: &db,
    });
    let expected_result = vec!["id", "name"];
    assert_eq!(result, expected_result);
}

// --- Expanded form ---
//
// Same test, with the macro call inlined. Expand a macro in the
// editor and you see this shape.

#[test]
fn drop_plain_column_expanded() {
    let db = LiveForever::new().unwrap();

    db.create_table(CreateTableIn {
        table_name: "users".to_string(),
        columns: vec![
            id_column(),
            not_null_text_col("name"),
            not_null_int_col("age"),
        ],
    })
    .unwrap();

    drop_column(DropColumn {
        table_name: "users",
        column_name: "age",
        db: &db,
    });

    let result = column_names(ColumnNames {
        table_name: "users",
        db: &db,
    });
    let expected_result = vec!["id", "name"];
    assert_eq!(result, expected_result);
}
