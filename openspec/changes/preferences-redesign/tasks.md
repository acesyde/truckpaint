## 1. Strings and shared pieces

- [x] 1.1 Add the new messages in en, fr, es and de with the same ids (`prefs-game-state-unset`, `prefs-game-state-problem`, `prefs-mcp-enable-hint`, `prefs-mcp-can-title`, `prefs-mcp-can-read`, `prefs-mcp-can-draw`, `prefs-mcp-can-import`, `prefs-mcp-cannot-save`, and the accessible names of the inline controls), American spelling and sentence case in English; verify with `mise run test` (the localization test compares the four locales)
- [x] 1.2 Add `state_chip`, the path field frame with inline buttons, the interactive `switch` helper and the quiet text button in `crates/tp-app/src/ui/dialogs.rs` (or `tp-ui` for the switch if the status bar can share it); verify with unit or headless tests that each reports its click and its accessible name, and that the switch reports its toggled state

## 2. General tab

- [x] 2.1 Reorder the General tab (Language first, label-and-value header over each slider, rule between Interface and Canvas, small-capital titles) and keep every value editable; verify with a headless test that Language is the first setting and the sliders, the grid spacing and the title bar option still change their preferences
- [x] 2.2 Move Reset to defaults to a quiet text button on the left of the footer, General tab only; verify with `tests/preferences_tabs.rs` that it is only there on General and still resets scale and grid spacing

## 3. Games tab

- [x] 3.1 Build one card per game with its state beside the name and **Detect** at the foot while the folder is empty; verify with a headless test of the three states (set and valid, empty, set but gone) showing Version, Not set and Folder problem as words
- [x] 3.2 Put **Choose…** and **Clear** inside the game folder and mod folder fields, with complete accessible names; verify with tests that Choose… for each of the four rows opens its own picker, Clear inside a set field clears it (and the mod folder goes back to the detected one), and typing a game folder path still works
- [x] 3.3 Keep the keyboard order path, Choose…, Clear within a row and verify it with a test that moves focus with Tab

## 4. AI Assistant tab

- [x] 4.1 Replace the checkbox with the switch, title and subtitle, then the state as a dot and words, the address field with Copy inside, and the port; verify with `tests/mcp.rs` (updated to find the switch by its title) that the server turns on and off and the address is copied
- [x] 4.2 Add the list of what the assistant can and cannot do and the client line; verify with a headless test that the four lines are shown, and with a unit test in `tp-mcp` that the tool list has no save, export or open-project tool

## 5. Tests and checks

- [x] 5.1 Update `tests/game.rs`, `mod_export.rs` and `preferences_tabs.rs` for the new controls and verify with `mise run test`
- [x] 5.2 Verify in German at 100% that no tab is truncated (headless check of the content width and the doc screenshots), and that the existing contrast test passes with the new text styles
- [x] 5.3 Run `mise run ci` through the rust-ui-tester agent and fix its failures; verify it passes

## 6. Documentation

- [x] 6.1 Update `ai-assistant/setup.mdx` (the switch, the list of what the assistant can do) and the Preferences paragraph of `vehicles/official-vehicles.mdx` in English, then translate to fr, es and de with the docs-translator agent; verify with `pnpm build` in `website/`
- [x] 6.2 Render the Preferences screenshots again (`vehicles/game-folders`, `ai-assistant/preferences`, and any other that shows a Preferences tab) with `mise run doc:media`, keep only the files that really changed if the command re-encodes others, and verify the English and German images

## 7. First launch, step 1

- [x] 7.1 Move the game cards into one function used by the Games tab and by the first launch (with the card's words as a parameter), give the first step the group titles and the plain language label, and drop its own rows and the strings they used; verify with `tests/first_run.rs` updated to the new fields and buttons (both games detected, none found, folder of the other game refused, mod folder chosen)
- [x] 7.2 Update `getting-started/first-launch.mdx` in English and translate to fr, es and de, then render the `getting-started/first-launch` screenshot again in four languages and verify English and German; verify with `pnpm build` in `website/`
