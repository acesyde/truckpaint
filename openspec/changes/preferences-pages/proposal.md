## Why

Preferences has grown into one long scrolling column: Interface, Canvas, Games (two games, each with an installation folder, I play this game and a mod folder) and AI assistant (MCP). It no longer fits a normal window, the AI assistant settings sit below the fold, and the status bar's MCP item needs a scroll trick to reach them (which broke once already). Players looking for one setting have to scan everything.

## What Changes

- **Preferences gets three tabs**, shown with the same segmented control as the Vehicle Library's tabs:
  - **General:** the Interface settings (UI scale, text size, language, Use the system title bar) and the Canvas setting (grid spacing), under their current headings;
  - **Games:** the Games section as it is today (installation folder, I play this game and mod folder for ETS2 and ATS);
  - **AI assistant:** the AI assistant (MCP) section as it is today.
- **Preferences opens on General.** Two entry points open a given tab instead:
  - the status bar's MCP item opens the **AI assistant** tab (this replaces scrolling to the section);
  - **Preferences…** in the My Game tab of the Vehicle Library opens the **Games** tab.
- **The dialog keeps the same size whichever tab is shown**, so it doesn't jump when switching; a tab's content scrolls only when the window is too small for it.
- **Reset to defaults moves onto the General tab**, next to the settings it resets (UI scale, text size, grid spacing). The footer keeps only **Done**. What it resets doesn't change.
- **Docs:** the pages that send players to a Preferences section name the tab instead, and the Preferences screenshots are rendered again.

Non-goals:
- new settings, or changes to what any setting does;
- remembering the last tab between sessions;
- a search field in Preferences.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `app-preferences`: the Preferences dialog is organized in three tabs (General, Games, AI assistant), opens on General, keeps its size across tabs, and has Reset to defaults on the General tab.
- `workspace-layout`: clicking the status bar's MCP item opens Preferences on the AI assistant tab.
- `vehicle-generation`: Preferences… in the My Game tab opens Preferences on the Games tab.

## Impact

- **App:** the Preferences dialog (`crates/tp-app/src/ui/dialogs.rs`), the state that opens it (`open_preferences`, the MCP reveal flag in `state.rs` and the status bar in `ui/workspace/mod.rs`), and the My Game tab's Preferences… button (`ui/vehicle_dialogs.rs`).
- **Strings:** three tab labels in en, fr, es and de; German must fit the dialog's width.
- **Tests:** headless UI tests that change a Games or AI assistant setting must switch tab first (`tests/ui.rs`, `game.rs`, `mcp.rs`, `mod_export.rs`, `persistence.rs`, `localization.rs`, `precision.rs`); the screenshot and doc media scenarios that show Preferences.
- **Docs:** `ai-assistant/setup`, `vehicles/official-vehicles`, `workshop/tools`, `getting-started/first-launch` and `getting-started/installation` in four languages; the `vehicles` and `ai-assistant` media.
- No change to the preferences file.
