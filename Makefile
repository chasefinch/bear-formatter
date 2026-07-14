default: format lint check test

format:
	@echo "Formatting Rust files..."
	@cargo fmt
	@echo "...done."

lint:
	@echo "Checking formatting..."
	@cargo fmt --check || (printf 'Found files which need formatting. Run \033[1mmake format\033[0m and re-lint.\n'; exit 1)
	@echo "Running Clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "...done. No issues found."

check:
	@echo "Type-checking..."
	@cargo check --all-targets
	@echo "...done. No issues found."

test:
	@echo "Running tests..."
	@cargo test
	@echo "All tests completed successfully!"

build:
	@echo "Building release binary..."
	@cargo build --release
	@echo "...done. Binary at target/release/bear-formatter"

coverage:
	@cargo llvm-cov --summary-only

setup:
	@rustup component add rustfmt clippy
	@echo "Toolchain components installed."

clean:
	@cargo clean

.PHONY: default format lint check test build coverage setup clean
