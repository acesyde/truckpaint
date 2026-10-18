## 1. Preferences model

- [x] 1.1 Remove `games`, `set_plays` and the seeding in `start_first_run` from `crates/tp-app/src/prefs.rs` and `ui/first_run.rs`; make `plays` mean "a folder is stored" with a `played()` helper in `Game::ALL` order; verify with unit tests in `prefs.rs` (played follows folders, order ETS2 then ATS)
- [x] 1.2 Verify a preferences file that still has a `games` list loads with its other fields and the list ignored (new test in `prefs.rs`)
- [x] 1.3 Change `NewProjectDraft::new` (`state.rs`) and its callers (`project_io.rs`, `end_first_run`) to take the played games from the folders: first game with installed vehicles, else the only game with a folder, else ETS2; verify with unit tests for each of the three cases (the existing `NewProjectDraft` tests are updated)

## 2. Preferences dialog

- [x] 2.1 Remove the "I play this game" checkbox, its refusal hint and `prefs-play` / `prefs-play-hint` from the Games tab (`ui/dialogs.rs`) and the four locales; verify the Games tab shows folder and mod folder rows only (headless UI test) and `mise run test` has no unused-string failure
- [x] 2.2 Verify Clear on a game's folder makes it no longer played (test in `tests/game.rs` or `tests/ui.rs`) and that choosing a folder of the other game is refused with the other game's message and stores nothing

## 3. First launch

- [x] 3.1 Replace the checkboxes of step 1 (`ui/first_run.rs`) with both games' installation and mod rows, run the Steam detection for both games, and remove the Next guard and `first-run-need-game`; verify with headless tests: both detected shows two "Game found" rows, none found keeps Next enabled
- [x] 3.2 Add the first-launch test for a folder of the other game (ATS picked for ETS2 and the reverse): message shown, nothing stored
- [x] 3.3 Rework the Vehicles step to use folders (ETS2 scannable when stored and valid; stored but unreadable gives the existing message; no folder gives no message and the samples hint; ATS stored gives the "not supported yet" line); verify with the existing first-launch tests adapted, plus a "no folder, no error" test
- [x] 3.4 Reword `first-run-games` in en, fr, es, de (Fluent, same message id) and check German still fits the step; verify with `mise run test` (localization tests) and the first-launch screenshot in German
- [x] 3.5 Verify Run First Launch Again preselects the current folders and no longer a games list (test in `tests/`)

## 4. Tests and checks

- [x] 4.1 Update the headless tests that check or uncheck a game or read `prefs.games` (`tests/ui.rs`, `game.rs`, `mod_export.rs`, `persistence.rs`, `localization.rs` and the screenshot and doc media scenarios) to set a folder instead; verify with `mise run test`
- [x] 4.2 Run `mise run ci` through the rust-ui-tester agent and fix its failures; verify it passes

## 5. Documentation

- [x] 5.1 Update `website/src/content/docs/en/getting-started/first-launch.mdx` (the game checkbox line) and any page that mentions "I play this game" or the Games checkbox, then translate to fr, es and de with the docs-translator agent; verify with `pnpm build` in `website/` (unknown ids fail the build)
- [x] 5.2 Render the first-launch and Preferences screenshots again with `mise run doc:media getting-started` (and the other sections that show Preferences); verify the committed images in all four languages show no checkbox
