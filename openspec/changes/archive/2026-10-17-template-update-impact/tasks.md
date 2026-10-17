## 1. Package changelog

- [x] 1.1 Add `Manifest.changelog: Option<String>` with the 1000-character limit and `PackageError::BadChangelog` in `tp-vehicles`, and the message in `vehicles.rs`; verify unit tests: absent, valid, 1001 characters refused, older reader unaffected (manifest without the field).
- [x] 1.2 Make `tp-pack` write the field unchanged and `tpv check` print it; verify a pack/check test with a changelog, one over the limit (non-zero exit, no file) and byte-identical repacking.
- [x] 1.3 Add the "What changed" field to New Version… (`ui/custom_vehicle.rs`, `tp-pack/src/custom.rs`) with a length counter; verify `crates/tp-app/tests/custom_vehicles.rs` shows the installed new version carrying the text, empty writes none, and 1001 characters disables Create.

## 2. Pure diff and counting

- [x] 2.1 Add `diff(old, new) -> ManifestDiff` (layout raised, resized, added, removed, parts by texture id) in `tp-vehicles`; verify unit tests for each kind, for main and accessory parts, and for identical manifests.
- [x] 2.2 Add `objects_on(surface)` (non-group objects, recursive, hidden counted, instance = 1) in `tp-core` and use it in `vehicle_project::plan()` so `TextureChange::Replaced` carries `objects: usize` and `UpdatePlan` carries `changelog`; verify unit tests with groups, hidden objects, instances.
- [x] 2.3 Add `impact(summary, manifest) -> ProjectImpact` (textures with a changed layout, objects, resized, no longer in this version, "Nothing to reposition") next to the vehicle code; verify unit tests including a recorded version that is no longer installed.

## 3. Project file

- [x] 3.1 Add `SurfaceTemplate.flagged` and `FileTemplate.flagged_objects` (optional, format 1) with load-time pruning of unknown ids and of non-flagged textures; verify `crates/tp-file/tests/format.rs`: round trip, file without the field opens not-migrated, unknown ids dropped, and an old-shape file still parses.
- [x] 3.2 Add `tp_file::summary(path)` (vehicles, per-surface template layout, size, object count; no assets loaded, file unchanged); verify a test that it matches `read()` for a fixture, leaves the file's bytes and mtime unchanged, and reports a damaged file as an error.

## 4. Flagging objects

- [x] 4.1 In `apply_update`, set `flagged` to the ids from `objects_on` for each texture that becomes or stays `LayoutChanged`, in the same undo step; verify unit tests: flagged ids equal the dialog count, undo removes them, a second update merges.
- [x] 4.2 Clear ids in the one commit point of committed transforms (move, resize, rotate, nudge, align, node edit, group move) and on delete, and clear all in `dismiss_layout_change`, each undoable; verify one test per route, and that fill/stroke/text/rename edits keep the flag.
- [x] 4.3 Mark flagged rows in the Layers list (`panels/layers.rs`, group row when a descendant is flagged, icon plus "To reposition" accessible name and tooltip); verify a headless egui_kittest test in `crates/tp-app/tests/panels.rs` for marked rows, group row, cleared after a move, and none for an object added after the update.

## 5. Vehicle Library and Update Template UI

- [x] 5.1 Show the changelog and per-texture "objects to reposition" in `update_template_modal` (`ui/vehicle_dialogs.rs`); verify a headless test in `crates/tp-app/tests/vehicles.rs` for the counts equal to the flagged objects and for no changelog section.
- [x] 5.2 Add the Update summary (changelog and detected changes, "No changes listed", hidden with one version) to the Vehicle Library detail pane; verify a headless test with sample truck 1.0.0 and 1.1.0.
- [x] 5.3 Load the known projects (recent list with existing files, open project from memory, skipping duplicates) on a worker thread through `tp_file::summary`, cached by path and mtime, and list the lines with "Only recent projects are listed"; verify headless tests: counts line, missing file skipped, up-to-date skipped, unreadable named, files unmodified, dialog opens without waiting.
- [x] 5.4 Add Later and Update to the summary (Update runs Update Template on the open project, disabled with a hint otherwise); verify headless tests for Later (nothing changes), Update with an open project and Update from the home screen.

## 6. Translations

- [x] 6.1 Add every new string (summary, change kinds, project lines with plural forms, "Only recent projects are listed", Later, Update, hint, "Can't be read", What changed and counter, changelog error, To reposition mark and tooltip, update dialog counts) to `crates/tp-i18n/locales/{en,fr,es,de}/*.ftl` with identical message ids; verify the locale parity test passes and German screens fit in the headless tests at 960 × 600.

## 7. Documentation

- [x] 7.1 Document the update summary and impact in `website/src/content/docs/en/vehicles/library.mdx` and "What changed" in `en/vehicles/custom-vehicles.mdx`, the update flow in the fleet pages and the mark in `en/workshop/layers.mdx`, using `<Ui id="…" />` and `<Shot>`; verify `mise run doc:build`.
- [x] 7.2 Document `changelog` in `en/vehicles/package-format.mdx` (English only, see design Open Questions); verify `mise run doc:build`.
- [x] 7.3 Translate the pages of 7.1 into fr, es and de (same files and `sidebar.order`); verify `mise run doc:build` and the docs-drift-checker agent reports nothing stale.
- [x] 7.4 Add the new states to `crates/tp-app/tests/doc_media.rs` and run `mise run doc:media vehicles`, `mise run doc:media fleet` and `mise run doc:media workshop`; verify new media under `website/public/media/<lang>/` and that the build finds every `<Shot>`.

## 8. Finish

- [x] 8.1 Run `mise run ci` and verify format, lint, tests and build pass; report GPU-only checks that could not run.
