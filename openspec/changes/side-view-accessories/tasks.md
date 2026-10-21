## 1. Spike: trailers and rotations

- [ ] 1.1 On the installed game, list the locators of one `scs.box` chassis model and check where a body (`curtain_136`) is mounted (expected: a `body` locator on the chassis), and find one rotated exterior locator on a truck chassis (rear lights or exhaust) with its accessory; verify by recording the findings in design.md (decision 2 and the risks) and stopping to ask the user if trailer bodies do not hang on a chassis locator

## 2. Locators in the model reader

- [ ] 2.1 In `tp-scs/src/model.rs`, read the locator table (count at header offset 20, table at 80, 44 bytes each: token, position, scale, quaternion, hookup offset) and each part's `locator_count` / `first_locator`, failing the file like the other tables when out of bounds; verify with a unit test on a synthetic `.pmg` (extend `tp-scs/src/synthetic.rs`) that names, positions, rotations and the part each locator belongs to round-trip, and that a truncated table names the file

## 3. Preview format

- [ ] 3.1 In `tp-vehicles`, add the optional `locators` and `slots` (id, parent, options with id, name, default, pieces, chassis locators) to the preview mesh, with the validation of the vehicle-packages delta (repeated ids, two defaults, missing parent locator, chassis parent without chassis slot); verify with unit tests for a valid mesh with slots, each refusal, and a format-1 mesh without slots still read as before
- [ ] 3.2 Add the placement function (`position + rotate(q, scale · p)`, at every locator of the slot's name of its parent) next to the format; verify with unit tests: the DAF XD door figures of design.md give x from −1.245 to 1.245, a 90° yaw locator turns +x into −z (or the order found in 1.1), two locators of one name place two copies

## 4. Extraction

- [ ] 4.1 Resolve each painted accessory's slot (last word of its unit name) and parent (cabin's visible EU locators first, then the suitable chassis), and build per main texture the cabin pieces, the cabin locators, the `chassis` slot (options with their locators) and one slot per accessory slot with its painted options in model space, unpainted pieces and accessories left out, slots without a locator reported; verify with `tp-pack/tests/extract.rs` on the synthetic game: a door slot with parent cabin, a bumper slot with parent chassis, an unpainted accessory absent, a missing locator reported
- [ ] 4.2 Merge options with identical painted pieces (millimetre rounding), name a merged option after the default or the first one; verify with a synthetic test where two door definitions differing only inside become one option
- [ ] 4.3 Mark defaults by following `defaults[]` from the cabin (and the chassis it lists), recursively; an unpainted default leaves the slot without default; verify with a synthetic test, and on the installed DAF XD that `cab_door` and `f_bumper` have a default and `r_deflector` is as the game fits it
- [ ] 4.4 Trailers: the chassis is the root slot, the body and other slots hang on it, and a blank main texture still gets a preview mesh; verify with a synthetic trailer test and on the installed `scs.box` (body slot with its options, curtain 13.6 m drawn on the chassis)
- [ ] 4.5 Record per main texture in `report.json` the cabin, the slots with their options and default, and every accessory left out with its reason; verify in the synthetic tests' report
- [ ] 4.6 Count changed preview meshes as a package version change (not a template version change) in My Game generation; verify with a test regenerating a package whose meshes have no slots: next patch, template versions unchanged, projects not flagged
- [ ] 4.7 Measure package growth on the DAF XD and `scs.box`; verify the numbers are in the PR description and stay within a few MB zipped (otherwise stop and raise it, see design risks)

## 5. Project file

- [ ] 5.1 Add the per-vehicle Side View choices to the project model and `FileVehicle` (`side_view`, written only for choices that differ from the default), outside the undo history, marking the project changed; verify with tests: choices reopen after save, an unknown option falls back to the default, undo after a choice and an edit only undoes the edit

## 6. The Side View

- [ ] 6.1 Build the configuration (cabin of the open or last main texture, chosen chassis, chosen option per slot, defaults when nothing is recorded) and place its pieces; verify with headless tests on the sample truck: default configuration, switching cabins keeps the choices, opening an accessory texture keeps the cabin
- [ ] 6.2 Draw several textures: request every drawn surface from the 1024 px thumbnails, draw pieces of a texture not in the project in the neutral color, emit one `egui::Mesh` per texture run after the global sort, cache per configuration and direction; verify with unit tests on a two-texture box (depth order kept across textures) and the existing live-update test extended to an accessory texture
- [ ] 6.3 Add the Accessories button and list in the panel header (one row per slot, drop-down with None and the options, no None for the chassis, disabled with no vehicle, the "generate again" note for a mesh without slots); verify with `egui_kittest` tests: picking None removes a piece, one option per slot, the chassis has no None, the header still fits in the four languages at the smallest window
- [ ] 6.4 Remove the old "base color only" message case and keep the others; verify the empty-state tests

## 7. Strings and samples

- [ ] 7.1 Add the strings (Accessories, None, the "generate again" note, the slot names table) in en/fr/es/de with the same ids; verify the i18n tests
- [ ] 7.2 Add the sample truck 1.1.0 `sideskirt` slot (Side skirts texture) and the sample trailer `body` slot (Curtain body 13.6 m default, 10.5 m), with the locators they need, rebuild the committed packages, and extend the sample preview test to check every face against the template of the texture that paints it; verify the package and sample tests pass

## 8. Documentation

- [ ] 8.1 Update the Blue Wave scene and `doc_media.rs` so the Side View shot shows the sample truck with its side skirts and add a shot of the sample trailer with its curtain body; verify `mise run doc:media workshop` renders them in the four languages
- [ ] 8.2 Update the Side View section of the Workshop page (whole vehicle, only painted parts, the Accessories list, saved in the project, trailers) and remove the "parts mounted as accessories are missing" bullet with `docs-writer`, then fr/es/de with `docs-translator`; update the package format page (`locators`, `slots`) in English; verify `mise run doc:build` passes

## 9. Wrap-up

- [ ] 9.1 Run `mise run ci` through the `rust-ui-tester` agent and fix what fails; verify it passes (GPU-only tests named as skipped)
- [ ] 9.2 Check in the real app with the DAF XD and an official trailer generated from My Game: the door in place, toggling the roof deflector, an accessory texture edit showing live, the choices kept after reopening the project; verify with `mise run run` (done by the user)
