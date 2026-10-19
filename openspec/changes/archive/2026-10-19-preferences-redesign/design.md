## Context

The Preferences dialog is built by hand in `ui/dialogs.rs`: `general_tab`, `games_section` (with `mod_folder_row` and `folder_button`) and `mcp_section`, in a 440 px modal with a segmented control and a footer. Shared pieces already exist in `tp-ui`: `SegmentedControl`, `ThinSlider`, `secondary_button`, `SquareCheckbox`, and `paint_switch` with `SWITCH_SIZE` (used as a switch by the status bar), plus `labelled`, `committed_text`, `problem` and `section` in `dialogs.rs`. The colors come from the design tokens (`TEXT_SECONDARY` is held at 4.5:1 by the contrast test). The design file draws the same three tabs with cards, a state per game and a list of what the assistant can do. See proposal.md for motivation and scope.

## Goals / Non-Goals

**Goals:**
- Match the design's structure and hierarchy on the three tabs, in the app's own tokens and widgets.
- Keep every control reachable by keyboard and named for a screen reader, and never carry meaning by color alone.
- Keep the wording of Detect, Choose…, Clear and Reset to defaults, so the docs ids and tests that name them keep working.

**Non-Goals:**
- The default template opacity setting (kept for a later change).
- Pixel-matching the mock: its dim text `#66666c` is about 3.2:1 on the dialog, below the 4.5:1 the design system requires; the app's `TEXT_SECONDARY` is used instead.

## Decisions

**A field that holds its buttons: one `Frame`, widgets drawn inline.** egui's `TextEdit` has no trailing widget, so a "field with Choose… inside" is a `Frame` with the field fill and stroke whose content is a horizontal row: the path (a borderless `TextEdit` for the game folder, so typing still works; a truncated label for the read-only mod folder and the address), then a ghost **Choose…** and, when set, a small **Clear** icon button. Each inline button gets its own accessible name through `widget_info`, built with the existing `prefs-button-for` pattern. The frame keeps one stroke width at all times and the focus ring is painted over it, outside the frame, so focusing a field moves nothing (a test compares the rectangles before and after). Inside the field, **Choose…** (and Copy) is a real button: its own fill and outline, an icon and the words; Clear stays a plain cross. **Detect automatically** is an outlined button with a search icon. Both keep the theme's hover and pressed fills and outlines (only the idle outline is made stronger, in a local style scope), and show a pointing hand. After Copy, the address's button reads **Copied** with a check for 1.5 s: the time of the click is kept in egui's temporary data and the frame is repainted when it runs out. The widgets are made in reading order (path, Clear, action), so Tab follows what is seen; the path gets the width the buttons left, measured the frame before. Alternative: keep the buttons under the field — it is what ships today and is what the design removes (half the height).

**State as a dot plus words, one small helper.** `state_chip(ui, kind, text)` draws a filled dot in the state's color (success, muted, warning) and the text in `TEXT_SECONDARY`; the three states have different words and the problem state uses the warning icon shape rather than a dot, so a reader without color still gets the state. The words reuse `prefs-game-version` ("Version { $version }") for a valid folder and add `prefs-game-state-unset` ("Not set") and `prefs-game-state-problem` ("Folder problem"). The longer message about the problem stays under the card, in `problem()`, as today.

**Cards are a `Frame` with the raised surface.** One per game, `radius::MD`, border `BORDER`; **Detect** sits at the foot of the card while the game folder is empty, as a secondary button. Not a new `tp-ui` widget: only the Games tab uses it.

**An interactive switch from `paint_switch`.** A helper `switch(ui, &mut bool, title, subtitle)` allocates a click-and-focus response of `SWITCH_SIZE`, paints it with `paint_switch`, and reports on/off to the accessibility tree. The role is set to switch with `ctx.accesskit_node_builder(id, |b| b.set_role(Role::Switch))` and its toggled state; if that proves unreliable under `egui_kittest`, the fallback is egui's checkbox widget type with the toggled state and the switch painting kept, and the spec scenario is checked against what the tree reports. The title and subtitle are two labels beside it; the title's accessible name is the control's name. `SquareCheckbox` for "Use the system title bar" is unchanged.

**The capability list is text with marks, from three Fluent messages plus a header.** A check mark icon for what it can do and a cross for what it cannot, each line with its words (`prefs-mcp-can-read`, `prefs-mcp-can-draw`, `prefs-mcp-can-import`, `prefs-mcp-cannot-save`, header `prefs-mcp-can-title`). The wording follows the `mcp-server` capability, not the mock: the assistant's `import_image` tool reads an image file by absolute path, so "cannot open files" would be false; "every change can be undone" and "no file is written" are in that spec. The undo shortcut is not typed in the text (it differs between macOS and the other systems).

**General tab: reorder, no new setting.** Language first, then the two scale rows (a header with the label and the editable value, the thin slider below), the existing note, the title bar option, a rule, then Canvas with the grid spacing. Text reads in three levels (`section_title`, setting labels, hints): a group title is small, bold, in capitals and the secondary color (11 pt, `label_strong_style`; egui's `strong()` only changes the color, so the bold comes from the text style, and it cannot space letters), a label is regular size in the primary color, a hint is small and in the muted color. The capitals are drawn, while the label's accessible name stays as written. Language is a plain label above its drop-down, not the small secondary field label, so it is not mistaken for a title. The footer puts **Reset to defaults** as a ghost (text) button on the left and **Done** on the right.

**Layout width.** Everything stays inside the 440 px dialog; the Games tab is still the tallest and `PREFERENCES_PAGE` is re-measured with the cards (the dialog scrolls inside when a window is small, as today). German is checked by a headless test of the tab and by the doc screenshot.

## Risks / Trade-offs

- [A `Frame` with inline widgets is a custom control] → Keep it to one helper used by the three path rows, with its own tests: click Choose… inside, type in the game folder, Clear inside, keyboard Tab order path → Choose… → Clear.
- [The switch role may not map as expected in the accessibility tree] → The fallback above, verified by a test that finds the control by its title and reads its toggled state.
- [The capability list becomes stale if the MCP tools change] → It is checked against the tool list in `tp-mcp` by a unit test that fails when a save or export tool appears.
- [Moving Language first may surprise players used to the old order] → It is the most searched setting; nothing else about it changes.
