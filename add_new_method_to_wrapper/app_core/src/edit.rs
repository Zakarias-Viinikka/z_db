use crate::scan::{scan_macro_core_text, Method, ScanError};

#[derive(Debug, Clone)]
pub enum Position {
    Named(String),
    Bottom,
}

#[derive(Debug, Clone)]
pub enum EditError {
    Scan(ScanError),
    NotFound(String),
    WorkerJs(String),
    DbHelper(String),
    Tidy(String),
}

impl From<ScanError> for EditError {
    fn from(e: ScanError) -> Self {
        EditError::Scan(e)
    }
}

pub fn insert_method_into_macro_core(
    text: &str,
    position: &Position,
    pasted: &str,
) -> Result<String, EditError> {
    let methods = scan_macro_core_text(text)?;
    let target = match position {
        Position::Bottom => methods.last().cloned(),
        Position::Named(name) => methods.iter().find(|m| &m.name == name).cloned(),
    };
    let Some(target) = target else {
        let what = match position {
            Position::Bottom => "no methods in the impl block".to_string(),
            Position::Named(n) => format!("method '{}' not found", n),
        };
        return Err(EditError::NotFound(what));
    };

    let tidied = tidy_pasted_method(pasted, &target.indent);
    let mut out = String::with_capacity(text.len() + tidied.len() + 4);
    out.push_str(&text[..target.insert_offset]);
    out.push_str("\n\n");
    out.push_str(&tidied);
    out.push_str(&text[target.insert_offset..]);
    Ok(out)
}

fn tidy_pasted_method(text: &str, target_indent: &str) -> String {
    let trimmed = text.trim_end_matches('\n');
    let lines: Vec<&str> = trimmed.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let first = lines[0].trim_start();
    let rest = &lines[1..];

    let common = rest
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| *c == ' ').count())
        .min()
        .unwrap_or(0);

    let mut out = String::new();
    out.push_str(target_indent);
    out.push_str(first);
    for line in rest {
        out.push('\n');
        if line.trim().is_empty() {
            continue;
        }
        let stripped: String = line.chars().skip(common).collect();
        out.push_str(target_indent);
        out.push_str(&stripped);
    }
    out
}

pub fn add_to_worker_js(text: &str, name: &str, has_input: bool) -> Result<String, EditError> {
    let header = if has_input {
        "const serializedCommands = {"
    } else {
        "const noInputCommands = {"
    };

    let lines: Vec<&str> = text.lines().collect();
    let mut header_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == header {
            header_idx = Some(i);
            break;
        }
    }
    let Some(header_idx) = header_idx else {
        return Err(EditError::WorkerJs(format!("couldn't find '{}'", header)));
    };

    let mut close_idx: Option<usize> = None;
    for i in (header_idx + 1)..lines.len() {
        if lines[i].trim() == "};" {
            close_idx = Some(i);
            break;
        }
    }
    let Some(close_idx) = close_idx else {
        return Err(EditError::WorkerJs(format!(
            "couldn't find closing '}};' for '{}'",
            header
        )));
    };

    if close_idx == header_idx + 1 {
        return Err(EditError::WorkerJs(format!(
            "object for '{}' is empty; nothing to append after",
            header
        )));
    }

    let prev_line = lines[close_idx - 1];
    let code = match prev_line.find("//") {
        Some(i) => &prev_line[..i],
        None => prev_line,
    };
    if !code.trim_end().ends_with(',') {
        return Err(EditError::WorkerJs(format!(
            "entry above '}};' in '{}' doesn't end with a comma: {:?}",
            header, prev_line
        )));
    }

    let indent: String = lines[close_idx - 1]
        .chars()
        .take_while(|c| *c == ' ')
        .collect();

    let entry = if has_input {
        format!("{}{}: (msg) => db_manager.{}(msg[1]),", indent, name, name)
    } else {
        format!("{}{}: () => db_manager.{}(),", indent, name, name)
    };

    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i == close_idx {
            out.push_str(&entry);
            out.push('\n');
        }
        out.push_str(line);
        out.push('\n');
    }
    if !text.ends_with('\n') {
        out.pop();
    }
    Ok(out)
}

const HEADERS: [&str; 4] = [
    "// no input, output",
    "// no input, no output",
    "// input, no output",
    "// input, output",
];

pub fn add_to_db_helper(
    text: &str,
    name: &str,
    input_type: Option<&str>,
    output_type: Option<&str>,
) -> Result<String, EditError> {
    let header = match (input_type.is_some(), output_type.is_some()) {
        (false, true) => HEADERS[0],
        (false, false) => HEADERS[1],
        (true, false) => HEADERS[2],
        (true, true) => HEADERS[3],
    };

    let lines: Vec<&str> = text.lines().collect();
    let mut header_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == header {
            header_idx = Some(i);
            break;
        }
    }
    let Some(header_idx) = header_idx else {
        return Err(EditError::DbHelper(format!(
            "couldn't find section header '{}'",
            header
        )));
    };

    let mut next_header_idx: Option<usize> = None;
    for i in (header_idx + 1)..lines.len() {
        if HEADERS.iter().any(|h| lines[i].trim() == *h) {
            next_header_idx = Some(i);
            break;
        }
    }

    let fn_text = build_db_helper_fn(name, input_type, output_type);

    let section_is_empty = match next_header_idx {
        Some(n) => lines[(header_idx + 1)..n].iter().all(|l| l.trim().is_empty()),
        None => lines[(header_idx + 1)..].iter().all(|l| l.trim().is_empty()),
    };

    let mut out = String::new();

    if section_is_empty {
        for (i, line) in lines.iter().enumerate() {
            out.push_str(line);
            out.push('\n');
            if i == header_idx {
                out.push_str(&fn_text);
            }
        }
    } else {
        let insert_before = match next_header_idx {
            Some(n) => n,
            None => lines.len(),
        };
        for (i, line) in lines.iter().enumerate() {
            if i == insert_before {
                out.push_str(&fn_text);
                out.push('\n');
            }
            out.push_str(line);
            out.push('\n');
        }
        if insert_before >= lines.len() {
            out.push_str(&fn_text);
            out.push('\n');
        }
    }

    if !text.ends_with('\n') {
        out.pop();
    }
    Ok(out)
}

fn build_db_helper_fn(name: &str, input_type: Option<&str>, output_type: Option<&str>) -> String {
    match (input_type, output_type) {
        (None, Some(out)) => format!(
            "pub async fn {}() -> Result<{}, DbError> {{\n    let bytes = tell_worker_to_do(\"{}\", None).await?;\n    {}::un_payloadify(&bytes)\n}}\n",
            name, out, name, out
        ),
        (None, None) => format!(
            "pub async fn {}() -> Result<(), DbError> {{\n    let bytes = tell_worker_to_do(\"{}\", None).await?;\n    Result::<(), DbError>::un_payloadify(&bytes)?\n}}\n",
            name, name
        ),
        (Some(inp), None) => format!(
            "pub async fn {}(input: {}) -> Result<(), DbError> {{\n    let bytes = tell_worker_to_do(\"{}\", Some(input.to_payload())).await?;\n    Result::<(), DbError>::un_payloadify(&bytes)?\n}}\n",
            name, inp, name
        ),
        (Some(inp), Some(out)) => format!(
            "pub async fn {}(\n    input: {},\n) -> Result<{}, DbError> {{\n    let bytes = tell_worker_to_do(\"{}\", Some(input.to_payload())).await?;\n    {}::un_payloadify(&bytes)\n}}\n",
            name, inp, out, name, out
        ),
    }
}

#[allow(dead_code)]
fn _unused(_: &Method) {}
