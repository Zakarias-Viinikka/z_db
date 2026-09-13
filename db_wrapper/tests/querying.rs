use db::black_magic;
use db::black_magic_extension;
use db::black_magic_read;
use protocol::new_table::{self, ColumnDef, ColumnType};
use protocol::payload::*;
use protocol::row_col::{Col, Row};
use rusqlite::Connection;

// ============================================================
// TABLE MACROS
// ============================================================

macro_rules! create_users_table {
    ($conn:expr) => {
        black_magic::create_table(
            $conn,
            "users",
            vec![
                new_table::id_column(),
                not_null_text_col("name"),
                not_null_int_col("age"),
            ],
        )
        .unwrap();
    };
}

macro_rules! seed_users {
    ($conn:expr) => {
        insert_user(InsertUser {
            name: "alice",
            age: 30,
            conn: $conn,
        });
        insert_user(InsertUser {
            name: "bob",
            age: 25,
            conn: $conn,
        });
        insert_user(InsertUser {
            name: "carol",
            age: 30,
            conn: $conn,
        });
        insert_user(InsertUser {
            name: "dave",
            age: 40,
            conn: $conn,
        });
    };
}

fn not_null_text_col(name: &str) -> ColumnDef {
    new_table::not_null_col(ColumnType::Text, name)
}

fn not_null_int_col(name: &str) -> ColumnDef {
    new_table::not_null_col(ColumnType::Integer, name)
}

// ============================================================
// ACTION HELPERS
// ============================================================

struct InsertUser<'a> {
    name: &'a str,
    age: i64,
    conn: &'a Connection,
}

fn insert_user(params: InsertUser) {
    black_magic::insert_into_table(
        params.conn,
        "users",
        vec![
            ColumnValue {
                column_name: "name".into(),
                value: Col::Text(params.name.into()),
            },
            ColumnValue {
                column_name: "age".into(),
                value: Col::Integer(params.age),
            },
        ],
    )
    .unwrap();
}

struct Read<'a> {
    conn: &'a Connection,
    args: SelectArguments,
}

fn read(params: Read) -> Vec<Row> {
    black_magic_read::read_from_db(
        params.conn,
        &GetDataIn {
            table_name: "users".into(),
            arguments: params.args,
            columns_to_read: vec![],
        },
    )
    .unwrap()
}

struct ReadOrderedNames<'a> {
    conn: &'a Connection,
    order_by: &'a str,
}

fn read_ordered_names(params: ReadOrderedNames) -> Vec<String> {
    black_magic_read::read_from_db_ordered(
        params.conn,
        &GetDataOrderedIn {
            table_name: "users".into(),
            arguments: SelectArguments::Single(SelectArgument::All),
            columns_to_read: vec!["name".into()],
            order_by: params.order_by.into(),
        },
    )
    .unwrap()
    .iter()
    .map(|r| r.cols[0].as_str().unwrap().to_string())
    .collect()
}

struct CountAllRows<'a> {
    conn: &'a Connection,
}

fn count_all_rows(params: CountAllRows) -> u64 {
    black_magic_extension::count_all_rows(
        params.conn,
        &CountAllRowsIn {
            table_name: "users".into(),
        },
    )
    .unwrap()
    .count
}

struct CountRowsWithFilter<'a> {
    conn: &'a Connection,
    column_name: &'a str,
    value: &'a str,
}

fn count_rows_with_filter(params: CountRowsWithFilter) -> u64 {
    black_magic_extension::count_rows(
        params.conn,
        &CountRowsIn {
            table_name: "users".into(),
            filters: vec![ColumnFilter {
                col_name: params.column_name.into(),
                arguments: SelectArguments::Single(SelectArgument::XEqualY {
                    x: params.column_name.into(),
                    y: params.value.into(),
                }),
                join: None,
            }],
        },
    )
    .unwrap()
    .count
}

// ============================================================
// TESTS
// ============================================================

#[test]
fn fetch_all_and_count_all() {
    let conn = Connection::open_in_memory().unwrap();
    create_users_table!(&conn);
    seed_users!(&conn);

    let result = read(Read {
        conn: &conn,
        args: SelectArguments::Single(SelectArgument::All),
    })
    .len();
    let expected_result = 4;
    assert_eq!(result, expected_result);

    let result = count_all_rows(CountAllRows { conn: &conn });
    let expected_result = 4;
    assert_eq!(result, expected_result);
}

#[test]
fn and_join_narrows_results() {
    let conn = Connection::open_in_memory().unwrap();
    create_users_table!(&conn);
    seed_users!(&conn);

    let result = read(Read {
        conn: &conn,
        args: SelectArguments::Single(SelectArgument::XEqualY {
            x: "age".into(),
            y: "30".into(),
        }),
    })
    .len();
    let expected_result = 2;
    assert_eq!(result, expected_result);

    let result = read(Read {
        conn: &conn,
        args: SelectArguments::Two {
            first: SelectArgument::XEqualY {
                x: "age".into(),
                y: "30".into(),
            },
            join: JoinType::And,
            second: SelectArgument::XEqualY {
                x: "name".into(),
                y: "alice".into(),
            },
        },
    })
    .len();
    let expected_result = 1;
    assert_eq!(result, expected_result);
}

#[test]
fn or_join_broadens_results() {
    let conn = Connection::open_in_memory().unwrap();
    create_users_table!(&conn);
    seed_users!(&conn);

    let result = read(Read {
        conn: &conn,
        args: SelectArguments::Two {
            first: SelectArgument::XEqualY {
                x: "age".into(),
                y: "25".into(),
            },
            join: JoinType::And,
            second: SelectArgument::XEqualY {
                x: "name".into(),
                y: "dave".into(),
            },
        },
    })
    .len();
    let expected_result = 0;
    assert_eq!(result, expected_result);

    let result = read(Read {
        conn: &conn,
        args: SelectArguments::Two {
            first: SelectArgument::XEqualY {
                x: "age".into(),
                y: "25".into(),
            },
            join: JoinType::Or,
            second: SelectArgument::XEqualY {
                x: "name".into(),
                y: "dave".into(),
            },
        },
    })
    .len();
    let expected_result = 2;
    assert_eq!(result, expected_result);
}

#[test]
fn ordered_read() {
    let conn = Connection::open_in_memory().unwrap();
    create_users_table!(&conn);
    seed_users!(&conn);

    let result = read_ordered_names(ReadOrderedNames {
        conn: &conn,
        order_by: "name ASC",
    });
    let expected_result = vec!["alice", "bob", "carol", "dave"];
    assert_eq!(result, expected_result);

    let result = read_ordered_names(ReadOrderedNames {
        conn: &conn,
        order_by: "name DESC",
    });
    let expected_result = vec!["dave", "carol", "bob", "alice"];
    assert_eq!(result, expected_result);
}

#[test]
fn count_rows_with_filters() {
    let conn = Connection::open_in_memory().unwrap();
    create_users_table!(&conn);
    seed_users!(&conn);

    let result = read(Read {
        conn: &conn,
        args: SelectArguments::Single(SelectArgument::XEqualY {
            x: "age".into(),
            y: "30".into(),
        }),
    })
    .len();
    let expected_result = 2;
    assert_eq!(result, expected_result);

    let result = count_rows_with_filter(CountRowsWithFilter {
        conn: &conn,
        column_name: "age",
        value: "30",
    });
    let expected_result = 2;
    assert_eq!(result, expected_result);
}
