## 1. Dependencies

- [ ] 1.1 Add `ureq` (rustls) and `sha2` to the workspace; check license (`deny`/audit config if the repo has one) and that CI builds on all targets
- [ ] 1.2 Note the build-time and binary-size difference in the PR

## 2. Catalog types (tp-vehicles)

- [ ] 2.1 `Catalog`, `Entry` with serde, unknown fields ignored, `schema` check with a typed error
- [ ] 2.2 `newest_compatible` for an id given the app version; ordering by semver
- [ ] 2.3 Unit tests: parsing, unknown schema, `min_app` filtering, a vehicle with no compatible version

## 3. `tpv index` (tp-pack)

- [ ] 3.1 `tpv index <folder> --base-url <url> [-o file]`: validate like `check`, fill entries, sort, write
- [ ] 3.2 `--check`: differing file, invalid package, duplicate `(id, version)`, missing `license`, changed SHA-256 of an existing version, removed entry
- [ ] 3.3 Tests: deterministic output, each failure above, the sample packages from `examples/vehicles/`
- [ ] 3.4 Update the `tpv` usage text and the package-format page for authors

## 4. Catalog client (tp-app)

- [ ] 4.1 Address resolution (env, preference, built-in) and HTTPS / `file://` validation
- [ ] 4.2 Fetch with timeout and size cap on a worker thread; result delivered to the UI without blocking
- [ ] 4.3 Cache write (temp file + rename) and read; offline state with the cache date
- [ ] 4.4 Download, size and SHA-256 check, then `install_bytes`; cancel between packages
- [ ] 4.5 Tests with a local `file://` catalog: success, checksum mismatch, bad catalog keeps cache, no cache offline, cancel

## 5. Marketplace tab (tp-app, tp-i18n)

- [ ] 5.1 Third tab in the Vehicle Library: list, search, game and kind filters with counts, status, detail pane
- [ ] 5.2 Install / Update with several selected, progress, Cancel, confirmation, error messages
- [ ] 5.3 Preferences: Catalog address field, the environment note, error on a refused address
- [ ] 5.4 The legal notice in the tab and the entry `notice` in the detail pane
- [ ] 5.5 Strings in en, fr, es, de (same ids; check German widths)
- [ ] 5.6 Headless UI tests (`egui_kittest`): listing, filter, install state, offline, update

## 6. Catalog repository (acesyde/truckpaint-marketplace)

- [ ] 6.1 `validate` workflow: build `tpv` from a pinned app tag, run `tpv index --check` on pull requests
- [ ] 6.2 `release` workflow: create the release with the `.tpv` assets, regenerate and publish `catalog.json` (Pages)
- [ ] 6.3 `CONTRIBUTING.md` (how to submit, immutable versions, license required) and the sample packages as first entries
- [ ] 6.4 README legal notice and contribution rule on game-derived templates; confirm SCS's modding policy allows the sharing before the first release

## 7. Documentation

- [ ] 7.1 Player page: installing from the Marketplace, offline, updating (English first, then fr/es/de)
- [ ] 7.2 Author page: publishing a package, versions never change
- [ ] 7.3 `mise run doc:media` for the new screen; `docs-drift-checker` before archiving

## 8. Finish

- [ ] 8.1 `mise run ci` passes
- [ ] 8.2 Update `first-run` and `template-update-impact` notes that depend on the catalog
