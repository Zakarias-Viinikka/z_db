#!/usr/bin/env bash
cd "$(dirname "$0")/add_new_method_to_wrapper"
cargo build --release -p app
exec ./target/release/app