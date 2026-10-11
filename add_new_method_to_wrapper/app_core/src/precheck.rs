pub struct PreCheckInput<'a> {
    pub name: &'a str,
    pub input_type: Option<&'a str>,
    pub output_type: Option<&'a str>,
    pub macro_core_text: &'a str,
    pub worker_js_text: &'a str,
    pub db_helper_text: &'a str,
    pub payload_text: &'a str,
}

pub fn precheck(input: PreCheckInput) -> Vec<String> {
    let mut problems = Vec::new();

    if name_exists_as_method(input.macro_core_text, input.name) {
        problems.push(format!("'{}' already exists as a method in macro_core.rs", input.name));
    }
    if let Some(where_) = name_exists_in_worker_js(input.worker_js_text, input.name) {
        problems.push(format!("'{}' already exists as a key in worker.js ({})", input.name, where_));
    }
    if name_exists_as_fn(input.db_helper_text, input.name) {
        problems.push(format!("'{}' already exists as a fn in db_helper.rs", input.name));
    }

    for ty in [input.input_type, input.output_type].into_iter().flatten() {
        if !type_exists_in_payload(input.payload_text, ty) {
            problems.push(format!("type '{}' is not declared in payload.rs", ty));
        }
    }

    problems
}

fn name_exists_as_method(macro_core: &str, name: &str) -> bool {
    use crate::scan::scan_macro_core_text;
    match scan_macro_core_text(macro_core) {
        Ok(methods) => methods.iter().any(|m| m.name == name),
        Err(_) => false,
    }
}

fn name_exists_in_worker_js(js: &str, name: &str) -> Option<&'static str> {
    let mut section: Option<&'static str> = None;
    for line in js.lines() {
        let t = line.trim();
        if t == "const serializedCommands = {" {
            section = Some("serializedCommands");
            continue;
        }
        if t == "const noInputCommands = {" {
            section = Some("noInputCommands");
            continue;
        }
        if t == "};" {
            section = None;
            continue;
        }
        if let Some(sec) = section {
            if let Some(colon) = line.find(':') {
                let key = line[..colon].trim();
                let key = key.rsplit(' ').next().unwrap_or(key);
                if key == name {
                    return Some(sec);
                }
            }
        }
    }
    None
}

fn name_exists_as_fn(db_helper: &str, name: &str) -> bool {
    let needle = format!("fn {}(", name);
    let needle2 = format!("fn {} (", name);
    db_helper.contains(&needle) || db_helper.contains(&needle2)
}

fn type_exists_in_payload(payload: &str, ty: &str) -> bool {
    for kw in ["pub struct ", "pub enum ", "pub type "] {
        let needle = format!("{}{}", kw, ty);
        for line in payload.lines() {
            if line.trim_start().starts_with(&needle) {
                return true;
            }
        }
    }
    false
}
