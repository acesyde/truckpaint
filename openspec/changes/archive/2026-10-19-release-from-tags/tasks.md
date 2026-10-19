## 1. One version constant

- [x] 1.1 Add the version constant to `tp-core` (`option_env!("TRUCKPAINT_VERSION")`, fallback `"dev"`) and a unit test that checks the fallback when the variable is not set
- [x] 1.2 Use it in the About dialog and the startup log (`tp-app`) and in the MCP server information (`tp-mcp`); leave `tp_scs::VERSION` as it is; verify with `grep -rn CARGO_PKG_VERSION crates` that only `tp-scs` and the manifests still read it
- [x] 1.3 Give the extraction report's `tool_version` the app's version through `tp_core::VERSION` (`tp-pack` depends on `tp-core` for it, see design.md) and verify the extraction tests still pass and the report says `dev` in a test build
- [x] 1.4 Add a headless UI test that opens About and checks it shows `Version dev`, and run `TRUCKPAINT_VERSION=0.2.0-beta.1 cargo test -p tp-app` for the same test with the variable set to prove the number follows it
- [x] 1.5 Delegate `mise run ci` to the rust-ui-tester agent and fix its failures

## 2. License

- [x] 2.1 Add `LICENSE` (standard MIT text, `Copyright (c) 2026 acesyde`) at the root, and verify it matches `license = "MIT"` in `Cargo.toml`
- [x] 2.2 Point the README's "Credits and licenses" and `CONTRIBUTING.md`'s License section to the file

## 3. Packaging task

- [x] 3.1 Write the packaging script under `scripts/` that detects the platform, builds with `--release --locked` and writes `dist/truckpaint-<version>-<platform>.<tar.gz|zip>` with one top folder holding the program and `LICENSE`; verify by running it on this Mac and listing the archive (macOS arm64 `.zip`, executable bit kept)
- [x] 3.2 Add its test script next to the script (same pattern as `ci-changes-test.sh`) covering the names for the three platforms, the content of the archive and the `dev` default; verify it passes
- [x] 3.3 Register the `package` task in `mise.toml` (version taken from `TRUCKPAINT_VERSION`, default `dev`) and verify `mise run package` produces the archive and a checksum line in the same format as `SHA256SUMS`
- [x] 3.4 Add `dist/` to `.gitignore` and verify `git status` is clean after packaging

## 4. Tag checks

- [x] 4.1 Write the tag-validation script under `scripts/` (valid `vMAJOR.MINOR.PATCH[-suffix]`, prints the version, says release or pre-release) with a test script covering `v0.2.0`, `v0.2.0-beta.1`, `v0.2`, `v1.0.0-`, `v1.2.3-!` and leading zeros; verify it passes
- [x] 4.2 Add the "last stable tag below this one" helper used for the release notes, with test cases (no stable tag yet, pre-releases only, betas then stable); verify it passes
- [x] 4.3 Expose both as mise tasks like `ci:changes` and verify they run locally the same way as in CI

## 5. Workflows

- [x] 5.1 Write `.github/workflows/release-build.yml` (`workflow_call`, inputs `version` and `prerelease`): the three-platform matrix running `mise run package`, then one job that collects the archives, writes `SHA256SUMS`, creates or reuses the draft, uploads with replacement, sets the mark and publishes; same SHA-pinned actions, mise version and Linux system packages as `ci.yml`; verify the file passes `actionlint` and `mise run ci:changes` still treats workflow files as code
- [x] 5.2 Write `.github/workflows/release.yml` (tag filter for releases, tag validation, ancestor-of-`main` check in its own first job, `contents: write`, calls the reusable workflow); verify with `actionlint`
- [x] 5.3 Write `.github/workflows/prerelease.yml` (tag filter for suffixes, tag validation, no branch check, calls the reusable workflow); verify with `actionlint`, and that the two tag filters match no common tag (check the examples of the release spec by hand)
- [x] 5.4 Add a concurrency group per tag to the two trigger workflows (a group shared with the called workflow would cancel it) and verify it in the files
- [x] 5.5 Update the `continuous-integration` spec only if `ci.yml` had to change; otherwise state in the change that it did not

- [x] 5.6 Pin `actionlint` and `shellcheck` in `mise.toml` and add `mise run lint:workflows` (actionlint on the workflows, shellcheck on `scripts/*.sh`); verify it passes, and that it fails on a workflow with an unknown property and an unquoted shell variable

## 6. Documentation

- [x] 6.1 Add a "Releasing" section to `CONTRIBUTING.md` (tags to push, release only from `main`, version from the tag, `Cargo.toml` not the app's version, how to delete a mistaken tag, `mise run package` locally) and drop the README's "no installers yet" sentence for a link to the releases page; verify by reading both
- [x] 6.2 With the docs-writer agent, update the English Installation page, which says there are no downloads yet, (where to download, which file, unsigned programs: SmartScreen on Windows and Gatekeeper right-click on macOS, how to check `SHA256SUMS`); with docs-translator, the French, Spanish and German pages with matching `sidebar.order`; verify `mise run doc:build` passes
- [x] 6.3 Delegate the docs-drift-checker agent before archiving and list the stale pages in the archive summary

## 7. First releases (by the maintainer)

- [x] 7.1 Push `v0.1.0-beta.1` on a feature branch and verify the Actions run builds the three platforms, a pre-release appears with the three archives and `SHA256SUMS`, it is not Latest, and the downloaded app's About says `0.1.0-beta.1`
- [ ] 7.2 Push `v0.1.0` on a commit that is not on `main` and verify the run fails with the "not on `main`" message and creates no draft; delete that tag (Not done: replaced by the manual workflows of `release-immutable-attestations`, which rehearse the circuit again.)
- [ ] 7.3 Push `v0.1.0` on `main` and verify it is published as Latest with notes since the start of the history; run the workflow again from the Actions page and verify no second release appears (Not done: replaced by the manual workflows of `release-immutable-attestations`, which rehearse the circuit again.)
