// Process tests. These run the pipeline end to end against a temp copy of
// the four files and a shell command as the "check" step.
//
// The check commands are `true` and `false`, not real cargo. We're testing
// the pipeline's control flow and the guard, not whether cargo can compile
// a pasted method — that's what the actual tool is for.

use app_core::events::Event;
use app_core::guard;
use app_core::pipeline::{hash_bytes, run, Action, Request};
use app_core::edit::Position;
use app_core::settings::{CheckCommand, Settings};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const GOOD_METHOD: &str = r#"pub fn zzz_process_test_method(&self) -> method_return_type!(()) {
    return_nothing!()
}
"#;

struct TestEnv {
    _tmp: tempfile::TempDir,
    settings: Settings,
}

fn setup_with_check(command_line: &str) -> TestEnv {
    let tmp = tempfile::tempdir().unwrap();
    let real = tmp.path().join("real");
    std::fs::create_dir_all(&real).unwrap();

    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for name in ["macro_core.rs", "worker.js", "db_helper.rs", "payload.rs"] {
        std::fs::copy(fixtures.join(name), real.join(name)).unwrap();
    }

    let settings = Settings {
        macro_core_path: real.join("macro_core.rs"),
        worker_js_path: real.join("worker.js"),
        db_helper_path: real.join("db_helper.rs"),
        payload_path: real.join("payload.rs"),
        working_directory: tmp.path().to_path_buf(),
        check_commands: vec![CheckCommand {
            label: "test command".into(),
            command_line: command_line.into(),
            enabled: true,
        }],
        use_mock_paths: false,
        mock_root: tmp.path().join("mock"),
        cache_root: tmp.path().join("cache"),
    };

    TestEnv { _tmp: tmp, settings }
}

fn no_emit(_: Event) {}

fn original_macro_core(_env: &TestEnv) -> Vec<u8> {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    std::fs::read(fixtures.join("macro_core.rs")).unwrap()
}

#[test]
fn check_only_reverts_macro_core_on_pass() {
    let env = setup_with_check("true");

    let request = Request {
        action: Action::CheckOnly,
        position: Position::Bottom,
        pasted: GOOD_METHOD.to_string(),
    };

    let result = run(
        &env.settings,
        &request,
        Arc::new(AtomicBool::new(false)),
        &no_emit,
    );

    assert!(result.is_ok(), "pipeline should succeed, got {:?}", result.err());

    let after = std::fs::read(&env.settings.macro_core_path).unwrap();
    let expected_result = original_macro_core(&env);
    assert_eq!(after, expected_result, "macro_core.rs should be byte-identical to the original");
}

#[test]
fn check_only_reverts_macro_core_on_fail() {
    let env = setup_with_check("false");

    let request = Request {
        action: Action::CheckOnly,
        position: Position::Bottom,
        pasted: GOOD_METHOD.to_string(),
    };

    let result = run(
        &env.settings,
        &request,
        Arc::new(AtomicBool::new(false)),
        &no_emit,
    );

    assert!(result.is_err(), "pipeline should fail");

    let after = std::fs::read(&env.settings.macro_core_path).unwrap();
    let expected_result = original_macro_core(&env);
    assert_eq!(after, expected_result, "macro_core.rs should be reverted after a failed check");
}

#[test]
fn check_and_apply_with_failed_check_reverts_and_writes_nothing() {
    let env = setup_with_check("false");

    let worker_before = std::fs::read(&env.settings.worker_js_path).unwrap();
    let db_helper_before = std::fs::read(&env.settings.db_helper_path).unwrap();

    let request = Request {
        action: Action::CheckAndApply,
        position: Position::Bottom,
        pasted: GOOD_METHOD.to_string(),
    };

    let result = run(
        &env.settings,
        &request,
        Arc::new(AtomicBool::new(false)),
        &no_emit,
    );

    assert!(result.is_err());

    let macro_after = std::fs::read(&env.settings.macro_core_path).unwrap();
    let worker_after = std::fs::read(&env.settings.worker_js_path).unwrap();
    let db_helper_after = std::fs::read(&env.settings.db_helper_path).unwrap();

    assert_eq!(macro_after, original_macro_core(&env));
    assert_eq!(worker_after, worker_before);
    assert_eq!(db_helper_after, db_helper_before);
}

#[test]
fn check_and_apply_with_passing_check_writes_all_three() {
    let env = setup_with_check("true");

    let request = Request {
        action: Action::CheckAndApply,
        position: Position::Bottom,
        pasted: GOOD_METHOD.to_string(),
    };

    let result = run(
        &env.settings,
        &request,
        Arc::new(AtomicBool::new(false)),
        &no_emit,
    );

    assert!(result.is_ok(), "pipeline should succeed, got {:?}", result.err());

    let macro_text = std::fs::read_to_string(&env.settings.macro_core_path).unwrap();
    let worker_text = std::fs::read_to_string(&env.settings.worker_js_path).unwrap();
    let db_helper_text = std::fs::read_to_string(&env.settings.db_helper_path).unwrap();

    assert!(macro_text.contains("zzz_process_test_method"), "macro_core.rs should have the new method");
    assert!(worker_text.contains("zzz_process_test_method"), "worker.js should have the new entry");
    assert!(db_helper_text.contains("zzz_process_test_method"), "db_helper.rs should have the new fn");
}

#[test]
fn cancelling_a_check_reverts_macro_core() {
    let env = setup_with_check("sleep 5");

    let request = Request {
        action: Action::CheckAndApply,
        position: Position::Bottom,
        pasted: GOOD_METHOD.to_string(),
    };

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_for_thread = cancel.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        cancel_for_thread.store(true, Ordering::Relaxed);
    });

    let result = run(&env.settings, &request, cancel, &no_emit);

    assert!(result.is_err(), "cancelled pipeline should fail");

    let after = std::fs::read(&env.settings.macro_core_path).unwrap();
    let expected_result = original_macro_core(&env);
    assert_eq!(after, expected_result, "macro_core.rs should be reverted after cancel");
}

#[test]
fn startup_recovery_restores_a_modified_file() {
    let env = setup_with_check("true");
    let cache = &env.settings.cache_root;
    std::fs::create_dir_all(cache).unwrap();

    let target = &env.settings.macro_core_path;
    let original = std::fs::read(target).unwrap();
    let original_hash = hash_bytes(&original);

    let backup = cache.join("macro_core_backup.rs");
    std::fs::write(&backup, &original).unwrap();

    // Pretend a run died mid-check: target file is wrong, marker still there.
    std::fs::write(target, b"corrupted contents").unwrap();
    let marker = cache.join("macro_core_backup.marker");
    let marker_text = format!(
        "path={}\nhash={}\nbackup={}\n",
        target.display(),
        original_hash,
        backup.display()
    );
    std::fs::write(&marker, marker_text).unwrap();

    let recovery = guard::check_startup_marker(cache).expect("recovery should be offered");
    assert!(recovery.needs_restore);

    guard::restore_from_backup(&recovery).unwrap();

    let after = std::fs::read(target).unwrap();
    assert_eq!(after, original, "the file should have been restored from backup");
    assert!(!marker.exists(), "the marker should be gone after a successful restore");
}

#[test]
fn startup_recovery_does_nothing_when_file_matches_marker_hash() {
    let env = setup_with_check("true");
    let cache = &env.settings.cache_root;
    std::fs::create_dir_all(cache).unwrap();

    let target = &env.settings.macro_core_path;
    let original = std::fs::read(target).unwrap();
    let original_hash = hash_bytes(&original);

    let backup = cache.join("macro_core_backup.rs");
    std::fs::write(&backup, &original).unwrap();

    let marker = cache.join("macro_core_backup.marker");
    let marker_text = format!(
        "path={}\nhash={}\nbackup={}\n",
        target.display(),
        original_hash,
        backup.display()
    );
    std::fs::write(&marker, marker_text).unwrap();

    // The file on disk still hashes to what the marker says, so no recovery.
    let recovery = guard::check_startup_marker(cache);
    assert!(recovery.is_none());
    assert!(!marker.exists(), "the stale marker should be tidied up");
}
