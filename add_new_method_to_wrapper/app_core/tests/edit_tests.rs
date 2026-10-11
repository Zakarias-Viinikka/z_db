// Edit tests. The builders should be pure text-in, text-out. The core
// invariant: taking the added text back out gives you the original.

use app_core::edit::{add_to_db_helper, add_to_worker_js, insert_method_into_macro_core, Position};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("couldn't read {}: {}", path.display(), e))
}

const NO_IN_NO_OUT: &str = r#"pub fn do_nothing(&self) -> method_return_type!(()) {
    return_nothing!()
}
"#;

const IN_NO_OUT: &str = r#"pub fn do_thing(&self, data: method_input_type!(DoThingIn)) -> method_return_type!(()) {
    let input: DoThingIn = decode_input!(data);
    let conn = unwrap_or_bail!(self.get_conn());
    unwrap_or_bail!(::db::black_magic::do_thing(&*conn, &input.field));
    return_nothing!()
}
"#;

const NO_IN_OUT: &str = r#"pub fn get_thing(&self) -> method_return_type!(GetThingOut) {
    let conn = unwrap_or_bail!(self.get_conn());
    let thing = unwrap_or_bail!(::db::black_magic::get_thing(&*conn));
    finish_output!(GetThingOut { value: thing })
}
"#;

const IN_OUT: &str = r#"pub fn get_thing_for(&self, data: method_input_type!(GetThingForIn)) -> method_return_type!(GetThingForOut) {
    let input: GetThingForIn = decode_input!(data);
    let conn = unwrap_or_bail!(self.get_conn());
    let value = unwrap_or_bail!(::db::black_magic::get_thing_for(&*conn, &input.key));
    finish_output!(GetThingForOut { value })
}
"#;

#[test]
fn insert_below_a_named_method() {
    let text = fixture("macro_core.rs");

    let result = insert_method_into_macro_core(
        &text,
        &Position::Named("insert_data".to_string()),
        NO_IN_NO_OUT,
    )
    .unwrap();

    let insert_at = result.find("pub fn insert_data(").unwrap();
    let new_at = result.find("pub fn do_nothing(").unwrap();
    let drop_at = result.find("pub fn drop_table(").unwrap();
    assert!(
        insert_at < new_at && new_at < drop_at,
        "do_nothing should land between insert_data and drop_table"
    );
}

#[test]
fn insert_at_bottom() {
    let text = fixture("macro_core.rs");

    let result =
        insert_method_into_macro_core(&text, &Position::Bottom, NO_IN_NO_OUT).unwrap();

    let new_at = result.find("pub fn do_nothing(").unwrap();
    let after = &result[new_at..];
    let closing = after
        .find("        }\n    };\n}")
        .expect("the impl macro's closing braces should still come after the last method");
    let fn_count = after[..closing].matches("pub fn ").count();
    assert_eq!(
        fn_count, 1,
        "do_nothing should be the only fn after its own declaration, got {}",
        fn_count
    );
}

#[test]
fn macro_core_edit_is_a_pure_insertion() {
    let text = fixture("macro_core.rs");

    let result = insert_method_into_macro_core(
        &text,
        &Position::Named("insert_data".to_string()),
        NO_IN_NO_OUT,
    )
    .unwrap();

    let added = "\n\n            pub fn do_nothing(&self) -> method_return_type!(()) {\n                return_nothing!()\n            }";
    let recovered = result.replacen(added, "", 1);
    assert_eq!(recovered, text);
}

#[test]
fn macro_core_reindents_a_pasted_method_that_arrives_at_column_zero() {
    let text = fixture("macro_core.rs");

    let flat = "pub fn flat_thing(&self) -> method_return_type!(()) {\n    return_nothing!()\n}\n";

    let result = insert_method_into_macro_core(&text, &Position::Bottom, flat).unwrap();

    let needle = "pub fn flat_thing";
    let i = result.find(needle).unwrap();
    let after = &result[i..];
    assert!(
        after.contains("\n                return_nothing!()"),
        "the body should have been reindented"
    );
}

#[test]
fn worker_js_entry_for_a_method_with_input() {
    let text = fixture("worker.js");

    let result = add_to_worker_js(&text, "do_thing", true).unwrap();

    let needle = "  do_thing: (msg) => db_manager.do_thing(msg[1]),";
    assert!(
        result.contains(needle),
        "the entry should be in serializedCommands style"
    );
    let sc = result.find("const serializedCommands = {").unwrap();
    let nic = result.find("const noInputCommands = {").unwrap();
    let entry = result.find(needle).unwrap();
    assert!(sc < entry && entry < nic);
}

#[test]
fn worker_js_entry_for_a_method_with_no_input() {
    let text = fixture("worker.js");

    let result = add_to_worker_js(&text, "get_thing", false).unwrap();

    let needle = "  get_thing: () => db_manager.get_thing(),";
    assert!(
        result.contains(needle),
        "the entry should be in noInputCommands style"
    );
    let nic = result.find("const noInputCommands = {").unwrap();
    let entry = result.find(needle).unwrap();
    assert!(nic < entry);
}

#[test]
fn worker_js_edit_is_a_pure_insertion() {
    let text = fixture("worker.js");

    let result = add_to_worker_js(&text, "do_thing", true).unwrap();

    let added = "  do_thing: (msg) => db_manager.do_thing(msg[1]),\n";
    let recovered = result.replacen(added, "", 1);
    assert_eq!(recovered, text);
}

#[test]
fn db_helper_lands_in_the_right_section_for_each_shape() {
    let text = fixture("db_helper.rs");

    let cases: [(&str, Option<&str>, Option<&str>, &str); 4] = [
        ("a_no_in_out", None, Some("NoInOut"), "// no input, output"),
        ("b_no_in_no_out", None, None, "// no input, no output"),
        ("c_in_no_out", Some("InNoOutIn"), None, "// input, no output"),
        ("d_in_out", Some("InOutIn"), Some("InOutOut"), "// input, output"),
    ];

    for (name, input, output, header) in cases {
        let result = add_to_db_helper(&text, name, input, output).unwrap();
        let header_at = result.find(header).unwrap();
        let fn_at = result.find(&format!("pub async fn {}(", name)).unwrap();

        let after_header = &result[header_at + header.len()..];
        let next_header_at = after_header
            .find("\n// ")
            .map(|i| header_at + header.len() + i)
            .unwrap_or(result.len());
        assert!(
            fn_at > header_at && fn_at < next_header_at,
            "{} should land under '{}'",
            name,
            header
        );
    }
}

#[test]
fn db_helper_edit_is_a_pure_insertion() {
    let text = fixture("db_helper.rs");

    let result = add_to_db_helper(&text, "a_no_in_out", None, Some("NoInOut")).unwrap();

    let added = "pub async fn a_no_in_out() -> Result<NoInOut, DbError> {\n    let bytes = tell_worker_to_do(\"a_no_in_out\", None).await?;\n    NoInOut::un_payloadify(&bytes)\n}\n\n";
    let recovered = result.replacen(added, "", 1);
    assert_eq!(recovered, text);
}

#[test]
fn paste_parses_no_input_no_output() {
    let result = app_core::scan::parse_pasted_method(NO_IN_NO_OUT).unwrap();
    assert_eq!(result.name, "do_nothing");
    assert!(result.input_type.is_none());
    assert!(result.output_type.is_none());
}

#[test]
fn paste_parses_input_no_output() {
    let result = app_core::scan::parse_pasted_method(IN_NO_OUT).unwrap();
    assert_eq!(result.name, "do_thing");
    assert_eq!(result.input_type.as_deref(), Some("DoThingIn"));
    assert!(result.output_type.is_none());
}

#[test]
fn paste_parses_no_input_output() {
    let result = app_core::scan::parse_pasted_method(NO_IN_OUT).unwrap();
    assert_eq!(result.name, "get_thing");
    assert!(result.input_type.is_none());
    assert_eq!(result.output_type.as_deref(), Some("GetThingOut"));
}

#[test]
fn paste_parses_input_output() {
    let result = app_core::scan::parse_pasted_method(IN_OUT).unwrap();
    assert_eq!(result.name, "get_thing_for");
    assert_eq!(result.input_type.as_deref(), Some("GetThingForIn"));
    assert_eq!(result.output_type.as_deref(), Some("GetThingForOut"));
}

#[test]
fn paste_rejects_a_missing_self() {
    let text = "pub fn do_thing(data: method_input_type!(DoThingIn)) -> method_return_type!(()) {\n    return_nothing!()\n}\n";
    assert!(app_core::scan::parse_pasted_method(text).is_err());
}