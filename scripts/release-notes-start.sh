#!/usr/bin/env bash
# Finds where the notes of a release start: the highest stable tag (no suffix)
# below the one being released, so that a release that follows pre-releases
# lists everything they held.
#
# Usage: release-notes-start.sh TAG
#   TAG: the valid tag being released (see release-tag.sh).
#
# Prints `start=<tag>` (also appended to $GITHUB_OUTPUT when set); `start=` is
# empty when there is no stable tag yet. Tags are those of the repository in
# the current directory. A pre-release `vX.Y.Z-suffix` is below its release
# `vX.Y.Z`, so only stable tags of a lower X.Y.Z count for both.

set -euo pipefail

tag="${1:?release-notes-start: a tag is needed}"
core="${tag%%-*}"

start=""
while IFS= read -r candidate; do
    [[ "$candidate" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || continue
    [ "$candidate" != "$core" ] || continue
    lowest="$(printf '%s\n%s\n' "$candidate" "$core" | sort -V | head -n 1)"
    [ "$lowest" = "$candidate" ] || continue
    start="$candidate"
done < <(git tag -l 'v*' | sort -V)

echo "start=$start"
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "start=$start" >> "$GITHUB_OUTPUT"
fi
