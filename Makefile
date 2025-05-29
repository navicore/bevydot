.PHONY: all build test clippy clippy-fix fmt clean run help

# Default target
all: fmt clippy test build

# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test

# Run clippy with strict settings (similar to CI)
clippy:
	cargo clippy -- -D warnings -W clippy::all -W clippy::pedantic \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_panics_doc \
		-A clippy::missing_errors_doc

# Run clippy and automatically fix what it can
clippy-fix:
	cargo clippy --fix -- -D warnings -W clippy::all -W clippy::pedantic \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_panics_doc \
		-A clippy::missing_errors_doc

# Format code
fmt:
	cargo fmt

# Clean build artifacts
clean:
	cargo clean

# Run the program with a sample file
run:
	cargo run -- examples/hierarchy.dot

# Run with stdin example
run-stdin:
	cat examples/network_topology.dot | cargo run

# Help message
help:
	@echo "Available targets:"
	@echo "  make all        - Format, lint, test, and build"
	@echo "  make build      - Build the project in release mode"
	@echo "  make test       - Run all tests"
	@echo "  make clippy     - Run clippy with strict settings"
	@echo "  make clippy-fix - Run clippy and auto-fix issues"
	@echo "  make fmt        - Format code with rustfmt"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make run        - Run with hierarchy.dot example"
	@echo "  make run-stdin  - Run with stdin example"