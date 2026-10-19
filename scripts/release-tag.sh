#!/usr/bin/env bash
# Checks the tag typed to release (in the Release or Pre-release workflow) and
# says what it releases.
#
# Usage: release-tag.sh TAG
#   TAG: `v` and a semantic version, `vMAJOR.MINOR.PATCH` for a release,
#        `vMAJOR.MINOR.PATCH-suffix` for a pre-release (`v0.2.0-beta.1`).
#        Build metadata (`+...`) and leading zeros are refused.
#
# Prints `version=<tag without v>` and `prerelease=true|false` (also appended
# to $GITHUB_OUTPUT when set). Fails, printing nothing, on anything else.

set -euo pipefail

tag="${1:-}"

number='(0|[1-9][0-9]*)'
identifier='(0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)'
pattern="^v${number}\\.${number}\\.${number}(-${identifier}(\\.${identifier})*)?\$"

if ! [[ "$tag" =~ $pattern ]]; then
    echo "release: '$tag' is not a release tag: use vMAJOR.MINOR.PATCH or vMAJOR.MINOR.PATCH-suffix" >&2
    exit 1
fi

version="${tag#v}"
prerelease=false
[[ "$version" == *-* ]] && prerelease=true

emit() {
    echo "$1"
    if [ -n "${GITHUB_OUTPUT:-}" ]; then
        echo "$1" >> "$GITHUB_OUTPUT"
    fi
}
emit "version=$version"
emit "prerelease=$prerelease"
