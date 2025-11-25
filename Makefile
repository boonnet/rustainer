.PHONY: build release run test clean check fmt lint

# Default target
all: build

# Build debug binary
build:
	cargo build

# Build release binary
release:
	cargo build --release

# Run the application
run:
	cargo run

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Check code without building
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run clippy linter
lint:
	cargo clippy -- -W clippy::all

# Build and run with example
example: build
	@echo "Run with: sudo ./target/debug/rustainer run -f <rootfs> /bin/sh"

