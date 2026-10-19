## Why

TruckPaint has no installers: to try it, a player has to build it from source. We want to publish a versioned download for Windows, Linux and macOS from GitHub, and to do it by pushing a tag, so a release (or a test build from any branch) is one command and nothing else.

The version a player sees in the app must be the version of the download they installed.

## What Changes

- Pushing a tag `vMAJOR.MINOR.PATCH` publishes a **release** (marked Latest). The tagged commit must be on `main`; a stable tag on any other commit fails and publishes nothing.
- Pushing a tag `vMAJOR.MINOR.PATCH-suffix` (for example `v0.2.0-beta.1`) publishes a **pre-release** from any branch. It is never marked Latest.
- Each release holds three downloads, each with a SHA-256 checksum: Linux x86_64, Windows x86_64 and macOS arm64 (Apple silicon only). Each is an archive with the `truckpaint` program; no installer, no signature, no `.app` bundle yet.
- The release is created as a draft, filled with the downloads, then published only when all three were built, so a failed build never leaves a public release with a missing download. A failed run can be run again for the same tag.
- Release notes are generated from the merged pull requests since the last stable tag.
- The version comes from the tag, not from `Cargo.toml`. The app shows it in About, in the startup log, in the MCP server information and in the vehicle extraction report, from one place. A build that is not a release shows `dev`.
- A mise task builds the download of the current platform, so the same steps run locally and in CI.
- The release workflows are two small files (stable and pre-release) that share one reusable workflow for the build and the publication.

Out of scope, planned as a second change: an icon and a macOS `.app` bundle.

## Capabilities

### New Capabilities
- `release`: how a pushed tag becomes a GitHub release or pre-release, which branches may release, what the downloads are, and how a failed release is run again.
- `app-version`: the version number of the app, where it is shown, and what a development build shows.

### Modified Capabilities

None. The `continuous-integration` capability (checks on pull requests and `main`) is not changed.

## Impact

- New: `.github/workflows/release.yml`, `prerelease.yml` and `release-build.yml`; a `package` task in `mise.toml` with its script under `scripts/`.
- Code: the four reads of the app's version (About dialog and startup log, MCP server, vehicle extraction report) go through one constant that reads the build's version. The SCS reader keeps its own crate version: it is the version of that reader, not of the app.
- `actionlint` and `shellcheck` pinned in `mise.toml`, with a `lint:workflows` task; not part of `mise run ci`, so `ci.yml` is not changed.
- New `LICENSE` file (MIT) at the root, shipped in every download.
- `Cargo.toml`'s `version` is no longer the app's version; `CONTRIBUTING.md` says so and describes how to release.
- The README stops saying there are no installers, and the documentation site gets an install page in the four languages (done in the apply step with the docs agents).
- No change to project files, vehicle packages or the mod export.
