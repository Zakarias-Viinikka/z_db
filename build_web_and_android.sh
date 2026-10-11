#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

# --- protocol (android) ---
cd protocol
cargo ndk -o ./output_for_android/jniLibs -t arm64-v8a build --release
cargo run --bin uniffi-bindgen -- generate \
    --library ./output_for_android/jniLibs/arm64-v8a/libprotocol.so \
    --language kotlin \
    --out-dir ./output_for_android/kotlin \
    --config ./uniffi.toml \
    --no-format
cd ..

# --- db_wrapper (android) ---
cd db_wrapper
cargo ndk -o ./android_output/jniLibs -t arm64-v8a build --release --features android
cargo run --package protocol --bin uniffi-bindgen -- generate \
    --library ./android_output/jniLibs/arm64-v8a/libdb_wrapper.so \
    --language kotlin \
    --out-dir ./android_output/kotlin \
    --config ./uniffi.toml \
    --no-format

# --- db_wrapper (web) ---
# RUSTFLAGS from local_cargo.sh sets -fuse-ld=mold, which rust-lld
# doesn't understand. Clear it for the wasm build.
RUSTFLAGS="" wasm-pack build --target web --out-dir src/web_output --out-name db_wrapper --no-typescript --no-pack --features web
cd ..
