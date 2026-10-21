## Why

A painter draws on flat textures whose pieces (UV islands) are cut, turned and scattered in a way only the vehicle knows. While designing, they cannot tell which piece is the left side, which way is up, whether a logo is cut by a door or whether a band joins up from one piece to the next. Today the only answer is to export and look in the game, which is slow. The check in the game stays the final test; what is missing is a guide while designing.

The 3D preview removed on 2026-10-08 (`remove-3d-preview`) was a placeholder that never rendered anything. Since then the app reads the game's vehicle models itself (`ets2-game-extraction`), so the painted surfaces of every official vehicle are already at hand: only their vertex positions are skipped. A simple view built from them is now cheap.

## What Changes

- **A Side View panel in the Workshop** shows the vehicle of the open texture, painted with the livery being edited, from four fixed directions: left, right, front and back. It is a flat (orthographic) drawing, not a 3D scene: no orbit, no zoom on the model, no lighting beyond a plain shade that makes the shape readable.
- **It follows the work live:** every change on the canvas shows in the view. Switching texture, cabin or vehicle switches the view.
- **Main texture only:** a truck shows the cabin of the open main texture, a trailer its body. Accessory textures (deflectors, side skirts, sun visors…) are not drawn in this change.
- **View › Show Side View** turns the panel on and off; the choice and the chosen direction are remembered in the preferences.
- **Extraction stores a preview mesh in each official package:** for each main texture, the positions, paint UVs and triangles of its painted surfaces, nothing else (no other parts, materials or normals). Projects do not embed it, so `.truckpaint` files stay small and carry no game geometry.
- **The sample truck and trailer get a simple hand-made preview mesh:** a few boxes whose faces map the labeled zones of their templates (cab sides, front, roof; trailer sides, front, back). The documentation can then show the Side View with the Blue Wave Biarritz livery, without any game file.
- **When there is no preview mesh the panel says why:** "Not available for custom vehicles" for a custom vehicle, and a message asking to generate the vehicle again from My Game for an official package made before this change (the app is not released, so there is no migration).

Non-goals:
- free 3D (orbit, zoom, perspective), realistic rendering (base color, metallic, flip-flake, airbrush): the game remains the place to judge the final result;
- linking the view and the canvas (hover a point of the view to mark it on the texture, click to select the object there): the natural next step, kept for a later change;
- a preview for custom vehicles (Custom Vehicle packs templates only), and any tooling or guide for package authors to make a preview mesh: the package format accepts one, but only official packages and the samples carry it in this change.

Follow-up, high priority: **accessories.** For a player, the vehicle seen from the side is the cabin with its accessories, so a view without them is incomplete. It already shows in this change: on some trucks (the DAF XD for one) the doors, bumper, fenders and deflectors are accessories painted with the main texture, so the view of the main texture has gaps where they go. Drawing them needs the game's attachment points and a choice of one variant per slot (otherwise a mirror or a deflector would be drawn over the livery in every variant at once), which is why it waits for the follow-up. They come from other models, attach at the cabin's locators and depend on the configuration chosen in the game, which makes them a change of their own. The preview mesh stored in the package is laid out so that positioned accessory pieces can be added later without changing what this change stores.

## Capabilities

### New Capabilities

- `side-view`: the Side View panel of the Workshop, its directions, live update, what it shows for a truck and a trailer, and its messages when no preview mesh exists.

### Modified Capabilities

- `ets2-game-extraction`: model reading also keeps vertex positions; each official package stores a preview mesh per main texture.
- `vehicle-generation`: a package that gains a preview mesh is a new package version, so packages made before this change are regenerated.
- `vehicle-packages`: a package may carry a preview mesh per main texture; packages without one stay valid.
- `vehicle-authoring`: the sample truck and trailer carry hand-made preview meshes, checked against their templates.
- `workspace-layout`: the Workshop gains the Side View panel and View › Show Side View.
- `app-preferences`: the panel's visibility and direction are remembered.

## Impact

- **tp-scs:** the `.pmg` reader keeps the position stream it already skips.
- **tp-pack / vehicle generation (My Game):** builds and writes the preview mesh next to the templates; the `tpv extract` tool does the same, being the same code.
- **tp-vehicles:** reads the optional preview mesh from a package.
- **tp-app:** a new panel drawing the projected, textured mesh with egui (no new wgpu pipeline), fed by the texture the app already renders for the open texture; View menu item, preference, empty states.
- **tp-i18n:** new strings (panel title, directions, command, empty-state messages) in en/fr/es/de.
- **Sample vehicles:** `examples/vehicles/sample-truck` and `sample-trailer` gain a preview mesh file and their `.tpv` are packed again; the mesh follows the zone rectangles of their SVG templates, so it stays valid as long as the zones do.
- **Documentation:** a page or section about the Side View in the four languages, with media rendered from the sample fleet and Blue Wave Biarritz (`doc_media.rs`), and the Blue Wave headless test extended to check the view is drawn.
- **Official packages already generated** must be generated again to get the view.
