# release Specification

## Purpose
Defines how pushing a version tag to GitHub publishes a downloadable release of TruckPaint for Windows, Linux and macOS, which tags make a release or a pre-release, and what a failed release leaves behind.

## Requirements

### Requirement: Tag names
A release is started by pushing a tag named `v` followed by a semantic version: `vMAJOR.MINOR.PATCH` for a release, `vMAJOR.MINOR.PATCH-suffix` for a pre-release, where the suffix is one or more dot-separated identifiers of letters, digits and hyphens (`v0.2.0-beta.1`, `v0.2.0-hotfix.2`). A tag that is not a valid semantic version SHALL NOT publish anything.

#### Scenario: Release tag
- **WHEN** the tag `v0.2.0` is pushed
- **THEN** a release named `v0.2.0` is published

#### Scenario: Pre-release tag
- **WHEN** the tag `v0.2.0-beta.1` is pushed
- **THEN** a pre-release named `v0.2.0-beta.1` is published

#### Scenario: Invalid tag
- **WHEN** the tag `v0.2` or `v1.0.0-` is pushed
- **THEN** nothing is published and no download is built

### Requirement: Releases come from main
A release (a tag without a suffix) SHALL be published only when the tagged commit is part of `main`. A release tag on any other commit SHALL fail before anything is built, with a message saying the commit is not on `main`, and SHALL NOT create a release, a draft or a download.

#### Scenario: Release from main
- **WHEN** `v0.2.0` is pushed on a commit that is on `main`
- **THEN** the release is built and published

#### Scenario: Release from a feature branch
- **WHEN** `v0.2.0` is pushed on a commit that is only on the branch `feat/gradients`
- **THEN** the run fails with a message that the commit is not on `main` and no release is created

### Requirement: Pre-releases come from any branch
A pre-release (a tag with a suffix) SHALL be published whatever branch holds the tagged commit: `main`, a feature, documentation or hotfix branch, or no branch at all. A pre-release SHALL be marked as a pre-release on GitHub and SHALL NOT be the Latest release.

#### Scenario: Pre-release from a feature branch
- **WHEN** `v0.2.0-beta.1` is pushed on a commit that is only on the branch `feat/gradients`
- **THEN** a pre-release is published and the Latest release does not change

#### Scenario: Latest ignores pre-releases
- **WHEN** `v0.1.0` is the Latest release and `v0.2.0-beta.1` is published
- **THEN** the link to the Latest release still opens `v0.1.0`

### Requirement: Downloads
Each release and pre-release SHALL hold one download per platform, built from the tagged commit in release mode, and a checksum file listing the SHA-256 of each:
- Linux x86_64: a `.tar.gz` archive;
- Windows x86_64: a `.zip` archive;
- macOS arm64 (Apple silicon): a `.zip` archive.

Each archive SHALL contain the `truckpaint` program (`truckpaint.exe` on Windows) and the license. Its file name SHALL name the version and the platform (`truckpaint-v0.2.0-macos-arm64.zip`). No Intel macOS download is published.

#### Scenario: Files of a release
- **WHEN** `v0.2.0` is published
- **THEN** the release holds `truckpaint-v0.2.0-linux-x86_64.tar.gz`, `truckpaint-v0.2.0-windows-x86_64.zip`, `truckpaint-v0.2.0-macos-arm64.zip` and `SHA256SUMS`

#### Scenario: Checksums match
- **WHEN** a player checks a downloaded archive against `SHA256SUMS`
- **THEN** the SHA-256 of the file is the one listed for it

### Requirement: License file
The repository SHALL hold a `LICENSE` file at its root with the text of the MIT license, matching the license declared by the project, and every download SHALL ship that file next to the program.

#### Scenario: License in the archive
- **WHEN** a player opens a downloaded archive
- **THEN** it contains `LICENSE` with the MIT license text

### Requirement: Published only when complete
The release SHALL be created as a draft, receive its downloads and checksum file, and be published only once all three downloads are built. When any build fails, the release SHALL stay unpublished and no tag SHALL be moved or deleted automatically.

#### Scenario: One platform fails
- **WHEN** the Windows build fails and the other two succeed
- **THEN** the release stays a draft and nothing is public

### Requirement: A failed release can be run again
Running a failed release again for the same tag SHALL complete it: an existing draft is reused, files already attached are replaced, and the release is published. Running it again for a release that is already published SHALL NOT create a second release.

#### Scenario: Run again after a failure
- **WHEN** the macOS build failed, then the run is started again for the same tag and succeeds
- **THEN** one release exists, with the three downloads and the checksum file, and it is published

#### Scenario: Release created by hand
- **WHEN** a release for the tag was already created in the GitHub interface before the run
- **THEN** the downloads are attached to it and its pre-release or Latest mark follows the tag

### Requirement: Release notes
The notes of a release SHALL list the pull requests merged since the last release (no suffix), so that a release that follows pre-releases lists everything those pre-releases held. The notes of a pre-release SHALL list the pull requests since the last release as well.

#### Scenario: Release after betas
- **WHEN** `v0.2.0` is published after `v0.2.0-beta.1` and `v0.2.0-beta.2`, and the last release was `v0.1.0`
- **THEN** the notes list every pull request merged since `v0.1.0`

### Requirement: Same steps locally
The steps that turn the built program into the download of the current platform SHALL be available as a task that can be run on a developer machine, and the release SHALL use that same task.

#### Scenario: Package locally
- **WHEN** a contributor runs the package task on a Mac with Apple silicon
- **THEN** it produces the macOS arm64 archive, with the same content and layout as the published one

### Requirement: Contributor guide
`CONTRIBUTING.md` SHALL describe how to publish a release and a pre-release (the tag to push), that a release tag needs a commit on `main`, that the app's version comes from the tag and not from `Cargo.toml`, and how to remove a tag pushed by mistake.

#### Scenario: Releasing
- **WHEN** a maintainer reads the guide
- **THEN** it gives the exact commands to publish a release and a pre-release
