#!/usr/bin/env bash
# Builds TruckPaint in release mode and packs the download of this platform:
# one archive holding a folder with the program and the license.
#
# Usage: package.sh [PLATFORM]
#   PLATFORM: linux-x86_64, windows-x86_64 or macos-arm64; detected from the
#             machine when omitted. Any other platform is refused.
#
# Environment:
#   TRUCKPAINT_VERSION   the version to build in (a release tag without its
#                        `v`); `dev` when unset, as for any local build.
#   PACKAGE_DIST         where the files are written (default: dist).
#   PACKAGE_SKIP_BUILD   set to 1 to pack the program already in
#                        $CARGO_TARGET_DIR/release (the script's test does).
#
# Writes dist/truckpaint-<version>-<platform>.<tar.gz|zip> and next to it
# <archive>.sha256, one line in the format of a SHA256SUMS file. A release
# archive is named after its tag (`truckpaint-v0.2.0-macos-arm64.zip`).

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

version="${TRUCKPAINT_VERSION:-dev}"
dist="${PACKAGE_DIST:-dist}"
target="${CARGO_TARGET_DIR:-target}"

if ! [[ "$version" =~ ^[0-9A-Za-z][0-9A-Za-z.+-]*$ ]]; then
    echo "package: '$version' is not a version" >&2
    exit 1
fi

detect_platform() {
    local system machine
    system="$(uname -s)"
    machine="$(uname -m)"
    case "$system-$machine" in
        Linux-x86_64) echo linux-x86_64 ;;
        Darwin-arm64) echo macos-arm64 ;;
        MINGW*-x86_64 | MSYS*-x86_64 | CYGWIN*-x86_64) echo windows-x86_64 ;;
        *)
            echo "package: no download is published for $system $machine" >&2
            return 1
            ;;
    esac
}

platform="${1:-$(detect_platform)}"
case "$platform" in
    linux-x86_64) program=truckpaint extension=tar.gz ;;
    windows-x86_64) program=truckpaint.exe extension=zip ;;
    macos-arm64) program=truckpaint extension=zip ;;
    *)
        echo "package: unknown platform '$platform'" >&2
        exit 1
        ;;
esac

label="$version"
[ "$version" = dev ] || label="v$version"
name="truckpaint-$label-$platform"

if [ "${PACKAGE_SKIP_BUILD:-}" != 1 ]; then
    if [ -n "${TRUCKPAINT_VERSION:-}" ]; then
        export TRUCKPAINT_VERSION
    fi
    cargo build --release --locked -p tp-app
fi

binary="$target/release/$program"
if [ ! -f "$binary" ]; then
    echo "package: $binary does not exist" >&2
    exit 1
fi

mkdir -p "$dist"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
mkdir "$stage/$name"
cp "$binary" "$stage/$name/$program"
chmod 755 "$stage/$name/$program"
cp LICENSE "$stage/$name/LICENSE"

archive="$name.$extension"
rm -f "$dist/$archive"
out="$(cd "$dist" && pwd)/$archive"
if [ "$extension" = tar.gz ]; then
    tar -czf "$out" -C "$stage" "$name"
elif command -v zip > /dev/null; then
    (cd "$stage" && zip -qr "$out" "$name")
elif command -v 7z > /dev/null; then
    # Windows runners: Git Bash has no zip.
    (cd "$stage" && 7z a -tzip -bso0 -bsp0 "$out" "$name")
else
    echo "package: neither zip nor 7z is installed" >&2
    exit 1
fi

if command -v sha256sum > /dev/null; then
    sum="$(sha256sum "$out" | cut -d' ' -f1)"
else
    sum="$(shasum -a 256 "$out" | cut -d' ' -f1)"
fi
echo "$sum  $archive" > "$dist/$archive.sha256"

echo "$dist/$archive"
