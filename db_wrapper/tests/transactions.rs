#![cfg(feature = "testing")]

mod common;
use common::*;

use db_wrapper::testing_mascot::LiveForever;
use protocol::new_table::id_column;
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

macro_rules! seed_bob {
    ($db:expr) => {
        insert_user(InsertUser {
            name: "bob",
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
            table_name: "users".to_string(),
            values: vec![ColumnValue {
                column_name: "name".to_string(),
                value: Col::Text(params.name.to_string()),
            }],
        })
        .unwrap();
}

// ============================================================
// TESTS
// ============================================================

// Verifies that commit makes changes permanent and rollback discards them.
#[test]
fn commit_persists_and_rollback_discards() {
    let db = setup_empty_db();
    create_users_table!(&db);

    db.begin_all_or_nothing().unwrap();
    seed_alice!(&db);
    db.everything_went_perfectly().unwrap();
    let result = row_count(RowCount {
        table_name: "users",
        db: &db,
    });
    let expected_result = 1;
    assert_eq!(result, expected_result);

    db.begin_all_or_nothing().unwrap();
    seed_bob!(&db);
    db.regret_everything().unwrap();
    let result = row_count(RowCount {
        table_name: "users",
        db: &db,
    });
    let expected_result = 1;
    assert_eq!(result, expected_result);
}

// Verifies that uncommitted writes are visible to reads on the same connection.
#[test]
fn reads_see_own_writes_inside_transaction() {
    let db = setup_empty_db();
    create_users_table!(&db);

    db.begin_all_or_nothing().unwrap();
    seed_alice!(&db);
    let result = row_count(RowCount {
        table_name: "users",
        db: &db,
    });
    let expected_result = 1;
    assert_eq!(result, expected_result);
}

// Verifies that misuse of the transaction API surfaces as errors:
// commit/rollback without begin, and begin while already in a transaction.
#[test]
fn transaction_misuse_returns_errors() {
    let db = setup_empty_db();
    create_users_table!(&db);

    assert!(db.everything_went_perfectly().is_err());
    assert!(db.regret_everything().is_err());

    db.begin_all_or_nothing().unwrap();
    assert!(db.begin_all_or_nothing().is_err());

    db.regret_everything().unwrap();
}

// Verifies that if a statement fails mid-transaction, rolling back
// still leaves the database clean (no partial writes persist).
#[test]
fn error_mid_transaction_then_rollback_leaves_nothing() {
    let db = setup_empty_db();
    create_users_table!(&db);

    db.begin_all_or_nothing().unwrap();
    seed_alice!(&db);

    // name is NOT NULL, so this should fail
    let result = db.insert_data(InsertDataIn {
        table_name: "users".to_string(),
        values: vec![ColumnValue {
            column_name: "name".to_string(),
            value: Col::Null,
        }],
    });
    assert!(result.is_err());

    db.regret_everything().unwrap();
    let result = row_count(RowCount {
        table_name: "users",
        db: &db,
    });
    let expected_result = 0;
    assert_eq!(result, expected_result);
}
