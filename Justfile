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

# Run Clippy for macOS target
clippy-mac:
	cargo clippy --target x86_64-apple-darwin --all-targets --all-features -- -D warnings

# Run Clippy for Windows target
clippy-win:
	cargo clippy --target x86_64-pc-windows-gnu --all-targets --all-features -- -D warnings

# Run Clippy across all platforms (Linux, macOS, Windows)
clippy-all: clippy clippy-mac clippy-win

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
