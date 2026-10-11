// All the state the HomePage needs, plus the methods that change it.
// No drawing lives here — see view.rs for that.

use app_core::edit::Position;
use app_core::events::{Event, LogKind};
use app_core::guard::{self, StartupRecovery};
use app_core::pipeline::{self, Action, Request};
use app_core::report::Report;
use app_core::scan::{self, Method, PastedMethod};
use app_core::settings::{self, Settings};
use app_core::ui_state::{self, UiState};
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub struct HomePageState {
    pub settings: Settings,
    pub settings_open: bool,
    pub methods: Vec<Method>,
    pub scan_error: Option<String>,
    pub selection: Selection,
    pub pasted: String,
    pub parsed: Option<PastedMethod>,
    pub parse_error: Option<String>,
    pub last_parsed_input: Option<String>,
    pub run: RunState,
    pub log: Vec<LogLine>,
    pub report: Option<Report>,
    pub banner: Option<Banner>,
    pub startup_recovery: Option<StartupRecovery>,
    pub last_progress: Option<(u64, u64)>,
    pub stream_folder: app_core::linefold::LineFolder,
}

#[derive(Debug, Clone)]
pub enum Selection {
    Bottom,
    Named(String),
}

pub struct LogLine {
    pub kind: LogKind,
    pub text: String,
}

#[derive(Clone)]
pub enum Banner {
    DontCommit(String),
    Recovery,
}

pub enum RunState {
    Idle,
    Running {
        cancel: Arc<AtomicBool>,
        cancel_pressed: bool,
        events: Receiver<Event>,
        phase: String,
        /// Reload the method list when the run finishes. True for the
        /// build-everything button.
        reload_on_done: bool,
    },
}

impl HomePageState {
    pub fn new() -> Self {
        let settings = settings::load();
        let ui_state = ui_state::load();
        let selection = match ui_state.selection_name {
            Some(name) => Selection::Named(name),
            None => Selection::Bottom,
        };
        let mut state = Self {
            settings,
            settings_open: ui_state.settings_open,
            methods: Vec::new(),
            scan_error: None,
            selection,
            pasted: ui_state.paste,
            parsed: None,
            parse_error: None,
            last_parsed_input: None,
            run: RunState::Idle,
            log: Vec::new(),
            report: None,
            banner: None,
            startup_recovery: None,
            last_progress: None,
            stream_folder: app_core::linefold::LineFolder::default(),
        };
        state.check_startup_recovery();
        state.reload_methods();
        state.reparse_if_paste_changed();
        state
    }

    /// Persist UI state. Called on exit and can be called at any time.
    pub fn save_ui_state(&self) {
        let state = UiState {
            selection_name: match &self.selection {
                Selection::Bottom => None,
                Selection::Named(n) => Some(n.clone()),
            },
            paste: self.pasted.clone(),
            settings_open: self.settings_open,
        };
        let _ = ui_state::save(&state);
    }

    fn check_startup_recovery(&mut self) {
        if let Some(recovery) = guard::check_startup_marker(&self.settings.cache_root) {
            self.startup_recovery = Some(recovery);
            self.banner = Some(Banner::Recovery);
        }
    }

    pub fn save_settings(&mut self) {
        if let Err(e) = settings::save(&self.settings) {
            self.log.push(LogLine {
                kind: LogKind::Error,
                text: e,
            });
        }
    }

    pub fn reload_methods(&mut self) {
        match scan::scan_macro_core(&self.settings.macro_core_path) {
            Ok(methods) => {
                self.methods = methods;
                self.scan_error = None;
            }
            Err(e) => {
                self.methods.clear();
                self.scan_error = Some(format!("{:?}", e));
            }
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.run, RunState::Running { .. })
    }

    pub fn phase_text(&self) -> &str {
        match &self.run {
            RunState::Idle => "idle",
            RunState::Running { phase, .. } => phase,
        }
    }

    pub fn cancel_pressed(&self) -> bool {
        match &self.run {
            RunState::Idle => false,
            RunState::Running { cancel_pressed, .. } => *cancel_pressed,
        }
    }

    pub fn reparse_if_paste_changed(&mut self) {
        if self.last_parsed_input.as_deref() == Some(self.pasted.as_str()) {
            return;
        }
        self.last_parsed_input = Some(self.pasted.clone());
        if self.pasted.trim().is_empty() {
            self.parsed = None;
            self.parse_error = None;
            return;
        }
        match scan::parse_pasted_method(&self.pasted) {
            Ok(parsed) => {
                self.parsed = Some(parsed);
                self.parse_error = None;
            }
            Err(e) => {
                self.parsed = None;
                self.parse_error = Some(format!("{:?}", e));
            }
        }
    }

    pub fn current_position(&self) -> Position {
        match &self.selection {
            Selection::Bottom => Position::Bottom,
            Selection::Named(n) => Position::Named(n.clone()),
        }
    }

    pub fn start_run(&mut self, action: Action, ctx: egui::Context) {
        if self.is_running() {
            return;
        }
        if self.parsed.is_none() || self.scan_error.is_some() {
            return;
        }

        let request = Request {
            action,
            position: self.current_position(),
            pasted: self.pasted.clone(),
        };
        let settings = self.settings.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_for_thread = cancel.clone();
        let (tx, rx) = std::sync::mpsc::channel::<Event>();
        let tx_for_emit = tx.clone();
        let ctx_for_thread = ctx.clone();
        let ctx_for_closure = ctx.clone();

        self.log.clear();
        self.stream_folder.clear();
        self.report = None;

        std::thread::spawn(move || {
            let base = std::time::Instant::now();
            let last_repaint_ms = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let emit = move |e: Event| {
                let _ = tx_for_emit.send(e);
                let now_ms = base.elapsed().as_millis() as u64;
                let prev = last_repaint_ms.load(std::sync::atomic::Ordering::Relaxed);
                if now_ms.saturating_sub(prev) >= 33 {
                    last_repaint_ms.store(now_ms, std::sync::atomic::Ordering::Relaxed);
                    ctx_for_closure.request_repaint();
                }
            };
            let result = pipeline::run(&settings, &request, cancel_for_thread, &emit);
            match result {
                Ok(report) => {
                    let _ = tx.send(Event::Report(report));
                }
                Err(msg) => {
                    let _ = tx.send(Event::Log(LogKind::Error, msg));
                }
            }
            let _ = tx.send(Event::Done);
            ctx_for_thread.request_repaint();
        });

        self.run = RunState::Running {
            cancel,
            cancel_pressed: false,
            events: rx,
            phase: "starting".into(),
            reload_on_done: false,
        };
    }

    /// Run the configured "build everything and check" shell line. Streams
    /// output into the same log, reloads the method list when done.
    pub fn start_build_everything(&mut self, ctx: egui::Context) {
        if self.is_running() {
            return;
        }
        let settings = self.settings.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_for_thread = cancel.clone();
        let (tx, rx) = std::sync::mpsc::channel::<Event>();
        let tx_for_emit = tx.clone();
        let ctx_for_thread = ctx.clone();
        let ctx_for_closure = ctx.clone();

        self.log.clear();
        self.stream_folder.clear();
        self.report = None;

        std::thread::spawn(move || {
            let base = std::time::Instant::now();
            let last_repaint_ms = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let emit = move |e: Event| {
                let _ = tx_for_emit.send(e);
                let now_ms = base.elapsed().as_millis() as u64;
                let prev = last_repaint_ms.load(std::sync::atomic::Ordering::Relaxed);
                if now_ms.saturating_sub(prev) >= 33 {
                    last_repaint_ms.store(now_ms, std::sync::atomic::Ordering::Relaxed);
                    ctx_for_closure.request_repaint();
                }
            };
            emit(Event::Phase("building everything".into()));
            let result = app_core::runner::run_check(
                &settings.build_everything_command,
                &settings.working_directory,
                cancel_for_thread,
                &emit,
            );
            match result {
                Ok(outcome) if outcome.cancelled => {
                    emit(Event::Log(LogKind::Warning, "cancelled".into()));
                }
                Ok(outcome) if outcome.passed => {
                    emit(Event::Log(LogKind::Plain, "build everything: ok".into()));
                }
                Ok(_) => {
                    emit(Event::Log(LogKind::Error, "build everything: failed".into()));
                }
                Err(e) => {
                    emit(Event::Log(LogKind::Error, e));
                }
            }
            let _ = tx.send(Event::Done);
            ctx_for_thread.request_repaint();
        });

        self.run = RunState::Running {
            cancel,
            cancel_pressed: false,
            events: rx,
            phase: "building".into(),
            reload_on_done: true,
        };
    }

    pub fn request_cancel(&mut self) {
        if let RunState::Running {
            cancel,
            cancel_pressed,
            ..
        } = &mut self.run
        {
            cancel.store(true, Ordering::Relaxed);
            *cancel_pressed = true;
        }
    }

    /// Drain events from the running thread. Call once per frame.
    pub fn pump_events(&mut self) {
        let mut new_report: Option<Report> = None;
        let mut new_logs: Vec<LogLine> = Vec::new();
        let mut done = false;
        let mut reload_on_done = false;
        let mut new_chunks: Vec<String> = Vec::new();

        if let RunState::Running {
            events,
            phase,
            reload_on_done: rod,
            ..
        } = &mut self.run
        {
            reload_on_done = *rod;
            while let Ok(event) = events.try_recv() {
                match event {
                    Event::Phase(p) => {
                        *phase = p;
                    }
                    Event::Log(kind, text) => new_logs.push(LogLine { kind, text }),
                    Event::RawStream(text) => new_chunks.push(text),
                    Event::Report(r) => new_report = Some(r),
                    Event::Done => {
                        done = true;
                    }
                }
            }
        }

        for chunk in new_chunks {
            self.stream_folder.push_str(&chunk);
        }
        if let Some(p) = self.stream_folder.progress_from_current() {
            self.last_progress = Some(p);
        }

        self.log.extend(new_logs);
        if let Some(r) = new_report {
            self.report = Some(r);
        }
        if self.log.len() > 3000 {
            let overflow = self.log.len() - 3000;
            self.log.drain(0..overflow);
        }
        if done {
            self.stream_folder.flush();
            self.run = RunState::Idle;
            self.last_progress = None;
            if reload_on_done {
                self.reload_methods();
            }
        }
    }
}
