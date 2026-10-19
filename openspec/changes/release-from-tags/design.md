## Context

See proposal.md for why. Today:

- `.github/workflows/ci.yml` checks pull requests and `main`; its build matrix only runs on Ubuntu to save minutes. Actions are pinned by commit SHA, mise is pinned to `2026.10.3`, and every step goes through a `mise run` task.
- The version is the workspace `version = "0.1.0"`, read through `env!("CARGO_PKG_VERSION")` in five places. Four are the app's version: the About dialog and the startup log (`tp-app`), the MCP server information (`tp-mcp`) and the extraction report's `tool_version` (`tp-pack`). The fifth, `tp_scs::VERSION`, is the version of the SCS reader and is reported next to the app's in the extraction report; it stays as it is.
- `tp-core` is a dependency of `tp-app` and `tp-mcp`, but not of `tp-pack`, which only depends on `tp-vehicles` and `tp-scs`.
- The repository has no license file: `MIT` is only declared in `Cargo.toml`, the README and CONTRIBUTING.
- The program is `target/release/truckpaint` (`.exe` on Windows). There is no icon, no bundle and no signing.
- The app does not parse or compare its own version; `NewerVersion` in `tp-file` is about the file format.

## Goals / Non-Goals

**Goals:**
- A pushed tag is the only thing a maintainer does to release.
- The same packaging steps run locally and in CI.
- The version in the app cannot drift from the tag.

**Non-Goals:**
- Installers, macOS `.app` bundle, icons, code signing or notarization (second change, and signing is not planned).
- Intel macOS, Linux arm64, Windows arm64.
- An in-app update check.
- Changing `ci.yml`, or the way it is built.

## Decisions

### Two small trigger workflows and one reusable workflow
`release.yml` and `prerelease.yml` only decide *whether* a tag may release; `release-build.yml` (`workflow_call`) builds and publishes. Their tag filters do not overlap, because GitHub matches a filter against the whole tag name:

```
release.yml      tags: 'v[0-9]+.[0-9]+.[0-9]+'
prerelease.yml   tags: 'v[0-9]+.[0-9]+.[0-9]+-*'
```

Each trigger workflow validates the tag in full (the filters are only globs; `v1.2.3-!` matches the second one) and passes `version` and `prerelease` to the reusable workflow. The "commit is on `main`" check lives only in `release.yml`, so the rule has no condition to cross-read.

Alternative: one workflow with `if:` conditions on the tag shape. Smaller, but the guard and the permissions then sit among conditions, and the Actions list would not tell a release from a pre-release at a glance.

A malformed tag such as `v0.2` matches neither filter and starts nothing, silently. Accepted: a third catch-all workflow just to fail loudly costs a file and runner time for a rare typo. CONTRIBUTING says it.

### "On `main`" means an ancestor of `main`
A tag has no branch. `release.yml` checks out with full history and runs `git merge-base --is-ancestor <tag commit> origin/main`. A release tag on an older `main` commit passes (it is reachable); a tag on a feature commit does not. The check runs in its own first job, so a refusal costs seconds and builds nothing. The failed tag is left alone: the workflow never deletes a tag.

### The version comes from a build variable, through one constant
The workflow sets `TRUCKPAINT_VERSION` (the tag without `v`) when it builds. One constant in `tp-core` is `option_env!("TRUCKPAINT_VERSION")` with `"dev"` as the fallback, and the four reads of the app's version use it. `option_env!` makes Cargo rebuild the crate when the variable changes, and the constant is evaluated in one crate, so the four places cannot differ. `tp-pack` gains a dependency on `tp-core` for it: the report is written only by the `tpv extract` command, and one shared constant cannot drift, which a second copy could.

Alternatives:
- Rewrite `version` in `Cargo.toml` and `Cargo.lock` in CI: the workspace members' entries in `Cargo.lock` change too, so `--locked` fails unless the lock is rewritten as well (`cargo update --workspace`), and it is a text edit on a TOML file in every release.
- Require `Cargo.toml` to equal the tag (a bump pull request before every release): the repository then says the true version, but it adds a step to each release and none to pre-releases from branches, which could not edit the manifest without polluting their pull request. Not chosen.

Consequence: `Cargo.toml`'s version is no longer the app's. It stays `0.1.0`, CONTRIBUTING states it, and a local build says `dev` rather than a number that looks real.

### Packaging is a mise task
`mise run package` builds with `--release --locked` and writes `dist/truckpaint-<version>-<platform>.<ext>`; the platform is detected from the machine. A script under `scripts/` does the work (the same pattern as `ci-changes.sh`), with a test script next to it. The workflow calls only the task, then uploads `dist/*`. The version for a local run is `dev`, so the file is `truckpaint-dev-<platform>.<ext>`.

Archive layout: a single top folder `truckpaint-<version>-<platform>/` holding the program and `LICENSE`. Linux uses `tar.gz` (keeps the executable bit), Windows and macOS use `zip`. The macOS program is built on the arm64 runner and is not signed: Gatekeeper blocks it until the player opens it with a right-click; the release notes and the documentation say so.

### Draft, upload, publish
The `package` jobs run in a matrix (`ubuntu-latest`, `windows-latest`, `macos-latest`, which is arm64) and upload their archive as a workflow artifact. A final job, only when all three succeeded, collects them, writes `SHA256SUMS`, then:

1. creates the release as a draft for the tag (`gh release create --draft`), or reuses it if it exists (a release made by hand in the GitHub interface);
2. uploads the files with replacement (`--clobber`);
3. sets the pre-release or Latest mark from the tag and publishes.

Publishing is the last step, so a failed run leaves nothing public. Running the failed workflow again repeats the same steps with the same results. Concurrency is grouped by tag, in the two trigger workflows only (a group set in both caller and called workflow makes the called one cancel itself), so two runs for the same tag wait for each other.

The release is created with the tag pointing at the tagged commit; the workflow never moves a tag.

### Release notes
`gh release create --generate-notes` with the last stable tag as the start (`--notes-start-tag`), found by listing tags without a suffix and taking the highest below the one being released. With no stable tag yet (the first release), GitHub's default start is used. Pull requests in this repository are written for players, so the generated list is readable as it is.

### License file
The new `LICENSE` holds the standard MIT text with `Copyright (c) 2026 acesyde`, as given by the maintainer. Each archive copies it as is.

### `ci.yml` is not changed
The release workflows are new files and share nothing with `ci.yml` but its pinned action versions, so the `continuous-integration` spec stays as it is.

### Permissions
Only the release jobs have `contents: write`; `ci.yml` keeps `contents: read`. A reusable workflow cannot raise the permissions of its caller, so the two trigger workflows grant it.

## Risks / Trade-offs

- [A tag with a wrong number ships as that number, because nothing else holds the version] → the tag is validated as semantic versioning, the notes name the commit, and a bad pre-release can be deleted; a bad release is fixed with a new tag.
- [Pre-release tags accumulate on commits that are never merged] → harmless to players (Latest ignores them); a cleanup is left for later.
- [Windows or macOS builds were never run in CI, because the matrix is Ubuntu only] → the first pre-release exercises them; a system-specific build failure shows up as a failed, unpublished draft, not a bad release. The Linux packages need the same system libraries as `ci.yml`.
- [Unsigned programs: SmartScreen on Windows, Gatekeeper on macOS] → documented in the notes and the install page; signing is a separate decision.
- [Tests are not run by the release] → they ran on the pull request and on `main` already; the release builds the same commit. A pre-release from a branch has the pull request's checks if it has one, otherwise none.

## Migration Plan

No data migration. Order: land the version constant and the package task first (they change nothing for players), then the workflows, then publish `v0.1.0-beta.1` from a branch to try the three platforms, then the first release from `main`. Rollback is deleting the workflows; releases already published are kept or deleted by hand.
