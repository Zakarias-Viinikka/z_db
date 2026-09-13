#!/usr/bin/env bash
cd "$(dirname "$0")"

cargo check -p protocol --all-targets
cargo check -p sql_builder --all-targets
cargo check -p db --all-targets
cargo check -p db_wrapper --no-default-features --features testing --all-targets
