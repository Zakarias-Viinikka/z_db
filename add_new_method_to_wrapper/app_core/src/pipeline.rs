use crate::edit::{self, EditError, Position};
use crate::events::{Event, LogKind};
use crate::guard::Guard;
use crate::precheck::{self, PreCheckInput};
use crate::report::{Report, Snippet};
use crate::runner;
use crate::scan::{self, PasteError};
use crate::settings::Settings;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    CheckOnly,
    CheckAndApply,
}

pub struct Request {
    pub action: Action,
    pub position: Position,
    pub pasted: String,
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn read_bytes(p: &PathBuf) -> Result<Vec<u8>, String> {
    std::fs::read(p).map_err(|e| format!("couldn't read {}: {}", p.display(), e))
}

fn read_text(p: &PathBuf) -> Result<String, String> {
    let b = read_bytes(p)?;
    String::from_utf8(b).map_err(|e| format!("{} isn't utf-8: {}", p.display(), e))
}

pub fn run(
    settings: &Settings,
    request: &Request,
    cancel: Arc<AtomicBool>,
    emit: &dyn Fn(Event),
) -> Result<Report, String> {
    let paths = settings.resolved();

    emit(Event::Phase("reading files".into()));

    let macro_core_bytes = read_bytes(&paths.macro_core_path)?;
    let macro_core_text = String::from_utf8(macro_core_bytes.clone())
        .map_err(|e| format!("macro_core.rs isn't utf-8: {}", e))?;
    let worker_js_text = read_text(&paths.worker_js_path)?;
    let db_helper_text = read_text(&paths.db_helper_path)?;
    let payload_text = read_text(&paths.payload_path)?;

    emit(Event::Phase("parsing pasted method".into()));
    let pasted = scan::parse_pasted_method(&request.pasted).map_err(|e| describe_paste(&e))?;

    emit(Event::Phase("scanning macro_core.rs".into()));
    let methods = scan::scan_macro_core_text(&macro_core_text)
        .map_err(|e| format!("couldn't scan macro_core.rs: {:?}", e))?;

    emit(Event::Phase("pre-checking".into()));
    let problems = precheck::precheck(PreCheckInput {
        name: &pasted.name,
        input_type: pasted.input_type.as_deref(),
        output_type: pasted.output_type.as_deref(),
        macro_core_text: &macro_core_text,
        worker_js_text: &worker_js_text,
        db_helper_text: &db_helper_text,
        payload_text: &payload_text,
    });
    if !problems.is_empty() {
        for p in &problems {
            emit(Event::Log(LogKind::Error, p.clone()));
        }
        return Err(format!("pre-check found {} problem(s)", problems.len()));
    }

    // Resolve the position now, so an early error doesn't touch disk
    match &request.position {
        Position::Named(n) => {
            if !methods.iter().any(|m| &m.name == n) {
                return Err(format!("method '{}' not found in macro_core.rs", n));
            }
        }
        Position::Bottom => {}
    }

    emit(Event::Phase("building new macro_core.rs".into()));
    let new_macro_core = edit::insert_method_into_macro_core(
        &macro_core_text,
        &request.position,
        &request.pasted,
    )
    .map_err(describe_edit)?;

    let mut guard = Guard::arm(
        &paths.macro_core_path,
        macro_core_bytes.clone(),
        &settings.cache_root,
    )
    .map_err(|e| format!("couldn't arm guard: {:?}", e))?;

    emit(Event::Phase("writing macro_core.rs for the check".into()));
    std::fs::write(&paths.macro_core_path, new_macro_core.as_bytes())
        .map_err(|e| format!("couldn't write macro_core.rs: {}", e))?;
    guard.written_hash = Some(hash_bytes(new_macro_core.as_bytes()));

    emit(Event::Phase("running checks".into()));
    let mut all_passed = true;
    let mut cancelled = false;
    for cmd in settings.check_commands.iter().filter(|c| c.enabled) {
        let outcome = runner::run_check(
            &cmd.command_line,
            &paths.working_directory,
            cancel.clone(),
            emit,
        )?;
        if outcome.cancelled {
            cancelled = true;
            all_passed = false;
            break;
        }
        if !outcome.passed {
            emit(Event::Log(LogKind::Error, format!("'{}' failed", cmd.label)));
            all_passed = false;
            break;
        }
    }

    if settings.check_commands.iter().all(|c| !c.enabled) {
        return Err("no check commands enabled".into());
    }

    emit(Event::Phase("restoring macro_core.rs".into()));
    if let Err(e) = guard.finish() {
        emit(Event::Log(LogKind::Error, format!("restore failed: {:?}", e)));
        return Err("restore failed; see log".into());
    }

    let _ = &guard; // keep guard alive past the finish call

    if request.action == Action::CheckOnly {
        if !all_passed {
            emit(Event::Phase("check failed".into()));
            return Err("checks didn't pass".into());
        }
        emit(Event::Phase("done (check only)".into()));
        return Ok(Report { snippets: vec![] });
    }

    if !all_passed {
        emit(Event::Phase("not applying: checks didn't pass".into()));
        return Err("checks didn't pass; nothing applied".into());
    }

    if guard.restore_problem.is_some() {
        return Err("restore verification failed; not applying".into());
    }

    if cancelled {
        return Err("cancelled".into());
    }

    emit(Event::Phase("building worker.js and db_helper.rs".into()));
    let new_worker_js = edit::add_to_worker_js(
        &worker_js_text,
        &pasted.name,
        pasted.input_type.is_some(),
    )
    .map_err(describe_edit)?;
    let new_db_helper = edit::add_to_db_helper(
        &db_helper_text,
        &pasted.name,
        pasted.input_type.as_deref(),
        pasted.output_type.as_deref(),
    )
    .map_err(describe_edit)?;

    // re-read macro_core to make sure nothing changed in the meantime
    let current_macro_core = read_bytes(&paths.macro_core_path)?;
    if current_macro_core != macro_core_bytes {
        return Err("macro_core.rs changed since the check; not applying".into());
    }

    // sanity: each new name appears exactly once in each new text
    for (label, text) in [
        ("macro_core.rs", &new_macro_core),
        ("worker.js", &new_worker_js),
        ("db_helper.rs", &new_db_helper),
    ] {
        let count = text.matches(&format!("fn {}(", pasted.name)).count()
            + text.matches(&format!("fn {} (", pasted.name)).count();
        let js_count = text
            .matches(&format!("{}: ", pasted.name))
            .count();
        if label == "worker.js" {
            if js_count == 0 {
                return Err(format!("'{}' didn't land in {}", pasted.name, label));
            }
        } else if count == 0 {
            return Err(format!("'{}' didn't land in {}", pasted.name, label));
        }
    }

    emit(Event::Phase("writing the three files".into()));
    let apply_dir = settings.cache_root.join("apply_backup");
    std::fs::create_dir_all(&apply_dir)
        .map_err(|e| format!("couldn't create {}: {}", apply_dir.display(), e))?;
    let macro_core_backup = apply_dir.join("macro_core.rs");
    let worker_js_backup = apply_dir.join("worker.js");
    let db_helper_backup = apply_dir.join("db_helper.rs");
    std::fs::write(&macro_core_backup, &macro_core_bytes)
        .map_err(|e| format!("couldn't back up macro_core: {}", e))?;
    std::fs::write(&worker_js_backup, worker_js_text.as_bytes())
        .map_err(|e| format!("couldn't back up worker.js: {}", e))?;
    std::fs::write(&db_helper_backup, db_helper_text.as_bytes())
        .map_err(|e| format!("couldn't back up db_helper.rs: {}", e))?;

    let writes: [(PathBuf, &str); 3] = [
        (paths.macro_core_path.clone(), new_macro_core.as_str()),
        (paths.worker_js_path.clone(), new_worker_js.as_str()),
        (paths.db_helper_path.clone(), new_db_helper.as_str()),
    ];

    let mut done: Vec<PathBuf> = Vec::new();
    for (path, text) in &writes {
        if let Err(e) = std::fs::write(path, text.as_bytes()) {
            emit(Event::Log(
                LogKind::Error,
                format!("write to {} failed: {}", path.display(), e),
            ));
            // roll back the ones already written
            for p in &done {
                let originals: Option<&[u8]> = match p.file_name().and_then(|n| n.to_str()) {
                    Some("macro_core.rs") => Some(macro_core_bytes.as_slice()),
                    Some("worker.js") => Some(worker_js_text.as_bytes()),
                    Some("db_helper.rs") => Some(db_helper_text.as_bytes()),
                    _ => None,
                };
                if let Some(bytes) = originals {
                    let _ = std::fs::write(p, bytes);
                }
            }
            return Err("a write failed; rolled back".into());
        }
        done.push(path.clone());
    }

    let report = Report {
        snippets: vec![
            Snippet {
                file: paths.macro_core_path.display().to_string(),
                name: pasted.name.clone(),
            },
            Snippet {
                file: paths.worker_js_path.display().to_string(),
                name: pasted.name.clone(),
            },
            Snippet {
                file: paths.db_helper_path.display().to_string(),
                name: pasted.name.clone(),
            },
        ],
    };

    let _ = cancel.load(Ordering::Relaxed);

    emit(Event::Phase("done".into()));
    Ok(report)
}

fn describe_paste(e: &PasteError) -> String {
    match e {
        PasteError::Tokenize(s) => format!("couldn't tokenize pasted text: {}", s),
        PasteError::NotOneMethod => "pasted text must be exactly one fn".into(),
        PasteError::MissingSelf => "the fn must take &self first".into(),
        PasteError::BadInputParam(s) => format!("bad input parameter: {}", s),
        PasteError::BadReturn(s) => format!("bad return type: {}", s),
    }
}

fn describe_edit(e: EditError) -> String {
    match e {
        EditError::Scan(s) => format!("scan error: {:?}", s),
        EditError::NotFound(s) => s,
        EditError::WorkerJs(s) => format!("worker.js: {}", s),
        EditError::DbHelper(s) => format!("db_helper.rs: {}", s),
        EditError::Tidy(s) => format!("tidy: {}", s),
    }
}
