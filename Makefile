.PHONY: test build example clean clippy

## Run all tests
test:
	cargo test --lib --verbose

## Build
build:
	cargo build

## Run the example
example:
	cargo run --example demo

## Run clippy lints
clippy:
	cargo clippy -- -D warnings

## Run tests + clippy
check: clippy test

## Clean build artifacts
clean:
	cargo clean
