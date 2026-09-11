use protocol::payload::{JoinType, SelectArgument};
use sql_builder::{generate_get_data_by_order_sql, generate_read_from_table_sql, to_sql_condition};

fn arg_eq(col: &str, val: &str, join: Option<JoinType>) -> SelectArgument {
    SelectArgument::XEqualY {
        x: col.into(),
        y: val.into(),
        join,
    }
}

fn arg_not_eq(col: &str, val: &str, join: Option<JoinType>) -> SelectArgument {
    SelectArgument::XNotEqualY {
        x: col.into(),
        y: val.into(),
        join,
    }
}

fn arg_greater(col: &str, val: &str, join: Option<JoinType>) -> SelectArgument {
    SelectArgument::XGreaterThanY {
        x: col.into(),
        y: val.into(),
        join,
    }
}

fn arg_less(col: &str, val: &str, join: Option<JoinType>) -> SelectArgument {
    SelectArgument::XLessThanY {
        x: col.into(),
        y: val.into(),
        join,
    }
}

fn arg_greater_or_eq(col: &str, val: &str, join: Option<JoinType>) -> SelectArgument {
    SelectArgument::XGreaterThanOrEqualY {
        x: col.into(),
        y: val.into(),
        join,
    }
}

fn arg_less_or_eq(col: &str, val: &str, join: Option<JoinType>) -> SelectArgument {
    SelectArgument::XLessThanOrEqualY {
        x: col.into(),
        y: val.into(),
        join,
    }
}

fn arg_like(col: &str, val: &str, join: Option<JoinType>) -> SelectArgument {
    SelectArgument::XLikeY {
        x: col.into(),
        y: val.into(),
        join,
    }
}

fn arg_in(col: &str, vals: &[&str], join: Option<JoinType>) -> SelectArgument {
    SelectArgument::XInY {
        x: col.into(),
        y: vals.iter().map(|s| s.to_string()).collect(),
        join,
    }
}

// --- to_sql_condition ---

#[test]
fn to_sql_condition_empty_returns_empty_string() {
    let result = to_sql_condition(&[]);
    let expected = "";
    assert_eq!(result, expected);
}

#[test]
fn to_sql_condition_single_condition() {
    let result = to_sql_condition(&[arg_eq("name", "x", None)]);
    let expected = " WHERE \"name\" = 'x'";
    assert_eq!(result, expected);
}

#[test]
fn to_sql_condition_all_operators() {
    let cases = vec![
        (arg_eq("c", "v", None), " WHERE \"c\" = 'v'"),
        (arg_not_eq("c", "v", None), " WHERE \"c\" != 'v'"),
        (arg_greater("c", "v", None), " WHERE \"c\" > 'v'"),
        (arg_less("c", "v", None), " WHERE \"c\" < 'v'"),
        (arg_greater_or_eq("c", "v", None), " WHERE \"c\" >= 'v'"),
        (arg_less_or_eq("c", "v", None), " WHERE \"c\" <= 'v'"),
        (arg_like("c", "v", None), " WHERE \"c\" LIKE 'v'"),
        (arg_in("c", &["a", "b"], None), " WHERE \"c\" IN ('a', 'b')"),
    ];

    for (arg, expected) in cases {
        let result = to_sql_condition(&[arg]);
        assert_eq!(result, expected);
    }
}

#[test]
fn to_sql_condition_joins() {
    let args = vec![
        arg_eq("a", "1", None),
        arg_eq("b", "2", Some(JoinType::And)),
        arg_eq("c", "3", Some(JoinType::Or)),
    ];
    let result = to_sql_condition(&args);
    let expected = " WHERE \"a\" = '1' AND \"b\" = '2' OR \"c\" = '3'";
    assert_eq!(result, expected);
}

#[test]
fn to_sql_condition_all_is_skipped() {
    let args = vec![
        arg_eq("a", "1", None),
        SelectArgument::All,
        arg_not_eq("b", "2", Some(JoinType::And)),
    ];
    let result = to_sql_condition(&args);
    let expected = " WHERE \"a\" = '1' AND \"b\" != '2'";
    assert_eq!(result, expected);
}

// --- generate_read_from_table_sql ---

#[test]
fn read_sql_columns_and_where() {
    let result = generate_read_from_table_sql("t", "", &[] as &[&str]);
    let expected = "SELECT * FROM \"t\";";
    assert_eq!(result, expected);

    let result = generate_read_from_table_sql("t", "", &["a", "b"]);
    let expected = "SELECT \"a\", \"b\" FROM \"t\";";
    assert_eq!(result, expected);

    let result = generate_read_from_table_sql("t", " WHERE x = 1", &[] as &[&str]);
    let expected = "SELECT * FROM \"t\" WHERE x = 1;";
    assert_eq!(result, expected);
}

// --- generate_get_data_by_order_sql ---

#[test]
fn ordered_sql_columns_where_order() {
    let result = generate_get_data_by_order_sql("t", "", &[] as &[&str], "id");
    let expected = "SELECT * FROM \"t\" ORDER BY id;";
    assert_eq!(result, expected);

    let result = generate_get_data_by_order_sql("t", " WHERE x = 1", &["a"], "id");
    let expected = "SELECT \"a\" FROM \"t\" WHERE x = 1 ORDER BY id;";
    assert_eq!(result, expected);
}
