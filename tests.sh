#!/usr/bin/env bash
cd "$(dirname "$0")"

export CARGO_TERM_COLOR=always

{
  cargo test -p protocol
  cargo test -p sql_builder
  cargo test -p db
  cargo test -p db_wrapper --no-default-features --features testing --no-fail-fast
} 2>&1 \
| stdbuf -oL awk '
    {
        orig = $0
        line = $0
        gsub(/\033\[[0-9;]*m/, "", line)
    }
    line ~ /warning:/ { in_warn=1; next }
    in_warn && line == "" { in_warn=0; next }
    in_warn { next }
    { print orig; fflush() }
  '
