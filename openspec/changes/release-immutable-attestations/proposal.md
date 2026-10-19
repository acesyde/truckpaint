## Why

Releases are started today by pushing a tag from a terminal, with a signed annotated tag. That is easy to get wrong (a first try failed on the tag message), and a mistake is permanent once releases are immutable.

We want releases that nobody can change after they are published, and downloads whose origin a player can check. GitHub's immutable releases lock a published release's tag and files, but they only work if every file is attached while the release is still a draft. The workflow already does that; what must change is how a release is started, and what is added around it.

## What Changes

- A release is started from the Actions tab, with a button, instead of by pushing a tag. There are two workflows, **Release** and **Pre-release**, each with two choices: the `tag` (for example `v0.2.0`) and `publish` (ticked by default).
- **Release** only runs from `main`, for a tag without a suffix. **Pre-release** runs from any branch, for a tag with a suffix. Choosing the workflow chooses the kind, so a tag and a checkbox can never disagree.
- Wrong requests stop in seconds, before anything is built: a tag that is not a version, a suffix in a release or none in a pre-release, a release started from another branch than `main`, a tag that already exists.
- Unticking `publish` builds everything and leaves the release as a draft, so the maintainer can look at it and publish it by hand. This is the safe way to rehearse, because a published release is permanent.
- The build starts from the head of the chosen branch at the moment of the click. The draft is created for that exact commit, and GitHub creates the tag when the release is published.
- Pushing a tag no longer starts anything. The tag is a result, not a trigger.
- Releases are immutable: once published, their tag and files never change, and a release is never given files after it is published. The maintainer turns the setting on in the repository settings. A published release and its tag name can never be reused, so a mistake is fixed with the next version.
- The three downloads and `SHA256SUMS` get a GitHub **artifact attestation** before publishing. A player can check a file with `gh attestation verify <file> -R acesyde/truckpaint`. This proves which workflow and commit built the file; it is not code signing, so Windows SmartScreen and macOS Gatekeeper still warn on first launch.
- Title and notes stay editable after publishing (immutability locks only the tag and the files).
- `CONTRIBUTING.md` and the Installation page (four languages) describe the new way to release and how to check a download. A short rehearsal plan is added: a pre-release with a new version first, then the real release.

**BREAKING** (for maintainers): `git tag` and `git push` of a tag no longer publish a release; the release workflows are run from GitHub.

## Capabilities

### New Capabilities

None.

### Modified Capabilities
- `release`: the release starts from a manual workflow with a tag and a publish choice instead of a pushed tag; the main-only and any-branch rules and the failed-release rule follow the new start; releases are immutable; downloads carry attestations; a draft can be kept for review; players can verify a download; the contributor guide describes the new way.

## Impact

- Changed: `.github/workflows/release.yml`, `prerelease.yml` and `release-build.yml` (manual start, checks before the build, draft for the built commit, attestation step, optional publication, permissions `id-token` and `attestations` for the release job only); `scripts/release-test.sh` with a new guard script for the start rules.
- The `app-version` capability is unchanged: the app's version still comes from the tag, now typed in the form.
- Docs: `CONTRIBUTING.md` (Releasing), the Installation page in English, French, Spanish and German.
- Settings outside the code, done by the maintainer: turn on immutable releases for the repository.
- The existing `v0.1.0-beta.1` pre-release is not touched.
- No change to the app, project files, vehicle packages or the mod export, and none to `ci.yml`.
