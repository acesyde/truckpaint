## Context

See proposal.md for why. What exists today:

- `tp-scs::Model::parse` reads `.pmg` pieces (material, UV streams, triangles). The position stream sits at piece offset 64 (12 bytes per vertex, the first of the vertex stride) and is already counted in the stride, but not read.
- `tp-pack::extract::geometry::painted` selects the painted pieces of one look and variant and returns `Mesh { uv, triangles, … }`; `geometry::svg` draws the template from those same UVs, so a mesh built from `painted` matches the template by construction.
- `tp-vehicles` reads and validates `.tpv` manifests; a project embeds its templates and keeps the package id, so the installed package can be found from a project.
- `tp-app::surface_thumbnails::SurfaceThumbnails` renders a surface's artwork (without template) in a background thread, detects staleness by `Arc` pointer comparison, and waits for the pointer to be released before rendering. Its size is fixed by `SIDE = 128`.
- Custom vehicles are recognised by their id prefix (`tp_pack::custom::is_custom`).
- The sample templates are labelled rectangular panels (`<rect>` + label), the same zones `tests/common/blue_wave.rs` already parses.

## Goals / Non-Goals

**Goals:**
- No new GPU pipeline and no new dependency: the view is an `egui::Mesh` drawn by the existing painter.
- The canvas never waits on the Side View (rendering stays off the UI thread).
- The stored mesh format can take accessory pieces later without a format change (`pieces[].texture`).

**Non-Goals:**
- Mesh simplification (decimation). The painted surfaces of one cabin are small enough to draw as is; revisit only if a measurement says otherwise.
- Reading normals, locators or any `.pmg` stream other than positions.

## Decisions

**1. Preview mesh as JSON in the package, one file per main texture.**
`preview/<texture id>.json`, named from the main texture's `preview_mesh` field. JSON keeps the package rule "only data: JSON and images", is hand-writable for the samples and readable in a diff; the ZIP compresses it well. Positions are rounded to 1 mm and UVs keep the 32-bit values (`f32` round trip) to stay on the template lines.
- *Alternative: a binary buffer.* Smaller and faster to load, but a new kind of file in a package, not hand-writable, and the size difference is small once zipped. Rejected for V1.
- *Alternative: one file per vehicle.* Would force a choice of cabin for the whole vehicle; per main texture follows the paint job's structure.

**2. One cabin (one configuration) per main texture.**
A main texture's template superposes every cabin of its layout; in 3D those cabins would overlap. The extractor keeps the pieces of the first game id's cabin only (see the ets2-game-extraction delta). A truck's mesh is that cabin alone: the other accessories painted on the main texture sit at locators the mesh does not know, so they would pile up at the origin. A trailer's mesh is its first painted definition for the same reason (for several official trailers this is a small part, which is why the follow-up on accessories is high priority). The cabins of one layout share their UVs, so the artwork lands the same way; only the shape differs slightly (e.g. 8x4). A cabin picker is left for later.

**3. Orthographic projection drawn with `egui::Mesh`, painter's algorithm.**
For each direction, vertices are projected on a fixed plane (Left: `(z, y)` seen from −x, Right: `(−z, y)` from +x, Front: `(−x, y)` from −z, Back: `(x, y)` from +z; seen from the front, the vehicle's right side is on the left of the drawing; Top: `(z, x)` seen from +y, so the front (−z) is on the left and the vehicle's right side (+x) at the top of the drawing, with depth −y); triangles are sorted back to front by their centroid's depth, once per mesh and direction (cached, not per frame). Shade = `0.55 + 0.45 · |n·d|` from the triangle's geometric normal, multiplied into the vertex color. No back-face culling: the winding of SCS meshes is not guaranteed consistent, and sorting already hides far faces.
- *Alternative: wgpu paint callback with a depth buffer.* Exact occlusion, orbit possible later, but a pipeline, shaders and GPU-only tests for a fixed-view guide. Kept as the upgrade path if sorting artifacts prove annoying.

**4. Texture from a second `SurfaceThumbnails` at a larger size.**
`SurfaceThumbnails` gains a side parameter (`named(name, side)`; the existing default stays at 128). The Side View owns one instance at 1024 px, updated with `update_only(&[drawn surface])`. It reuses the background render, the staleness check and the "wait for pointer release" rule, which is exactly the spec's live-update behavior. 1024 px is enough for a panel at most a few hundred points high; an 8192 × 1024 trailer texture keeps its shape (1024 × 128).

**5. Panel placement: a strip under the canvas, inside the canvas area column.**
Wide strips suit side views (a truck or trailer is much longer than tall). It does not compete with the inspector's width, and the canvas keeps its resize rule since it only loses height. Default height 220 pt, clamped 140–60 % of the column. It is drawn as an `egui::TopBottomPanel` inside the central area, before the canvas.

**6. Which texture is drawn.**
The Side View tracks "last main texture opened per vehicle" in UI state (not saved in the project). When an accessory texture is active, it draws that vehicle's last main texture, or its first one.

**7. Coordinate convention.** Positions are stored as read from the game (x right, y up, z towards the back, meters). Checked on an installed game (task 1.2): the position stream is at piece offset 64 and the cabins of Scania R 2016 and MAN TGX 2020 measure 2.5 m wide, 2.2 to 2.8 m tall and 2.3 to 2.6 m long, y up (roof highest). The front is towards −z: the windshield frame's top lies at larger z than its bottom. No conversion is needed. The sign of x (right) follows the exporter's convention and is not independently verified, since the cabs are symmetric; the manual check (task 8.2) is to look at an asymmetric logo.

**8. Sample meshes: written as JSON sources, checked against the templates.**
Each sample main texture gets a hand-written `preview/<id>.json` next to its SVG (boxes, two triangles per face, UVs at the corners of a labelled `<rect>`). The existing "committed package matches its sources" test covers packing; a new test parses the template rects (shared with `blue_wave.rs`'s zone parser) and checks every face's UVs lie inside one labelled rect. The sample versions keep their numbers: the app is not released, so the committed packages are rebuilt in place.

## Risks / Trade-offs

- [Painter's algorithm artifacts where triangles interpenetrate (mirror arms, grilles)] → main textures are mostly large body panels; a wgpu depth pipeline (decision 3) is the fallback.
- [Sample trailer's Base only covers the chassis and front wall; Blue Wave's artwork is on the curtain accessory] → the documentation media show the Side View with the truck's Standard cab; the trailer view stays poor until accessories are drawn, which is exactly the follow-up the proposal flags.
- [Many official trailers have a blank 64 × 64 main texture] → they show the "base color only" message; same follow-up.
- [JSON size for detailed cabins] → measure on two or three real cabins during task 2; if a package grows by more than a few MB zipped, switch to a quantized or binary form before release (the app is not released, so no compatibility cost).
- [Position offset 64 is inferred from the stride table, not yet checked on a real file] → task 1.2 checks it on an installed cabin (bounding box of a few meters, y up).

## Migration Plan

None for players: the app is not released. Official packages generated before this change show the "generate again" message; regenerating installs a new patch version (vehicle-generation delta). Sample packages are rebuilt in place.
