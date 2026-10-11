// Precheck tests. Collect every problem before touching disk.

use app_core::precheck::{precheck, PreCheckInput};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("couldn't read {}: {}", path.display(), e))
}

struct Inputs {
    macro_core: String,
    worker_js: String,
    db_helper: String,
    payload: String,
}

fn fixtures() -> Inputs {
    Inputs {
        macro_core: fixture("macro_core.rs"),
        worker_js: fixture("worker.js"),
        db_helper: fixture("db_helper.rs"),
        payload: fixture("payload.rs"),
    }
}

fn run_check_with(
    inputs: &Inputs,
    name: &str,
    input_type: Option<&str>,
    output_type: Option<&str>,
) -> Vec<String> {
    precheck(PreCheckInput {
        name,
        input_type,
        output_type,
        macro_core_text: &inputs.macro_core,
        worker_js_text: &inputs.worker_js,
        db_helper_text: &inputs.db_helper,
        payload_text: &inputs.payload,
    })
}

#[test]
fn no_problems_for_a_fresh_name_and_known_types() {
    let inputs = fixtures();

    let result = run_check_with(
        &inputs,
        "totally_new_name",
        Some("InsertDataIn"),
        Some("GetDataOut"),
    );

    let expected_result: Vec<String> = vec![];
    assert_eq!(result, expected_result);
}

#[test]
fn catches_a_name_that_is_already_a_method_in_macro_core() {
    let inputs = fixtures();

    let result = run_check_with(&inputs, "insert_data", Some("InsertDataIn"), None);

    assert!(
        result.iter().any(|p| p.contains("macro_core.rs")),
        "should have flagged macro_core.rs, got: {:?}",
        result
    );
}

#[test]
fn catches_a_name_that_is_a_worker_js_alias() {
    let inputs = fixtures();

    let result = run_check_with(&inputs, "delete_table", Some("DropTableIn"), None);

    assert!(
        result.iter().any(|p| p.contains("worker.js")),
        "should have flagged worker.js, got: {:?}",
        result
    );
}

#[test]
fn catches_a_name_that_is_already_a_fn_in_db_helper() {
    let inputs = fixtures();

    let result = run_check_with(&inputs, "list_tables", None, Some("ListTablesOut"));

    assert!(
        result.iter().any(|p| p.contains("db_helper.rs")),
        "should have flagged db_helper.rs, got: {:?}",
        result
    );
}

#[test]
fn catches_a_missing_type() {
    let inputs = fixtures();

    let result = run_check_with(&inputs, "totally_new_name", Some("ThisTypeDoesNotExist"), None);

    assert_eq!(result.len(), 1);
    assert!(result[0].contains("ThisTypeDoesNotExist"));
}

#[test]
fn accepts_a_type_alias_defined_in_payload() {
    let inputs = fixtures();

    let result = run_check_with(&inputs, "totally_new_name", None, Some("SearchFts5Out"));

    let expected_result: Vec<String> = vec![];
    assert_eq!(result, expected_result);
}

#[test]
fn collects_every_problem_not_just_the_first() {
    let inputs = fixtures();

    let result = run_check_with(&inputs, "insert_data", Some("NopeIn"), Some("NopeOut"));

    assert!(
        result.len() >= 3,
        "should have collected multiple problems, got: {:?}",
        result
    );
}