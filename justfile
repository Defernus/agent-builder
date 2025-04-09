set export
set dotenv-load

default:
    @just --list

run:
    cargo run --bin agent

lint:
    cargo fmt --all --check
    cargo check
    cargo clippy

test:
    cargo nextest run --run-ignored default
    cargo test --doc

test-integration:
    cargo nextest run --run-ignored ignored-only

test-all:
    cargo nextest run --run-ignored all
    cargo test --doc