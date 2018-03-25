all: build

build-web:
	cargo +nightly build --target=wasm32-unknown-unknown --release
	mkdir -p target/site
	cp target/wasm32-unknown-unknown/release/*.wasm target/site/
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
