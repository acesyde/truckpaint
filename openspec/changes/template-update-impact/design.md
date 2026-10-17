## Context

See proposal.md for why. What exists today (checked in the code):

- `Manifest` (`crates/tp-vehicles/src/lib.rs`) has optional `description`, `homepage` and no `deny_unknown_fields`, so a new optional `changelog` is ignored by older apps. `Texture.layout_version` is per part.
- `vehicle_project::plan()` / `apply_update()` (`crates/tp-app/src/vehicle_project.rs`) already compute `Replaced { layout_changed, resized }` / `Removed`, flag `TemplateStatus::LayoutChanged` and clear it with `dismiss_layout_change` (Mark as Checked). They know nothing about objects.
- A project records, per surface, `SurfaceTemplate { package_id, texture_id, layout_version, size via surface, status }` and per vehicle `ProjectVehicle { package_id, version }`. That is enough to compare with the newest manifest without the old one.
- The `.truckpaint` file is a ZIP with `project.ron` (`FileProject`, format 1, `FORMAT_VERSION = 1`, `crates/tp-file/src/lib.rs`) plus asset entries. `FileProject`/`FileTemplate` have no `deny_unknown_fields`; new optional fields were already added to format 1 this way (`game_ids`, `main_index`). `tp_file::read` loads every asset; there is no light reader.
- `prefs.recent` holds `RecentProject { name, path, last_opened }`, capped by `MAX_RECENT = 12`, with `recent_exists(path)`.
- The Vehicle Library dialog is in `crates/tp-app/src/ui/vehicle_dialogs.rs` (not `library_dialog.rs`, which is the brand library import). Update Template's dialog is in the same file (`update_template_modal`). The Layers rows are in `crates/tp-app/src/ui/workspace/panels/layers.rs`.

## Goals / Non-Goals

**Goals:** answer "how much work is this update" from data already recorded; keep the project format at 1 and compatible both ways.

**Non-Goals:** knowing which areas moved; an index of all projects; any network. No project file is written unless opened.

## Decisions

### D1. Object flags: stored ids on the template, validity derived from its status
`FileTemplate` gets `flagged_objects: Vec<u64>` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]`); `SurfaceTemplate` gets `flagged: BTreeSet<ObjectId>`. An object is shown as To reposition iff its texture is `LayoutChanged` and its id is in `flagged`. Mark as Checked clears the status, and the same undo step clears the set. Moving or transforming an object removes its id in the same undo step; deleting prunes it.
- Format stays 1: older TruckPaint ignores the field (the texture stays To check); newer reading older files gets an empty set. The one loss is an older app re-saving a project, which drops the list: acceptable, the flag is a hint.
- Reading ignores ids absent from the surface and sets on non-flagged textures, so a stale file can't show phantom marks.
- Alternatives: derive only from "texture To check" (flags every object forever, can't clear one by moving it, new objects flagged); a field on each `FileObject` (touches every object type and the frozen object shape). Rejected.
- Texture-status's "Project files SHALL be unchanged" sentence is amended accordingly (delta).

### D2. What the count and the flag cover
All non-group objects on the texture, recursively, hidden ones too; a symbol instance counts once. The flagged set holds ids of those objects; a group shows the mark when a descendant is flagged, and moving a group clears all its descendants. One function `objects_on(surface)` backs the Update Template count, the project lines and the flagging, so the counts agree (tested).

### D3. Moving clears, look does not
"Moved or transformed" = a committed change to a frame's center, size or rotation (path node edits count). Look edits, text content and renames keep the flag. The hook is in the single place committed transforms go through, not in each tool.

### D4. Reading projects without opening them
Add `tp_file::summary(path) -> Summary` that opens the ZIP, parses only `project.ron` (migrating if needed) and returns the vehicles (`package_id`, `name`, `version`) and, per surface with a template: texture id, name, template layout version, size, and the count from D2. It does not call `current::into_project` and loads no asset bytes. It runs on a worker thread when the library opens, one file after the other, results cached by (path, modified time) in the app and the dialog shows lines as they arrive. The open project is summarized from memory (`Project`) with the same shared counting code, and its path is skipped in the recent list.

### D5. Pure manifest diff in `tp-vehicles`
`diff(old: &Manifest, new: &Manifest) -> ManifestDiff { layout_raised, resized, added, removed }` over parts by texture id; no UI, unit-tested. The per-project lines use a second pure function, `impact(summary, new_manifest)`, so neither needs the old manifest. "Textures no longer in this version" come from the project's own surfaces (the proposal's third open question: yes, enough).

### D6. Changelog
`Manifest.changelog: Option<String>`, plain text, at most 1000 characters, validated in `tp-vehicles` (shared by app install, `tpv pack` and `tpv check`) with a new `PackageError::BadChangelog`. Shown as the newest version's changelog only; Update Template shows the target version's. The summary is a diff of newest vs. the previous installed version; the per-project lines compare with what the project records, so a project several versions behind still gets correct counts. Custom vehicles: `tp-pack::custom` writes the "What changed" text; the field is empty by default and is not copied from the previous version.

### D7. Library actions
Update reuses the existing "Update Template…" path (closes the library, opens the project's dialog). Later closes. Other projects are never touched. The existing Update status (open project only) is unchanged.

## Risks / Trade-offs

- [Counts overstate the work, since every object counts] → worded "objects to reposition" and documented; no overlap guessing.
- [Large project files slow the list] → ZIP document only, off the UI thread, cached by mtime.
- [Older app re-saves and drops flags] → texture stays To check; documented in the file-format notes.
- [Recent list only] → stated under the list.
- [Move hook missed by some tool, flag stays] → one commit point plus a test per transform route (move, resize, rotate, nudge, align, node edit); Mark as Checked always clears.

## Migration Plan

No migration: format 1, optional fields. Rollback = ignore the fields.

## Open Questions

- `website/.../vehicles/package-format.mdx` has no fr/es/de translation (AGENTS.md says to ask before translating it). The tasks add `changelog` to the English page only; translating it is left to the user's decision.
