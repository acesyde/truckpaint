## 1. Preferences and folders

- [x] 1.1 Add `first_run_done`, `games` and `mod_folders` to `Prefs` (serde defaults) with the "existing user counts as done" rule; unit tests: round trip, absent fields, recent projects set done.
- [x] 1.2 Make `mod_export::destination` prefer the chosen mod folder when it exists; unit tests for chosen, missing chosen (falls back), none.
- [x] 1.3 New Project's initial game from `games` (when its vehicles are installed); unit test.
- [x] 1.4 Preferences' Games section: I play this game (at least one stays checked) and the mod folder (Detect, Choose…, Clear) beside C's installation folder for ETS2 and ATS; kittest: first-launch choices shown, mod folder changed then used by Export Mod, last game can't be unchecked.

## 2. Flow state and screen

- [x] 2.1 Add `FirstRun` state (step, games, filters, checks, job) and show it from the home screen while not done; `egui_kittest`: first start shows it, a done prefs file shows home.
- [x] 2.2 Step indicator, Back, Skip on every step (sets done, keeps choices); kittest for Back and Skip.
- [x] 2.3 Step 1: language (no save when unchanged), games (at least one), installation folder rows via C's detection with Choose folder…, mod folder rows with Choose folder…; kittest with a synthetic game folder: found, not found, chosen.
- [x] 2.4 Step 2: scan via C in the background, list with search, All/Trucks/Trailers counts, checkboxes, Select all; kittest with a synthetic game folder (counts, filter, search, select all).
- [x] 2.5 Generate and continue with progress, Cancel (keeps generated), per-vehicle errors, Continue when nothing is checked; kittest for generate, cancel, nothing, one failure.
- [x] 2.6 Game not found or ATS: message, Install the sample vehicles (uses `install_samples`), Custom Vehicle… note; kittest for both.
- [x] 2.7 Step 3: set done, save prefs, open New Project at the Vehicle step on the first game; kittest for the hand-off with and without vehicles.
- [x] 2.8 Help › Run First Launch Again… command (command id, menu, palette): current settings preselected, generated vehicles shown as such, Close Project first from a project; kittest from home, from a dirty project (Cancel keeps it), Skip changes nothing.

## 3. Strings

- [x] 3.1 Add the flow's strings to `tp-i18n/locales/{en,fr,es,de}/*.ftl` with the same ids (English Title Case for buttons; sentence case elsewhere); run `mise run test` for the locale parity test.
- [x] 3.2 Check the three steps in German at the smallest window width with a kittest or screenshot; nothing truncated.

## 4. Documentation

- [x] 4.1 Write the English getting-started page for the first launch (use `<Ui id>`, `<Shot>`, `<Clip>`), then fr/es/de with matching `sidebar.order`; update pages that mention the home screen's first run.
- [x] 4.2 Add the first-launch screens to `crates/tp-app/tests/doc_media.rs`, run `mise run doc:media getting-started`, commit the media; `mise run doc:build` passes.

## 5. Wrap up

- [ ] 5.1 `mise run ci` passes; run docs-drift-checker; re-sync the shared specs with the parallel changes before archiving.
