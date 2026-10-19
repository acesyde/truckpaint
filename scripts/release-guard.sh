#!/usr/bin/env bash
# Checks a request to release, before anything is built.
#
# Usage: release-guard.sh KIND TAG REF_TYPE REF_NAME
#   KIND:     `release` (the Release workflow) or `prerelease` (Pre-release).
#   TAG:      the tag typed in the workflow (see release-tag.sh).
#   REF_TYPE: `branch` or `tag`, what the workflow was started from.
#   REF_NAME: the name of that branch or tag.
#
# The rules: the tag is a valid version; a release has no suffix and a
# pre-release has one; a release is started from the branch `main`, a
# pre-release from anywhere.
#
# Prints `version=<tag without v>` and `prerelease=true|false` (also appended
# to $GITHUB_OUTPUT when set) when the request is allowed. Otherwise it prints
# nothing on the standard output, explains why on the standard error and fails.
# Whether the tag already exists needs the network: the workflow checks it.

set -euo pipefail

kind="${1:-}"
tag="${2:-}"
ref_type="${3:-}"
ref_name="${4:-}"

case "$kind" in
    release | prerelease) ;;
    *)
        echo "release: the kind is 'release' or 'prerelease', not '$kind'" >&2
        exit 1
        ;;
esac

# The tag script's own output is read here, not appended to $GITHUB_OUTPUT: a
# refused request must leave nothing behind.
if ! parsed="$(env -u GITHUB_OUTPUT bash "$(dirname "$0")/release-tag.sh" "$tag")"; then
    exit 1
fi
prerelease=false
grep -qx 'prerelease=true' <<< "$parsed" && prerelease=true

if [ "$kind" = release ] && [ "$prerelease" = true ]; then
    echo "release: $tag has a suffix, so it is a pre-release: run the Pre-release workflow" >&2
    exit 1
fi
if [ "$kind" = prerelease ] && [ "$prerelease" = false ]; then
    echo "release: $tag has no suffix, so it is a release: run the Release workflow, or add a suffix such as $tag-beta.1" >&2
    exit 1
fi
if [ "$kind" = release ] && { [ "$ref_type" != branch ] || [ "$ref_name" != main ]; }; then
    echo "release: a release comes from main, but this run is on the $ref_type '$ref_name': start Release from main, or run Pre-release for a test version" >&2
    exit 1
fi

emit() {
    echo "$1"
    if [ -n "${GITHUB_OUTPUT:-}" ]; then
        echo "$1" >> "$GITHUB_OUTPUT"
    fi
}
emit "version=${tag#v}"
emit "prerelease=$prerelease"
