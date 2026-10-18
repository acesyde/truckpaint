## Why

Making an official vehicle package needs ConverterPIX, an outside program that players would have to find and install. It has no macOS build, it is slow (a first run converts about 10,000 models and keeps an 11 GB cache), and it needs a licence and a separate download. Packages are no longer shared: players will make them from their own game (see `in-app-vehicle-generation`). The app must therefore read the game files itself.

## What Changes

- TruckPaint reads the game's `.scs` archives itself, on Linux, Windows and macOS. Files in later archives still win over files in earlier ones, as today.
- TruckPaint reads the vehicle models (`.pmd` and `.pmg`) itself, only as far as painted surfaces need: looks, variants, parts, materials, UV streams and triangles.
- **BREAKING**: `tpv extract` no longer has `--converter`. The ConverterPIX executable is not needed anywhere.
- The model cache, the converted-model folders and the converter logs are removed. `report.json` records a reader version where it recorded the converter.
- A game file of a version TruckPaint does not know stops with a clear message that names the file and the version. It is never skipped without notice.
- The result must not change: on the full ETS2 1.61 install, all 69 vehicles come out the same as with ConverterPIX.
- Documentation stops mentioning ConverterPIX (README, CONTRIBUTING, the plan, the official vehicles page in four languages).

Non-goals:
- Texture (`.dds`, `.tobj`) conversion, animations, collision or skeleton files.
- Zip-based `.scs` mods and the older HashFS version 1, unless the check on the real game shows they are needed.
- Game paths, scanning and the Vehicle Library screen (`in-app-vehicle-generation`).
- ATS extraction.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `ets2-game-extraction`: the command no longer takes a converter; archives and models are read natively; unsupported format versions fail with a clear message; the report records a reader version; the model cache requirement is removed.

## Impact

- New small crate `tp-scs` (archive and model reader), used by `tp-pack` now and by `tp-app` later. New dependency: `flate2` (already in the build through `zip`); a small CityHash64 written in the crate.
- `tp-pack`: `extract/source.rs` (command, listing, cache) rewritten on `tp-scs`; `extract/geometry.rs` reads model data instead of `.pim`/`.pit` text; `extract/mod.rs` loses the converter and cache plumbing; `bin/tpv.rs` loses `--converter`.
- Tests: converter-fake tests become synthetic-archive tests; new unit tests for archives, hashing and models; a checked comparison on the full game.
- Docs: `README.md`, `CONTRIBUTING.md`, `docs/game-extraction-plan.md`, `website/.../vehicles/official-vehicles.mdx` (en, fr, es, de), fixtures README. The `user-documentation` requirement that names ConverterPIX is owned by the generation changes (see design).
