#!/bin/bash

DIR="$(dirname "$0")"

if cargo build --target=wasm32-unknown-emscripten; then
    cp $DIR/target/wasm32-unknown-emscripten/debug/rusty_microphone.js "$DIR/web/"
    cp $DIR/target/wasm32-unknown-emscripten/debug/deps/*.wasm "$DIR/web/"
    cp $DIR/target/wasm32-unknown-emscripten/debug/deps/*.wast "$DIR/web/"
    cp $DIR/target/wasm32-unknown-emscripten/debug/deps/*.map "$DIR/web/"
    cp $DIR/target/wasm32-unknown-emscripten/debug/deps/*.js "$DIR/web/"
fi
