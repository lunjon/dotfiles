default: check test lint 
alias i := install
alias t := test

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
