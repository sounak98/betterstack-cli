.PHONY: build test lint fmt check clean release install

# Development
build:
	cargo build

# Quality
fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

lint:
	cargo clippy -- -D warnings

check: fmt-check lint test

# Testing
test:
	cargo test

test-verbose:
	cargo test -- --nocapture

# Release
release:
	cargo build --release

install: release
	mkdir -p $(HOME)/.local/bin
	cp target/release/bs $(HOME)/.local/bin/bs

clean:
	cargo clean
