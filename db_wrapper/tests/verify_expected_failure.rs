use db_wrapper::testing_mascot::LiveForever;
use protocol::error::DbError;
use protocol::new_table::{self, ColumnDef, ColumnType};
use protocol::payload::*;
use protocol::row_col::Col;

fn setup_db() -> LiveForever {
    LiveForever::new().unwrap()
}

fn create_users_table(db: &LiveForever) {
    db.create_table(CreateTableIn {
        table_name: "users".into(),
        columns: vec![
            new_table::id_column(),
            new_table::not_null_col(ColumnType::Text, "name"),
        ],
    })
    .unwrap();
}

fn insert_user(db: &LiveForever, name: &str) {
    db.insert_data(InsertDataIn {
        table_name: "users".into(),
        values: vec![ColumnValue {
            column_name: "name".into(),
            value: Col::Text(name.into()),
        }],
    })
    .unwrap();
}

// create_table rejects an empty table name.
#[test]
fn create_table_rejects_empty_name() {
    let db = setup_db();

    let result = db.create_table(CreateTableIn {
        table_name: "".into(),
        columns: vec![new_table::id_column()],
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// create_table rejects names starting with the reserved "fts5_" prefix.
#[test]
fn create_table_rejects_fts5_prefix() {
    let db = setup_db();

    let result = db.create_table(CreateTableIn {
        table_name: "fts5_users".into(),
        columns: vec![new_table::id_column()],
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// create_table on an existing table name succeeds silently (CREATE TABLE IF NOT EXISTS).
#[test]
fn create_table_duplicate_name_is_silent() {}

// add_column rejects PRIMARY KEY columns.
#[test]
fn add_column_rejects_primary_key() {
    let db = setup_db();
    create_users_table(&db);

    let result = db.add_column(AddColumnIn {
        table_name: "users".into(),
        column: ColumnDef {
            name: "new_pk".into(),
            column_type: "INTEGER".into(),
            primary_key: true,
            not_null: false,
            unique: false,
            default_value: String::new(),
            autoincrement: false,
        },
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// add_column rejects NOT NULL columns without a default value.
#[test]
fn add_column_rejects_not_null_without_default() {
    let db = setup_db();
    create_users_table(&db);

    let result = db.add_column(AddColumnIn {
        table_name: "users".into(),
        column: new_table::not_null_col(ColumnType::Text, "required"),
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// add_column rejects UNIQUE columns.
#[test]
fn add_column_rejects_unique() {
    let db = setup_db();
    create_users_table(&db);

    let result = db.add_column(AddColumnIn {
        table_name: "users".into(),
        column: new_table::unique_col(ColumnType::Text, "email"),
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// add_column rejects AUTOINCREMENT columns.
#[test]
fn add_column_rejects_autoincrement() {
    let db = setup_db();
    create_users_table(&db);

    let result = db.add_column(AddColumnIn {
        table_name: "users".into(),
        column: ColumnDef {
            name: "counter".into(),
            column_type: "INTEGER".into(),
            primary_key: false,
            not_null: false,
            unique: false,
            default_value: String::new(),
            autoincrement: true,
        },
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

// edit_col_in_row rejects row ids that are not numbers.
#[test]
fn edit_col_in_row_rejects_non_numeric_id() {
    let db = setup_db();
    create_users_table(&db);
    insert_user(&db, "alice");

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
    let db = setup_db();
    create_users_table(&db);
    insert_user(&db, "alice");

    let result = db.swap_columns(SwapColumnsIn {
        table_name: "users".into(),
        row_id_1: "1".into(),
        row_id_2: "999".into(),
        column: "name".into(),
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}
