## 1. Artwork and palette

- [x] 1.1 Sample the three colors from the reference image, and draw `mark.svg`, `waves.svg` and `coast.svg` under `crates/tp-app/tests/fixtures/blue-wave/` (original drawings, no third-party marks); verify each opens in a browser and that only the three palette colors (plus white and transparency) are used
- [x] 1.2 Write `crates/tp-app/tests/fixtures/blue-wave/brief.md` from the design's "The brief" (identity, colors with sampled hex values, logo lockup, motifs, composition rules) and state that the artwork is original and MIT licensed and that the reference image is not part of the repository; verify the file exists and `git status` shows no copy of the reference image

## 2. The livery scene

- [x] 2.1 Create `tests/common/blue_wave.rs` building the sample fleet (truck with its default textures, trailer with its main texture and the 13.6 m body) with the three swatches, the White base and Blue panel graphic styles (the waves are images, which take no graphic style) and the Tagline text style; verify the headless test finds 3 swatches and the 3 styles
- [x] 2.2 Read the labeled zones (rectangle and label) of the three templates (Standard cab, Curtain body 13.6 m, Base) from the sample packages, and write the layout table of the design as a constant; verify with a throwaway print that every zone name resolves to one rectangle (the two CURTAIN zones in template order)
- [x] 2.3 Import the SVGs as assets, convert the logo to a symbol and place one instance on a truck texture and one on the trailer texture, each from that texture's own layout table; verify the symbol has six instances (three on the truck, three on the trailer) and the placements differ
- [x] 2.4 Paint every row of the layout table (truck Standard cab, trailer curtain body, trailer base), the elements linked to swatches or styles, the logo placed once per texture; verify by rendering each of the three textures to a PNG and checking them against the brief; the table may be adjusted for a better result, and the design's table updated to match
- [x] 2.5 Add `tests/blue_wave.rs` (headless) covering: the fleet exists, no off-palette color outside the images, the logo symbol has an instance on the truck and on the trailer, every layout row lands inside its zone, every listed zone is painted, an exported texture is not blank; verify `mise run test` passes, and fails when the logo instance is removed or a row's box is made larger than its zone

## 3. Documentation media

- [x] 3.1 Move the `workshop` section of `doc_media.rs` to the livery (workshop, text-selected, layers, layers-to-reposition) and keep the gesture animations on an empty texture where they teach a gesture; verify `mise run doc:media workshop` renders and the images look right in the four languages
- [x] 3.2 Move the `brand` section (palette, styles, symbols, brand space) to the livery; verify the palette lists White, Light blue and Blue and the symbols list shows the logo
- [x] 3.3 Move the `export` and `fleet` sections and the sample-vehicle shots of the `vehicles` section to the livery; verify `mise run doc:media export`, `fleet` and `vehicles` render
- [x] 3.4 Check `getting_started`, `ai_assistant` and the project shots: use the livery where a finished paint job is shown, keep empty states where they teach first use; verify `mise run doc:media` runs end to end

## 4. Documentation text

- [x] 4.1 Update the alt and label texts that name ACE Logistics or the old shapes in `website/src/content/docs/en/` (workshop overview and text-and-images, vehicles library, and any page `rg "ACE Logistics|ellipse|plate"` finds), then the same pages in `fr`, `es` and `de` with the `docs-translator` agent; verify `rg "ACE Logistics" website/src` returns nothing
- [x] 4.2 Mention the example livery once where the docs introduce the sample vehicles (a line saying the screenshots show a fictional carrier), in the four languages; verify `mise run doc:build` passes

## 5. Agents and contributor guides

- [x] 5.1 Update `.claude/agents/docs-writer.md`: when a page describes a painted example, it names the Blue Wave Biarritz livery (never invented shapes or "ACE Logistics"), reads `tests/fixtures/blue-wave/brief.md` and the layout table for what the images show, and asks for a new scenario that uses the livery scene when a medium is missing; verify with `rg "Blue Wave|brief.md" .claude/agents/docs-writer.md`
- [x] 5.2 Update `.claude/agents/docs-translator.md`: the livery's name and tagline are artwork and stay unchanged in fr, es and de (the "example names" rule, which says to follow what each language's screenshot shows, gets this exception); verify the rule reads consistently with the existing example about "Background" and "Fondo"
- [x] 5.3 Update `.claude/agents/docs-drift-checker.md`: add a check that no page, alt text or label still names the old example (`ACE Logistics`, the old shapes), that the alt texts match what the livery scene shows, and that a change to the brief, the layout table or the artwork lists every section to render again with `mise run doc:media`; verify with `rg "livery|ACE Logistics" .claude/agents/docs-drift-checker.md`
- [x] 5.4 Add to `AGENTS.md` (Documentation site) and `CONTRIBUTING.md` (documentation media) a short note: the media show the example livery, defined by its brief and layout table under `crates/tp-app/tests/fixtures/blue-wave/`, to change in one place; verify both files mention it and that a docs-only change still skips the Rust CI jobs

## 6. Checks

- [ ] 6.1 Render all media once more from a clean `target/doc-media`, run `mise run doc:media` twice and verify the encoded files are identical between runs
- [x] 6.2 Run `mise run ci` and verify it passes (GPU-only tests stay ignored)
- [x] 6.3 Run the `docs-drift-checker` agent and verify it reports no stale page or medium
