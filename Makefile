.PHONY: help all build release test fmt clippy install run clean

help:
	@echo "chio - available make targets:"
	@echo "  make all        Run fmt + build + clippy + test (everything)"
	@echo "  make build      Build the CLI (debug)"
	@echo "  make release    Build the CLI (release)"
	@echo "  make test       Run the test suite"
	@echo "  make fmt        Format the code"
	@echo "  make clippy     Lint with clippy (warnings as errors)"
	@echo "  make install    Install chio locally (cargo install --path .)"
	@echo "  make run        Run the CLI (pass args via ARGS=\"...\")"
	@echo "  make clean      Remove build artifacts"

all: fmt build clippy test

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

fmt:
	cargo +nightly fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

install:
	cargo install --path . --force

run:
	cargo run -- $(ARGS)

clean:
	cargo clean
