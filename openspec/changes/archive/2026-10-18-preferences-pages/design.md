## Context

Preferences is one modal (`Modal::Preferences`) drawn by `preferences()` in `crates/tp-app/src/ui/dialogs.rs`. Its body, `preferences_body()`, lays out four sections in one column: Interface, Canvas, `games_section()` and `mcp_section()`. Whether the body scrolls is decided from its height measured on the previous frame (`PREFERENCES_HEIGHT`, `PREFERENCES_CHROME`, the `preferences_body_height` temp value). Reaching the AI assistant section from the status bar relies on `AppState::reveal_mcp_preferences`, consumed by `mcp_section()` with `scroll_to_me`. A recent fix had to keep that flag alive on the first, unmeasured frame. The Vehicle Library already shows tabs with `tp_ui::widgets::SegmentedControl` and a `LibraryTab` enum kept on its dialog state.

## Goals / Non-Goals

**Goals:**
- One way to open Preferences on a given tab, used by the menu, the home screen, the status bar and My Game.
- A dialog whose size doesn't depend on the tab shown or on the previous frame.
- The section functions (`games_section`, `mcp_section`) stay as they are, apart from losing the reveal.

**Non-Goals:**
- Splitting `dialogs.rs`, or moving the Games or AI assistant code elsewhere.
- Keyboard shortcuts to switch tabs (Tab reaches the segmented control as it does in the Vehicle Library).

## Decisions

**1. Tabs with `SegmentedControl`, not a side list.** With three pages, a side list would take width from the settings: the dialog is 440 pt wide and German labels already fill it. The Vehicle Library uses the segmented control for its tabs, so Preferences looks and behaves the same. *Alternative:* a left list of categories, as in larger settings windows. It is better past six or so pages, which TruckPaint doesn't have.

**2. Canvas goes on General instead of its own tab.** It holds one setting, and a tab with one field reads as empty. The Interface and Canvas headings stay inside General so players still find the grid spacing by its section name.

**3. `PrefsTab` lives on `AppState`, set by `open_preferences(tab)`.** `enum PrefsTab { General, Games, Ai }` replaces `reveal_mcp_preferences`:
- `CommandId::Preferences` and the home screen call `open_preferences(PrefsTab::General)`;
- the status bar calls it with `Ai`;
- My Game's Preferences… calls it with `Games`;
- the dialog's segmented control writes the same field.

It is not saved in the preferences file. `mcp_section()` loses its `reveal` argument and its `scroll_to_me`. *Alternative:* keep the tab in egui memory. That would not let the status bar or My Game choose it before the dialog is drawn.

**4. A fixed settings height, scrolling only when the content is taller.** The settings area is `PAGE_HEIGHT` points tall on every tab (sized so the tallest tab, Games, fits whole at 100% UI scale and text size in German), clamped to the room the window leaves after the title, tabs, footer and margins. The width stays 440 pt. A tab's content is drawn directly and measured; once it is taller than the area, it is wrapped in a `ScrollArea` the next frame (a `ScrollArea` around the settings also made the first click after opening get lost in the headless tests, so it is only added when needed). The measure is per tab, so switching tabs doesn't carry a scroll offset over. The reveal flag and its first-frame special case are gone. *Alternatives:*
- the height of the tallest tab, measured: you only see a tab after drawing it, so the dialog would jump the first time each tab is opened;
- each tab at its own height: the dialog would jump on every switch, which is what the spec rules out.

**5. Reset to defaults stays in the footer, on General only.** It resets only the settings of General, so the other tabs don't show it. In the footer it stays in reach when the settings scroll, which is when it matters most (a UI scale that went too large pushes anything at the bottom of the tab out of view). The label and what it resets don't change.

## Risks / Trade-offs

- [`PAGE_HEIGHT` too short at large UI scales or text sizes] → the tab's content scrolls inside the fixed body, which is the specified behavior. A headless test checks that the Games tab doesn't scroll at the default window size in German.
- [The General tab shows empty space under its settings] → accepted: the dialog stays the same size, and the empty space is below the content, not between controls.
- [Tests that change a Games or AI assistant setting stop finding their widgets] → each such test selects the tab first. The affected files are `game.rs`, `mcp.rs`, `mod_export.rs`, `screenshots.rs` and `doc_media.rs`.
- [Docs and screenshots name the Games and AI assistant "sections"] → they are rewritten to name the tab, and the `vehicles` and `ai-assistant` media are rendered again.
