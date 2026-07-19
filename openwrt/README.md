# OpenWrt Packaging & Development Guide

This directory contains the Makefile, init configuration, and containerized build pipeline to compile `cf-ddns` for OpenWrt 25.12+ (Alpine APK format).

## Directory Layout

- **[Makefile](Makefile)**: Standard OpenWrt package Makefile.
- **[Containerfile](Containerfile)**: Multi-stage build environment.
- **[build.sh](build.sh)**: A helper script to execute the container build.
- **[files/cf-ddns.init](files/cf-ddns.init)**: Procd init service configuration.

## How to Build the Package

To compile the package, run the build script from the repository root:

```bash
./openwrt/build.sh <architecture> <sdk-version>
```

For example, to build for standard x86_64 routers running OpenWrt 25.12:

```bash
./openwrt/build.sh x86-64 openwrt-25.12
```

The compiled Alpine `.apk` files will be exported directly to the local directory `openwrt/out/` (which is git-ignored and docker-ignored).

## Deploying & Installing

Copy the built `.apk` file from `openwrt/out/` to the target router, then run:

```bash
apk add --allow-untrusted cf-ddns_*.apk
```
