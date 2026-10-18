## Context

`tp_pack::extract::run(&Options)` does everything for the `tpv extract` command: it mounts the archives, finds the vehicles, converts models with ConverterPIX, builds each package and writes a folder and a report, using half the cores. It refuses anything but ETS2 and gets the package version from its caller. `native-game-reader` replaced ConverterPIX and the model cache with the `tp-scs` crate, used by `tp-pack`; this change builds on it. Installing goes through `VehicleLibrary::install_bytes` (`tp-app/src/vehicles.rs`); background work in the app uses an `mpsc` channel and a thread (`custom_vehicles.rs`, `export.rs`). `mod_export.rs` already finds the mod folder per OS. See proposal.md for motivation and the specs for the behavior.

## Goals / Non-Goals

**Goals:**
- Pieces with no UI: `detect games`, `check folder`, `scan`, `generate one vehicle`, so `first-run` reuses them.
- The `tpv extract` command and the app produce the same package for the same game.
- The UI never waits for the game files.

**Non-Goals:**
- ATS generation. Extraction reads `eut2` only, and ATS trucks and trailers have not been checked against its rules. The ATS folder can be set and checked now so the setting exists when support comes; the tab says it is not supported. Each game's support is a later change.
- A cache of scans or of models on disk.
- Writing the ConverterPIX-free reader (see `native-game-reader`).

## Decisions

**A library API in `tp-pack::extract`, split from the command.** Three functions, with no printing and no folders:
- `inspect(folder) -> GameInfo { game, version }`: opens `version.scs` / `def.scs` only; cheap, used to check a folder.
- `scan(folder) -> Scan`: game info plus the vehicles found (id, kind, name, brand), using the same discovery and paintability rules as `run`.
- `generate(&Scan, vehicle, previous: Option<&Package>) -> Result<Generated { bytes, manifest, skipped }, Failure>`: builds one package in memory.

`run` becomes a loop over these (and keeps its report and `--vehicle`/`--previous`). One code path means the app cannot drift from the command. Not shelling out to `tpv`: no process to ship, and no progress or cancel. A `Scan` holds the opened archives (file tables only), shared read-only by the generation threads.

**Detection lives in `tp-app` (`game.rs`), not in `tp-pack`:** it is about the player's computer, not about the game's files. It reads Steam's `libraryfolders.vdf` (a small key-value text, parsed by hand for the `path` entries) from, in order: Windows `Program Files (x86)\Steam` and `Program Files\Steam`, plus the Steam path in the registry if it can be read without a new dependency (otherwise the default path only); Linux `~/.local/share/Steam`, `~/.steam/steam` and `~/.var/app/com.valvesoftware.Steam/.local/share/Steam`; macOS `~/Library/Application Support/Steam`. In each library it looks for `steamapps/common/Euro Truck Simulator 2` and `American Truck Simulator`; on macOS also the `.app` bundle's resources folder, since that is where the archives are. Anything found goes through `inspect`, so a wrong folder is never trusted. GOG and Epic are not searched: the player chooses the folder by hand. Path building is a pure function taking the home and data folders as inputs (like `mod_folder_in`), so every OS is tested on every OS with a temp folder.

**Package version from the game's version: `MAJOR.MINOR.PATCH` of the game's four numbers.** Game 1.61.1.9 gives 1.61.1. Alternatives: the whole version as `1.61.1+9` (build metadata is ignored when comparing, so two builds would collide); a counter owned by TruckPaint (nothing links it to the game, and two players would have different numbers for the same game). The fourth number is a build number that rarely changes a template, so dropping it keeps the version readable and stable. Because two builds can then share a version, a rule protects installed packages: if a changed template would be installed under a version already installed, the version is the installed newest plus one patch (1.61.1 becomes 1.61.2). Versions therefore always rise and old ones stay installed beside the new one (existing behavior), so a project pinned to 1.61.1 keeps its templates.

**Template versions reuse the extraction's `--previous` logic.** `generate` takes the installed newest package of the same id as `previous`, exactly the argument `run` builds from a folder, so an unchanged template keeps its `layout_version` and a changed one gets the next. The `template-update-impact` capability then flags projects. `previous_packages` is changed to accept one package, and the folder scan stays for the command.

**Supported game versions are the minor series** (`>=1.61, <1.62`), as extraction writes today. The status is computed from that range alone, with no stored game version: Generated when the scanned version satisfies the installed newest package's range, Game updated when it does not. A patch inside the series is therefore not flagged, which fits since a patch rarely changes UVs; the player can still Regenerate, and if nothing changed the vehicle is reported as up to date and nothing is installed.

**Background work: one worker thread per run, a channel to the UI.** The UI sends `Scan(folder)` or `Generate(vehicles)`; the worker sends `Scanned`, `Progress { done, total, current }`, `Installed`/`Failed`/`Skipped` per vehicle and `Finished`. Cancel is an atomic flag checked between vehicles, because one vehicle is short (see below) and a half-built package is never written. The worker only builds bytes; `install_bytes` is called on the UI thread when it receives each result, because `VehicleLibrary` is owned by the UI (no lock, no shared mutable state). Generation inside a run is sequential, one vehicle at a time, as the player asked: the progress is readable vehicle by vehicle (bar of finished out of checked, current vehicle, per-row state), memory stays one vehicle's worth, and Cancel takes effect at the next vehicle. `Progress` carries the index and id of the vehicle starting, so the UI marks rows waiting / in progress / installed / skipped / failed. The command line `tpv extract` keeps its parallel loop; only the app's worker is sequential. The run keeps going if the dialog is closed; its end is a message. Only one run at a time.

**Cost.** With the native reader there is no model cache and no conversion: the disk cost is the installed packages (about 1 MB each on average, around 70 packages), nothing else. Each worker holds one vehicle's models and its SVG templates in memory (tens of MB). Time is expected in the order of a second per vehicle; the measurement is a task, and the progress and Cancel design does not depend on it. The scan reads directory tables and a few definition files, so it should finish in a second or two; it still runs off the UI thread because archive tables of a full install with all DLC are large.

**Where the tab gets its state.** The UI keeps `GameState { folder, info, vehicles, run }` in the Vehicle Library dialog's state and refreshes it when the tab opens, on Refresh and when Preferences changes the folder. Status is derived by comparing `scan.vehicles` with `VehicleLibrary::get`, as the Installed tab does, so the two cannot disagree.

**Manifest wording.** Generated manifests keep the license from extraction. Their `homepage` field is removed by `remove-marketplace` (it pointed to the catalog repository) and is not set again: a generated package is the player's own and links nowhere. The `description` says "Generated from ETS2 <version> on this computer".

**First run.** `first-run` calls `detect`, `inspect`, `scan` and the generation worker directly; nothing here depends on the dialog.

## Risks / Trade-offs

- **A game patch changes a template inside the series** → not flagged as Game updated; Regenerate finds it, and the version rule keeps old packages. Documented in the tab's help text.
- **Windows registry unread, non-Steam installs** → the player picks the folder; the empty state explains how.
- **Game files on a slow or removed drive** → every read error becomes a message naming the folder; nothing partial is installed.
- **A full run is slower than the parallel command** (about 70 vehicles one after another) → accepted for a readable progress; the measured time per vehicle goes in the PR, and the tab shows the count so the player knows how long is left.
- **macOS archive location unverified** → the check reads the archives, not a path, so detection tries the likely folders and the player can choose by hand; confirm on a Mac before release.
- **Packages differ between players' game versions** → intended; ids stay the same and versions say which game they come from.

## Migration Plan

None: the app is not released. The `tpv extract` command keeps its options; `--converter` and the model cache are removed by `native-game-reader`.

## Open Questions

