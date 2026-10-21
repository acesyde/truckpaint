## 1. Positions in the model reader

- [x] 1.1 In `tp-scs/src/model.rs`, read the position stream (piece offset 64, 3 × f32, first in the vertex stride) into `Piece::positions`, failing the file like the other streams when it is absent or out of bounds; verify with a unit test on a synthetic `.pmg` (`tp-scs/src/synthetic.rs`) that positions round-trip exactly and that a truncated position stream names the file
- [x] 1.2 Check the offset and the axes on an installed game (`tpv extract --vehicle` on one truck and one trailer): the painted pieces' bounding box is a few meters, y is up, and the cab front is towards −z; record the result in the design's decision 7 (convert in the extractor if the axes differ); verify by noting the measured bounding boxes in the PR description

## 2. Preview mesh format and validation

- [x] 2.1 In `tp-vehicles`, read the optional `preview_mesh` field of a main texture and parse the JSON file (`format`, `pieces[].texture/positions/uv/triangles`) with the validation of the vehicle-packages delta (missing file, unknown texture id, length mismatch, index out of range, non-finite number), each error naming the file; verify with unit tests for a valid mesh and each refusal, and that a package with no preview mesh is still accepted
- [x] 2.2 Make sure a project never embeds a preview mesh (`tp-file` writes only templates) and that the app can load the mesh from the installed package by the project's package id and version; verify with a test that saves a project from a package with a mesh and finds no mesh in the `.truckpaint` bytes

## 3. Extraction and generation

- [x] 3.1 In `tp-pack` extraction, build each non-blank main texture's preview mesh from `geometry::painted` for the first cabin of its game ids (first cabin of the vehicle when it has none; first configuration for a trailer), round positions to 1 mm, write `preview/<texture id>.json` and set `preview_mesh`; record the chosen configuration (or why there is none) per main texture in `report.json`; verify with `tp-pack/tests/extract.rs` on the synthetic game: a two-layout truck gets two meshes without each other's pieces, a blank trailer main texture gets none and the report says why
- [x] 3.2 Add a test that every edge of a generated preview mesh, drawn in UV space, is an edge of the template SVG built from the same meshes; verify it passes on the synthetic vehicles
- [x] 3.3 In the vehicle generation (My Game), count a main texture gaining a preview mesh as a change for the package version but not for its template version; verify with a test regenerating a package without meshes: a next-patch package is installed, template versions unchanged, projects not flagged
- [x] 3.4 Measure the zipped package size growth on two or three real cabins and one trailer; verify the numbers are noted in the PR description and stay within a few MB per vehicle (otherwise raise it before going on, see the design's risks)

## 4. Sample meshes

- [x] 4.1 Write `preview/<id>.json` for the Standard cab and High roof of sample truck 1.0.0 and 1.1.0 (cab box: sides, front, rear, roof, each face's UV corners on its labelled panel) and for the sample trailer's Base (chassis frame and front wall), name them in each `vehicle.json`, and rebuild the committed `.tpv`; verify the existing "package matches its sources" test passes
- [x] 4.2 Add a test that every face of each sample mesh lies inside one labelled `<rect>` of its template, sharing the zone parser of `tests/common/blue_wave.rs`; verify it passes, and fails when a panel of the Standard cab template is moved

## 5. The Side View panel

- [x] 5.1 Give `SurfaceThumbnails` a side parameter (default 128, unchanged for its current uses) and use a 1024 px instance for the drawn surface with `update_only`; verify the existing thumbnail tests pass and a new one checks a 1024 instance keeps an 8192 × 1024 shape at 1024 × 128
- [x] 5.2 Project and sort the mesh for the four directions (parallel projection, back-to-front by centroid depth, shade from the face normal), cached per mesh and direction; verify with unit tests: a unit box seen from Left draws its −x face last and its +x face not at all on top, and the drawing fits a given rectangle keeping proportions
- [x] 5.3 Draw the Side View as a strip under the canvas in the Workshop's canvas area, with the Left / Right / Front / Back segmented control in its header and a resizable top edge clamped between its minimum and maximum; keep the canvas resize rule; verify with an `egui_kittest` test: showing the panel keeps the zoom and the center point, dragging the edge changes the height within its limits, Tab keeps the panel shown
- [x] 5.4 Pick the drawn texture: the open main texture, or for an accessory the vehicle's last opened main texture (first main texture otherwise); verify with a headless test on the sample truck: Standard cab → High roof switches the drawing, opening the Chassis keeps the High roof
- [x] 5.5 Show the empty-state messages (custom vehicle, official package without mesh, blank main texture, package not installed); verify with a headless test for each case
- [x] 5.6 Add the View › Show Side View command (check mark, enabled only in the Workshop) to the menu structure shared with the command palette; verify the View menu test lists it after the snapping item and the palette shows its menu path

## 6. Preferences and strings

- [x] 6.1 Persist the Side View's visibility, direction and height in the workspace layout, with defaults (hidden, Left, 220 pt) when the fields are missing; verify with tests for the restart scenario and for an older preferences file
- [x] 6.2 Add the strings (command, panel title, four directions, four empty-state messages) in en, fr, es and de with the same message ids, following the capitalization rules; verify the i18n tests pass and a German screenshot of the panel header fits at the minimum panel width

## 7. Documentation

- [x] 7.1 Add a Side View doc scenario to `crates/tp-app/tests/doc_media.rs` with Blue Wave Biarritz on the sample truck's Standard cab, seen from the left, and extend `tests/blue_wave.rs` to check the Side View draws the cab with the livery (not blank); verify `mise run doc:media workshop` renders it in the four languages
- [x] 7.2 Write the Side View section of the Workshop documentation in English with the `docs-writer` agent (what it shows, the directions, main textures only, the messages), then fr, es and de with `docs-translator`; verify `mise run doc:build` passes
- [x] 7.3 Document `preview_mesh` and the mesh file in `website/src/content/docs/en/vehicles/package-format.mdx` (English only, as that page has no translation); verify `mise run doc:build` passes

## 7b. Top direction

- [x] 7b.1 Add the Top direction (seen from above, front towards the left, right side at the top) to the projection, the segmented control, the persisted direction and the strings in en/fr/es/de; verify with a unit test that a box's +y face is the one drawn on top and that its +x side lands at the top of the drawing, and with the headless test that the header still fits in the four languages at the smallest window
- [x] 7b.2 Mention Top in the Side View section of the Workshop documentation in en, fr, es and de; verify `mise run doc:build` passes

## 8. Wrap-up

- [x] 8.1 Run `mise run ci` through the `rust-ui-tester` agent and fix what fails; verify it passes (GPU-only tests named as skipped)
- [x] 8.2 Check in the real app with an official truck generated from My Game and the sample truck: the panel, the five directions, a live edit, and an accessory texture open; verify with `mise run run` and a screenshot in the PR
