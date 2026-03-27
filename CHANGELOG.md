# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `exec` lookup provider: run a shell command and use its stdout as the public IP address.
  Useful when a custom method of IP discovery is needed.
  ```toml
  [lookup]
  v4 = { provider = "exec", cmd = "curl -s ipv4.icanhazip.com" }
  ```
- Per-protocol lookup configuration: the `[lookup]` table now accepts separate `v4` and `v6` keys,
  allowing each protocol to use a different provider. Each key accepts either a provider name
  string or a full provider config table.
  ```toml
  [lookup]
  v4 = "icanhazip"
  v6 = { provider = "exec", cmd = "dig -6 +short myip.opendns.com @resolver1.opendns.com" }
  ```
- Retry configuration: a new `[retry]` section controls how failed updates are retried within
  each interval. Retries stop when `max_attempts` is reached or the next backoff delay would
  exceed the update interval, whichever comes first.
  ```toml
  [retry]
  base_delay = 5        # seconds before the first retry (default: 5)
  backoff_multiplier = 2.0  # exponential growth factor (default: 2.0)
  max_attempts = 5      # total attempt budget per interval (default: 5)
  ```
- `interface` lookup provider: read the public IP address from a named network interface.
  This is especially useful for IPv6 deployments where the desired address is already
  assigned locally.
  ```toml
  [lookup]
  v6 = { provider = "interface", interface = "eth0" }
  ```

### Fixed

- Ensured proper connection timeouts on CloudFlare API and IP lookup requests to prevent
  indefinitely hanging connections.

## [0.4.0] - 2024-12-23

### Changed

- Re-license under the GNU Affero General Public License v3.0 or later.
  This change is to ensure that the software remains open-source and free for everyone.
- Renamed the `zones` configuration property to `records` for accuracy.
  It is recommended to update the configuration file accordingly.
  The old name is still supported for backward compatibility, it is planned to be removed in the upcoming releases.

## [0.3.0] - 2024-05-19

### Added

- Skip updating the record if the IP address has not changed.

## [0.2.0] - 2024-04-16

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

[unreleased]: https://github.com/unlimitedsola/cf-ddns/compare/v0.4.0...HEAD

[0.4.0]: https://github.com/unlimitedsola/cf-ddns/compare/v0.3.0...v0.4.0

[0.3.0]: https://github.com/unlimitedsola/cf-ddns/compare/v0.2.0...v0.3.0

[0.2.0]: https://github.com/unlimitedsola/cf-ddns/compare/v0.1.0...v0.2.0

[0.1.0]: https://github.com/unlimitedsola/cf-ddns/releases/tag/v0.1.0
