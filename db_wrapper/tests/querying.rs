use db::black_magic;
use db::black_magic_extension;
use db::black_magic_read;
use protocol::new_table;
use protocol::payload::*;
use protocol::row_col::{Col, Row};
use rusqlite::Connection;

fn setup_table(conn: &Connection) {
    let columns = vec![
        new_table::id_column(),
        new_table::not_null_col(new_table::ColumnType::Text, "name"),
        new_table::not_null_col(new_table::ColumnType::Integer, "age"),
    ];
    assert!(black_magic::create_table(conn, "users", columns).is_ok());
}

fn insert(conn: &Connection, name: &str, age: i64) {
    black_magic::insert_into_table(
        conn,
        "users",
        vec![
            ColumnValue {
                column_name: "name".into(),
                value: Col::Text(name.into()),
            },
            ColumnValue {
                column_name: "age".into(),
                value: Col::Integer(age),
            },
        ],
    )
    .unwrap();
}

fn read(conn: &Connection, args: Vec<SelectArgument>) -> Vec<Row> {
    black_magic_read::read_from_db(
        conn,
        &GetDataIn {
            table_name: "users".into(),
            arguments: args,
            columns_to_read: vec![],
        },
    )
    .unwrap()
}

fn read_ordered(conn: &Connection, order_by: &str) -> Vec<String> {
    black_magic_read::read_from_db_ordered(
        conn,
        &GetDataOrderedIn {
            table_name: "users".into(),
            arguments: vec![SelectArgument::All],
            columns_to_read: vec!["name".into()],
            order_by: order_by.into(),
        },
    )
    .unwrap()
    .iter()
    .map(|r| r.cols[0].as_str().unwrap().to_string())
    .collect()
}

// Smoke test: fetch everything and count everything.
#[test]
fn fetch_all_and_count_all() {
    let conn = Connection::open_in_memory().unwrap();
    setup_table(&conn);
    insert(&conn, "alice", 30);
    insert(&conn, "bob", 25);
    insert(&conn, "carol", 30);
    insert(&conn, "dave", 40);

    let result = read(&conn, vec![SelectArgument::All]).len();
    let expected = 4;
    assert_eq!(result, expected);

    let result = black_magic_extension::count_all_rows(
        &conn,
        &CountAllRowsIn {
            table_name: "users".into(),
        },
    )
    .unwrap()
    .count;
    let expected = 4;
    assert_eq!(result, expected);
}

// AND should narrow results compared to one filter alone.
#[test]
fn and_join_narrows_results() {
    let conn = Connection::open_in_memory().unwrap();
    setup_table(&conn);
    insert(&conn, "alice", 30);
    insert(&conn, "bob", 25);
    insert(&conn, "carol", 30);
    insert(&conn, "dave", 40);

    let result = read(
        &conn,
        vec![SelectArgument::XEqualY {
            x: "age".into(),
            y: "30".into(),
            join: None,
        }],
    )
    .len();
    let expected = 2;
    assert_eq!(result, expected);

    let result = read(
        &conn,
        vec![
            SelectArgument::XEqualY {
                x: "age".into(),
                y: "30".into(),
                join: None,
            },
            SelectArgument::XEqualY {
                x: "name".into(),
                y: "alice".into(),
                join: Some(JoinType::And),
            },
        ],
    )
    .len();
    let expected = 1;
    assert_eq!(result, expected);
}

// OR should broaden results compared to AND.
#[test]
fn or_join_broadens_results() {
    let conn = Connection::open_in_memory().unwrap();
    setup_table(&conn);
    insert(&conn, "alice", 30);
    insert(&conn, "bob", 25);
    insert(&conn, "carol", 30);
    insert(&conn, "dave", 40);

    let result = read(
        &conn,
        vec![
            SelectArgument::XEqualY {
                x: "age".into(),
                y: "25".into(),
                join: None,
            },
            SelectArgument::XEqualY {
                x: "name".into(),
                y: "dave".into(),
                join: Some(JoinType::And),
            },
        ],
    )
    .len();
    let expected = 0;
    assert_eq!(result, expected);

    let result = read(
        &conn,
        vec![
            SelectArgument::XEqualY {
                x: "age".into(),
                y: "25".into(),
                join: None,
            },
            SelectArgument::XEqualY {
                x: "name".into(),
                y: "dave".into(),
                join: Some(JoinType::Or),
            },
        ],
    )
    .len();
    let expected = 2;
    assert_eq!(result, expected);
}

// Ordered read should return rows in the requested order.
#[test]
fn ordered_read() {
    let conn = Connection::open_in_memory().unwrap();
    setup_table(&conn);
    insert(&conn, "alice", 30);
    insert(&conn, "bob", 25);
    insert(&conn, "carol", 30);
    insert(&conn, "dave", 40);

    let result = read_ordered(&conn, "name ASC");
    let expected = vec!["alice", "bob", "carol", "dave"];
    assert_eq!(result, expected);

    let result = read_ordered(&conn, "name DESC");
    let expected = vec!["dave", "carol", "bob", "alice"];
    assert_eq!(result, expected);
}

// count_rows with a filter should match what a read returns.
#[test]
fn count_rows_with_filters() {
    let conn = Connection::open_in_memory().unwrap();
    setup_table(&conn);
    insert(&conn, "alice", 30);
    insert(&conn, "bob", 25);
    insert(&conn, "carol", 30);
    insert(&conn, "dave", 40);

    let result = read(
        &conn,
        vec![SelectArgument::XEqualY {
            x: "age".into(),
            y: "30".into(),
            join: None,
        }],
    )
    .len();
    let expected = 2;
    assert_eq!(result, expected);

    let result = black_magic_extension::count_rows(
        &conn,
        &CountRowsIn {
            table_name: "users".into(),
            filters: vec![ColumnFilter {
                col_name: "age".into(),
                arguments: vec![SelectArgument::XEqualY {
                    x: "age".into(),
                    y: "30".into(),
                    join: None,
                }],
                join: None,
            }],
        },
    )
    .unwrap()
    .count;
    let expected = 2;
    assert_eq!(result, expected);
}
