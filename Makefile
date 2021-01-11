check: test lint

test:
	cargo test

build:
	cargo build

lint:
	cargo clippy

fix:
	cargo fix --allow-dirty --allow-staged
	cargo fmt

