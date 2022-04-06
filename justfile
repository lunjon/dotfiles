default: lint check test

check:
    cargo check

test:
    cargo test

fmt:
    cargo fmt

lint: fmt
    cargo clippy
