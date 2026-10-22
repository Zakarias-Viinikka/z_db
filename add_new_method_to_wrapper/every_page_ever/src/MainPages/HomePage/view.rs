use app_core::events::LogKind;
use app_core::pipeline::Action;
use app_core::scan::PastedMethod;
use eframe::egui;
use std::path::PathBuf;

use super::state::{Banner, HomePageState, Selection};

// ---- Primer-ish palette ----
const BG: egui::Color32 = egui::Color32::from_rgb(0x0d, 0x11, 0x17);
const PANEL: egui::Color32 = egui::Color32::from_rgb(0x0d, 0x11, 0x17);
const PANEL_HEAD: egui::Color32 = egui::Color32::from_rgb(0x16, 0x1b, 0x22);
const BORDER: egui::Color32 = egui::Color32::from_rgb(0x30, 0x36, 0x3d);
const FIELD_BG: egui::Color32 = egui::Color32::from_rgb(0x01, 0x04, 0x09);
const TEXT: egui::Color32 = egui::Color32::from_rgb(0xc9, 0xd1, 0xd9);
const TEXT_HEAD: egui::Color32 = egui::Color32::from_rgb(0xf0, 0xf6, 0xfc);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(0x8b, 0x94, 0x9e);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x1f, 0x6f, 0xeb);
const GREEN_BTN: egui::Color32 = egui::Color32::from_rgb(0x23, 0x86, 0x36);
const GREEN_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(0x2e, 0xa0, 0x43);
const GHOST_BTN: egui::Color32 = egui::Color32::from_rgb(0x21, 0x26, 0x2d);
const GHOST_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(0x30, 0x36, 0x3d);
const OK: egui::Color32 = egui::Color32::from_rgb(0x3f, 0xb9, 0x50);
const ERR: egui::Color32 = egui::Color32::from_rgb(0xf8, 0x51, 0x49);
const WARN: egui::Color32 = egui::Color32::from_rgb(0xd2, 0x99, 0x22);
const DIM_LOG: egui::Color32 = egui::Color32::from_rgb(0x6e, 0x76, 0x81);

pub fn install_primer_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = FIELD_BG;
    visuals.faint_bg_color = PANEL_HEAD;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = ACCENT;
    visuals.hyperlink_color = ACCENT;
    visuals.window_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.bg_fill = GHOST_BTN;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.bg_fill = GHOST_BTN_HOVER;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, ACCENT);
    ctx.set_visuals(visuals);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.spacing.interact_size.y = 26.0;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.5));
        style
            .text_styles
            .insert(egui::TextStyle::Small, egui::FontId::proportional(13.0));
    });
}

pub fn draw(ui: &mut egui::Ui, state: &mut HomePageState) {
    state.pump_events();
    if state.is_running() {
        ui.ctx().request_repaint();
    }

    egui::ScrollArea::vertical()
        .id_salt("page_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            draw_header(ui, state);
            draw_banner(ui, state);
            draw_settings(ui, state);
            draw_section(ui, "insert position", |ui| draw_picker(ui, state));
            draw_section(ui, "paste one method", |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let button_col_w = 280.0;
                    let gap = 12.0;
                    let paste_w = (ui.available_width() - button_col_w - gap).max(200.0);
                    ui.allocate_ui(egui::vec2(paste_w, ui.available_height()), |ui| {
                        ui.set_width(paste_w);
                        draw_paste_box(ui, state);
                    });
                    ui.add_space(gap);
                    ui.vertical(|ui| {
                        ui.set_width(button_col_w);
                        draw_buttons(ui, state);
                    });
                });
                draw_understood(ui, state);
            });
            draw_phase(ui, state);
            draw_progress(ui, state);
            draw_section(ui, "log", |ui| draw_log(ui, state));
            draw_report(ui, state);
            ui.add_space(40.0);
        });
}

fn draw_progress(ui: &mut egui::Ui, state: &HomePageState) {
    let Some((current, total)) = state.last_progress else {
        return;
    };
    let frac = if total == 0 {
        0.0
    } else {
        current as f32 / total as f32
    };
    ui.add(
        egui::ProgressBar::new(frac)
            .text(
                egui::RichText::new(format!("{current} / {total}"))
                    .color(egui::Color32::WHITE)
                    .size(13.0),
            )
            .desired_width(ui.available_width())
            .fill(ACCENT),
    );
    ui.add_space(8.0);
}

fn draw_header(ui: &mut egui::Ui, state: &HomePageState) {
    ui.add_space(4.0);

    let (text, color, bg) = if state.settings.use_mock_paths {
        (
            "Test mode — mock files only, nothing important will be affected",
            OK,
            egui::Color32::from_rgb(0x0d, 0x2b, 0x12),
        )
    } else {
        (
            "Live mode — real files will be affected",
            WARN,
            egui::Color32::from_rgb(0x3a, 0x2d, 0x00),
        )
    };
    ui.horizontal(|ui| {
        // Center the badge across the panel width.
        let full_w = ui.available_width();
        let font = egui::FontId::proportional(14.0);
        let galley = ui
            .painter()
            .layout_no_wrap(text.to_string(), font.clone(), color);
        let pad = egui::vec2(14.0, 6.0);
        let badge_w = galley.size().x + pad.x * 2.0;
        let badge_h = galley.size().y + pad.y * 2.0;
        let left = ((full_w - badge_w) / 2.0).max(0.0);
        ui.add_space(left);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(badge_w, badge_h), egui::Sense::hover());
        ui.painter().rect_filled(rect, 6.0, bg);
        ui.painter()
            .text(rect.center(), egui::Align2::CENTER_CENTER, text, font, color);
    });

    ui.add_space(8.0);
}

fn draw_banner(ui: &mut egui::Ui, state: &mut HomePageState) {
    let banner = state.banner.clone();
    let Some(banner) = banner else { return };
    match banner {
        Banner::DontCommit(msg) => {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(0x3a, 0x0d, 0x0d))
                .stroke(egui::Stroke::new(1.0, ERR))
                .corner_radius(6.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("DON'T COMMIT — {msg}"))
                            .color(ERR)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "backup at {}",
                            state
                                .settings
                                .cache_root
                                .join("macro_core_backup.rs")
                                .display()
                        ))
                        .color(TEXT_DIM)
                        .size(13.0),
                    );
                });
        }
        Banner::Recovery => {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(0x3a, 0x2d, 0x00))
                .stroke(egui::Stroke::new(1.0, WARN))
                .corner_radius(6.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("zapcli died mid-check. A backup exists.")
                            .color(WARN)
                            .strong(),
                    );
                    ui.horizontal(|ui| {
                        if green_button(ui, "Restore from backup").clicked() {
                            let result = state
                                .startup_recovery
                                .as_ref()
                                .map(app_core::guard::restore_from_backup);
                            if let Some(Err(e)) = result {
                                state.log.push(super::state::LogLine {
                                    kind: LogKind::Error,
                                    text: e,
                                });
                            }
                            state.startup_recovery = None;
                            state.banner = None;
                            state.reload_methods();
                        }
                        if ghost_button(ui, "Ignore").clicked() {
                            if let Some(rec) = &state.startup_recovery {
                                app_core::guard::ignore_recovery(rec);
                            }
                            state.startup_recovery = None;
                            state.banner = None;
                        }
                    });
                });
        }
    }
    ui.add_space(8.0);
}

fn draw_section(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(6.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::Frame::new()
                .fill(PANEL_HEAD)
                .corner_radius(egui::CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                })
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        egui::RichText::new(title)
                            .color(TEXT_HEAD)
                            .strong()
                            .size(14.0),
                    );
                });
            egui::Frame::new()
                .inner_margin(14.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    body(ui);
                });
        });
    ui.add_space(10.0);
}

fn draw_settings(ui: &mut egui::Ui, state: &mut HomePageState) {
    let mut changed = false;
    let mut open = state.settings_open;
    egui::Frame::new()
        .fill(PANEL)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(6.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let header = egui::CollapsingHeader::new(
                egui::RichText::new("settings").color(TEXT_HEAD).strong().size(14.0),
            )
            .default_open(open);
            let resp = header.show(ui, |ui| {
                let running = state.is_running();
                ui.add_enabled_ui(!running, |ui| {
                    changed |= draw_path_row(ui, "macro_core.rs", &mut state.settings.macro_core_path);
                    changed |= draw_path_row(ui, "worker.js", &mut state.settings.worker_js_path);
                    changed |= draw_path_row(ui, "db_helper.rs", &mut state.settings.db_helper_path);
                    changed |= draw_path_row(ui, "payload.rs", &mut state.settings.payload_path);
                    changed |= draw_path_row(ui, "working dir", &mut state.settings.working_directory);

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    if ui
                        .checkbox(&mut state.settings.use_mock_paths, "use mock paths")
                        .changed()
                    {
                        changed = true;
                    }
                    changed |= draw_path_row(ui, "mock root", &mut state.settings.mock_root);
                    if ui.button("populate mock folder").clicked() {
                        if let Err(e) = app_core::settings::populate_mock_folder(&state.settings) {
                            state.log.push(super::state::LogLine {
                                kind: LogKind::Error,
                                text: e,
                            });
                        }
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.label(egui::RichText::new("check commands").color(TEXT_DIM).size(13.0));
                    let mut remove_at: Option<usize> = None;
                    for (i, cmd) in state.settings.check_commands.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut cmd.enabled, "").changed() {
                                changed = true;
                            }
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut cmd.label)
                                        .desired_width(140.0),
                                )
                                .changed()
                            {
                                changed = true;
                            }
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut cmd.command_line)
                                        .desired_width(ui.available_width() - 40.0),
                                )
                                .changed()
                            {
                                changed = true;
                            }
                            if ui.button("×").clicked() {
                                remove_at = Some(i);
                            }
                        });
                    }
                    if let Some(i) = remove_at {
                        state.settings.check_commands.remove(i);
                        changed = true;
                    }
                    if ui.button("add command").clicked() {
                        state.settings.check_commands.push(
                            app_core::settings::CheckCommand {
                                label: "new".into(),
                                command_line: "cargo check".into(),
                                enabled: true,
                            },
                        );
                        changed = true;
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.label(
                        egui::RichText::new("build everything command")
                            .color(TEXT_DIM)
                            .size(13.0),
                    );
                    if ui
                        .add(
                            egui::TextEdit::singleline(
                                &mut state.settings.build_everything_command,
                            )
                            .desired_width(ui.available_width()),
                        )
                        .changed()
                    {
                        changed = true;
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.label(
                        egui::RichText::new("post-build scripts (run after build succeeds)")
                            .color(TEXT_DIM)
                            .size(13.0),
                    );
                    let mut remove_script_at: Option<usize> = None;
                    for (i, script) in state.settings.post_build_scripts.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            let reserved = 90.0;
                            let edit_w = (ui.available_width() - reserved).max(120.0);
                            let mut s = script.to_string_lossy().to_string();
                            if ui
                                .add(egui::TextEdit::singleline(&mut s).desired_width(edit_w))
                                .changed()
                            {
                                *script = PathBuf::from(s);
                                changed = true;
                            }
                            let exists = script.exists();
                            ui.label(
                                egui::RichText::new(if exists { "✓" } else { "✗" })
                                    .color(if exists { OK } else { ERR })
                                    .strong(),
                            );
                            if ui.button("×").clicked() {
                                remove_script_at = Some(i);
                            }
                        });
                    }
                    if let Some(i) = remove_script_at {
                        state.settings.post_build_scripts.remove(i);
                        changed = true;
                    }
                    if ui.button("add script").clicked() {
                        state.settings.post_build_scripts.push(PathBuf::new());
                        changed = true;
                    }
                    ui.add_space(10.0);
                });
            });
            open = resp.openness > 0.5;
        });
    state.settings_open = open;
    if changed {
        state.save_settings();
    }
    ui.add_space(10.0);
}

fn draw_path_row(ui: &mut egui::Ui, label: &str, path: &mut PathBuf) -> bool {
    let mut s = path.to_string_lossy().to_string();
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_sized(
            egui::vec2(120.0, 24.0),
            egui::Label::new(egui::RichText::new(label).color(TEXT_DIM).size(14.0)),
        );
        if ui
            .add(egui::TextEdit::singleline(&mut s).desired_width(ui.available_width() - 30.0))
            .changed()
        {
            *path = PathBuf::from(s);
            changed = true;
        }
        let exists = path.exists();
        ui.label(
            egui::RichText::new(if exists { "✓" } else { "✗" })
                .color(if exists { OK } else { ERR })
                .strong(),
        );
    });
    changed
}

fn draw_picker(ui: &mut egui::Ui, state: &mut HomePageState) {
    ui.horizontal(|ui| {
        let current = match &state.selection {
            Selection::Bottom => "append to bottom".to_string(),
            Selection::Named(n) => n.clone(),
        };
        let enabled = !state.is_running() && state.scan_error.is_none();
        ui.add_enabled_ui(enabled, |ui| {
            egui::ComboBox::from_id_salt("position_picker")
                .selected_text(current)
                .width(ui.available_width() - 90.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            matches!(state.selection, Selection::Bottom),
                            "append to bottom",
                        )
                        .clicked()
                    {
                        state.selection = Selection::Bottom;
                        state.save_ui_state();
                    }
                    for m in &state.methods {
                        let selected =
                            matches!(&state.selection, Selection::Named(n) if n == &m.name);
                        if ui.selectable_label(selected, &m.name).clicked() {
                            state.selection = Selection::Named(m.name.clone());
                            state.save_ui_state();
                        }
                    }
                });
        });
        if ghost_button(ui, "reload")
            .clicked()
        {
            state.reload_methods();
        }
    });
    if let Some(err) = &state.scan_error {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(format!("scan error: {err}")).color(ERR));
    }
}

fn draw_paste_box(ui: &mut egui::Ui, state: &mut HomePageState) {
    let enabled = !state.is_running();
    ui.add_enabled_ui(enabled, |ui| {
        let response = ui.add(
            egui::TextEdit::multiline(&mut state.pasted)
                .desired_rows(10)
                .code_editor()
                .desired_width(f32::INFINITY),
        );
        if response.changed() {
            state.reparse_if_paste_changed();
        }
    });
    state.reparse_if_paste_changed();
}

fn draw_understood(ui: &mut egui::Ui, state: &HomePageState) {
    ui.add_space(10.0);
    if let Some(err) = &state.parse_error {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x3a, 0x0d, 0x0d))
            .stroke(egui::Stroke::new(1.0, ERR))
            .corner_radius(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(format!("parse error: {err}")).color(ERR));
            });
        return;
    }
    let Some(parsed) = &state.parsed else {
        ui.label(egui::RichText::new("(nothing pasted yet)").color(TEXT_DIM).italics());
        return;
    };
    let rows = describe_understood_rows(parsed);
    egui::Frame::new()
        .fill(PANEL_HEAD)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(6.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (k, v) in rows {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        egui::vec2(160.0, 20.0),
                        egui::Label::new(egui::RichText::new(k).color(TEXT_DIM).size(13.5)),
                    );
                    ui.label(egui::RichText::new(v).color(TEXT).strong().size(13.5));
                });
            }
        });
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(
            "note: web_mascot.rs uses different helper macros than the others, so a method that passes the testing check can still fail in the web build.",
        )
        .color(TEXT_DIM)
        .size(12.5),
    );
}

fn describe_understood_rows(parsed: &PastedMethod) -> Vec<(&'static str, String)> {
    let has_input = parsed.input_type.is_some();
    let has_output = parsed.output_type.is_some();
    let table = if has_input {
        "serializedCommands"
    } else {
        "noInputCommands"
    };
    let section = match (has_input, has_output) {
        (false, true) => "// no input, output",
        (false, false) => "// no input, no output",
        (true, false) => "// input, no output",
        (true, true) => "// input, output",
    };
    vec![
        ("name", parsed.name.clone()),
        (
            "input",
            parsed.input_type.clone().unwrap_or_else(|| "(none)".into()),
        ),
        (
            "output",
            parsed.output_type.clone().unwrap_or_else(|| "(none)".into()),
        ),
        ("JS table", table.to_string()),
        ("db_helper section", section.to_string()),
    ]
}

fn draw_buttons(ui: &mut egui::Ui, state: &mut HomePageState) {
    let running = state.is_running();
    let cancelled = state.cancel_pressed();
    let ready = state.parsed.is_some() && state.scan_error.is_none();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        let check_only_clicked = ui
            .add_enabled_ui(!running && ready, |ui| {
                styled_button(ui, "Check only", GREEN_BTN, GREEN_BTN_HOVER, egui::Color32::WHITE)
                    .clicked()
            })
            .inner;
        let check_and_apply_clicked = ui
            .add_enabled_ui(!running && ready, |ui| {
                styled_button(
                    ui,
                    "Check and apply",
                    GREEN_BTN,
                    GREEN_BTN_HOVER,
                    egui::Color32::WHITE,
                )
                .clicked()
            })
            .inner;
        let cancel_clicked = ui
            .add_enabled_ui(running && !cancelled, |ui| {
                styled_button(ui, "Cancel", GHOST_BTN, GHOST_BTN_HOVER, TEXT).clicked()
            })
            .inner;
        ui.separator();
        let build_clicked = ui
            .add_enabled_ui(!running, |ui| {
                styled_button(
                    ui,
                    "Build everything and check",
                    GHOST_BTN,
                    GHOST_BTN_HOVER,
                    TEXT,
                )
                .clicked()
            })
            .inner;

        if build_clicked {
            let ctx = ui.ctx().clone();
            state.start_build_everything(ctx);
        }
        if check_only_clicked {
            let ctx = ui.ctx().clone();
            state.start_run(Action::CheckOnly, ctx);
        }
        if check_and_apply_clicked {
            let ctx = ui.ctx().clone();
            state.start_run(Action::CheckAndApply, ctx);
        }
        if cancel_clicked {
            state.request_cancel();
        }
    });
}

fn draw_phase(ui: &mut egui::Ui, state: &HomePageState) {
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(format!("phase: {}", state.phase_text()))
            .color(TEXT_DIM)
            .size(14.0),
    );
    ui.add_space(10.0);
}

fn draw_log(ui: &mut egui::Ui, state: &mut HomePageState) {
    ui.horizontal(|ui| {
        let mut copy = false;
        let mut clear = false;
        if ghost_button(ui, "Copy log").clicked() {
            copy = true;
        }
        if ghost_button(ui, "Clear").clicked() {
            clear = true;
        }
        if clear {
            state.log.clear();
            state.stream_folder.clear();
        }
        if copy {
            let mut text = String::new();
            for line in &state.log {
                text.push_str(&line.text);
                text.push('\n');
            }
            text.push_str(&state.stream_folder.lines.join("\n"));
            if !state.stream_folder.current.is_empty() {
                text.push('\n');
                text.push_str(&state.stream_folder.current);
            }
            ui.ctx().copy_text(text);
        }
        let total = state.log.len() + state.stream_folder.lines.len();
        ui.label(
            egui::RichText::new(format!("{total} lines"))
                .color(TEXT_DIM)
                .size(13.0),
        );
    });
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(FIELD_BG)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(6.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical()
                .id_salt("log_scroll")
                .max_height(520.0)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &state.log {
                        let color = match line.kind {
                            LogKind::Plain => TEXT_DIM,
                            LogKind::Error => ERR,
                            LogKind::Warning => WARN,
                        };
                        ui.label(
                            egui::RichText::new(&line.text)
                                .color(color)
                                .monospace()
                                .size(12.5),
                        );
                    }
                    for line in &state.stream_folder.lines {
                        let color = classify_line(line);
                        ui.label(
                            egui::RichText::new(line)
                                .color(color)
                                .monospace()
                                .size(12.5),
                        );
                    }
                    if !state.stream_folder.current.is_empty() {
                        ui.label(
                            egui::RichText::new(&state.stream_folder.current)
                                .color(classify_line(&state.stream_folder.current))
                                .monospace()
                                .size(12.5),
                        );
                    }
                });
        });
}

fn classify_line(line: &str) -> egui::Color32 {
    let t = line.trim_start();
    if t.starts_with("error") || t.starts_with("Error") || t.contains("error:") {
        ERR
    } else if t.starts_with("warning") || t.starts_with("warning:") {
        WARN
    } else if t.starts_with("Compiling ")
        || t.starts_with("Checking ")
        || t.starts_with("Finished ")
    {
        OK
    } else if t.starts_with("Downloaded ")
        || t.starts_with("Downloading ")
        || t.starts_with("Updating ")
        || t.starts_with("Blocking ")
        || t.starts_with("Adding ")
        || t.starts_with("Locking ")
        || t.starts_with("Building ")
    {
        DIM_LOG
    } else {
        TEXT_DIM
    }
}

fn draw_report(ui: &mut egui::Ui, state: &HomePageState) {
    let Some(report) = &state.report else { return };
    if report.snippets.is_empty() {
        return;
    }
    ui.add_space(10.0);
    draw_section(ui, "what was written", |ui| {
        for s in &report.snippets {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("›").color(OK).strong());
                ui.label(
                    egui::RichText::new(&s.name)
                        .color(TEXT_HEAD)
                        .monospace()
                        .strong(),
                );
                ui.label(egui::RichText::new("→").color(TEXT_DIM));
                ui.label(egui::RichText::new(&s.file).color(TEXT_DIM).monospace().size(12.5));
            });
        }
    });
}

fn green_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    styled_button(ui, label, GREEN_BTN, GREEN_BTN_HOVER, egui::Color32::WHITE)
}

fn ghost_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    styled_button(ui, label, GHOST_BTN, GHOST_BTN_HOVER, TEXT)
}

/// A button whose fill changes on hover. `egui::Button::fill` overrides
/// egui's hover visuals, so we draw the rect ourselves.
fn styled_button(
    ui: &mut egui::Ui,
    label: &str,
    base: egui::Color32,
    hover: egui::Color32,
    text: egui::Color32,
) -> egui::Response {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let padding = egui::vec2(14.0, 8.0);
    let desired = egui::vec2(ui.available_width(), font.size + padding.y * 2.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
    let enabled = ui.is_enabled();
    let fill = if !enabled {
        base.gamma_multiply(0.4)
    } else if response.is_pointer_button_down_on() {
        hover.gamma_multiply(0.85)
    } else if response.hovered() {
        hover
    } else {
        base
    };
    ui.painter().rect_filled(rect, 6.0, fill);
    let text_color = if enabled {
        text
    } else {
        text.gamma_multiply(0.5)
    };
    ui.painter()
        .with_clip_rect(rect)
        .text(rect.center(), egui::Align2::CENTER_CENTER, label, font, text_color);
    response
}

#[allow(dead_code)]
const GREEN_BTN_HOVER_UNUSED: egui::Color32 = GREEN_BTN_HOVER;
#[allow(dead_code)]
const GHOST_BTN_HOVER_UNUSED: egui::Color32 = GHOST_BTN_HOVER;