# restore point

Copies of the four files the tool interacts with, from before the process tests first ran. If a test or a run leaves the real files in a broken state, copy the right one back:

- macro_core.rs  -> db_wrapper/src/macro_core.rs
- worker.js      -> db_wrapper/src/web_output/worker.js
- db_helper.rs   -> web_internal_db/src/db_helper.rs
- payload.rs     -> protocol/src/payload.rs

payload.rs is only here for completeness — the tool never writes it.
