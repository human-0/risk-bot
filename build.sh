#!/bin/bash
RUSTFLAGS="-Ctarget-feature=+simd128" cargo build --profile wasm-release --target wasm32-wasip1 --package puct_bot
echo "WASM = \"$(base64 --wrap=0 target/wasm32-wasip1/wasm-release/puct_bot.wasm)\"\n$(cat stub.py)" > bot.py

