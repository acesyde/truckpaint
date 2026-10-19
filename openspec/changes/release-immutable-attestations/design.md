## Context

See proposal.md for why, and the `release` spec for the behavior. Today, on `main`:

- `release.yml` and `prerelease.yml` start on a pushed tag (`push: tags`), validate it, check that a release commit is an ancestor of `main`, and call `release-build.yml`. That reusable workflow builds the three downloads, then a `publish` job creates a draft (`gh release create --draft --verify-tag`, or reuses an existing release), uploads the files with `--clobber`, and publishes with `gh release edit --draft=false`.
- `scripts/release-tag.sh` validates a tag and says whether it is a pre-release; `scripts/release-notes-start.sh` finds the last stable tag; `scripts/release-test.sh` tests both.
- The first run worked end to end (`v0.1.0-beta.1`). The draft-first order is already what immutable releases need. What does not fit is the tag that must exist before the run (`--verify-tag`), the reuse of a release made by hand, and the "ancestor of `main`" check that exists only because a tag has no branch.
- Facts checked while exploring: `gh release view`, `upload` and `edit` find a draft by its pending tag name, so they work before the tag exists; `gh release create --target` takes a branch or a full commit SHA; `gh release edit` can change the target of a draft; `actions/attest` takes `subject-path` (globs) or `subject-checksums`, and needs `id-token: write`, `attestations: write` and `contents: read`; the repository is public, so attestations are free; an immutable release locks its tag and files, not its title, notes, pre-release mark or Latest mark, and its tag name can never be reused.

## Goals / Non-Goals

**Goals:**
- A release is one click (or one `gh workflow run`) with two choices, and a wrong request stops before any build.
- Nothing is public until the three downloads are built, attached and attested.
- Every file of a release has a verifiable origin, and is locked once published.

**Non-Goals:**
- Code signing or notarization of the programs (the warnings of Windows and macOS stay).
- Changing the build, the packaging or the platforms.
- Turning on immutable releases from the workflows: it is a repository setting, set by the maintainer.
- Cleaning up old pre-releases or the existing `v0.1.0-beta.1`.

## Decisions

### Two manual workflows, one reusable workflow
`release.yml` ("Release") and `prerelease.yml` ("Pre-release") both start with `workflow_dispatch` and the inputs `tag` (string, required) and `publish` (boolean, default `true`). The `push: tags` triggers are removed, so a tag typed by hand starts nothing and a tag created by GitHub at publication cannot start a second run. `release-build.yml` stays the shared builder and publisher, with a new `publish` input.

Choosing the workflow chooses the kind (release or pre-release), so there is no checkbox that could disagree with the tag. Alternatives:
- A release form on GitHub starting the build when it is published: rejected, it publishes before the files exist, which immutable releases forbid.
- Keeping the tag push: rejected by the maintainer, a signed annotated tag is easy to get wrong and the tag is irreversible once published.

### The rules live in one script that is tested
A new `scripts/release-guard.sh KIND TAG REF_TYPE REF_NAME` holds the rules that need no network: the tag is valid (it calls `release-tag.sh`), a release tag has no suffix and a pre-release tag has one, and a release runs from the branch `main` (not another branch, not a tag). It prints the version and the pre-release mark, like `release-tag.sh`, and fails with a message that names the other workflow when the kind is wrong. `release-test.sh` gets cases for every combination, as it has for the tag script.

Each trigger workflow's `check` job runs the script, then checks that the tag does not exist with `git ls-remote --exit-code --tags origin "refs/tags/$TAG"`, which needs the network and so stays in the workflow. The `check` job needs no build and no write permission, so a refused request costs seconds.

`release.yml` does not need the "ancestor of `main`" check any more: the run itself is on `main`.

### The built commit is pinned
A manual run is on the head of the chosen branch when it starts. The workflows read `github.sha` once and every job checks out `ref: ${{ github.sha }}` (the three package jobs and the publish job), so a branch that moves during the build cannot mix two commits. The draft's target is the same SHA, so GitHub creates the tag on exactly the built commit when the release is published.

### Draft, reuse, attest, publish
In the `publish` job, after downloading the artifacts and writing `SHA256SUMS`:

1. **Draft.** If a release with this tag exists, it can only be a draft (a published one means the tag exists, so the `check` job refused the request): it is reused and its target is set to the built SHA with `gh release edit --target`. Otherwise `gh release create --draft --target "$GITHUB_SHA" --title "$TAG" --generate-notes [--notes-start-tag START] [--prerelease | --latest]`, without `--verify-tag` because the tag does not exist yet.
2. **Upload** with `--clobber`, as today (a draft accepts replacements).
3. **Attest** the three archives and `SHA256SUMS` with `actions/attest`, pinned by commit SHA (v4.2.2, older than two weeks), `subject-path: dist/*`. It runs on the exact files that were downloaded and uploaded, after the upload, so a failed upload leaves no attestation for a release that never got its files.
4. **Publish** only when `publish` is true: `gh release edit --draft=false` with `--latest=true` for a release and `--prerelease=true --latest=false` for a pre-release. GitHub creates the tag at the draft's target at this moment, and the release becomes immutable (if the setting is on).

When `publish` is false the job stops after the attestation and writes a summary that says the draft is ready and how to publish it, by hand or by running the workflow again with `publish` ticked.

Alternatives: the `softprops/action-gh-release` action for steps 1, 2 and 4 (rejected: `gh` is already installed, first party and tested by the first run, and the action would only add a dependency with write access); `subject-checksums` to attest the archives listed in `SHA256SUMS` (rejected: it would leave `SHA256SUMS` itself unattested).

### Permissions
The reusable workflow keeps `contents: read` for the three package jobs. Only the `publish` job declares `contents: write`, `id-token: write` and `attestations: write`. A reusable workflow can never get more than its caller gives, so the `release` and `prerelease` jobs of the two trigger workflows, which call it, grant those three; their `check` jobs keep `contents: read`.

### Concurrency
The group is per tag, `release-${{ inputs.tag }}`, in the two trigger workflows only (a group set in both the caller and the called workflow makes the called one cancel itself), without cancelling a running one.

### Nothing else changes
`ci.yml`, the packaging task, the version through `TRUCKPAINT_VERSION` and the `app-version` spec stay as they are. The Purpose of the `release` spec still says a pushed tag starts a release, and deltas cannot change a Purpose, so the apply step edits it directly in `openspec/specs/release/spec.md`.

## Risks / Trade-offs

- [A published release is permanent, and so is its tag name] → `publish` unticked gives a draft to review first, the checks refuse bad requests before any build, and the first real release is rehearsed with a pre-release of a new version.
- [The attestation of a reusable workflow may name `release-build.yml` rather than the trigger workflow] → check the certificate details on the first rehearsal and write the right `gh attestation verify` options (for example `--signer-workflow`) into the Installation page and `CONTRIBUTING.md`.
- [A draft published by hand may lose its Latest or pre-release mark] → set the marks when the draft is created, and check on the rehearsal that the mark survives; if it does not, the guide tells the maintainer to tick it when publishing.
- [`gh release edit --target` on a draft, and a tag created at publication on a commit that is on no branch, are untested here] → both are covered by the rehearsal with `publish` unticked, then ticked.
- [The Run workflow button needs the workflow on `main`, and a feature branch created earlier does not contain the files] → written in `CONTRIBUTING.md`; the run fails at once with GitHub's own error.
- [Immutability is a setting outside the code and can be forgotten] → the rehearsal ends by trying to change a file of the published pre-release, which must be refused.
- [Windows and macOS still warn on first launch] → unchanged, documented on the Installation page next to the new verification.

## Migration Plan

1. Land the workflow, script, docs and spec changes (nothing is published by a merge).
2. The maintainer turns on immutable releases in the repository settings.
3. Rehearse: run Pre-release on a feature branch with `publish` unticked; look at the draft; run it again with `publish` ticked for `v0.1.0-beta.2`; run `gh attestation verify` on a download; try to replace a file of that release and see GitHub refuse; run Release from a feature branch and see it refuse before building; run Release on `main` with the tag of an existing release and see it refuse.
4. Release `v0.1.0` from `main`, first with `publish` unticked, then publish.

Rollback is restoring the previous workflows from Git; no release is changed. The existing `v0.1.0-beta.1` is not touched.

## Open Questions

- Whether the marks of a draft survive when it is published by hand (see Risks); it only changes one sentence of the guide.
