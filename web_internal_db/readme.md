web_internal_db
Rust-side client for the z_db OPFS worker. Lets a web app (Leptos) talk to
the sqlite database that lives inside a browser worker.

What's inside
db_helper — the calling layer. Declares the JS extern, serializes each
call's input to bytes, hands them to the worker, deserializes the reply.
One pub async fn per db command.

db_gui — a reusable page of buttons for inspecting and poking at the db.
Uses db_helper internally.

Deps
toml
web_internal_db = { path = "/path/to/z_db/web_internal_db" }
Also needs protocol (it's a transitive dep, but if you're calling
db_helper directly you'll want it in scope too).

What the caller must provide
db_helper calls a global JS function named javascript_im_begging_you.
That function is created by worker_wrapper.js, which ships in z_db's
db_wrapper/src/web_output/. So a page using this crate must load the
worker files first.

Copy z_db's web_output/ into the web project, then in index.html:

html
<link data-trunk rel="copy-dir" href="zdb_web_output" />
...
<script type="module" src="./zdb_web_output/worker_wrapper.js"></script>
That auto-initializes the worker and puts the global function on the page.
Until it does, every db_helper call fails.

Using db_helper
rust
use web_internal_db::db_helper;
use protocol::payload::{CreateTableIn, ListTablesOut};

spawn_local(async move {
    match db_helper::list_tables().await {
        Ok(out) => log!("{:?}", out.table_names),
        Err(e) => log!("{:?}", e),
    }
});
Every fn is async and returns Result<Out, DbError>. Call them inside
spawn_local.

Command names must match worker.js
Each db_helper fn hardcodes the command string that worker.js expects
(e.g. "list_tables"). worker.js has a hand-maintained map of command
names to LiveForever methods. If you add a method to the db crate, you
must add it in both places:

db_wrapper/src/macro_core.rs — the method

db_wrapper/src/web_output/worker.js — the command name, in
serializedCommands or noInputCommands

web_internal_db/src/db_helper.rs — the async fn

Nothing enforces that they match. If a name in db_helper doesn't exist in
worker.js, the worker returns ['error', ...] and the call fails.