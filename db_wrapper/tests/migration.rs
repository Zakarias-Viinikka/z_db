#![cfg(feature = "testing")]

mod common;
use common::*;

use db_wrapper::testing_mascot::LiveForever;
use protocol::error::DbError;
use protocol::new_table::id_column;
use protocol::payload::*;
use protocol::row_col::Col;

// ============================================================
// TABLE MACROS
//
// Each macro defines one full table shape. Expand and you see
// the exact column list. Changing the base table means editing
// one macro, not every test.
// ============================================================

macro_rules! create_users_table {
    ($db:expr) => {
        create_table(
            "users",
            vec![
                id_column(),
                not_null_text_col("name"),
                not_null_int_col("age"),
            ],
            $db,
        );
    };
}

macro_rules! create_users_table_with_email {
    ($db:expr) => {
        create_table(
            "users",
            vec![
                id_column(),
                not_null_text_col("name"),
                not_null_int_col("age"),
                unique_text_col("email"),
            ],
            $db,
        );
    };
}

macro_rules! seed_users {
    ($db:expr) => {
        insert_user(InsertUser {
            name: "alice",
            age: 30,
            db: $db,
        });
        insert_user(InsertUser {
            name: "bob",
            age: 25,
            db: $db,
        });
        insert_user(InsertUser {
            name: "carol",
            age: 30,
            db: $db,
        });
    };
}

// ============================================================
// ACTION HELPERS
// ============================================================

struct InsertUser<'a> {
    name: &'a str,
    age: i64,
    db: &'a LiveForever,
}

struct InsertUserWithEmail<'a> {
    name: &'a str,
    age: i64,
    email: &'a str,
    db: &'a LiveForever,
}

fn insert_user(params: InsertUser) {
    params
        .db
        .insert_data(InsertDataIn {
            table_name: "users".to_string(),
            values: vec![
                ColumnValue {
                    column_name: "name".to_string(),
                    value: Col::Text(params.name.to_string()),
                },
                ColumnValue {
                    column_name: "age".to_string(),
                    value: Col::Integer(params.age),
                },
            ],
        })
        .unwrap();
}

fn insert_user_with_email(params: InsertUserWithEmail) {
    params
        .db
        .insert_data(InsertDataIn {
            table_name: "users".to_string(),
            values: vec![
                ColumnValue {
                    column_name: "name".to_string(),
                    value: Col::Text(params.name.to_string()),
                },
                ColumnValue {
                    column_name: "age".to_string(),
                    value: Col::Integer(params.age),
                },
                ColumnValue {
                    column_name: "email".to_string(),
                    value: Col::Text(params.email.to_string()),
                },
            ],
        })
        .unwrap();
}

// ============================================================
// TESTS
// ============================================================

#[test]
fn add_unique_and_not_null_columns_to_populated_table() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_users!(&db);

    // unique with no default, on a populated table -> succeeds
    // existing rows get NULL, and NULL is exempt from uniqueness.
    add_column(AddColumn {
        table_name: "users",
        column: unique_text_col("email"),
        db: &db,
    })
    .unwrap();
    let result = is_unique(IsUnique {
        table_name: "users",
        column_name: "email",
        db: &db,
    });
    let expected_result = true;
    assert_eq!(result, expected_result);

    // unique with a default, on a populated table -> fails
    // all existing rows get filled with the same default value.
    // unique means no two rows can match, so the constraint fires and it fails.
    let result = add_column(AddColumn {
        table_name: "users",
        column: unique_text_col_with_default("username", "'x'"),
        db: &db,
    });
    let expected_result = true;
    assert_eq!(result.is_err(), expected_result);

    // not-null with a default, on a populated table -> succeeds
    // existing rows get filled with the default.
    add_column(AddColumn {
        table_name: "users",
        column: not_null_text_col_with_default("note", "'seed'"),
        db: &db,
    })
    .unwrap();
    let result = string_values_in_col(StringValuesInCol {
        table_name: "users",
        column_name: "note",
        db: &db,
    });
    let expected_result = vec!["seed", "seed", "seed"];
    assert_eq!(result, expected_result);
}

#[test]
fn drop_plain_column() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_users!(&db);

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

#[test]
fn drop_indexed_column_preserves_everything_else() {
    let db = setup_empty_db();
    create_users_table_with_email!(&db);
    index_column(IndexColumn {
        table_name: "users",
        column_name: "name",
        db: &db,
    });
    index_column(IndexColumn {
        table_name: "users",
        column_name: "age",
        db: &db,
    });

    insert_user_with_email(InsertUserWithEmail {
        name: "alice",
        age: 30,
        email: "a@x",
        db: &db,
    });
    insert_user_with_email(InsertUserWithEmail {
        name: "bob",
        age: 25,
        email: "b@x",
        db: &db,
    });

    drop_column(DropColumn {
        table_name: "users",
        column_name: "age",
        db: &db,
    });

    let result = column_names(ColumnNames {
        table_name: "users",
        db: &db,
    });
    let expected_result = vec!["id", "name", "email"];
    assert_eq!(result, expected_result);

    let result = is_unique(IsUnique {
        table_name: "users",
        column_name: "email",
        db: &db,
    });
    let expected_result = true;
    assert_eq!(result, expected_result);

    let result = is_indexed(IsIndexed {
        table_name: "users",
        column_name: "name",
        db: &db,
    });
    let expected_result = true;
    assert_eq!(result, expected_result);
}

//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//
//i verified tests up to here.
// dont remove this comment untill i verify the rest
//
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT
//IMPORTANT

#[test]
fn rename_column() {
    let db = setup_empty_db();
    create_users_table!(&db);
    index_column(IndexColumn {
        table_name: "users",
        column_name: "name",
        db: &db,
    });
    seed_users!(&db);

    edit_column(EditColumn {
        table_name: "users",
        column_name: "name",
        new_column: not_null_text_col("full_name"),
        source: ColumnSource::SameName,
        db: &db,
    })
    .unwrap();

    let result = string_values_in_col(StringValuesInCol {
        table_name: "users",
        column_name: "full_name",
        db: &db,
    });
    let expected_result = vec!["alice", "bob", "carol"];
    assert_eq!(result, expected_result);

    let result = is_indexed(IsIndexed {
        table_name: "users",
        column_name: "full_name",
        db: &db,
    });
    let expected_result = true;
    assert_eq!(result, expected_result);
}

#[test]
fn change_column_type() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_users!(&db);

    edit_column(EditColumn {
        table_name: "users",
        column_name: "age",
        new_column: not_null_text_col("age"),
        source: ColumnSource::Expression("CAST(age AS TEXT)".to_string()),
        db: &db,
    })
    .unwrap();

    let result = column_types(ColumnTypes {
        table_name: "users",
        db: &db,
    })
    .contains(&"TEXT".to_string());
    let expected_result = true;
    assert_eq!(result, expected_result);

    let result = string_values_in_col(StringValuesInCol {
        table_name: "users",
        column_name: "age",
        db: &db,
    });
    let expected_result = vec!["30", "25", "30"];
    assert_eq!(result, expected_result);
}

#[test]
fn edit_column_with_use_default() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_users!(&db);

    edit_column(EditColumn {
        table_name: "users",
        column_name: "name",
        new_column: text_col_with_default("name", "'unknown'"),
        source: ColumnSource::UseDefault,
        db: &db,
    })
    .unwrap();

    let result = string_values_in_col(StringValuesInCol {
        table_name: "users",
        column_name: "name",
        db: &db,
    });
    let expected_result = vec!["unknown", "unknown", "unknown"];
    assert_eq!(result, expected_result);
}

#[test]
fn edit_column_to_a_name_that_already_exists() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_users!(&db);

    let result = edit_column(EditColumn {
        table_name: "users",
        column_name: "name",
        new_column: not_null_text_col("age"),
        source: ColumnSource::SameName,
        db: &db,
    });

    assert!(matches!(result, Err(DbError::IllegalInput(_))));
}

#[test]
fn edit_column_inside_an_outer_transaction() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_users!(&db);

    db.begin_all_or_nothing().unwrap();
    edit_column(EditColumn {
        table_name: "users",
        column_name: "name",
        new_column: not_null_text_col("full_name"),
        source: ColumnSource::SameName,
        db: &db,
    })
    .unwrap();
    db.regret_everything().unwrap();

    let result = column_names(ColumnNames {
        table_name: "users",
        db: &db,
    });
    let expected_result = vec!["id", "name", "age"];
    assert_eq!(result, expected_result);
}

#[test]
fn rebuild_that_fails_partway() {
    let db = setup_empty_db();
    create_users_table!(&db);
    seed_users!(&db);

    // alice and carol both have age 30 -> adding UNIQUE on age fails at
    // index-recreate time. The original table must be untouched.
    let result = edit_column(EditColumn {
        table_name: "users",
        column_name: "age",
        new_column: not_null_unique_int_col("age"),
        source: ColumnSource::SameName,
        db: &db,
    });
    let expected_result = true;
    assert_eq!(result.is_err(), expected_result);

    let result = is_unique(IsUnique {
        table_name: "users",
        column_name: "age",
        db: &db,
    });
    let expected_result = false;
    assert_eq!(result, expected_result);
}
