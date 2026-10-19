#!/usr/bin/env bash
# Self-test of package.sh: names, content of the archives and the dev default,
# with a stand-in program (nothing is compiled).

set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/package.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

export PACKAGE_SKIP_BUILD=1
export CARGO_TARGET_DIR="$work/target"
export PACKAGE_DIST="$work/dist"
mkdir -p "$CARGO_TARGET_DIR/release"
printf '#!/bin/sh\necho stand-in\n' > "$CARGO_TARGET_DIR/release/truckpaint"
printf 'stand-in' > "$CARGO_TARGET_DIR/release/truckpaint.exe"

failures=0
check() {
    local name="$1" ok="$2"
    if [ "$ok" = yes ]; then
        echo "ok   $name"
    else
        echo "FAIL $name"
        failures=$((failures + 1))
    fi
}

# package VERSION PLATFORM prints the path of the archive.
package() {
    if [ "$1" = dev ]; then
        env -u TRUCKPAINT_VERSION bash "$script" "$2"
    else
        TRUCKPAINT_VERSION="$1" bash "$script" "$2"
    fi
}

# The platform is part of the name, and so is the version.
archive="$(package 0.2.0 linux-x86_64)"
[ "$archive" = "$PACKAGE_DIST/truckpaint-v0.2.0-linux-x86_64.tar.gz" ]
check "linux name" "$([ -f "$archive" ] && echo yes || echo no)"
listing="$(tar -tzf "$archive")"
check "linux content" "$(grep -qx 'truckpaint-v0.2.0-linux-x86_64/truckpaint' <<< "$listing" \
    && grep -qx 'truckpaint-v0.2.0-linux-x86_64/LICENSE' <<< "$listing" && echo yes || echo no)"
mode="$(tar -tvzf "$archive" | grep 'linux-x86_64/truckpaint$' | cut -c1-10)"
check "linux program is executable" "$([ "$mode" = "-rwxr-xr-x" ] && echo yes || echo no)"

archive="$(package 0.2.0-beta.1 windows-x86_64)"
check "windows name" "$([ "$archive" = "$PACKAGE_DIST/truckpaint-v0.2.0-beta.1-windows-x86_64.zip" ] && [ -f "$archive" ] && echo yes || echo no)"
listing="$(unzip -Z1 "$archive")"
check "windows content" "$(grep -qx 'truckpaint-v0.2.0-beta.1-windows-x86_64/truckpaint.exe' <<< "$listing" \
    && grep -qx 'truckpaint-v0.2.0-beta.1-windows-x86_64/LICENSE' <<< "$listing" && echo yes || echo no)"

archive="$(package 0.2.0 macos-arm64)"
check "macos name" "$([ "$archive" = "$PACKAGE_DIST/truckpaint-v0.2.0-macos-arm64.zip" ] && [ -f "$archive" ] && echo yes || echo no)"
listing="$(unzip -Z1 "$archive")"
check "macos content" "$(grep -qx 'truckpaint-v0.2.0-macos-arm64/truckpaint' <<< "$listing" \
    && grep -qx 'truckpaint-v0.2.0-macos-arm64/LICENSE' <<< "$listing" && echo yes || echo no)"
check "macos has no exe" "$(grep -q 'truckpaint.exe' <<< "$listing" && echo no || echo yes)"

# Without a version the build is a development one.
archive="$(package dev linux-x86_64)"
check "dev name" "$([ "$archive" = "$PACKAGE_DIST/truckpaint-dev-linux-x86_64.tar.gz" ] && [ -f "$archive" ] && echo yes || echo no)"

# The checksum line is the one a SHA256SUMS file holds.
archive="$PACKAGE_DIST/truckpaint-v0.2.0-macos-arm64.zip"
line="$(cat "$archive.sha256")"
if command -v sha256sum > /dev/null; then
    want="$(cd "$PACKAGE_DIST" && sha256sum "$(basename "$archive")")"
else
    want="$(cd "$PACKAGE_DIST" && shasum -a 256 "$(basename "$archive")")"
fi
check "checksum line" "$([ "$line" = "$want" ] && echo yes || echo no)"
check "checksum verifies" "$( (cd "$PACKAGE_DIST" && { command -v sha256sum > /dev/null && sha256sum -c "$(basename "$archive").sha256" > /dev/null || shasum -a 256 -c "$(basename "$archive").sha256" > /dev/null; }) && echo yes || echo no)"

# Refused: a platform nobody publishes, and a version that is not one.
check "unknown platform refused" "$(bash "$script" linux-arm64 > /dev/null 2>&1 && echo no || echo yes)"
check "bad version refused" "$(TRUCKPAINT_VERSION='../x' bash "$script" linux-x86_64 > /dev/null 2>&1 && echo no || echo yes)"
check "missing program refused" "$(CARGO_TARGET_DIR="$work/none" bash "$script" linux-x86_64 > /dev/null 2>&1 && echo no || echo yes)"

if [ "$failures" -gt 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all checks passed"
