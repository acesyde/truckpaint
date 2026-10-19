#!/usr/bin/env bash
# Self-test of release-tag.sh, release-guard.sh and release-notes-start.sh (the
# last on a throwaway Git repository).

set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
tag_script="$here/release-tag.sh"
guard_script="$here/release-guard.sh"
start_script="$here/release-notes-start.sh"

failures=0
report() {
    if [ "$2" = yes ]; then
        echo "ok   $1"
    else
        echo "FAIL $1${3:+: $3}"
        failures=$((failures + 1))
    fi
}

# expect_tag TAG VERSION PRERELEASE: the tag is valid and means this.
expect_tag() {
    local out
    out="$(bash "$tag_script" "$1" 2> /dev/null)" || out="refused"
    local want
    want="$(printf 'version=%s\nprerelease=%s' "$2" "$3")"
    if [ "$out" = "$want" ]; then report "tag $1" yes; else report "tag $1" no "got $out"; fi
}

# refuse_tag TAG: the tag is refused and nothing is printed.
refuse_tag() {
    local out
    if out="$(bash "$tag_script" "$1" 2> /dev/null)"; then
        report "refuse $1" no "accepted: $out"
    elif [ -n "$out" ]; then
        report "refuse $1" no "printed: $out"
    else
        report "refuse $1" yes
    fi
}

expect_tag v0.2.0 0.2.0 false
expect_tag v10.20.30 10.20.30 false
expect_tag v0.2.0-beta.1 0.2.0-beta.1 true
expect_tag v0.2.0-hotfix.2 0.2.0-hotfix.2 true
expect_tag v1.0.0-rc-1 1.0.0-rc-1 true
expect_tag v1.0.0-0 1.0.0-0 true
refuse_tag v0.2
refuse_tag v1
refuse_tag v1.0.0-
refuse_tag 'v1.2.3-!'
refuse_tag v01.2.3
refuse_tag v1.2.03
refuse_tag v1.0.0-beta.01
refuse_tag v1.0.0-beta..1
refuse_tag v1.0.0+build
refuse_tag 0.2.0
refuse_tag V0.2.0
refuse_tag ''

# expect_guard KIND TAG REF_TYPE REF_NAME VERSION PRERELEASE: the request is
# allowed, and says so on the standard output and in $GITHUB_OUTPUT.
expect_guard() {
    local out file name="guard $1 $2 on $3 $4"
    file="$(mktemp)"
    out="$(GITHUB_OUTPUT="$file" bash "$guard_script" "$1" "$2" "$3" "$4" 2> /dev/null)" || out="refused"
    local want
    want="$(printf 'version=%s\nprerelease=%s' "$5" "$6")"
    if [ "$out" = "$want" ] && [ "$(cat "$file")" = "$want" ]; then
        report "$name" yes
    else
        report "$name" no "got $out"
    fi
    rm -f "$file"
}

# refuse_guard KIND TAG REF_TYPE REF_NAME MESSAGE: the request is refused with
# a message that contains MESSAGE, and nothing is printed or written.
refuse_guard() {
    local out err file name="refuse $1 $2 on $3 $4"
    file="$(mktemp)"
    err="$(mktemp)"
    if out="$(GITHUB_OUTPUT="$file" bash "$guard_script" "$1" "$2" "$3" "$4" 2> "$err")"; then
        report "$name" no "accepted: $out"
    elif [ -n "$out" ] || [ -s "$file" ]; then
        report "$name" no "left output behind"
    elif ! grep -qF -- "$5" "$err"; then
        report "$name" no "message was: $(cat "$err")"
    else
        report "$name" yes
    fi
    rm -f "$file" "$err"
}

expect_guard release v0.2.0 branch main 0.2.0 false
expect_guard prerelease v0.2.0-beta.1 branch main 0.2.0-beta.1 true
expect_guard prerelease v0.2.0-beta.1 branch feat/gradients 0.2.0-beta.1 true
expect_guard prerelease v1.0.0-rc.1 branch docs/install 1.0.0-rc.1 true
refuse_guard release v0.2.0 branch feat/gradients "comes from main"
refuse_guard release v0.2.0 branch mainline "comes from main"
refuse_guard release v0.2.0 tag main "comes from main"
refuse_guard release v0.2.0-beta.1 branch main "Pre-release workflow"
refuse_guard prerelease v0.2.0 branch feat/gradients "Release workflow"
refuse_guard prerelease v0.2.0 branch main "Release workflow"
refuse_guard release v0.2 branch main "not a release tag"
refuse_guard prerelease v1.0.0- branch main "not a release tag"
refuse_guard release "" branch main "not a release tag"
refuse_guard publish v0.2.0 branch main "'release' or 'prerelease'"

repo="$(mktemp -d)"
trap 'rm -rf "$repo"' EXIT
cd "$repo"
git init -q -b main
git config user.email "test@example.invalid"
git config user.name "release test"
git config commit.gpgsign false
git config tag.gpgsign false
git config tag.forceSignAnnotated false
git commit -q --allow-empty -m initial

# start_of TAG prints the notes start for TAG in the repository.
start_of() {
    bash "$start_script" "$1" | tail -n 1
}

expect_start() {
    local got
    got="$(start_of "$1")"
    if [ "$got" = "start=$2" ]; then report "notes of $1 start at '${2:-the beginning}'" yes; else report "notes of $1" no "got $got"; fi
}

# No stable tag yet: the notes start at the beginning of the history.
expect_start v0.1.0 ""
git tag v0.1.0-beta.1
git tag v0.1.0-beta.2
expect_start v0.1.0-beta.3 ""
expect_start v0.1.0 ""

# After a first release.
git tag v0.1.0
expect_start v0.2.0-beta.1 v0.1.0
expect_start v0.2.0 v0.1.0

# Betas of the next release do not count: a release lists everything since the
# last release.
git tag v0.2.0-beta.1
git tag v0.2.0-beta.2
expect_start v0.2.0 v0.1.0
expect_start v0.2.0-beta.3 v0.1.0

# The highest lower release is chosen, numerically (v0.9.0 < v0.10.0).
git tag v0.9.0
git tag v0.10.0
expect_start v0.10.1 v0.10.0
expect_start v0.10.0 v0.9.0
expect_start v1.0.0-rc.1 v0.10.0

# A hotfix on an older line starts at the release below it.
expect_start v0.9.1 v0.9.0

if [ "$failures" -gt 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all checks passed"
