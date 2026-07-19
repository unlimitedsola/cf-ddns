# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Per-record lookup overrides: individual records can override the global lookup configuration
  by specifying a custom lookup provider configuration under `v4` or `v6`.

  ```toml
  [[records]]
  name = "internal.example.com"
  zone = "example.com"
  v4 = { lookup = { provider = "interface", interface = "eth0" } }
  ```

- IP matchers for interface lookup: specify prefix (e.g. `2001:db8::/64`) and/or suffix (e.g. `::20/-64`)
  matching rules to filter candidate IP addresses when multiple addresses are assigned to the network interface.

  ```toml
  [[records]]
  name = "suffix-filtered.example.com"
  zone = "example.com"
  v6 = { lookup = { provider = "interface", interface = "eth0", matchers = { v6 = ["::20/-64"] } } }
  ```

- Custom configuration and cache locations: specify custom paths for the configuration TOML file
  and the zone/record ID cache JSON file using command-line arguments or environment variables.

  ```bash
  cf-ddns --config /path/to/config.toml --id-cache /path/to/id_cache.json
  # Or via environment variables
  CF_DDNS_CONFIG=/path/to/config.toml CF_DDNS_ID_CACHE=/path/to/id_cache.json cf-ddns
  ```

- OpenWrt system package: build and install the daemon on OpenWrt 25.12+ (Alpine `.apk` format)
  with a standard `procd` init script. Runtime cache writes are redirected to memory-backed tmpfs (`/var/run/`)
  to protect physical flash.

## [0.5.0] - 2026-04-01

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

[unreleased]: https://github.com/unlimitedsola/cf-ddns/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/unlimitedsola/cf-ddns/compare/v0.5.0...v0.4.0
[0.4.0]: https://github.com/unlimitedsola/cf-ddns/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/unlimitedsola/cf-ddns/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/unlimitedsola/cf-ddns/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/unlimitedsola/cf-ddns/releases/tag/v0.1.0
