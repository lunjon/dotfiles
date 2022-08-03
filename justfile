default: check test lint 
alias i := install

check:
    cargo check

test:
    cargo test

fmt:
    cargo fmt

lint: fmt
    cargo clippy

install:
    cargo install --path .
