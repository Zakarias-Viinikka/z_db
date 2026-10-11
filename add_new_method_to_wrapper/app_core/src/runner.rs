use crate::events::Event;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct RunOutcome {
    pub passed: bool,
    pub cancelled: bool,
}

enum RunnerEvent {
    Chunk(String),
}

fn find_local_cargo(start: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("local_cargo.sh");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn resolve_cargo_wrapper(
    command_line: &str,
    working_directory: &std::path::Path,
) -> String {
    let Some(rest) = command_line.strip_prefix("cargo ") else {
        return command_line.to_string();
    };
    let Some(wrapper) = find_local_cargo(working_directory) else {
        return command_line.to_string();
    };
    let wrapper = wrapper.to_string_lossy();
    format!("\"{}\" {}", wrapper, rest)
}

/// Read any `export VAR="..."` or `export VAR=...` lines from a
/// local_cargo.sh and return them as (name, value) pairs. Everything the
/// app spawns — including the .sh scripts it calls — inherits these, so
/// the RUSTFLAGS the wrapper sets apply whether the wrapper is invoked
/// explicitly or not.
fn env_from_local_cargo(start: &std::path::Path) -> Vec<(String, String)> {
    let Some(path) = find_local_cargo(start) else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("export ") else {
            continue;
        };
        let Some((name, value)) = rest.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
            .unwrap_or(value);
        out.push((name.to_string(), value.to_string()));
    }
    out
}

pub fn run_check(
    command_line: &str,
    working_directory: &std::path::Path,
    cancel: Arc<AtomicBool>,
    emit: &dyn Fn(Event),
) -> Result<RunOutcome, String> {
    let resolved = resolve_cargo_wrapper(command_line, working_directory);
    emit(Event::RawStream(format!("$ {}\n", resolved)));

    let mut child = Command::new("sh");
    child
        .arg("-c")
        .arg(&resolved)
        .current_dir(working_directory)
        .env("CARGO_TERM_COLOR", "never")
        .env("NO_COLOR", "1")
        .env("CARGO_TERM_PROGRESS_WHEN", "always")
        .env("CARGO_TERM_PROGRESS_WIDTH", "60")
        .env("TERM", "xterm-256color");
    for (name, value) in env_from_local_cargo(working_directory) {
        child.env(name, value);
    }
    let mut child = child
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|e| format!("couldn't spawn '{}': {}", command_line, e))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<RunnerEvent>();
    let tx2 = tx.clone();

    let t_out = std::thread::spawn(move || stream_lines(stdout, tx));
    let t_err = std::thread::spawn(move || stream_lines(stderr, tx2));

    let pgid = child.id() as i32;
    let mut cancelled = false;
    let start = Instant::now();

    loop {
        while let Ok(ev) = rx.try_recv() {
            forward(ev, emit);
        }

        if cancel.load(Ordering::Relaxed) && !cancelled {
            cancelled = true;
            unsafe {
                libc::kill(-pgid, libc::SIGTERM);
            }
            let kill_start = Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if kill_start.elapsed() > Duration::from_secs(3) {
                            unsafe {
                                libc::kill(-pgid, libc::SIGKILL);
                            }
                            let _ = child.wait();
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
            break;
        }

        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(format!("couldn't wait on '{}': {}", command_line, e)),
        }

        if start.elapsed() > Duration::from_secs(1800) {
            unsafe {
                libc::kill(-pgid, libc::SIGKILL);
            }
            let _ = child.wait();
            return Err(format!("'{}' exceeded 30 minutes", command_line));
        }
    }

    let status = child.wait().map_err(|e| format!("wait failed: {}", e))?;
    let _ = t_out.join();
    let _ = t_err.join();

    while let Ok(ev) = rx.try_recv() {
        forward(ev, emit);
    }

    Ok(RunOutcome {
        passed: status.success(),
        cancelled,
    })
}

fn forward(ev: RunnerEvent, emit: &dyn Fn(Event)) {
    match ev {
        RunnerEvent::Chunk(text) => emit(Event::RawStream(text)),
    }
}

/// Read a stream line by line, split on `\n` only (like before the
/// progress work). A line that contains a cargo progress fraction
/// (`] NNN/MMM`) becomes a Progress event and doesn't hit the log.
/// Everything else is logged normally.
fn stream_lines<R: Read>(mut reader: R, tx: std::sync::mpsc::Sender<RunnerEvent>) {
    let mut carry: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                let mut buf = std::mem::take(&mut carry);
                buf.extend_from_slice(&chunk[..n]);
                let valid_up_to = match std::str::from_utf8(&buf) {
                    Ok(_) => buf.len(),
                    Err(e) => e.valid_up_to(),
                };
                let tail = buf.len() - valid_up_to;
                if tail > 0 && tail < 4 {
                    carry = buf[valid_up_to..].to_vec();
                } else {
                    carry.clear();
                }
                let text = String::from_utf8_lossy(&buf[..valid_up_to]).into_owned();
                let text = strip_ansi(&text);
                if !text.is_empty() && tx.send(RunnerEvent::Chunk(text)).is_err() {
                    return;
                }
            }
            Err(_) => break,
        }
    }
}


pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('[') => {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if ('\x40'..='\x7e').contains(&nc) {
                        break;
                    }
                }
            }
            Some(_) => {
                chars.next();
            }
            None => {}
        }
    }
    out
}