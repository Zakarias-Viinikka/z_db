# db_wrapper_macro_explained

Sandbox for prototyping a macro to deduplicate the mascot wrapper methods in db_wrapper.

`mascot_without_macro_stuff.rs` is the hand-written "before" — one dummy operation
(`get_thing`) implemented three times, once per connection-access pattern used by
the real mascots:

- PlainMascot   -> mascot.rs / testing_mascot.rs style (direct `&self.conn`)
- WebMascot     -> web_mascot.rs style (`Option<Connection>` behind a `Result`)
- AndroidMascot -> android_mascot.rs style (`Mutex<Connection>`, locked)

Goal: write a macro that generates all three `get_thing` methods from one
declaration, without hand-writing the connection-access boilerplate each time.

Not covered yet: the byte-payload vs typed-struct split (mascot.rs/web_mascot.rs
take Vec<u8>, android/testing take typed structs directly). That's a separate
axis, added after the connection-access macro works.

Note: this macro generates the db methods, but it does NOT update the web
output. When a method is added or changed here, web_output/worker.js has to
be updated by hand to include the new command.
