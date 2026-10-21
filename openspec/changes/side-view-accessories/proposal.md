## Why

The Side View (`livery-side-view`) draws only the model of a main texture's cabin. On many trucks the parts a player paints most are mounted as accessories: the DAF XD's doors, bumper, fenders and deflectors are painted with the cabin texture but are separate models, so the view shows a gap where the door is. Side skirts, sun visors and trailer bodies have their own textures and are not drawn at all; for an official trailer the view shows little more than a ladder. A player cannot judge a livery on a vehicle that is missing half its panels.

The game places every accessory at an attachment point (a locator) of the cabin or chassis model, named after the accessory's slot (`cab_door`, `r_deflector`, `f_bumper`…). The extractor already reads those models; it only skips their locators.

## What Changes

- **The Side View draws the whole vehicle:** the cabin (or a trailer's chassis) and every painted accessory chosen for it, each with its own texture's artwork. Main and accessory textures appear together, whichever texture is open.
- **Only painted parts are drawn.** Glass, lights, plastic and chrome parts, unpainted mirrors and the like are left out, so the view shows where the livery goes, not every object that might hide it.
- **One choice per accessory slot, made in the panel:** an Accessories list in the Side View shows each slot that has painted options (Door, Roof deflector, Bumper, Body…) with "None" and its options. Options of one slot never show together. Options whose painted shape is the same (the six DAF XD doors, which only differ inside) are one option.
- **The game's default configuration first:** a vehicle starts with the accessories the game fits on a new vehicle (the `defaults` of its cabin and chassis); a slot with no painted default starts at "None".
- **The choices are saved in the project,** per vehicle, and reopen as they were.
- **Trailers use the same model:** the chassis and the body are slots like the others, so a trailer shows its chosen body with its livery. Only a single trailer is drawn (no doubles or B-doubles).
- **Official packages carry the accessories' preview pieces**, placed at their locators, with their slot, option and whether the game fits them by default. Packages made before this change keep working with the cabin only and are regenerated like any template change.
- **The sample truck and trailer get hand-made accessory pieces** (side skirts, the curtain bodies), so the documentation can show Blue Wave Biarritz on the whole sample fleet.

Non-goals:
- unpainted parts, even as grey shapes;
- several trailers in a combination, and the truck and trailer drawn together;
- UK variants (the EU attachment points are used, as the extraction does today);
- choosing the cabin from the panel: the cabin stays the one of the open (or last opened) main texture;
- linking the view and the canvas (pointing at the view to find the spot on a texture).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `side-view`: draws the whole vehicle with every texture, adds the Accessories list (one choice per slot, game defaults first), drops the rule that an accessory texture keeps the last main texture's drawing.
- `vehicle-packages`: preview pieces may carry a slot, an option and a default flag; a package may hold pieces for accessory textures and for accessories painted with a main texture.
- `ets2-game-extraction`: model reading also returns locators; the preview mesh holds every painted accessory option placed at its locator, grouped by slot, with the game's defaults.
- `vehicle-generation`: a package gaining accessory preview pieces is a new package version.
- `project-files`: a project records, per vehicle, the accessory chosen in each slot for the Side View.
- `vehicle-authoring`: the sample truck's side skirts and the sample trailer's curtain bodies get preview pieces.

## Impact

- **tp-scs:** read the locator table of `.pmg` files (name, position, rotation) per part.
- **tp-pack (extraction and My Game):** resolve each painted accessory's slot and locator, transform its pieces, group equal shapes, follow the `defaults` chains; the trailer chassis and body become slots.
- **tp-vehicles:** preview piece fields for slot, option and default; validation.
- **tp-file / tp-core:** the per-vehicle accessory choices in the project, saved and reopened.
- **tp-app:** the Side View draws several textures at once (one larger render per texture shown, off the UI thread), the Accessories list in the panel, strings in en/fr/es/de.
- **Size:** packages grow with every painted accessory option; to measure on a well-equipped truck and a trailer with many bodies.
- **Documentation:** the Side View section and its media in the four languages, the package format page.
