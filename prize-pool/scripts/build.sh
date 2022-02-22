#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
mkdir -p ./out
cp ../target/wasm32-unknown-unknown/release/*.wasm ../out/prize-pool.wasm
