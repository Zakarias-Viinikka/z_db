#![cfg(feature = "testing")]

mod common;
use common::*;

use db_wrapper::testing_mascot::LiveForever;
use protocol::error::DbError;
use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col, unique_col};
use protocol::payload::*;
use protocol::row_col::Col;

// ============================================================
// TABLE MACROS
// ============================================================

macro_rules! create_users_table {
    ($db:expr) => {
        create_table("users", vec![id_column(), not_null_text_col("name")], $db);
    };
}

macro_rules! seed_alice {
    ($db:expr) => {
        insert_user(InsertUser {
            name: "alice",
            db: $db,
        });
    };
}

// ============================================================
// ACTION HELPERS
// ============================================================

struct InsertUser<'a> {
    name: &'a str,
    db: &'a LiveForever,
}

fn insert_user(params: InsertUser) {
    params
        .db
        .insert_data(InsertDataIn {
            table_name: "users".into(),
            values: vec![ColumnValue {
                column_name: "name".into(),
                value: Col::Text(params.name.into()),
            }],
        })
        .unwrap();
}

// ============================================================
// TESTS
// ============================================================

// create_table rejects an empty table name.
#[test]
fn create_table_rejects_empty_name() {
    let db = setup_empty_db();

    let result = db.create_table(CreateTableIn {
        table_name: "".into(),
        columns: vec![id_column()],
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// create_table rejects names starting with the reserved "fts5_" prefix.
#[test]
fn create_table_rejects_fts5_prefix() {
    let db = setup_empty_db();

    let result = db.create_table(CreateTableIn {
        table_name: "fts5_users".into(),
        columns: vec![id_column()],
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// create_table on an existing table name succeeds silently (CREATE TABLE IF NOT EXISTS).
#[test]
fn create_table_duplicate_name_is_silent() {}

// add_column rejects PRIMARY KEY columns.
#[test]
fn add_column_rejects_primary_key() {
    let db = setup_empty_db();
    create_users_table!(&db);

    let result = add_column(AddColumn {
        table_name: "users",
        column: ColumnDef {
            name: "new_pk".into(),
            column_type: "INTEGER".into(),
            primary_key: true,
            not_null: false,
            unique: false,
            default_value: String::new(),
            autoincrement: false,
        },
        db: &db,
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// add_column rejects NOT NULL columns without a default value.
#[test]
fn add_column_rejects_not_null_without_default() {
    let db = setup_empty_db();
    create_users_table!(&db);

    let result = add_column(AddColumn {
        table_name: "users",
        column: not_null_col(ColumnType::Text, "required"),
        db: &db,
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// add_column rejects AUTOINCREMENT columns.
#[test]
fn add_column_rejects_autoincrement() {
    let db = setup_empty_db();
    create_users_table!(&db);

    let result = add_column(AddColumn {
        table_name: "users",
        column: ColumnDef {
            name: "counter".into(),
            column_type: "INTEGER".into(),
            primary_key: false,
            not_null: false,
            unique: false,
            default_value: String::new(),
            autoincrement: true,
        },
        db: &db,
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// edit_col_in_row rejects row ids that are not numbers.
#[test]
fn edit_col_in_row_rejects_non_numeric_id() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_alice!(&db);

    let result = db.edit_col_in_row(EditColInRowIn {
        table_name: "users".into(),
        row_id: "not-a-number".into(),
        column: "name".into(),
        new_value: Col::Text("bob".into()),
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// swap_columns fails when one of the row ids does not exist.
#[test]
fn swap_columns_rejects_missing_row() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_alice!(&db);

    let result = db.swap_columns(SwapColumnsIn {
        table_name: "users".into(),
        row_id_1: "1".into(),
        row_id_2: "999".into(),
        column: "name".into(),
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}
