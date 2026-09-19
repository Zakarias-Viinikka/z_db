#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

# protocol
cd protocol
cargo ndk -o ./output_for_android/jniLibs -t arm64-v8a build --release
cargo run --bin uniffi-bindgen -- generate \
    --library ./output_for_android/jniLibs/arm64-v8a/libprotocol.so \
    --language kotlin \
    --out-dir ./output_for_android/kotlin \
    --config ./uniffi.toml \
    --no-format
cd ..

# db_wrapper
cd db_wrapper
cargo ndk -o ./android_output/jniLibs -t arm64-v8a build --release --features android
cargo run --package protocol --bin uniffi-bindgen -- generate \
    --library ./android_output/jniLibs/arm64-v8a/libdb_wrapper.so \
    --language kotlin \
    --out-dir ./android_output/kotlin \
    --config ./uniffi.toml \
    --no-format
cd ..
