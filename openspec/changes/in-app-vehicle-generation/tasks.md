## 1. Library API (tp-pack, after native-game-reader)

- [ ] 1.1 Add `inspect(folder)` returning game and version, from the archives only; test with a synthetic game folder (small `.scs` files built in the test, no game assets committed): ETS2, ATS, missing `base.scs`, unreadable file
- [ ] 1.2 Add `scan(folder)` listing vehicles with kind, name and brand using the existing discovery and paintability rules; test the DLC-only and not-paintable cases on the synthetic folder
- [ ] 1.3 Add `generate(&Scan, vehicle, previous)` returning the package bytes in memory with version from the game (first three numbers), the minor-series range, the "this computer" description, license, and the version bump rule; unit tests for 1.61.1.9, 1.62.0.3, same-version changed template, unchanged template skipped
- [ ] 1.4 `previous` takes one installed package; rebuild `run` and `previous_packages` on these functions and verify `tpv extract` output is byte-identical to before on the synthetic folder
- [ ] 1.5 Check generated manifests have no `homepage` (removed by `remove-marketplace`) and `tpv check` accepts the package

## 2. Game folders (tp-app)

- [ ] 2.1 `game.rs`: pure Steam library parsing and per-OS candidate paths (home and data folders as inputs); tests for Windows, Linux native, Flatpak and macOS paths and a second library, using temp folders
- [ ] 2.2 `prefs.rs`: `game_folders: BTreeMap<String, PathBuf>` keyed by game id (`ets2`, `ats`), shared with `first-run`, `#[serde(default)]`; test round trip and a file without it
- [ ] 2.3 Preferences "Games" section (installation folders; `first-run` adds the games played and mod folders) in `ui/dialogs.rs`: Detect, Choose…, Clear, version, messages for refused or missing folders, not changed by Reset to defaults; headless UI test with a synthetic folder
- [ ] 2.4 Detect once on a first run with no folders; test that nothing is set and no error is shown when Steam is absent

## 3. Generation worker (tp-app)

- [ ] 3.1 Worker thread with channel, one vehicle at a time (no parallelism), `Progress` naming the vehicle starting: scan, generate list, progress, per-vehicle result, finished; cancel flag checked between vehicles; test with the synthetic folder (success, cancel after N, one failure continues)
- [ ] 3.2 Install each result on the UI thread with `install_bytes`; refresh the library; test that a failed or cancelled vehicle installs nothing and a skipped one installs nothing
- [ ] 3.3 One run at a time; message when a run ends with the dialog closed; test the disabled state
- [ ] 3.4 Measure per-vehicle time, peak memory and total disk on a real ETS2 install with all DLC (sequential run); note the numbers in the PR

## 4. My Game tab (tp-app, tp-i18n)

- [ ] 4.1 Third tab in `library` (`LibraryTab`): list with search and kind filter, status (Not generated, Generated version, Game updated), game and folder header, Refresh, Preferences…
- [ ] 4.2 Checkbox list with Select All / Select None / Select Updates, one Generate (n) button that generates or regenerates by status, progress bar (finished of checked, "3 of 12"), current vehicle name, per-row state (waiting, in progress, installed, up to date, failed), Cancel, final result with failures and their reasons
- [ ] 4.3 Empty states: no folder (propose the detected one), invalid folder, ATS not supported yet, nothing found; the SCS notice; empty-library state offers My Game
- [ ] 4.4 Fluent strings in en, fr, es, de with the same ids (Title Case for buttons and tab; check German widths)
- [ ] 4.5 Headless UI tests (`egui_kittest`) with the synthetic game folder: listing, statuses, check 3 and generate with the bar and row states advancing, regenerate a Game updated vehicle, cancel, ATS message, no folder

## 5. Documentation

- [ ] 5.1 Update `vehicles/library.mdx`, `vehicles/official-vehicles.mdx` and `getting-started/first-project.mdx` in English: setting the game folder, My Game, generating, updating after a game patch, ATS not yet, the notice
- [ ] 5.2 Translate to fr, es and de (same files and `sidebar.order`); the build passes with `<Ui id>` ids present in all four locales
- [ ] 5.3 `mise run doc:media vehicles` for the new screens (synthetic game folder in `doc_media.rs`); run `docs-drift-checker` before archiving

## 6. Finish

- [ ] 6.1 `mise run ci` passes
- [ ] 6.2 Tell `first-run` which pieces to reuse (detect, inspect, scan, worker) and check `native-game-reader` and `remove-marketplace` have landed first
