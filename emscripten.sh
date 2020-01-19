#!/bin/bash

# ../emsdk/upstream/emscripten/emcc -s WASM=1 -s NO_FILESYSTEM=1 -s SIDE_MODULE=1 -Oz --llvm-opts 3 --llvm-lto 3 -o ./cache/wasm/bundle.wasm ./src/c/bundle.c && \
../emsdk/upstream/emscripten/emcc -s WASM=1 -s EXPORT_ALL=0 -s TOTAL_MEMORY=33554432 -s NO_FILESYSTEM=1 -Oz --llvm-opts 3 --llvm-lto 3 -o ./cache/wasm/bundle.wasm ./src/c/bundle.c && \
../binaryen/bin/wasm-opt -o ./dist/emc.wasm ./cache/wasm/bundle.wasm -Oz --dce --merge-locals --strip --vacuum --minify-imports


# emcc -s WASM=1 -s NO_FILESYSTEM=1 -s SIDE_MODULE=1 -Oz -o dist\emc.wasm ^
# cache\c\logic.bc ^
# cache\c\map.bc ^
# cache\c\exported.bc ^
# cache\c\image.bc

# emcc -s WASM=1 -s NO_FILESYSTEM=1 -s SIDE_MODULE=1 -s EXPORTED_FUNCTIONS=['tick','getImage'] -Os --llvm-opts 3 -o dist\emc.wasm src\c\main.c
# emcc -s WASM=1 -s NO_FILESYSTEM=1 -s SIDE_MODULE=1 -s EXPORTED_FUNCTIONS=['tick','getImage'] -Os --llvm-opts 3 --llvm-lto 3 --closure 1 -o dist\emc.wasm src\c\main.c
