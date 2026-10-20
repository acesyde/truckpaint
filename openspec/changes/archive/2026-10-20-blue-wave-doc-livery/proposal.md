## Why

Every screenshot and animation of the documentation shows the same pile of test shapes (a red band, a grey plate, an ellipse, a bar, "ACE Logistics") on a plain light texture. It proves the tools work, but it does not show what a real livery looks like, or how the palette, symbols and styles work together on a fleet. A reader cannot tell what TruckPaint is capable of.

A complete, believable livery gives every page the same story. The reference is a presentation board for a fictional carrier, **Blue Wave Biarritz** (a wave logo, three blues on white, a tagline, flowing wave graphics, a lighthouse and coast silhouette), applied to a tractor and a curtainsider trailer.

## What Changes

- **One example livery, "Blue Wave Biarritz", replaces the demo shapes in every documentation medium.** It is painted on the two sample vehicles (the sample truck's Standard cab and the sample trailer's curtain body and chassis), so the docs show a fleet, not a single texture.
- **The livery is built from the brand elements the app teaches:** a three-color palette (White, Light blue, Blue), the wave logo as a symbol with one instance per texture, a graphic style for the wave band, a text style for the lettering and tagline. The Brand and Fleet pages finally have real content to explain.
- **A written brief and a layout table make the livery deterministic.** The ideas of the reference image (name, tagline, colors and their meaning, logo lockup, motifs, composition rules) are extracted into a brief kept in the repository, and the placement is a table of elements per labeled zone of each template. The images follow the table, not a developer's eye, so they stay the same over time.
- **The artwork is original and vector:** the wave logo, wave bands and coast silhouette are drawn for this repository as SVG files (MIT, like the sample vehicles). The reference image itself, which is AI generated and shows third-party marks, is not committed.
- **The same fixture serves the tests and the images.** A shared scene builds the livery in a headless test (no GPU) that checks it is valid, and the documentation media scenarios use the same scene, so the images cannot drift from what the tests cover.
- **The documentation agents and contributor guides learn the example.** `docs-writer`, `docs-translator`, `docs-drift-checker`, `AGENTS.md` and `CONTRIBUTING.md` are updated, so later pages and checks keep to the same livery.
- **All documentation media are rendered again** in the four languages, and the screenshot descriptions and animation labels that name "ACE Logistics" or the old shapes are updated in the four languages.

Non-goals:
- showing a 3D truck or a side view: TruckPaint edits flat textures, and the reference image is a presentation illustration; it inspires the livery, which is not copied from it (see Impact);
- new app features, new vehicles, or changes to the sample vehicle templates;
- moving `tests/screenshots.rs` and the unit-level tests off `demo_scene`: they need small, quick scenes;
- using Volvo, Krone or any official vehicle in the docs (they need the player's game).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `user-documentation`: the documentation media share one example livery, painted on the sample fleet from the brand elements, with original artwork, from a scene the headless tests also use.

## Impact

- **Tests (no app code):** a new fixture module and artwork under `crates/tp-app/tests/common/` and `crates/tp-app/tests/fixtures/blue-wave/`, a headless test for it, and the scenarios of `crates/tp-app/tests/doc_media.rs`. `demo_scene` and `demo_brand` stay for the other tests.
- **Documentation:** `website/public/media/<lang>/**` (every medium is rendered again, one large but one-time binary change) and the alt and label texts of the pages that name the old example, in `en`, `fr`, `es`, `de`.
- **Agents and guides:** `.claude/agents/docs-writer.md`, `docs-translator.md`, `docs-drift-checker.md`, `AGENTS.md`, `CONTRIBUTING.md` (documentation only, so the Rust CI jobs are skipped for these files).
- **App, crates, strings:** none. The brand name and tagline are artwork, not interface text, so no Fluent message is added.
- **Interpretation, not copy:** the reference is an illustration. The brief keeps its ideas and the layout is a composition adapted to the sample templates (abstract labeled zones such as *Door* and *Curtain*, not a truck silhouette), so the textures will not look like the board's renders. Richer sample templates would be a separate change.
- **Assumption:** the reference's three colors are taken from the image at implementation time (white, a light blue near `#3D9BE6`, a deep blue near `#0B2F6B`).
