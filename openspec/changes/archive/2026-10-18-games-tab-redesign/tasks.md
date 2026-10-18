## 1. Strings

- [x] 1.1 Add `prefs-game-folder-label`, `prefs-mod-folder-label`, `prefs-folder-not-set`, `prefs-button-for` and `prefs-game-folder-target` in en, fr, es and de (same ids, American spelling and sentence case in English; the game's own terms elsewhere), and remove `prefs-games`, `prefs-game-folder-hint` and `prefs-mod-none` when nothing else uses them; verify with `mise run test` (the localization test checks the four locales have the same ids)

## 2. Games tab

- [x] 2.1 Rebuild `games_section` and `mod_folder_row` in `crates/tp-app/src/ui/dialogs.rs`: a heading per game with its version beside it, two rows (Game folder, Mod folder) with the same field look, "Not set" for an empty one, the intro note first and no tab-name heading; verify with a headless test that the tab shows both game headings and the labels once each, and no "Games" heading
- [x] 2.2 Show the buttons by state (game folder: Detect + Choose… when empty, Choose… + Clear when set; mod folder: Choose…, plus Clear when chosen; no mod Detect, no disabled button) and verify with headless tests for the empty and set states of the game folder and the chosen and unchosen states of the mod folder
- [x] 2.3 Give every button its accessible name from `prefs-button-for` and verify with a test that the four Choose… are found by four distinct names, and that clicking the one for ATS's game folder opens the ATS picker

## 3. Tests and checks

- [x] 3.1 Update `tests/game.rs`, `tests/mod_export.rs` and `tests/preferences_tabs.rs` (they click or count `Choose…` and `Clear` by their old shared labels) and verify with `mise run test`
- [x] 3.2 Verify in German at 100% that nothing on the Games tab is truncated (headless test of the tab's content width), and that no new text is below the contrast tokens (the existing automated contrast test passes)
- [x] 3.3 Run `mise run ci` through the rust-ui-tester agent and fix its failures; verify it passes

## 4. Documentation

- [x] 4.1 Update the Preferences paragraph of `website/src/content/docs/en/vehicles/official-vehicles.mdx` (Detect only while empty, Clear once set, no Detect for the mod folder), then translate it to fr, es and de with the docs-translator agent; verify with `pnpm build` in `website/`
- [x] 4.2 Render `vehicles/game-folders` again with `mise run doc:media vehicles`, keep only that screenshot's four files if the command re-encodes others, and verify the English and German images show the new layout
