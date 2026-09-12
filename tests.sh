#!/usr/bin/env bash
cd "$(dirname "$0")"

cargo test -p protocol
cargo test -p sql_builder
cargo test -p db
cargo test -p db_wrapper --no-default-features --features testing --no-fail-fast
