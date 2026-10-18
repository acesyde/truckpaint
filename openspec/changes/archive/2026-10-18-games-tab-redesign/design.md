## Context

`games_section` (`ui/dialogs.rs`) loops over the two games and draws, for each, a `labelled` game-name label, a `committed_text` field (placeholder `prefs-game-folder-hint`), a row of three `secondary_button`s (Detect, Choose…, Clear, Clear disabled when no folder), the folder's problem or version line, then `mod_folder_row`, which draws a muted label, a plain text path or "Not found", and the same three buttons (Detect disabled unless a folder is detected, Clear disabled unless one is chosen). A section heading `prefs-games` and a closing hint `prefs-games-hint` frame it. Buttons carry only their visible text as accessible name. See proposal.md for motivation; the checks of a folder (`game::set_folder`, `check`) are not touched.

## Goals / Non-Goals

**Goals:**
- A tab a player scans by game, with the same look for both folders and no disabled button.
- Accessible names that tell the four Choose… (and the Detect and Clear) apart.

**Non-Goals:**
- Typing a mod folder path, a new component in `tp-ui`, or changes to other tabs or the first launch.

## Decisions

**Per-game group, built in `dialogs.rs` with the existing helpers.** For each game: a strong heading (`section`) with the version in secondary text beside it when `FolderStatus::Valid`, then two `labelled` rows. Labels are short (`prefs-game-folder-label` "Game folder", `prefs-mod-folder-label` "Mod folder") because the game's name is already the heading; the old "{ game } mod folder" repeated it. The `prefs-games` heading goes; `prefs-games-hint` moves above the groups. Alternative: a framed card per game — a new container style for little gain; whitespace and the heading carry the grouping in the other tabs too.

**One field look for both rows.** The game folder keeps `committed_text` (typing, Enter commits, empty clears). The mod folder uses a non-editable `TextEdit` with the same margin and height, so it reads as a path field without offering typing; a typed mod path would need its own validation, which is out of scope. Alternative: make the mod folder editable too — rejected for that reason.

**Empty reads "Not set".** The hint text of both fields becomes `prefs-folder-not-set`. egui draws a hint in the theme's weak text color, which this app already sets to `TEXT_SECONDARY` (the 4.5:1 token), so the colour stays; what changes is the wording, which no longer reads like an instruction or a label beside a labelled field. A path is drawn in `TEXT_PRIMARY`, so an empty field is visibly not a path. The mod folder shows the detected folder in `TEXT_SECONDARY` when none is chosen, and "Not set" when none is detected (this replaces `prefs-mod-none`, "Not found").

**Contextual buttons, not disabled ones.** Game folder: empty gives `[Detect] [Choose…]`; set gives `[Choose…] [Clear]`. Mod folder: `[Choose…]`, plus `[Clear]` once one is chosen. A button that cannot act is not drawn, which removes the disabled-contrast problem on this tab without touching the design system's disabled style (still held at 3:1 by its own test). Clear stays a button, not a cross drawn in the field: egui's `TextEdit` has no trailing widget, and an overlay would need hand-made hit testing and its own accessible name. The mod folder's Detect goes, because pinning the detected folder only copied what Export Mod already uses.

**Accessible names from one pattern.** A message `prefs-button-for = { $action }, { $target }` builds the name from the visible action and a target label (`prefs-game-folder-target = { $game } game folder`, and the existing `prefs-mod-folder` for the mod folder). It is applied with `response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name))` on each button, as `more_menu` and the MCP URL field already do. The visible label is unchanged, so the first-launch and docs wording stay the same. Alternative: `on_hover_text` only — it does not reach the accessibility tree.

**Layout width.** Field full width, buttons under it left-aligned (as today): German `Auswählen…`/`Erkennen` plus two buttons fit in 440 px, which a trailing in-line button next to a path field would not.

## Risks / Trade-offs

- [Buttons appearing and disappearing move the layout] → The button row keeps its height; only its contents change, and the problem line below stays where it is.
- [Removing the mod folder's Detect changes a documented control] → The docs paragraph is updated in four languages; the detected folder is still shown and used.
- [A read-only field next to an editable one] → It is announced as read-only text by the accessibility tree and is not given a text cursor; the label and the Choose… button make the way to change it clear.
- [The tab's height changes] → It scrolls inside the fixed dialog height already (`PREFERENCES_PAGE`); a headless test checks the German tab is whole at 100%.
