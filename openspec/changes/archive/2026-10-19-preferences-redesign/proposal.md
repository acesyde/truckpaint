## Why

A new design for the three Preferences tabs (General, Games, AI Assistant) is clearer and easier to use than today's: the most searched setting comes first, each game has a card with its state, a folder and its **Choose…** share one field instead of two rows, the AI assistant has a real switch and a readable list of what it can and cannot do. It also fixes things the current tabs get wrong: the AI assistant's "switch" is a checkbox, and what the assistant may do is a sentence nobody reads.

## What Changes

- **General:** Language comes first, then UI scale and text size (label and value on one line, the slider below), the note that changes apply at once, and the system title bar option where it is offered. A rule separates **Interface** from **Canvas**, and the section titles become small bold titles. **Reset to defaults** stays on the General tab only, as a quiet text button on the left of the footer, with **Done** on the right.
- **Games:** the note about what TruckPaint reads comes first. Each game is a card: its name, and at the right its state as a dot and words (**Version 1.61.1.9**, **Not set**, or **Folder problem**), so color is never the only cue. A card holds **Game folder** and **Mod folder**, each one field showing the path with **Choose…** inside it at the right and **Clear** inside it once something is chosen. **Detect automatically** is an outlined button with an icon at the foot of the card while the game folder is empty; **Choose…** is a button with its own outline, fill and a folder icon. Typing a game folder path still works.
- **AI Assistant:** a real switch (announced as a switch), with its title and a one-line subtitle, then the server state as a dot and words, the address with **Copy** in the same field, and the port. The sentence about what the assistant can do becomes a list with a mark and words on each line: it can read the open project; it can draw on the textures and every change can be undone; it can import an image from a file path it names; it cannot save or export files.
- **First launch, step 1 (the welcome page):** it uses the same pieces as Preferences so the two look alike: group titles in capitals, a plain label for the language, and the same game cards (name, state as a dot and words, game folder and mod folder fields with their buttons inside, **Detect automatically**). Its states read **Game found** / **Not found** / **Folder problem**, and it still shows no game version.
- **Accessible names stay complete** for every control, so repeated labels such as **Choose…** are told apart by game and folder.

Non-goals:
- the default template opacity setting that appears in the design: it is a new persistent setting and is kept for a later change;
- new wording for Choose…, Clear and Reset to defaults (they keep their words); Detect becomes **Detect automatically**, as in the design;
- middle-ellipsis truncation of long paths, editing the mod folder by typing, other dialogs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `first-run`: the first step shows the same cards, titles and labels as Preferences.
- `app-preferences`: the Preferences dialog's General tab order and footer, the AI assistant (MCP) setting (switch, state, list of what it can do), the Games tab (cards, state, field with its buttons inside) through Game folders, Mod folders in Preferences and Games tab layout.

## Impact

- **App:** the Preferences dialog (`crates/tp-app/src/ui/dialogs.rs`) and a few shared pieces in `tp-ui` (an interactive switch from the existing `paint_switch`, a state dot with text, a field frame that holds inline buttons).
- **Strings:** the AI assistant's subtitle, the capability list, the three card states and a few labels, in en, fr, es and de; German must fit 440 px.
- **Tests:** the headless tests that click the AI checkbox, **Choose…** and **Clear** by their labels (`tests/mcp.rs`, `game.rs`, `mod_export.rs`, `preferences_tabs.rs`).
- **Docs:** `ai-assistant/setup` and `vehicles/official-vehicles` in four languages; the Preferences screenshots (`vehicles/game-folders`, `ai-assistant/preferences`) in four languages.
