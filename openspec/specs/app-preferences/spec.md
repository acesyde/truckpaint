# app-preferences Specification

## Purpose

Remembers the user's interface preferences and workspace layout between sessions, stored in the platform's standard configuration location on Linux, Windows and macOS.

## Requirements

### Requirement: Persisted preferences
The application SHALL persist, in the operating system's standard per-user configuration directory, at least: UI scale, text size, the workspace layout, window size/position/maximized state, the recent projects list, the interface language when the user has chosen one, whether the grid and guides are shown, whether snapping is on, the grid spacing, on Windows and Linux whether the system title bar is used.

The workspace layout SHALL be: the active tab of the left panel (Textures, Layers or Resources), the width of the left panel, the width of the inspector, and whether the panels are hidden (Hide Panels). The active space (Project, Workshop or Brand) SHALL NOT be persisted: opening a project always shows the Project space.

A preferences file written by a build with the 3D preview, which also stores a view mode and the width of the 3D panel, SHALL load with all its other preferences. The view mode and the panel width are ignored.

A preferences file written by a build before the spaces, which stores a stack of panels (their order, visibility and collapsed state), a panel column width, and the Vehicles sidebar's visibility and width, SHALL load with all its other preferences. Those fields are ignored and the workspace layout takes its default values.

#### Scenario: Preferences survive restart
- **WHEN** the user sets the UI scale to 125%, shows the Layers tab, quits and restarts the application, then opens a texture in the Workshop
- **THEN** the UI scale is 125% and the left panel shows the Layers tab

#### Scenario: Panel widths and hidden panels survive restart
- **WHEN** the user widens the inspector, presses Tab to hide the panels, quits and restarts, then opens a texture in the Workshop
- **THEN** the left panel and the inspector are hidden, and pressing Tab shows them with the inspector at the width it had

#### Scenario: Grid and snapping settings survive restart
- **WHEN** the user shows the grid, sets the grid spacing to 128, turns snapping off, quits and restarts
- **THEN** the grid is shown with a 128 px spacing and snapping is off

#### Scenario: Language survives restart
- **WHEN** the user picks "Deutsch" in Preferences, quits and restarts the application
- **THEN** the interface is in German, whatever the system language

#### Scenario: Pack folder survives restart
- **WHEN** the application loads an older preferences file containing template pack folders and a UI scale of 125%
- **THEN** the UI scale remains 125%, obsolete pack fields are ignored, and no invalid-file backup or error is produced

#### Scenario: Active space not persisted
- **WHEN** the user is in the Brand space, quits, restarts and opens the project again
- **THEN** the Project space is shown

#### Scenario: Preferences from a build with the 3D preview
- **WHEN** the application starts with a preferences file that stores the UI scale 125%, the 3D view mode and a 3D panel width
- **THEN** the UI scale is 125%, the canvas is shown, no backup of the file is made, and no message about the preferences appears

#### Scenario: Preferences from a build before the spaces
- **WHEN** the application starts with a preferences file that stores the UI scale 125%, a panel stack with the Assets panel closed, a panel column width and a hidden Vehicles sidebar
- **THEN** the UI scale is 125%, the workspace layout has its default values (Textures tab, default widths, panels shown), no backup of the file is made, and no message about the preferences appears

Old template pack folder fields SHALL be ignored when loading preferences and SHALL no longer be persisted.

### Requirement: Preferences dialog
The application SHALL provide a Preferences dialog (Edit > Preferences, shortcut `⌘,` on macOS and `Ctrl+,` elsewhere) to change UI scale and text size with a live preview, the grid spacing (texture pixels, 4 to 1024) and the interface language, and a "Reset to defaults" action that restores UI scale, text size and grid spacing. The Language setting SHALL list "System default" followed by English, Français, Español and Deutsch, each named in its own language. "System default" follows the operating system's language, and choosing a language applies it immediately. "Reset to defaults" SHALL NOT change the language. On Windows and Linux, the dialog SHALL also offer **Use the system title bar** (see the window-title-bar capability), applied at once; "Reset to defaults" SHALL NOT change it.

The dialog SHALL NOT offer template pack folder settings or Carsmaniac import controls.

View > Reset Workspace SHALL restore the workspace layout to its defaults: the left panel and the inspector shown at their default widths, with the Textures tab active. It SHALL NOT change the active space, the document, the selection or the zoom.

#### Scenario: Reset to defaults
- **WHEN** the user clicks "Reset to defaults" in Preferences
- **THEN** UI scale and text size return to 100% and the grid spacing to 64 px immediately

#### Scenario: Reset workspace layout
- **WHEN** the panels are hidden, the inspector has been widened, the Resources tab is active, and the user chooses View > Reset Workspace
- **THEN** the left panel and the inspector are shown at their default widths and the left panel shows the Textures tab

#### Scenario: Back to the system language
- **WHEN** the language was set to "Español" on a French system and the user picks "System default"
- **THEN** the interface switches to French and no language choice is saved

#### Scenario: Change the pack folder
- **WHEN** Preferences is opened
- **THEN** no template pack folder chooser is offered

#### Scenario: Refused pack folder
- **WHEN** an older preferences file contains a path that is not a template pack
- **THEN** the retired field is ignored without validating it or changing installed vehicles

#### Scenario: Showing the pack folder
- **WHEN** an older preferences file contains an existing template pack path and Preferences is opened
- **THEN** no pack path field or action to open it is shown

#### Scenario: Pack folder gone
- **WHEN** an older preferences file contains a deleted template pack path
- **THEN** Preferences shows no pack-folder warning and other settings remain usable

### Requirement: Robust preferences loading
If the preferences file is missing, unreadable, from an unknown version or invalid, the application SHALL start with default preferences, SHALL NOT crash, and SHALL keep a backup copy of the invalid file. Saving preferences SHALL be atomic so that a crash during write never leaves a truncated file.

#### Scenario: Corrupted preferences file
- **WHEN** the preferences file contains invalid content at startup
- **THEN** the application starts with defaults and the invalid file is preserved next to it with a backup suffix

### Requirement: Window geometry restoration
The application SHALL restore the previous window size, position and maximized state, and SHALL ensure the window is visible on a currently connected display.

#### Scenario: Monitor disconnected
- **WHEN** the window was last placed on a monitor that is no longer connected
- **THEN** the window opens on a connected display, fully visible

### Requirement: AI assistant (MCP) setting
The Preferences dialog SHALL have an **AI assistant (MCP)** section with:
- a switch that turns the local MCP server on or off (see the mcp-server capability), applied at once, off by default;
- the port (1024 to 65535, default 7373), editable while the switch is off;
- the server's URL (`http://127.0.0.1:<port>/mcp`) in a read-only field with a **Copy** button;
- a line saying that an MCP client (such as Claude Desktop or Claude Code) can connect to this URL to draw on the open project, and that every change it makes can be undone;
- the server's state: off, on, or "Port <port> is in use".

The switch and the port SHALL be persisted with the other preferences. A preferences file without them SHALL load with the switch off and the default port. "Reset to defaults" SHALL NOT change them.

#### Scenario: Turning the server on
- **WHEN** the user turns on AI assistant (MCP) in Preferences
- **THEN** the server starts, the section shows it is on, and Copy puts `http://127.0.0.1:7373/mcp` on the clipboard

#### Scenario: Setting survives restart
- **WHEN** the user turns the server on, quits and restarts the application
- **THEN** the server is on again with the same port

#### Scenario: Older preferences file
- **WHEN** the application starts with a preferences file written before this setting existed
- **THEN** all its other preferences load, the server is off, and no message about the preferences appears
