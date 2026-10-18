## 1. Opening Preferences on a tab

- [x] 1.1 Add `PrefsTab { General, Games, Ai }` and `AppState::preferences_tab`, and make `open_preferences(tab)` set it. Callers: `CommandId::Preferences` and the home screen use General, the status bar's MCP item uses Ai, My Game's Preferences… uses Games. Remove `reveal_mcp_preferences`. Verify: `mise run lint` passes, and the `mcp.rs` status bar test asserts the tab is Ai instead of the flag being used up.

## 2. The dialog

- [x] 2.1 Draw a `SegmentedControl` with General, Games and AI assistant under the title, bound to `preferences_tab`. General shows the Interface and Canvas settings; Games shows `games_section`; AI assistant shows `mcp_section` without its reveal. The footer keeps Done, and Reset to defaults on General only. Verify: a kittest opens Edit > Preferences and finds UI scale, Grid spacing and Reset to defaults, then clicks Games and finds I play this game and that Reset to defaults is gone, then AI assistant and finds the address.
- [x] 2.2 Give the settings area the fixed height `PAGE_HEIGHT`, clamped to the window, per tab: content drawn directly, wrapped in a `ScrollArea` once its measured height exceeds the area. Verify:
  - a kittest compares the dialog's rect across the three tabs and finds it equal;
  - at the smallest window, the tabs, Reset to defaults and Done stay visible;
  - in German at the default window, the Games tab doesn't scroll.
- [x] 2.3 A value changed on one tab survives a tab switch, and Reset to defaults still resets UI scale, text size and grid spacing only. Verify: a kittest changes the grid spacing, switches to Games and back, and checks the value; the existing Reset to defaults tests pass unchanged.

## 3. Entry points and existing tests

- [x] 3.1 Status bar MCP item opens the AI assistant tab, and My Game's Preferences… opens the Games tab. Verify: kittests in `mcp.rs` and `game.rs`.
- [x] 3.2 Update the tests that change a Games or AI assistant setting so they select its tab first (`game.rs`, `mcp.rs`, `mod_export.rs`), and the Preferences scenarios in `screenshots.rs`. Verify: `mise run test` passes.

## 4. Strings

- [x] 4.1 Add the three tab labels to `tp-i18n/locales/{en,fr,es,de}/dialogs.ftl` with the same ids: English Title Case ("General", "Games", "AI Assistant"), and the games' own terms in the other languages. Verify: the locale parity test passes, and a German kittest shows the three tabs whole at 440 pt.

## 5. Documentation

- [x] 5.1 Pages that send players to a Preferences section name the tab: `ai-assistant/setup`, `vehicles/official-vehicles`, `workshop/tools`, `getting-started/first-launch`, `getting-started/installation`, in en then fr, es, de. Verify: `mise run doc:build` passes, and no page under `website/src/content/docs/` says "section" for Games or AI assistant in Preferences.
- [x] 5.2 Point the `vehicles/game-folders` and `ai-assistant/preferences` scenarios in `doc_media.rs` at their tabs. Run `mise run doc:media vehicles` and `mise run doc:media ai-assistant`, look at the images, and commit them. Verify: each image shows its tab whole, with no machine-specific path.

## 6. Wrap up

- [x] 6.1 `mise run ci` passes; run docs-drift-checker and act on its report.
