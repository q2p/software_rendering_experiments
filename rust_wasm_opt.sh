#!/bin/bash

cd ./src/rust_casted/ && \
cargo +nightly build --color=always --target wasm32-unknown-unknown --release && \
cd ../../ && \

wasm-snip --snip-rust-fmt-code --snip-rust-panicking-code ./src/rust_casted/target/wasm32-unknown-unknown/release/js_nano_r.wasm -o ./cache/wasm/snipped.wasm && \
../binaryen/bin/wasm-opt -o ./dist/emc.wasm ./cache/wasm/snipped.wasm -O4 --dce --merge-locals --strip --vacuum --minify-imports
# ../binaryen/bin/wasm-opt -o ./dist/emc.wasm ./cache/wasm/snipped.wasm -Oz --dce --merge-locals --strip --vacuum --minify-imports
# ../binaryen/bin/wasm-opt -o ./dist/emc.wasm ./cache/wasm/snipped.wasm -Oz --dce --merge-locals --strip --vacuum --minify-imports

# cp ./src/rust/target/wasm32-unknown-unknown/release/js_nano_r.wasm ./dist/emc.wasm