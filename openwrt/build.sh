#!/usr/bin/env bash
set -euo pipefail

# Resolve repository root path
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Help menu
show_help() {
    echo "Usage: $0 [TARGET] [TAG]"
    echo
    echo "Arguments:"
    echo "  TARGET    The OpenWrt architecture target (default: x86-64)"
    echo "  TAG       The OpenWrt SDK version/branch tag (default: openwrt-25.12)"
    echo
    echo "Example:"
    echo "  $0 aarch64-cortex-a53 openwrt-25.12"
    exit 0
}

# Check for help flags
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    show_help
fi

# Use arguments or defaults
TARGET="${1:-x86-64}"
TAG="${2:-openwrt-25.12}"

# Detect container engine: podman (preferred) or docker
if command -v podman &>/dev/null; then
    ENGINE="podman"
elif command -v docker &>/dev/null; then
    ENGINE="docker"
else
    echo "Error: Neither podman nor docker was found in PATH." >&2
    exit 1
fi

echo "Building cf-ddns APK for OpenWrt target '${TARGET}' (${TAG}) using ${ENGINE}..."

# Build container and export package directly to the local openwrt/out/ directory
# The -f flag specifies the path to the Containerfile, and -o specifies the output directory.
$ENGINE build \
    --build-arg TARGET="${TARGET}" \
    --build-arg TAG="${TAG}" \
    -f "${REPO_ROOT}/openwrt/Containerfile" \
    -o "${REPO_ROOT}/openwrt/out" \
    "${REPO_ROOT}"

echo "--------------------------------------------------------"
echo "Build complete! Staged APK packages in 'openwrt/out/' directory:"
ls -la "${REPO_ROOT}/openwrt/out"
