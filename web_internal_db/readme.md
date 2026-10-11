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

worker_wrapper.js hardcodes the connection name 'leptos_db' in its
initialize call at the bottom of the file. Each project that copies
web_output/ should change that string to its own name. If two projects
use the same name, they share the same OPFS database.

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

Updating macro_core manually means web_internal doesn't get updated.
web_internal has a custom worker solution for a web client's internal db.
Refer to SCRIPTS_EXPLAINED.md or the add_new_method_to_wrapper crate for
details. Updating manually is possible but requires changes in
macro_core.rs, worker.js, and db_helper.rs.
