#!/usr/bin/env bash
cd "$(dirname "$0")"

file=$(find ./db_wrapper/tests -maxdepth 1 -name '*.rs' | fzf --height 40% --reverse --preview 'head -50 {}')

[ -z "$file" ] && exit 0

name=$(basename "$file" .rs)

export CARGO_TERM_COLOR=always

cargo test -p db_wrapper --no-default-features --features testing --test "$name" 2>&1 \
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
