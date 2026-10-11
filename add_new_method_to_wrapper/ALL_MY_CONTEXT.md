# All my context

Everything a future AI needs to work on this crate.

## What it is

A GUI for adding one method to z_db's web path. The web path is three files: macro_core.rs (the method body), worker.js (the command name to LiveForever method map), and db_helper.rs (the async Rust client). Nothing enforces that the command names match. This tool takes one pasted method, checks it compiles, and writes the other two entries.

## The three files

db_wrapper/src/macro_core.rs — every method lives inside one create_the_entire_universe! macro. Each method uses helper macros: method_input_type!, method_return_type!, decode_input!, unwrap_or_bail!, finish_output!, return_nothing!. Those helpers are defined per mascot file, not here. In web_mascot.rs the input and output types expand to Vec<u8>; in the others they expand to the real types. So a method can pass one check and fail another.

db_wrapper/src/web_output/worker.js — two hand-maintained objects, serializedCommands and noInputCommands. A method with an input parameter goes in the first, one without goes in the second. export_database is the exception: no input, but it still passes an empty Uint8Array. It's hand-written.

web_internal_db/src/db_helper.rs — one pub async fn per command. Four shapes (input or not × output or not), grouped in four sections headed by "// no input, output", "// no input, no output", "// input, no output", "// input, output". Uses "use protocol::payload::*;", so no import handling needed. The command string equals the method name.

## Architecture

Three crates. app is the window, every_page_ever is the UI, app_core is the logic. app_core has no egui dependency, so it's testable without a window.

app_core modules:

settings.rs — the JSON config, load/save, resolved paths (real vs mock), populate_mock_folder.

scan.rs — parse macro_core.rs with proc-macro2, find the impl inside the macro, list methods with insertion offset and indent. Also parses the pasted method into name/input/output.

edit.rs — pure text builders: insert into macro_core, add to worker.js, add to db_helper.rs. No I/O.

precheck.rs — collect every problem before touching disk. Name collisions, missing types.

guard.rs — hash, backup, marker, restore, verify. Drop does a best-effort restore.

runner.rs — spawn a shell command in its own process group, stream stdout/stderr, handle cancel with SIGTERM then SIGKILL. Rewrites "cargo " to local_cargo.sh if found.

pipeline.rs — the whole flow glued together.

events.rs, report.rs — the callback types.

linefold.rs — fold a byte stream into lines. \r replaces the current line, \n commits. This is what makes cargo's progress bar disappear into one updating line instead of spamming the log. last_progress survives across \r-clears so the UI can read a live value.

ui_state.rs — the small bits of UI state that persist across sessions (selected position, pasted text, settings header open).

## The guard

The tool has to write macro_core.rs to check it. That means it temporarily breaks a real source file, so this is the part to get right.

Before the write: hash the bytes with SHA-256, save them to <cache_root>/macro_core_backup.rs, write a marker file next to it holding the file path, the hash, and the backup path.

After the check: read the file again. If it doesn't match what the tool wrote (someone else edited it), don't overwrite — warn. Otherwise write the original bytes back, read again, hash again, compare with the first hash. If the hashes match, delete the marker. If they don't, log both short hashes and set a restore problem on the guard.

On startup, check_startup_marker looks for the marker. If the file at the marker's path still hashes to the marker's hash, the marker is stale and gets deleted silently. If not, the tool shows a "restore from backup" banner.

The apply step (writing worker.js and db_helper.rs) only happens if the check passed and the restore verified.

## Running the check

Check commands run through "sh -c" in working_directory, with CARGO_TERM_COLOR=never, NO_COLOR=1, CARGO_TERM_PROGRESS_WHEN=always, CARGO_TERM_PROGRESS_WIDTH=60, TERM=xterm-256color.

If a command starts with "cargo ", the runner walks up from the working directory looking for local_cargo.sh. If one is found, the command becomes "<wrapper>" and the rest.

Progress ticks: cargo writes many \r-separated progress lines and one \n-terminated log line per real event. stream_lines hands raw chunks to the line folder, which handles the \r/\n split itself.

## Mock mode

settings.use_mock_paths redirects the four file paths to settings.mock_root, but leaves working_directory alone. The mock folder is not a cargo workspace, so the check still runs against the real workspace files. That's the point — you're testing the tool's behaviour, not whether cargo can compile the pasted method.

## The settings default

cargo check -p db_wrapper --no-default-features --features testing. This compiles the testing mascot. To also check the web mascot, enable the second default command: cargo check -p db_wrapper --no-default-features --features web --target wasm32-unknown-unknown. Disabled by default because the wasm target may not be installed.

## Reference files outside this crate

~/ProgStuff/egui/zap/zapcli/app/src/egui_setup/drawing_requirements.rs — the stream rendering pattern this tool copies. drain_stream_frames and draw_pane are the model.

~/ProgStuff/egui/zap/zap_wire/src/linefold.rs — the original LineFolder. app_core/linefold.rs is a copy with a last_progress field added.

~/ProgStuff/egui/zap/zapgui/app_core/src/runner.rs — the original process runner with pane_env, strip_ansi, and the process group cancel. app_core/runner.rs is closely related.

## What's tested, what isn't

Tested: scan, edit (pure text-in/text-out, and the pure-insertion invariant), precheck, guard (revert on pass, revert on fail, check-and-apply with a passing check writes all three, cancel reverts, startup recovery restores, startup recovery skips when the file matches), line fold.

Not tested: the check running against a modified macro_core.rs. Mock mode never exercises it, and the process tests use true/false as the check command.

## Rough edges

web_mascot.rs uses different helper macros than the others, so a method can pass the default check and fail the web build. There's a disabled web check command in the default settings — turn it on when you want that coverage.

The apply step doesn't verify db_helper.rs compiles. web_internal_db is a wasm crate and the tool doesn't try to build it. That's the one piece of the generated output that has no compile check behind it.
