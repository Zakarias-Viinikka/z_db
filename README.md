# z_db

## Crates

db handles the rusqlite stuff.

protocol is basically the shared stuff that everything depends on, which none of them own. The most important one is payload, which has all the structs for in and out data the library expects and works with.

sql_builder is self explanatory. Takes parameters, produces sql statements.

migration_helper — I don't remember.

migration_template is self explanatory, but basically it exists because migrating isn't really a library. It's a "this is how I've added support for it in my crud library, but it expects the code to be written in a certain way."

msg_build_n_debuild_helper — I don't remember.

db_wrapper_macro_explained is for macro_core, which is basically how I avoid rewriting implementations for every single wrapper. It requires a macro because the different wrapper targets have requirements that can't be satisfied with general rust traits and whatnot.

db_wrapper is the facade for the db, and is how the client talks to the client owned db — or whoever will be the front-facing target for db requests — and it doesn't matter how they get there or in what shape. The individual wrapper defines what it expects and how to parse and serialize.

web_internal_db is different from the other wrappers, for an important reason: it's a client owned db, it runs inside a worker. web_internal_db makes it easy to use without having to know how it works, but it requires manually adding new macro_core implementations to two different web_internal files.

## Migrations

The actual migration code lives in db/src/migration.rs — add_column, make_col_disappear, fundamentally_edit_existing_col. Those functions build their SQL through sql_builder. To understand how to migrate, refer to the migration_template and its simple_real_usage_example.
