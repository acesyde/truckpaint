## Why

The Games tab of Preferences is hard to read and hard to use with a screen reader. The tab repeats its own name as a heading; the two folders of a game and the two games look like four identical blocks; an empty field shows grey hint text in the same style as the labels, so it looks filled; the mod folder is plain text while the game folder is a field; twelve buttons repeat Detect, Choose… and Clear, half of them disabled and hard to see; and every button of the same name is announced the same way, so a screen reader cannot say which game or which folder a Choose… is for.

## What Changes

- **One group per game.** Each game has its name as a heading, with its version beside it when the folder is valid, then two rows: **Game folder** and **Mod folder**. The tab's own "Games" heading goes; the sentence about what TruckPaint reads and that nothing is sent moves to the top of the tab.
- **Both folders look the same:** a path field under a short label. An empty one says **Not set** in a text color that is clearly readable, not like a filled value. The game folder is still editable by typing; the mod folder shows its path as before and changes with Choose….
- **At most two buttons per row, none disabled.** **Choose…** is always there. **Detect** shows only while the game folder is empty; **Clear** shows only once a folder is set. The mod folder has **Choose…** and, once one is chosen, **Clear** (back to the detected folder).
- **The mod folder's Detect button goes.** The detected folder is already shown and used when none is chosen, so pinning it did nothing a player needs.
- **Buttons say what they are for** to a screen reader: for example "Choose…, Euro Truck Simulator 2 game folder". The visible text does not change.
- **Docs:** the Preferences paragraph of the official-vehicles page stops describing three buttons per row, and the Games tab screenshot is rendered again in four languages.

Non-goals:
- editing the mod folder by typing;
- changes to detection, checking of folders, or to the other tabs;
- changes to the first launch step.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `app-preferences`: the Games tab layout and its accessible names (new requirement), and the buttons of the game folder and mod folder rows (Game folders, Mod folders in Preferences, and the Games bullet of Preferences dialog).

## Impact

- **App:** the Games tab (`crates/tp-app/src/ui/dialogs.rs`: `games_section` and `mod_folder_row`).
- **Strings:** new short labels (game folder, mod folder, Not set) and an accessible-name pattern in en, fr, es and de; the Games heading and the placeholder hint are removed if unused elsewhere; German must fit the 440 px dialog.
- **Tests:** the headless tests that click `Choose…`, `Clear` or count them (`tests/game.rs`, `mod_export.rs`, `preferences_tabs.rs`).
- **Docs:** `vehicles/official-vehicles` in four languages; the `vehicles/game-folders` screenshot in four languages.
