use crate::report::Report;

#[derive(Debug, Clone)]
pub enum LogKind {
    Plain,
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub enum Event {
    Phase(String),
    /// A one-shot line to add to the log (errors from the pipeline itself,
    /// summaries, "build everything: ok").
    Log(LogKind, String),
    /// Raw text from a running command's stdout/stderr. Callers fold this
    /// into a LineFolder — `\r` replaces the current line, `\n` commits.
    RawStream(String),
    Report(Report),
    Done,
}