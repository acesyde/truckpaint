## Context

See proposal.md for why. Today `extract/source.rs` runs ConverterPIX with every archive mounted (`-b`) for two jobs: list/extract files, and convert models to `.pim`/`.pit` text that `geometry.rs` parses. Facts found in the reference source (`target/game-extraction-reference/ConverterPIX`, commit 3cd4e73a):

- All installed ETS2 1.61 archives start with `SCS#` and version 2 (checked on the 40+ archives of the maintainer's install): HashFS v2 is the format that matters. ConverterPIX also reads HashFS v1 and zip.
- HashFS v2 (`structs/hashfs_0x02.h`, `fs/hashfs_v2*.cpp`): a 53-byte header (magic, version 2, salt, hash method `CITY`, entry and metadata table sizes and offsets); an entry table sorted by hash (16 bytes: CityHash64 of `<salt><path without leading slash>`, metadata index and count, flags) and a metadata table of u32, both zlib-compressed when their compressed size differs from their size. Each entry lists metadata records; a file's "plain" record packs size, compressed size, offset and a 4-bit compression id (0 none, 1 zlib, 2 headerless zlib, 3 GDeflate, 4 zstd). Directory entries hold a text listing (`*name` for folders, `name` for files). Image metadata is for textures.
- ConverterPIX itself implements only none, zlib and GDeflate for v2 files. GDeflate (`libs/GDeflate`, a DirectStorage format) exists for GPU-friendly texture data; no text or model file is known to use it.
- Models: `.pmg` versions 0x13, 0x14, 0x15 (`structs/pmg_0x13..15.h`; `model.cpp` has one loader each) and `.pmd` version 4 (`structs/pmd.h`). `geometry.rs` only needs per piece: material index, texture-coordinate mask and width, float2 UV streams, triangle indices; per part: name and piece indices; from the descriptor: look and variant names, per look material aliases and effects (read from each `.mat` file), per variant part visibility attribute. ConverterPIX names a stream `_UV<k>` and gives it the alias `_TEXCOORD<i>` for each nibble `i` of the mask equal to `k`.
- The `.pim` text prints floats as exact 32-bit hex, so reading the binary floats gives the same values.

Constraints: no game asset in the repository; `tp-core` has no files; the app is not released, so the old command line and cache are deleted, not migrated.

## Goals / Non-Goals

**Goals:**
- Same packages as ConverterPIX on the full ETS2 1.61 install, with no external program.
- A reader `tp-app` can call later (`in-app-vehicle-generation`), cross-platform, with no GPU or Python.
- Clear failure on any version or damage it does not understand.

**Non-Goals:**
- A general SCS tool: no writing, no textures, no animations, no mod zips.
- Matching ConverterPIX's output text format; only the data `geometry.rs` uses is kept.

## Decisions

**New crate `tp-scs`, used by `tp-pack`.** It holds the archive reader, CityHash64, the model reader and the `.mat` effect read, with no UI and no image or texture dependencies. `tp-app` would otherwise depend on `tp-pack`, which pulls `image` and `texpresso`. The extraction logic (definitions, selection, packaging) stays in `tp-pack`; how the app calls it is decided in `in-app-vehicle-generation`. Alternative: a module of `tp-pack`. Rejected because the dependency weight and the app reuse above are real.

**HashFS v2 only, plus a clear error for anything else.** The real install is v2 throughout. Version 1 and zip are older or mod formats the product does not need ("official vehicles of an installed game"). Anything else fails with the archive name and version. If the full-game check or a supported game version shows a v1 archive, adding it is a small task (a 32-byte header and a flat entry table). Alternative: port v1 and zip now. Rejected as unused code.

**Archive reading: read the header and tables into memory; read each file on demand.** Entries are found by binary search on the hash, as the game does. A file is returned as `Vec<u8>` (defs and models are small; the largest model is a few MB). No memory map, so no `unsafe` and no platform difference. Recursive and per-folder listing come from directory entries' text, with the same `[D]`/`[F]` ordering the current code expects. Archives whose root cannot be listed keep working through per-folder lookups, as in `parallel-ets2-extraction`.

**Mounting: same list and order as `Sources::new` today, last mounted wins.** The order list (`base.scs` … `version.scs`, then sorted `dlc_*.scs`) and the file map built at inventory stay. Only the five ConverterPIX calls (`command`, `listing`, `locate`, extraction, listing parse) are replaced by in-process reads. Files are read from memory, so the per-worker extraction folders and the converter logs go away; the batch work folder only stages packages and is kept on failure as before.

**Decompression: zlib through `flate2`; other methods are an error.** `flate2` is already compiled through `zip`; `tp-scs` declares it directly, with the same pure-Rust backend (`zlib-rs`) the workspace already selects for `zip`. GDeflate and zstd are not implemented unless the spike below shows a needed file uses them. The plain-meta compression id is checked for every file read, so an unexpected method fails with archive, path and method rather than returning garbage. Alternative: implement GDeflate now (no mature Rust crate; the C++ library is large). Rejected until needed.

**CityHash64: own code, about 150 lines, in `tp-scs`.** CityHash64 v1.0.x as vendored by ConverterPIX (`libs/cityhash`): the game's hash is that exact version, whose output differs from newer CityHash releases for some lengths, so a general crate whose version we cannot check is a risk, and a crate for one function adds a dependency to audit. Tests: known vectors computed from the ConverterPIX C++ code on several path lengths (0-16, 17-32, 33-64, over 64 bytes), plus a lookup of real entries on the real install in the comparison task. Alternative: a crates.io `cityhash` crate. Accepted only if the spike shows it matches the vectors, then the own code is dropped.

**Models: read into plain structs, not into `.pim` text.** `tp-scs` returns `Model { pieces, parts }` and `Traits { looks, variants }`; `geometry.rs` selects from them. This removes the text re-parse and the PIX syntax code for models (`syntax.rs` stays for definitions if it parses them). Float values are kept as `f32`, as the hex floats did, then widened as today. Versions 0x13, 0x14 and 0x15 each have their own piece layout (field order differs: see the headers); each gets a small reader on one shared piece type. `.pmd` version 4 and the `.mat` effect line are read as ConverterPIX does. Only what `geometry.rs` uses is parsed; bones, locators, normals and weights are skipped by offsets, with bounds checks. Alternative: emit the same `.pim`/`.pit` text and keep `geometry.rs`. Rejected: it keeps a format only the converter needed, and parsing text again costs time on 10,000 models.

**Bounds and versions are checked once, at the reader.** Every offset and count from a file is checked against the file size before use. The error always carries the file and the version found (`.pmg` version byte, `.pmd` u32, archive u16).

**Report: `converter` becomes `reader`.** `report.json` keeps `format`, and `reader` is `{"name": "tp-scs", "version": <crate version>}`. `model_cache` and converter fields are removed. The archive list and precedence text are unchanged.

**Acceptance is a comparison, not a belief.** Before ConverterPIX code is deleted, a reference batch is made with the current build on the maintainer's full ETS2 1.61 install (`--previous` not used; 69 vehicles). The native build must give the same `.tpv` bytes; if the archive differs only by non-deterministic metadata, identical manifests and identical SVG templates are accepted, and the difference is recorded. `report.json` is compared apart from the reader field. Synthetic tests build small HashFS v2 archives and model files in memory (no game asset is committed).

## Risks / Trade-offs

[A file we need uses GDeflate or zstd in some DLC] → The spike (task 1.1) lists the compression ids of every entry of the install by extension; reading an unexpected id fails loudly, so the failure is visible, not silent.
[PMG layouts for 0x13/0x14 differ from my reading, or the installed models are all 0x15] → The spike counts versions in the real install; unused versions are not implemented, and are refused with the clear message instead. The comparison covers the used ones.
[CityHash version mismatch makes every lookup miss] → Entry lookups on the real install in the first task prove it before anything else is built.
[Native reading is faster but the new code is the only reader] → The comparison and synthetic tests; damaged-input tests for bounds.
[A future game update changes a format] → The version error names the file and version; formats are not guessed.
[`user-documentation` still names ConverterPIX] → That requirement ("Content sections") is also reworded by the Marketplace and generation changes; the pages are updated here, and the requirement text is left to the first of those changes to archive, to avoid conflicting deltas.

## Migration Plan

Not released, so no migration. Order: reader crate and tests, native source in `tp-pack` next to the converter for the comparison, comparison on the full game, then delete the converter path, the cache and the fake-converter tests, then docs. Rollback before the last step is the old code path; after it, git.

## Open Questions

- Does any file we read (defs, `.pmd`, `.pmg`, `.mat`) use GDeflate or zstd on ETS2 1.61 with all DLC? Expected no; answered by the spike (task 1.1). If yes, GDeflate becomes a task of this change.
- Which `.pmg` versions (0x13, 0x14, 0x15) do the 69 vehicles use? Answered by the spike; unused versions are refused, not implemented.
- Is HashFS v1 or zip needed for any installation we support (older game versions)? Expected no.
