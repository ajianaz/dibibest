#!/bin/sh
# Build the core WASM bridge and drop it where Vite imports it (src/lib/wasm, git-ignored).
set -e
cd "$(dirname "$0")/.."
cargo build -p dibibest-wasm --release --target wasm32-unknown-unknown
mkdir -p src/lib/wasm
cp ../../target/wasm32-unknown-unknown/release/dibibest_wasm.wasm src/lib/wasm/dibibest.wasm
