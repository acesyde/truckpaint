## Why

A new version of a vehicle package usually follows a game update that redrew some textures. Today the player learns what changed only inside one project, in Update Template: which textures are replaced, which have a new layout, which are new or gone. The Vehicle Library says nothing beyond the version number, and the package can't say why it changed. A player with three fleets using that truck can't tell whether updating is a five-minute check or an afternoon of repositioning.

Once the marketplace (`marketplace`) offers updates, this question comes with every one of them. It can be answered now for local packages, with what packages and projects already record.

## What Changes

- **Packages can describe a version:** an optional `changelog` text in `vehicle.json` ("Door layout redrawn, islands moved. Low cabin added."). It doesn't exist today; older TruckPaint versions ignore it, as they ignore unknown fields. `tpv` packs it, and New Version… of a custom vehicle gets a "What changed" field.
- **The Vehicle Library shows what an update changes,** for a vehicle with several installed versions, between the newest and the one before it (or the one a project uses, see below):
  - the changelog, when the package has one;
  - what TruckPaint detects from the two manifests: textures whose layout changed (`layout_version` raised), textures resized, main textures (cabin layouts) and accessories added or removed.
- **…and which projects it touches:** for each known project using an older version of the vehicle, a line such as "Transports Ardent: 2 textures, 3 objects to reposition". It compares the version the project records with the newest one: the textures it paints whose layout changed, and the objects on them. "Objects to reposition" counts the objects on those textures: TruckPaint can't tell which areas of a template moved.
- **Known projects only:** the projects listed are those in the recent projects list (at most 12) whose file still exists, read without opening them. The library says so under the list ("Only recent projects are listed"). There is no index of every project on disk.
- **Later / Update:** Later closes the panel; the update button stays in the project's sidebar. Update runs Update Template on the open project when it uses the vehicle; the other projects listed are updated when the player opens them, through their existing update button. No project file is changed without being opened.
- **Update Template shows the same information:** the changelog and, for each texture whose layout changed, the number of objects to reposition.
- **After the update, nothing moves:** textures whose layout changed become To check (see `texture-status`), and the objects that were on them when the update was applied are flagged in the Workshop's Layers list, until they are moved or the texture is marked as checked.

Non-goals:
- downloading updates (that is `marketplace`), or updating projects that aren't open;
- telling which objects actually overlap the redrawn areas, or moving them;
- an index of projects outside the recent list.

Open questions (to settle in design):
- Is an object's flag kept in the project file, so it survives closing the project, or does it live only while the texture is To check? Proposed: derived from the texture, plus the ids of the objects present at update time, saved as an optional field (format 1).
- Does editing an object's look (not its position) clear its flag? Proposed: no, only moving it, transforming it, or Mark as Checked on the texture.
- When the older installed versions were removed, the library has only the projects' recorded layout versions to compare with: the per-project lines still work, but cabins and accessories removed since the project's version can't be named without the old manifest. Is listing "textures no longer in this version" from the project's own surfaces enough? Proposed: yes.

## Capabilities

### New Capabilities
- `template-update-impact`: the update summary in the Vehicle Library (changelog, detected changes), the projects it touches from the recent list and how they are counted, the Later / Update actions, and the flag on objects of a changed texture.

### Modified Capabilities
- `vehicle-packages`: the optional `changelog` field of the package format; the Vehicle Library dialog shows the update summary.
- `vehicle-projects`: Update Template shows the changelog and the objects to reposition, and records the flagged objects.
- `custom-vehicles`: New Version… has a "What changed" field written as the changelog.
- `vehicle-authoring`: `tpv` packs and checks the `changelog` field.
- `layers-panel`: flagged objects are marked in the list.
- `project-files`: the optional list of flagged objects (if the first open question is settled that way).

## Impact

- **Depends on** `texture-status` (To check) and so on `new-design` (Workshop, Vehicle Library layout).
- **tp-vehicles:** `Manifest.changelog: Option<String>`; a pure diff of two manifests (parts added, removed, resized, layout raised), unit-tested.
- **tp-pack:** packs the field; length limit in `tpv check`.
- **tp-file:** reading a project's vehicles, surface templates and object counts without loading its assets (the recent list's projects); the optional flagged-object ids.
- **tp-app:**
  - `vehicles.rs` / `library_dialog.rs`: the update summary and project impact;
  - `prefs.rs`: the recent list as the source of known projects;
  - `vehicle_project.rs`: `UpdatePlan` gains the changelog and object counts, `TextureChange::Replaced` its count, and the update records flagged objects;
  - `vehicle_dialogs.rs`, `custom_vehicle.rs`, the Layers list;
  - strings in en/fr/es/de.
- **Docs:** the package format page (`website/src/content/docs/en/vehicles/package-format.mdx`) documents `changelog`.
