# List all available recipes
default:
	@just --list

# Run all CI checks (formatting, clippy, and native tests)
check: fmt-check clippy test-all

# Check formatting
fmt-check:
	cargo fmt --all --check

# Format the codebase
fmt:
	cargo fmt --all

# Run Clippy lints with warnings treated as errors
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Run all native test configurations
test-all: test test-no-features test-all-features

# Run standard native tests
test:
	cargo test

# Run native tests without default features
test-no-features:
	cargo test --no-default-features

# Run native tests with all features enabled
test-all-features:
	cargo test --all-features

# Run cross-compilation tests using cargo-cross (requires docker/podman and cross)
test-cross target:
	cross test --target {{target}}
	cross test --target {{target}} --no-default-features
	cross test --target {{target}} --all-features
