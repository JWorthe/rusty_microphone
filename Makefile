all: build

build-web:
	cargo build --target=wasm32-unknown-emscripten --release
	mkdir -p target/site
	cp target/wasm32-unknown-emscripten/release/rusty_microphone.js target/site/
	cp target/wasm32-unknown-emscripten/release/deps/*.wasm target/site/
	cp web/* target/site/

build-web-debug:
	cargo build --target=wasm32-unknown-emscripten
	mkdir -p target/site
	cp target/wasm32-unknown-emscripten/debug/rusty_microphone.js target/site/
	cp target/wasm32-unknown-emscripten/debug/deps/*.wasm target/site/
	cp web/* target/site/


build-desktop:
	cargo build --release

build: build-desktop build-web

test:
	cargo test --release

bench:
	cargo bench

clean:
	cargo clean


.PHONY: all build-web build-desktop build test bench clean
