#!/usr/bin/env bash
cd "$(dirname "$0")"

file=$(find ./db/src -name '*.rs' | fzf --height 40% --reverse --preview 'head -50 {}')

[ -z "$file" ] && exit 0

pkg=$(echo "$file" | cut -d/ -f2)

CARGO_TERM_COLOR=always cargo check -p "$pkg"
