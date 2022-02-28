#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
cd ..
pwd
cp target/wasm32-unknown-unknown/release/test_token.wasm res/
