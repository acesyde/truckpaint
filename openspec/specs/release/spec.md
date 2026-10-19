# release Specification

## Purpose
Defines how running the Release or Pre-release workflow on GitHub publishes a downloadable, immutable and attested release of TruckPaint for Windows, Linux and macOS, which tags make a release or a pre-release, and what a failed release leaves behind.

## Requirements

### Requirement: Tag names
The tag of a release is typed when the workflow is started: `v` followed by a semantic version, `vMAJOR.MINOR.PATCH` for Release and `vMAJOR.MINOR.PATCH-suffix` for Pre-release, where the suffix is one or more dot-separated identifiers of letters, digits and hyphens (`v0.2.0-beta.1`, `v0.2.0-hotfix.2`). The version of the app is the tag without its `v`. A tag that is not a valid semantic version SHALL NOT publish anything.

#### Scenario: Release tag
- **WHEN** Release is run with the tag `v0.2.0`
- **THEN** a release named `v0.2.0` is published

#### Scenario: Pre-release tag
- **WHEN** Pre-release is run with the tag `v0.2.0-beta.1`
- **THEN** a pre-release named `v0.2.0-beta.1` is published

#### Scenario: Invalid tag
- **WHEN** a workflow is run with the tag `v0.2` or `v1.0.0-`
- **THEN** nothing is published and no download is built

### Requirement: Releases come from main
Release SHALL run only from the branch `main`. Started from any other branch, or from a tag, it SHALL fail before anything is built, with a message saying a release comes from `main`, and SHALL NOT create a release, a draft, a tag or a download.

#### Scenario: Release from main
- **WHEN** Release is run on `main` with the tag `v0.2.0`
- **THEN** the release is built and published

#### Scenario: Release from a feature branch
- **WHEN** Release is run on the branch `feat/gradients` with the tag `v0.2.0`
- **THEN** the run fails with a message that a release comes from `main` and nothing is created

### Requirement: Pre-releases come from any branch
Pre-release SHALL run from any branch: `main`, a feature, documentation or hotfix branch. A pre-release SHALL be marked as a pre-release on GitHub and SHALL NOT be the Latest release. A release SHALL be marked Latest.

#### Scenario: Pre-release from a feature branch
- **WHEN** Pre-release is run on the branch `feat/gradients` with the tag `v0.2.0-beta.1`
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
The release SHALL be created as a draft, receive its downloads, checksum file and attestations, and be published only once all three downloads are built, attached and attested, and only when `publish` is ticked. When any build fails, the release SHALL stay a draft with no tag, and nothing SHALL be deleted automatically.

#### Scenario: One platform fails
- **WHEN** the Windows build fails and the other two succeed
- **THEN** the release stays a draft with no tag and nothing is public

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
`CONTRIBUTING.md` SHALL describe how to publish a release and a pre-release (running Release or Pre-release from the Actions tab, or with `gh workflow run`, and the `tag` and `publish` choices), that Release only runs from `main`, that the Run workflow button needs the workflow on `main` and the chosen branch must contain the workflow files, that the app's version comes from the tag and not from `Cargo.toml`, that releases are immutable and a published tag name is never reused, how to check a download with its attestation, and that a first release is rehearsed with a pre-release of a new version.

#### Scenario: Releasing
- **WHEN** a maintainer reads the guide
- **THEN** it gives the exact steps and commands to publish a release and a pre-release, and to rehearse with `publish` not ticked

#### Scenario: Branch older than the workflows
- **WHEN** a maintainer chooses a feature branch created before the workflow files existed
- **THEN** the guide says to update the branch from `main` first

### Requirement: Manual start
A release SHALL be started from the Actions tab of the repository (or with `gh workflow run`), by running one of two workflows: **Release**, which publishes a release, and **Pre-release**, which publishes a pre-release. Each SHALL ask for the `tag` (required) and `publish` (a choice, ticked by default). Pushing a tag SHALL NOT start a release, a build or a download.

#### Scenario: Starting a release
- **WHEN** the maintainer runs Release on `main` with the tag `v0.2.0` and `publish` ticked
- **THEN** the downloads are built and the release `v0.2.0` is published

#### Scenario: Pushing a tag
- **WHEN** a tag `v0.2.0` is pushed
- **THEN** no workflow starts and nothing is published

### Requirement: Checks before the build
A request SHALL be checked before anything is built, and refused with a message that says why, when:
- the tag is not a valid version;
- the tag already exists in the repository;
- the tag has a suffix in Release, or has no suffix in Pre-release.

A refused request SHALL NOT create a draft, a release, a tag or a download.

#### Scenario: Tag that already exists
- **WHEN** Release is run for `v0.1.0` and that tag already exists
- **THEN** the run fails at once, with a message that the tag exists, and nothing is built

#### Scenario: Suffix in the wrong workflow
- **WHEN** Release is run with the tag `v0.2.0-beta.1`, or Pre-release is run with the tag `v0.2.0`
- **THEN** the run fails at once with a message naming the other workflow, and nothing is built

### Requirement: The built commit is the one released
The downloads SHALL be built from the commit at the head of the chosen branch at the moment the workflow is started. The release SHALL be created for that exact commit, even if the branch moves during the build, and the tag SHALL be created on that commit when the release is published.

#### Scenario: Branch moves during the build
- **WHEN** a commit is pushed to the branch after the workflow started
- **THEN** the release and its tag are for the commit that was built, not for the new one

### Requirement: Draft for review
When `publish` is not ticked, the workflow SHALL build the downloads, attach them and their attestations, and leave the release as a draft without creating its tag. The maintainer SHALL be able to publish that draft by hand from GitHub. Running the workflow again for the same tag with `publish` ticked SHALL complete and publish it.

#### Scenario: Rehearsal
- **WHEN** Pre-release is run with `publish` not ticked
- **THEN** a draft exists with the three downloads and `SHA256SUMS`, no tag exists and nothing is public

#### Scenario: Publishing the draft by hand
- **WHEN** the maintainer publishes that draft in GitHub
- **THEN** the release becomes public and its tag is created on the built commit

### Requirement: Immutable releases
Releases SHALL be immutable: once a release is published, its tag and its files SHALL never change, and the workflows SHALL attach every file to a draft and SHALL NOT add, replace or remove files of a published release. The title, the notes and the pre-release mark MAY still be edited after publication. A published release and its tag name SHALL NOT be reused: a mistake is fixed by publishing the next version.

#### Scenario: Files of a published release
- **WHEN** someone tries to add or replace a file of a published release
- **THEN** GitHub refuses

#### Scenario: Notes edited after publication
- **WHEN** the maintainer edits the title and notes of a published release
- **THEN** the change is saved and the tag and files stay as they were

### Requirement: Artifact attestations
Before a release is published, the three downloads and `SHA256SUMS` SHALL receive a GitHub artifact attestation made by the release workflow, naming the repository, the workflow and the built commit. A file SHALL carry an attestation only if it is exactly the file that the workflow built. The attestations SHALL NOT replace the code signing of the operating systems: Windows and macOS still warn on the first launch of the programs.

#### Scenario: Verifying a download
- **WHEN** a player runs `gh attestation verify truckpaint-v0.2.0-macos-arm64.zip -R acesyde/truckpaint` on the downloaded file
- **THEN** the check succeeds and names the Release workflow and the built commit

#### Scenario: A changed file
- **WHEN** the same command is run on a file that was changed after the build
- **THEN** the check fails

### Requirement: Players can verify a download
The Installation page of the documentation, in the four languages of the app, SHALL explain how to check a download with its checksum and with its attestation, say that a published release never changes, and say that the attestation shows where the file was built and does not remove the warnings of Windows and macOS.

#### Scenario: Installation page
- **WHEN** a player reads the Installation page
- **THEN** it gives the `gh attestation verify` command and says what the check proves and does not prove

### Requirement: A failed release is run again before it is published
Running a release again for the same tag, before it is published, SHALL complete it: the existing draft is reused, files already attached are replaced, and the release is published (when `publish` is ticked). A tag that is already published SHALL be refused as an existing tag, so it never creates a second release, and files SHALL never be added to a release that someone published by hand.

#### Scenario: Run again after a failure
- **WHEN** the macOS build failed, then the workflow is run again for the same tag and succeeds
- **THEN** one release exists, with the three downloads and the checksum file, and it is published

#### Scenario: Run again for a published release
- **WHEN** a workflow is run for a tag whose release is already published
- **THEN** it fails at once because the tag exists, and the published release is unchanged

#### Scenario: Release published by hand beforehand
- **WHEN** a release for the tag was published in the GitHub interface before the workflow ran
- **THEN** the workflow refuses the tag and attaches nothing to that release
