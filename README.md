# cf-ddns: CloudFlare Dynamic DNS

A dynamic DNS update tool specifically designed for CloudFlare, written in [Rust].
It is designed to be run as a
service on a server or a device with a dynamically changing public IP address.
It periodically checks the current public IP address and updates
the DNS record if it has changed.

[Rust]: https://www.rust-lang.org/

[CloudFlare]: https://www.cloudflare.com/

## Features

- **Written in [Rust]**: Enjoy the benefits of Rust's memory-safety and performance.
- **Designed for [CloudFlare]**: Specifically designed to work with CloudFlare's API.
- **Run as a Service**: Can be run as a service and periodically update in the background.
- **IPv4 and IPv6**: Support both IPv4 and IPv6 addresses.
- **Cross-platform**: Works on Linux, macOS, and Windows.
- **User-Friendly**: Engineered for ease of use and straightforward configuration.

## Installation

The latest release binaries are available for download on the [releases page].

[releases page]: https://github.com/unlimitedsola/cf-ddns/releases/latest

## Configuration

First, you need to have a CloudFlare account and a token with the `Zone.DNS` permission.
You can create a token with the required permission in the [API Tokens] section of the CloudFlare dashboard.

[API Tokens]: https://dash.cloudflare.com/profile/api-tokens

Second, create a configuration file `config.toml` in the same directory as the binary.
Here is an example configuration file:

```toml
token = "<your-cloudflare-token>"
[[records]]
name = "abc.example.com"
zone = "example.com"
v4 = true
v6 = true

[[records]]
name = "v4.example.net"
zone = "example.net"
v4 = true

[[records]]
name = "v6.example.net"
zone = "example.net"
v6 = true
```

Replace `<your-cloudflare-token>` with your CloudFlare token.

The `records` section is a list of records, each containing the following fields:

- `name` will be the full DNS record name, e.g., `abc.example.com`.
- `zone` is the zone name, e.g., `example.com`.
- `v4` and `v6` are boolean values indicating whether to update the `A` and `AAAA` records, respectively.

> [!TIP]
> The updater is designed to automatically create the DNS record if it is missing.
> Once established, it will solely update the IP address and refrain from
> modifying other settings, such as TTL, priority, and similar parameters.

> [!TIP]
> For a more detailed configuration, see the [full configuration example].

[full configuration example]: ./config.example.toml

Finally, run the binary to update the DNS records:

```sh
./cf-ddns
```

## Install as a Service

You can install `cf-ddns` as a service on your server or device.
`cf-ddns` has built-in support to help you install and manage the service on various platforms.

To install the service, run the following command with administrative privileges:

```sh
./cf-ddns service install
```

> [!NOTE]
> Based on your platform, this command will:
> - On Linux, install the [systemd unit file] to `/etc/systemd/system/cf-ddns.service`
> - On macOS, install the [launchd plist file] to `/Library/LaunchDaemons/cf-ddns.plist`
> - On Windows, install the service to the Windows Service Manager

[systemd unit file]: ./src/service/linux/systemd.service

[launchd plist file]: ./src/service/macos/launchd.plist

> [!IMPORTANT]
> You should not move or delete the binary after installing the service.

The installed service will start automatically on boot and run in the background.

To uninstall the service, run the following command with administrative privileges:

```sh
./cf-ddns service uninstall
```

## Contributing

Contributions are welcome! Feel free to [open an issue] or [submit a pull request].

[open an issue]: https://github.com/unlimitedsola/cf-ddns/issues

[submit a pull request]: https://github.com/unlimitedsola/cf-ddns/pulls

## License

This program is free software: you can redistribute it and/or modify it under the terms of
the GNU Affero General Public License as published by the Free Software Foundation,
either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program.
If not, see <https://www.gnu.org/licenses/>.
