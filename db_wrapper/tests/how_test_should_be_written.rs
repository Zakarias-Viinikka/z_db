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

// ============================================================
// AI NOTES — not cleaned up, just here so future AI doesn't
// repeat the same mistakes.
// ============================================================
//
// Everything above is the polished style guide. This section is
// a raw list of stuff I've had to explain over and over.
//
// 1. File layout.
//    - Macros for a test file go in tests/macros/<test_file>.rs,
//      one file per test file, registered in tests/macros/mod.rs
//      with #[macro_use].
//    - Helpers (structs + fns) go at the bottom of the test file
//      under a "// HELPERS" divider. Tests at the top.
//    - common/mod.rs is for stuff shared across multiple test files,
//      not for one-off helpers.
//
// 2. What the call site should show vs hide.
//    - SHOW: the table shape. The column list at the setup call
//      site so you can see what the test operates on.
//    - HIDE: db / &db where possible. Tests shouldn't have db
//      noise everywhere.
//
// 3. macro_rules! hygiene.
//    A macro CANNOT see a `let db` from the call site. Local
//    variables are looked up at the macro's definition site, not
//    the call site. So you must either:
//      - pass db as an argument: create_x!(&db, [...]), OR
//      - put db in a thread_local, OR
//      - not use a macro and use a function instead.
//    There is no fourth option. Renaming db does not help.
//
// 4. No vec![] in macro bodies.
//    vec![] expands to alloc::boxed::box_assume_init_into_vec_unsafe(...)
//    in rust-analyzer's expand view. Use Vec::new() + push instead.
//    Same for anything else that makes the expanded form unreadable.
//
// 5. Pick one: macro or helper.
//    Don't wrap a function call in a macro that does nothing else.
//    If a macro expands to `some_helper(args)`, drop the macro and
//    call the helper directly.
//
// 6. One db per test file.
//    Even if tests in the same file are independent. If a test needs
//    its own db, split it into its own file.
//
// 7. Insert/update/delete helpers take only the fields that matter.
//    Don't make the caller pass `old_value: None` when they're
//    inserting. Either:
//      - split into insert_x / update_x / delete_x helpers, OR
//      - make a single helper and live with Option fields, but
//        don't put Nones at every call site.
//    What you don't do: make the caller think about fts5's three
//    modes every time they touch a row.
//
// 8. Don't answer questions with walls of text.
//    Read the whole correction before replying. A rule stated once
//    is permanent for the session, not for one message.
