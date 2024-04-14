# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support `aarch64-unknown-linux-gnu` target.
- Support `riscv64gc-unknown-linux-gnu` target.
- Build universal binary for macOS target.
- Start service automatically after installation on Windows.

### Changed

- The TLS backend is switched to `rustls`.
- Switched the configuration format from `YAML` to `TOML`.

### Removed

- No longer provide separate builds for `x86_64-apple-darwin` and `aarch64-apple-darwin` targets.
  Please use the universal binary instead.

## [0.1.0] - 2024-03-08

### Added

- Initial release.

[unreleased]: https://github.com/unlimitedsola/cf-ddns/compare/v0.1.0...HEAD

[0.1.0]: https://github.com/unlimitedsola/cf-ddns/releases/tag/v0.1.0
