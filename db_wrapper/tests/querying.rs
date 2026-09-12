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

fn read(conn: &Connection, args: SelectArguments) -> Vec<Row> {
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
            arguments: SelectArguments::Single(SelectArgument::All),
            columns_to_read: vec!["name".into()],
            order_by: order_by.into(),
        },
    )
    .unwrap()
    .iter()
    .map(|r| r.cols[0].as_str().unwrap().to_string())
    .collect()
}

#[test]
fn fetch_all_and_count_all() {
    let conn = Connection::open_in_memory().unwrap();
    setup_table(&conn);
    insert(&conn, "alice", 30);
    insert(&conn, "bob", 25);
    insert(&conn, "carol", 30);
    insert(&conn, "dave", 40);

    let result = read(&conn, SelectArguments::Single(SelectArgument::All)).len();
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
        SelectArguments::Single(SelectArgument::XEqualY {
            x: "age".into(),
            y: "30".into(),
        }),
    )
    .len();
    let expected = 2;
    assert_eq!(result, expected);

    let result = read(
        &conn,
        SelectArguments::Two {
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
    )
    .len();
    let expected = 1;
    assert_eq!(result, expected);
}

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
        SelectArguments::Two {
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
    )
    .len();
    let expected = 0;
    assert_eq!(result, expected);

    let result = read(
        &conn,
        SelectArguments::Two {
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
    )
    .len();
    let expected = 2;
    assert_eq!(result, expected);
}

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
        SelectArguments::Single(SelectArgument::XEqualY {
            x: "age".into(),
            y: "30".into(),
        }),
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
                arguments: SelectArguments::Single(SelectArgument::XEqualY {
                    x: "age".into(),
                    y: "30".into(),
                }),
                join: None,
            }],
        },
    )
    .unwrap()
    .count;
    let expected = 2;
    assert_eq!(result, expected);
}
