#!/bin/bash

DIR="$(dirname "$0")"

if cargo build --target=wasm32-unknown-emscripten --release; then
    cp $DIR/target/wasm32-unknown-emscripten/release/rusty_microphone.js "$DIR/web/"
    cp $DIR/target/wasm32-unknown-emscripten/release/deps/*.wasm "$DIR/web/"
    cp $DIR/target/wasm32-unknown-emscripten/release/deps/*.wast "$DIR/web/"
    cp $DIR/target/wasm32-unknown-emscripten/release/deps/*.map "$DIR/web/"
    cp $DIR/target/wasm32-unknown-emscripten/release/deps/*.js "$DIR/web/"
fi
