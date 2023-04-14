default: fmt check test lint 
alias i := install
alias t := test

check:
    cargo check

test:
    cargo test

fmt:
    cargo fmt

lint: fmt
    cargo clippy --fix --allow-dirty

install: check test
    cargo install --path . --locked
