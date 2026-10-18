## 1. Check the real game first

- [ ] 1.1 Spike (throwaway script or test, not committed with game data): on the maintainer's ETS2 1.61 install, list every entry of every archive with extension and compression id, and the version of every `.pmg` and `.pmd` under the vehicle folders; verify that the answer to "does any `.sii`, `.sui`, `.pmd`, `.pmg`, `.mat` use GDeflate, zstd or headerless zlib" and the list of `.pmg` versions are written into the design's Open Questions (resolved), and add a GDeflate task here if needed.
- [ ] 1.2 Record a reference batch with the current ConverterPIX build: all 69 vehicles into a kept folder; verify the folder holds 69 packages (or the known skipped ones) and its `report.json`.

## 2. Archive reader (`tp-scs`)

- [ ] 2.1 Create the `tp-scs` crate (workspace member, `flate2` dependency, lints) and write CityHash64 with its tests; verify the test vectors from the ConverterPIX C++ code pass for lengths 0-16, 17-32, 33-64 and over 64 (and replace the code by a crate only if that crate passes them all).
- [ ] 2.2 Read HashFS v2: header, compressed entry and metadata tables, lookup by salted hash, plain metadata, stored and zlib entries, directory listings (one level and recursive); verify unit tests on a synthetic archive built in the test (stored file, compressed file, nested folders, salt, a listing) pass.
- [ ] 2.3 Add every error path: wrong magic, version other than 2, hash method other than `CITY`, offsets and sizes outside the file, encrypted or unsupported compression, size mismatch after inflate; verify each returns a message naming archive, path and cause (or version) and none panics, including on truncated copies of the synthetic archive.
- [ ] 2.4 Mount archives in order with last-mounted-wins lookup and an inventory of all paths; verify a test with two archives holding one path returns the later one, and that an archive whose root has no listing is still searched by path.

## 3. Model reader (`tp-scs`)

- [ ] 3.1 Read `.pmd` version 4 (looks, variants, per look material paths, variant part visibility) and the effect of a `.mat` file; verify unit tests on a synthetic descriptor and material give the names, effects and visibility that `geometry.rs` expected from the `.pit` text.
- [ ] 3.2 Read `.pmg` versions found in 1.1 (0x15 first, then 0x14 and 0x13 only if used): header, parts, pieces, UV streams with aliases from the mask, triangles; verify synthetic models pass, and that an `_UV7` stream aliased `_TEXCOORD1`, an empty piece list and hidden pieces behave as the `geometry.rs` tests do.
- [ ] 3.3 Bounds-check every offset and count, and return unknown versions as an error naming file and version; verify truncated and bad-version model tests pass without panic.

## 4. Use the reader in `tp-pack`

- [ ] 4.1 Add a native source next to the converter one: archive inventory, `read`, `locate` and listing on `tp-scs` in memory; verify the existing provenance, overlay and inclusion tests pass with synthetic archives instead of the fake converter.
- [ ] 4.2 Make `geometry.rs` select from `tp-scs` model data instead of `.pim`/`.pit` text; verify the geometry unit tests (semantic alias, visibility, repeated looks, shader families, UV overflow, unsupported cases, replay of validated templates) pass unchanged in meaning.
- [ ] 4.3 Run the full batch with the native source on the full ETS2 1.61 install, all 69 vehicles; verify every `.tpv` is byte-identical to task 1.2 (or, where not, that manifests and SVG templates are identical and the cause is written in `docs/game-extraction-plan.md`), and that `report.json` is equal apart from the reader field. Record the duration and peak memory.

## 5. Remove the converter

- [ ] 5.1 Delete `--converter` (arguments, help, `ConverterOptions`), the converter `Command` code, the model cache and its key, converted/reused counters, converter logs and the per-worker extraction folders; write `reader` in `report.json`; verify `tpv extract --converter x` is refused as an unknown option, no `converter` or `model_cache` entry remains in the report, and no `target/game-extraction/cache` is created.
- [ ] 5.2 Replace the fake converter test harness (`crates/tp-pack/tests/extract.rs` and fixtures) with a synthetic-archive harness; verify the extract tests cover the batch scenarios of the spec (failure isolation, unique batches, concurrent equals single, previous layout versions, unreadable model, unsupported archive version) and pass.
- [ ] 5.3 Run `mise run ci`; verify format, lint, tests and build pass, and that the comparison of 4.3 still passes on the final code.

## 6. Documentation

- [ ] 6.1 Update `README.md`, `CONTRIBUTING.md`, `docs/game-extraction-plan.md` and `crates/tp-pack/tests/fixtures/game-extraction/README.md` to remove ConverterPIX and `--converter`, and to describe the native reader; verify `rg -i "converterpix|--converter"` finds only the plan's historical reference to the pinned source and the licence credit, if kept.
- [ ] 6.2 Update `website/src/content/docs/{en,fr,es,de}/vehicles/official-vehicles.mdx` (no converter step, no converter path, same example command shape); verify the four pages match in structure, `mise run doc:build` passes, and run the docs drift check agent.
