#!/usr/bin/env bash
here="$(dirname "$0")"
exec "$here/add_new_method_to_wrapper/local_cargo.sh" run --release --manifest-path "$here/add_new_method_to_wrapper/Cargo.toml" -p app