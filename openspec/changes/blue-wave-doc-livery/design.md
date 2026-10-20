## Context

The media scenarios in `crates/tp-app/tests/doc_media.rs` build their scenes with `common::scenes` (`demo_scene`, `demo_brand`, `brand_fleet`, `livery_tree`, `lettering_scene`, `vector_scene`, `gradient_scene`): a few rectangles and ellipses in made-up colors, drawn on the default sample truck texture (a light grey template overlay). `tests/screenshots.rs` shares the same scenes. See proposal.md for motivation.

Facts that shape the approach (observed in the code):
- The app has no 3D or side view. Every medium is a flat texture, optionally with the template overlay. The reference image is an illustration and is not reproduced one for one; it inspires a brief.
- The sample templates (`examples/vehicles/sample-*/…/templates/*.svg`) are abstract labeled zones, with different layouts per texture.
- The scenes already use the real building blocks: `add_swatch`, `new_graphic_style`, `new_text_style`, `convert_to_symbol`, `add_asset` (SVG), `Object` frames and paths, and `fleet_project` for a multi-vehicle project.
- Media are rendered locally (GPU) and committed; CI builds the site from them. A headless test run does not need a GPU.

## Goals / Non-Goals

**Goals:**
- One livery definition, used by the documentation media and by a headless test.
- A livery that shows palette, symbol instances per texture, styles and lettering working together on a truck and a trailer.
- Deterministic output: rendering twice gives the same images.

**Non-Goals:**
- Reproducing the reference image one for one, or making the flat textures look like its 3D renders.
- Replacing the small scenes of the other tests.

## Decisions

**The livery is a scene module, not a project file.** A new `tests/common/blue_wave.rs` builds it with the same project API the other scenes use (fleet of the sample truck and trailer, swatches, styles, symbol, objects). Alternative: a committed `.truckpaint` project opened by the tests. Rejected: a binary-like file hides the intent, goes stale silently when the file format changes, and cannot be reviewed in a diff.

**Artwork is hand-drawn SVG under `tests/fixtures/blue-wave/`, loaded with `include_bytes!`.** Files: `logo.svg` (wave mark), `waves.svg` (the flowing bands in the three blues), `coast.svg` (lighthouse and rocks silhouette). They enter the project as SVG assets (they stay vector, as the Image import spec requires) and the logo becomes a symbol. Alternative: build every curve from `Path` objects in Rust. Rejected: dozens of Bézier points in code are unreadable and impossible to restyle; SVG is editable in any tool. Alternative: trace the reference image. Rejected: it is AI generated, its terms are unclear, and it carries Volvo and Krone marks; the repository is MIT.

**Colors are three swatches taken from the reference** (White, Light blue, Blue), and every object links to a swatch (`fill_swatch`), like `brand_fleet` does. The SVG fills use the same hex values; the test checks that the project has no off-palette color outside the imported images. Hex values are sampled from the reference when implementing.

**Per-texture placement is a layout table, relative to the template's labeled zones** (see "Layout table"). Rows are fractions of a zone, so the result is deterministic and follows a template if a zone moves. This is the product rule made visible: shared elements, own placement. Alternative: one automatic layout. Rejected: layouts differ per vehicle, and an automatic copy would break the rule the docs explain.

**Lettering uses the text engine and a text style** ("Blue Wave" in the logo is artwork; the tagline "L'excellence du transport sur la Côte Basque" is a text object, italic with the app's bundled fonts if no script font is bundled). One text style is created from it, so the Brand pages show a real style.

**Media scenarios are migrated section by section, keeping medium names.** `workshop/workshop`, `brand/*`, `export/*`, `fleet/*`, `vehicles/*` keep their file names, so the `.mdx` pages need only new alt and label texts, not new `<Shot />` references. Scenarios that need a bare texture to teach a gesture (drawing a rectangle on an empty texture, the first-run and installation screens) keep their empty start; the livery appears where a finished paint job makes sense.

**One headless test for the fixture** (`tests/blue_wave.rs`, no GPU, using the existing `wgpu`-free harness): builds the scene and asserts the checks of the spec (swatch links, symbol instances on both vehicles, non-blank export through the existing texture export path).

**`demo_scene`, `demo_brand` and `tests/screenshots.rs` are left alone.** They are small, quick and many tests depend on their exact shapes. Merging them is an option for a later change.

## The brief (extracted from the reference image)

The reference image is an illustration, not a model to copy: the brief keeps its ideas (identity, colors, motifs, spirit) and the layout below is our own composition, adapted to the zones of the sample templates. Where the image and a zone disagree, the zone wins. The reference image is not committed, so the ideas it carries are written down here and copied to `tests/fixtures/blue-wave/brief.md`, the readable source of truth that the scene and the images follow.

**Identity**
- Name: Blue Wave Biarritz. Place: Biarritz, Pays Basque, France.
- Tagline: "L'excellence du transport sur la Côte Basque", italic script, underlined by a light-blue swoosh.
- Keywords: Mer, Transport, Confiance, Performance, Côte Basque.
- The name and the tagline are artwork: the same in all four documentation languages.

**Colors** (hex sampled from the reference when the artwork is drawn; the swatch names are the English labels of the reference board)
- White: purity, reliability. Base of every texture.
- Light blue: ocean, openness. Used for "Wave", the swoosh, the upper wave bands.
- Blue (deep navy): performance, trust. Used for "Blue", the lettering, the lower wave bands, the coast silhouette.

**Logo lockup** (`logo.svg`, one symbol "Logo")
- A wave crest in three stacked blues, the words "Blue Wave" in bold sans-serif ("Blue" Blue, "Wave" Light blue), and "BIARRITZ" small and letter-spaced between two thin rules.

**Motifs** (`waves.svg`, `coast.svg`)
- Flowing wave bands in the three blues; a lighthouse and rocks silhouette in Blue, as at the Biarritz coast.

**Composition rules** (our interpretation, free of the illustration's exact placement)
- White base everywhere; blue only for graphics and lettering.
- Waves rise from the lower front toward the rear and end in the coast silhouette at the rear of the trailer.
- The logo is large on the trailer, small on the cab doors, centered on the rear.
- The tagline goes under the logo wherever a zone is large enough to read it (the trailer sides; the cab rear wall); it is left out of small zones rather than shrunk until unreadable.
- Left and right sides use the same table; the waves and the coast are mirrored so they stay at the rear, the logo and text are never mirrored.

## Layout table

The livery is a data table, a Rust constant in `tests/common/blue_wave.rs`, so that the images are the same whenever they are rendered. Each row names a texture zone (a labeled rectangle of the sample template, read from the template SVG, never hard-coded in pixels), an element, and a box in fractions of that zone (`x y w h`, 0 to 1, top left origin). The scene resolves a row to document coordinates with the zone's rectangle; a change to a template's zone moves the artwork with it. Every zone not listed stays unpainted, as the template overlay shows it.

**Truck, Standard cab** (main texture, 4096)

| Zone | Element | Box (x y w h) | Notes |
|---|---|---|---|
| LEFT DOOR | White fill | 0 0 1 1 | base |
| LEFT DOOR | Logo | .10 .64 .80 .22 | below the window |
| LEFT DOOR | Waves | 0 .88 1 .12 | clipped to the zone, rising to the rear |
| RIGHT DOOR | same three rows as LEFT DOOR | | waves mirrored, logo not |
| FRONT | White fill | 0 0 1 1 | base |
| FRONT | Lettering "Blue Wave" | .20 .005 .60 .04 | strip above the windshield, Blue |
| FRONT | Waves | 0 .78 1 .22 | lower corners, under the lamps |
| ROOF | White fill | 0 0 1 1 | |
| REAR WALL | White fill | 0 0 1 1 | |
| REAR WALL | Logo | .25 .22 .50 .30 | centered |
| REAR WALL | Tagline | .15 .56 .70 .07 | text style "Tagline" |
| BUMPER | Blue fill | 0 0 1 1 | with a Light blue line at .8 of its height |

**Trailer, Curtain body 13.6 m** (accessory, 4096)

| Zone | Element | Box (x y w h) | Notes |
|---|---|---|---|
| CURTAIN (1st) | White fill | 0 0 1 1 | left side |
| CURTAIN (1st) | Logo | .30 .12 .32 .43 | large |
| CURTAIN (1st) | Tagline | .30 .58 .32 .09 | text style "Tagline" |
| CURTAIN (1st) | Waves | 0 .38 1 .62 | lower part, rising to the rear |
| CURTAIN (1st) | Coast | .82 .30 .18 .45 | rear end |
| CURTAIN (2nd) | same rows | | right side: waves and coast mirrored |
| REAR DOORS | White fill | 0 0 1 1 | |
| REAR DOORS | Logo | .20 .25 .60 .45 | centered |
| REAR DOORS | Waves | 0 .88 1 .12 | |
| FRONT BULKHEAD | White fill | 0 0 1 1 | |
| FRONT BULKHEAD | Waves | 0 .85 1 .15 | Light blue only |
| ROOF | White fill | 0 0 1 1 | |

**Trailer, Base** (main texture, 2048): both CHASSIS FRAME zones filled Blue; REAR BUMPER filled White. Nothing else.

The numbers are a starting composition: the implementer may move, resize or drop a row to make a texture look good, and records the change in this table. What stays fixed is the brief (identity, colors, motifs) and the rules below.

A box larger than its zone is a layout error: the headless test fails on it.

## Risks / Trade-offs

- [The sample templates are abstract, so the livery looks like a flat atlas] → It is a livery interpreted for the zones, not a copy; use the template overlay in the shots where mapping matters; richer templates are a separate change.
- [Original drawn artwork is time-consuming and may look amateur next to the reference] → Keep the shapes simple and bold (a wave mark, layered bands, one silhouette); review the images before the media are committed.
- [Every medium is re-rendered, a large binary diff] → Do it once, in one commit, after the scenes are final; no media are committed during development.
- [Alt texts in four languages drift] → The tasks update `en` first, then the three translations, and `docs-drift-checker` runs at the end.
- [Rendering needs a GPU, so CI cannot check the images] → The headless test guards the livery; the media remain a local step, as today.

## Open Questions

- Font for the tagline: a script style as in the reference only if a suitable font is already bundled; otherwise italic. Decide when the text is placed; it changes no spec.
