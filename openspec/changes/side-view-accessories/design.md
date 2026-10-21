## Context

See proposal.md for why. What exists after `livery-side-view`:

- `tp-scs::Model::parse` reads parts and pieces, and skips the locator table. `.pmg` 0x15 stores `locator_count` at header offset 20 and the locators' offset at 76 + 4 = 80; each locator is 44 bytes: name token (u64), position (3 × f32), scale (f32), rotation quaternion (4 × f32), hookup string offset (i32). Each part (24 bytes) carries `locator_count` and `first_locator` at offsets 16 and 20. These offsets were read by hand on the installed DAF XD (spike in the explore session) and gave sensible values; they are not yet in code or tests.
- `tp-pack::extract` already resolves, per main texture, the painted contributors (`parts.painted`): the cabin and every accessory definition painted with the main texture, plus the accessories with their own textures. It keeps `geometry::Mesh` per contributor (positions, UVs, triangles) and builds the preview from the cabin contributor only.
- `tp-vehicles::PreviewMesh { format, pieces }` with `PreviewPiece { texture, positions, uv, triangles }`; `Package::read_preview`.
- `tp-app::side_view` keeps one `SurfaceThumbnails::sized("side_view", 1024, …)` and calls `update_only(&[surface])` for the drawn surface; `Drawing` projects and sorts the triangles of one mesh per direction, cached.
- `tp-file::FileVehicle` holds the package id, version and game data of each project vehicle.

Facts read from the installed DAF XD (ETS2 1.61):

| Locator | Model | Position | Accessory model extent |
|---|---|---|---|
| `cab_door` (eu part) | cabin `cab_sl` | (1.075, 1.498, −2.429) | x −2.32 to 0.17 (both doors in one model) |
| `r_deflector` | cabin | (0, 3.018, −1.000) | |
| `f_bumper` | chassis `chs_4x2_sl` | (0, 0.517, −3.365) | z 0.07 to 1.01 |
| `f_fender_chs` | chassis | (0, 1.223, −1.770) | |
| `cabin` | chassis | (0, 1.383, −2.359) | |

Placed at their locators, the door spans exactly the cabin's width and the bumper sits just in front of the cabin, with the cabin model drawn as is. So the cabin model is already in the chassis's space; the chassis's `cabin` locator is a pivot (cab tilt), not an offset.

## Goals / Non-Goals

**Goals:**
- Placement done once, the same way for every slot, from data the game provides (locators, `defaults`, `suitable_for`), with no per-truck tables.
- A preview file whose size grows with the number of options, not with the number of combinations.
- The app redraws a configuration change at once, without a new background render (the textures do not change).

**Non-Goals:**
- Locator hookups, animations (doors opening), skeletons.
- Accessories mounted on other accessories (a light bar on a roof grill): only the cabin and the chassis are parents.

## Decisions

**1. Accessories stored in their own model space, placed by the app.**
Each option's pieces stay as read from its model; the preview file carries the cabin's locators and each chassis option's locators. The app places an option at every locator of the slot's name of its parent (cabin or the chosen chassis): `p' = position + rotate(q, scale · p)`.
- *Alternative: pieces already placed by the extractor.* Simpler to draw, but a chassis-mounted accessory's place depends on the chosen chassis (side skirts, fenders, bumpers move with the wheelbase), so the file would need one copy per chassis option. Rejected.

**2. Parent of a slot: the cabin first, then the chassis.**
The extractor looks for a locator named after the slot (the last word of the accessory unit name: `std_a.daf.xd.cab_door` → `cab_door`) in the cabin's visible parts for the EU variant, then in the chassis models. A slot found in neither is left out and reported. For a trailer, the chassis is the root (no parent) and every other slot, the body included, hangs on it. Trailer bodies are expected to use a `body` locator on the chassis; the first task checks it on `scs.box` and the design is updated if the game uses something else.

**3. The chassis is a slot without None.**
A truck's chassis options are the chassis definitions suitable for the cabin; a trailer's are its chassis definitions. The default is the cabin's (or the trailer's) default chassis. Making it a slot reuses the list, the saved choice and the placement of chassis-mounted slots, instead of a separate chassis picker.

**4. Options merged by painted shape.**
Two options of a slot merge when their painted pieces (positions, UVs, triangles, texture) are identical after millimetre rounding. The six DAF XD doors (interior colours) become one; three roof deflectors stay three.

**5. Defaults from the game's `defaults[]`.**
Starting from the cabin definition (a trailer's root definition), follow `defaults[]` recursively (the cabin lists the chassis, which lists its own defaults) and mark, per slot, the option whose definition is reached. When the reached definition is unpainted (a plastic fender), the slot has no default and starts at None.

**6. Names.**
Option names come from the game's English localization, like vehicle and cabin names today. Slot names come from a small Fluent table in the app (`side-view-slot-cab_door = Door`, …) for the common slots; an unknown slot shows its game name. The table is a convenience, not a contract: adding a slot needs no package change.

**7. Several textures in the view.**
The Side View asks `SurfaceThumbnails::update_only` for every surface its configuration draws (the main texture and the accessory textures of the chosen options that are in the project). It already renders off the UI thread and only the stale surfaces; the drawing waits for each texture and draws a missing one in the neutral color meanwhile. The 1024 px side stays; a truck with a cabin and four accessory textures means five textures of at most 4 MB each in GPU memory.

**8. Drawing cache per configuration.**
`Drawing` becomes the projection of all placed pieces of the current configuration and direction, each triangle tagged with its texture; it is rebuilt when the configuration, the cabin or the direction changes (not per frame). An `egui::Mesh` holds one texture, so the drawing emits one mesh per texture after the global back-to-front sort; consecutive triangles of the same texture are batched, which keeps the depth order right at the cost of more meshes where textures interleave.

**9. Project file.**
`FileVehicle` gains an optional `side_view: { slot id: option id or null }`, written only when a choice differs from the default. Reading tolerates unknown slots and options (they fall back to the default, as the spec says). The choice lives on the project's vehicle in `tp-core` as plain data, outside the undo history; changing it marks the document dirty through the same path as other non-undoable project settings.

**10. Format stays 1.**
`locators` and `slots` are optional fields of format 1: the app is not released, so there is no reader to keep compatible with, and a mesh without slots still reads as before (the cabin alone).

## Risks / Trade-offs

- [Quaternion component order or handedness wrong] → most exterior locators have no rotation, so a mistake shows only on the few rotated ones (rear lights, exhausts); a unit test with a 90° yaw locator and a check on a rotated chassis accessory pin it.
- [Trailer body mounting differs from the truck pattern] → the first task inspects `scs.box`'s chassis locators before any trailer code; if bodies are not on a `body` locator, design decision 2 is updated with the user before going on.
- [Package size with every option of every slot] → measure on the DAF XD and `scs.box` (25 bodies); if a package grows beyond a few MB zipped, merge options more aggressively or drop the interior-only variants earlier.
- [Painter's algorithm across many separate models] → more interpenetrating pieces (fenders over wheels, deflectors over the roof) than with the cabin alone; acceptable for a guide, wgpu depth stays the fallback named in `livery-side-view`.
- [Draw cost: several textures and up to a few hundred thousand triangles] → the projection and sort are cached per configuration and direction; measure on a well-equipped truck and note it in the PR.

## Migration Plan

None for players (the app is not released). Packages without slots keep drawing the cabin alone with the "generate again" hint in the Accessories list; generating again installs a new patch version (vehicle-generation delta). Sample packages are rebuilt in place.
