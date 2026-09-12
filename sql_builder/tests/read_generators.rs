use protocol::payload::{JoinType, SelectArgument, SelectArguments};
use sql_builder::{generate_get_data_by_order_sql, generate_read_from_table_sql, to_sql_condition};

fn arg_eq(col: &str, val: &str) -> SelectArgument {
    SelectArgument::XEqualY {
        x: col.into(),
        y: val.into(),
    }
}

fn arg_not_eq(col: &str, val: &str) -> SelectArgument {
    SelectArgument::XNotEqualY {
        x: col.into(),
        y: val.into(),
    }
}

fn arg_greater(col: &str, val: &str) -> SelectArgument {
    SelectArgument::XGreaterThanY {
        x: col.into(),
        y: val.into(),
    }
}

fn arg_less(col: &str, val: &str) -> SelectArgument {
    SelectArgument::XLessThanY {
        x: col.into(),
        y: val.into(),
    }
}

fn arg_greater_or_eq(col: &str, val: &str) -> SelectArgument {
    SelectArgument::XGreaterThanOrEqualY {
        x: col.into(),
        y: val.into(),
    }
}

fn arg_less_or_eq(col: &str, val: &str) -> SelectArgument {
    SelectArgument::XLessThanOrEqualY {
        x: col.into(),
        y: val.into(),
    }
}

fn arg_like(col: &str, val: &str) -> SelectArgument {
    SelectArgument::XLikeY {
        x: col.into(),
        y: val.into(),
    }
}

fn arg_in(col: &str, vals: &[&str]) -> SelectArgument {
    SelectArgument::XInY {
        x: col.into(),
        y: vals.iter().map(|s| s.to_string()).collect(),
    }
}

// --- to_sql_condition ---

#[test]
fn to_sql_condition_single_all_returns_empty_string() {
    let result = to_sql_condition(&SelectArguments::Single(SelectArgument::All));
    let expected = "";
    assert_eq!(result, expected);
}

#[test]
fn to_sql_condition_single_condition() {
    let result = to_sql_condition(&SelectArguments::Single(arg_eq("name", "x")));
    let expected = " WHERE \"name\" = 'x'";
    assert_eq!(result, expected);
}

#[test]
fn to_sql_condition_all_operators() {
    let cases = vec![
        (arg_eq("c", "v"), " WHERE \"c\" = 'v'"),
        (arg_not_eq("c", "v"), " WHERE \"c\" != 'v'"),
        (arg_greater("c", "v"), " WHERE \"c\" > 'v'"),
        (arg_less("c", "v"), " WHERE \"c\" < 'v'"),
        (arg_greater_or_eq("c", "v"), " WHERE \"c\" >= 'v'"),
        (arg_less_or_eq("c", "v"), " WHERE \"c\" <= 'v'"),
        (arg_like("c", "v"), " WHERE \"c\" LIKE 'v'"),
        (arg_in("c", &["a", "b"]), " WHERE \"c\" IN ('a', 'b')"),
    ];

    for (arg, expected) in cases {
        let result = to_sql_condition(&SelectArguments::Single(arg));
        assert_eq!(result, expected);
    }
}

#[test]
fn to_sql_condition_two_with_and() {
    let args = SelectArguments::Two {
        first: arg_eq("a", "1"),
        join: JoinType::And,
        second: arg_eq("b", "2"),
    };
    let result = to_sql_condition(&args);
    let expected = " WHERE \"a\" = '1' AND \"b\" = '2'";
    assert_eq!(result, expected);
}

#[test]
fn to_sql_condition_two_with_or() {
    let args = SelectArguments::Two {
        first: arg_eq("a", "1"),
        join: JoinType::Or,
        second: arg_eq("b", "2"),
    };
    let result = to_sql_condition(&args);
    let expected = " WHERE \"a\" = '1' OR \"b\" = '2'";
    assert_eq!(result, expected);
}

#[test]
fn to_sql_condition_two_with_all_first() {
    let args = SelectArguments::Two {
        first: SelectArgument::All,
        join: JoinType::And,
        second: arg_eq("b", "2"),
    };
    let result = to_sql_condition(&args);
    let expected = " WHERE \"b\" = '2'";
    assert_eq!(result, expected);
}

#[test]
fn to_sql_condition_two_with_all_second() {
    let args = SelectArguments::Two {
        first: arg_eq("a", "1"),
        join: JoinType::And,
        second: SelectArgument::All,
    };
    let result = to_sql_condition(&args);
    let expected = " WHERE \"a\" = '1'";
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
