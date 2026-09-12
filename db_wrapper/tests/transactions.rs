#![cfg(feature = "testing")]
use db_wrapper::testing_mascot::LiveForever;
use protocol::new_table;
use protocol::payload::*;
use protocol::row_col::Col;

fn setup_db() -> LiveForever {
    let db = LiveForever::new().unwrap();
    db.create_table(CreateTableIn {
        table_name: "users".to_string(),
        columns: vec![
            new_table::id_column(),
            new_table::not_null_col(new_table::ColumnType::Text, "name"),
        ],
    })
    .unwrap();
    db
}

fn insert_user_in(name: &str) -> InsertDataIn {
    InsertDataIn {
        table_name: "users".to_string(),
        values: vec![ColumnValue {
            column_name: "name".to_string(),
            value: Col::Text(name.to_string()),
        }],
    }
}

fn count_users_in() -> CountAllRowsIn {
    CountAllRowsIn {
        table_name: "users".to_string(),
    }
}

// Verifies that commit makes changes permanent and rollback discards them.
#[test]
fn commit_persists_and_rollback_discards() {
    let db = setup_db();

    db.begin_all_or_nothing().unwrap();
    db.insert_data(insert_user_in("alice")).unwrap();
    db.everything_went_perfectly().unwrap();
    assert_eq!(db.count_all_rows(count_users_in()).unwrap().count, 1);

    db.begin_all_or_nothing().unwrap();
    db.insert_data(insert_user_in("bob")).unwrap();
    db.regret_everything().unwrap();
    assert_eq!(db.count_all_rows(count_users_in()).unwrap().count, 1);
}

// Verifies that uncommitted writes are visible to reads on the same connection.
#[test]
fn reads_see_own_writes_inside_transaction() {
    let db = setup_db();

    db.begin_all_or_nothing().unwrap();
    db.insert_data(insert_user_in("alice")).unwrap();
    assert_eq!(db.count_all_rows(count_users_in()).unwrap().count, 1);
}

// Verifies that misuse of the transaction API surfaces as errors:
// commit/rollback without begin, and begin while already in a transaction.
#[test]
fn transaction_misuse_returns_errors() {
    let db = setup_db();

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
    let db = setup_db();

    db.begin_all_or_nothing().unwrap();
    db.insert_data(insert_user_in("alice")).unwrap();

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
    assert_eq!(db.count_all_rows(count_users_in()).unwrap().count, 0);
}
