default: build test

check: fmt lint

build:
    cargo build

test:
    cargo test

fmt:
    cargo fmt

lint: fmt
    cargo clippy
