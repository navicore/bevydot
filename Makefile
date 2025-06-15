.PHONY: all build test clippy clippy-fix fmt clean run help

# Default target
all: fmt clippy-all test build

# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test

# Run clippy with strict settings (matching GitHub Actions CI exactly)
clippy:
	cargo clippy -- \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::nursery \
		-D clippy::cargo \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc \
		-A clippy::missing_docs_in_private_items \
		-A clippy::missing_const_for_fn

# Run clippy on tests
clippy-tests:
	cargo clippy --tests -- \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::nursery \
		-D clippy::cargo \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc \
		-A clippy::missing_docs_in_private_items \
		-A clippy::missing_const_for_fn

# Run clippy on examples (with additional allows)
clippy-examples:
	cargo clippy --examples -- \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::nursery \
		-D clippy::cargo \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc \
		-A clippy::missing_docs_in_private_items \
		-A clippy::missing_const_for_fn \
		-A clippy::uninlined_format_args \
		-A clippy::map_unwrap_or \
		-A clippy::manual_let_else \
		-A clippy::needless_collect \
		-A clippy::single_match_else \
		-A clippy::option_if_let_else

# Run all clippy checks (main, tests, examples)
clippy-all: clippy clippy-tests clippy-examples

# Run clippy and automatically fix what it can
clippy-fix:
	cargo clippy --fix --allow-dirty -- \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::nursery \
		-D clippy::cargo \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc \
		-A clippy::missing_docs_in_private_items \
		-A clippy::missing_const_for_fn

# Check code formatting
fmt:
	cargo fmt -- --check

# Format code (fix formatting issues)
fmt-fix:
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
	@echo "  make all          - Format, lint, test, and build"
	@echo "  make build        - Build the project in release mode"
	@echo "  make test         - Run all tests"
	@echo "  make clippy       - Run clippy with CI settings"
	@echo "  make clippy-all   - Run clippy on code, tests, and examples"
	@echo "  make clippy-fix   - Run clippy and auto-fix issues"
	@echo "  make fmt          - Check code formatting"
	@echo "  make fmt-fix      - Fix code formatting with rustfmt"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make run          - Run with hierarchy.dot example"
	@echo "  make run-stdin    - Run with stdin example"