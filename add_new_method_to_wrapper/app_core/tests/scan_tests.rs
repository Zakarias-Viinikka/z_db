// Scan tests. Read the fixture copy of macro_core.rs and check the
// method list comes back in file order with usable offsets.

use app_core::scan::scan_macro_core_text;

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("couldn't read {}: {}", path.display(), e))
}

#[test]
fn lists_methods_in_file_order() {
    let text = fixture("macro_core.rs");

    let result: Vec<String> = scan_macro_core_text(&text)
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    let create = result.iter().position(|n| n == "create_table").unwrap();
    let insert = result.iter().position(|n| n == "insert_data").unwrap();
    let insert_and_get = result
        .iter()
        .position(|n| n == "insert_data_and_get_col")
        .unwrap();
    let drop = result.iter().position(|n| n == "drop_table").unwrap();

    assert!(create < insert, "create_table should come before insert_data");
    assert!(
        insert < insert_and_get,
        "insert_data should come before insert_data_and_get_col"
    );
    assert!(
        insert_and_get < drop,
        "insert_data_and_get_col should come before drop_table"
    );
}

#[test]
fn insert_offset_points_at_end_of_closing_brace_line() {
    let text = fixture("macro_core.rs");
    let methods = scan_macro_core_text(&text).unwrap();
    let insert_data = methods
        .iter()
        .find(|m| m.name == "insert_data")
        .expect("insert_data should be in the fixture");

    let byte_at = text.as_bytes()[insert_data.insert_offset];
    assert_eq!(
        byte_at, b'\n',
        "insert_offset should be a newline, got {:?}",
        byte_at as char
    );
}

#[test]
fn indent_matches_the_methods_first_line() {
    let text = fixture("macro_core.rs");
    let methods = scan_macro_core_text(&text).unwrap();
    let insert_data = methods
        .iter()
        .find(|m| m.name == "insert_data")
        .expect("insert_data should be in the fixture");

    assert_eq!(insert_data.indent, "            ");
}