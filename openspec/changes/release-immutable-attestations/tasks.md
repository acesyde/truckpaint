## 1. Start rules

- [x] 1.1 Write `scripts/release-guard.sh KIND TAG REF_TYPE REF_NAME` (valid tag through `release-tag.sh`, no suffix for `release`, a suffix for `prerelease`, `release` only on the branch `main`; prints `version=` and `prerelease=`, also to `$GITHUB_OUTPUT`; fails with a message naming the other workflow) and verify it by running it for a release and a pre-release
- [x] 1.2 Add cases to `scripts/release-test.sh` for every combination (release on `main`, on another branch and on a tag ref; pre-release on `main` and on a feature branch; wrong suffix both ways; invalid tags such as `v0.2` and `v1.0.0-`) and verify `mise run release:test` passes
- [x] 1.3 Expose the script as a `release:guard` task in `mise.toml` like `release:tag`, and verify `mise run release:guard -- release v0.2.0 branch main` prints the version

## 2. Workflows

- [x] 2.1 Change `.github/workflows/release.yml` and `prerelease.yml` to start with `workflow_dispatch` (inputs `tag` and `publish`, default `true`), remove `push: tags`, run the guard and a `git ls-remote --exit-code --tags origin "refs/tags/$TAG"` check in a `check` job (with `contents: read`), and call `release-build.yml` with the version, the pre-release mark and `publish`; the calling job grants `contents: write`, `id-token: write` and `attestations: write`; group concurrency per tag (`release-${{ inputs.tag }}`); verify with `mise run lint:workflows`
- [x] 2.2 Update `.github/workflows/release-build.yml`: new `publish` input; every job checks out `ref: ${{ github.sha }}`; the publish job declares its own permissions; create the draft with `--target "$GITHUB_SHA"` and no `--verify-tag`, or reuse an existing draft and set its target with `gh release edit --target`; set the pre-release or Latest mark when the draft is created; publish only when `publish` is true; write a summary that says how to publish a draft; verify with `mise run lint:workflows`
- [x] 2.3 Add the attestation step to the publish job after the upload: `actions/attest` pinned by the commit SHA of v4.2.2 (resolved with `gh api`, checked against the tag), `subject-path: dist/*`; verify with `mise run lint:workflows` and by reading the workflow that the three archives and `SHA256SUMS` are in `dist/` at that step
- [x] 2.4 Check that no `push: tags` trigger is left in any workflow and that `ci.yml` and `docs.yml` are unchanged, and verify with `git diff --stat .github/workflows`

## 3. Documentation

- [x] 3.1 Rewrite the Releasing section of `CONTRIBUTING.md` for the new way (Release and Pre-release from the Actions tab or `gh workflow run`, the `tag` and `publish` choices, Release only from `main`, the button needs the workflow on `main` and the branch must contain the workflow files, the version comes from the tag, immutable releases and the never-reused tag name, how to check a download with `gh attestation verify`, the rehearsal with a pre-release of a new version and `publish` unticked, and how to turn on immutable releases in the repository settings); verify by reading it against the `release` spec
- [x] 3.2 With the docs-writer agent, update the English Installation page (check a download with its attestation and checksum, a published release never changes, the attestation shows where a file was built and does not remove the Windows and macOS warnings); with docs-translator, the French, Spanish and German pages with matching ids and anchors; verify `mise run doc:build` passes
- [x] 3.3 Delegate the docs-drift-checker agent and list any other stale page in the apply summary

## 4. Spec and checks

- [x] 4.1 Edit the Purpose of `openspec/specs/release/spec.md` directly so it no longer says a pushed tag starts a release (deltas cannot change a Purpose), and verify `openspec validate release --type spec --strict` passes
- [x] 4.2 Run `mise run lint:workflows`, `mise run release:test` and `mise run package:test` and verify all three pass
- [x] 4.3 Delegate `mise run ci` to the rust-ui-tester agent and read its output for the exit code of each step, since nothing in Rust changes but the workflow tests must still be reported

## 5. Rehearsal and first release (by the maintainer)

- [x] 5.1 Turn on immutable releases in the repository settings, and verify the setting is on
- [x] 5.2 Run Pre-release from a feature branch (updated from `main` first) with the tag `v0.1.0-beta.2` and `publish` not ticked; verify a draft exists with the three archives and `SHA256SUMS`, that no tag exists and that nothing is public
- [x] 5.3 Run it again with `publish` ticked; verify the pre-release is published, not Latest, the tag points to the built commit, and `gh attestation verify <archive> -R acesyde/truckpaint` succeeds for the three archives and `SHA256SUMS`; note which workflow the attestation names and fix the options in the guide and the Installation page if needed
- [ ] 5.4 Try to replace a file of that published release and verify GitHub refuses; edit its notes and verify that works
- [ ] 5.5 Check the refusals: run Release from the feature branch (refused: not `main`), Release with `v0.1.0-beta.2` (refused: suffix, and tag exists), Pre-release with `v0.1.0` (refused: no suffix); verify each fails in seconds and creates no draft
- [ ] 5.6 Run Release on `main` with `v0.1.0` and `publish` not ticked, look at the draft, then publish it (by hand or by running it again with `publish` ticked); verify it is Latest, and note whether a draft published by hand keeps its marks
