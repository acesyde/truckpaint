# app-preferences Specification

## Purpose

Remembers the user's interface preferences and workspace layout between sessions, stored in the platform's standard configuration location on Linux, Windows and macOS.

## Requirements

### Requirement: Persisted preferences
The application SHALL persist, in the operating system's standard per-user configuration directory, at least: UI scale, text size, the workspace layout, window size/position/maximized state, the recent projects list, the interface language when the user has chosen one, whether the first launch is done, the installation folder and the mod folder of each game, whether the grid and guides are shown, whether snapping is on, the grid spacing, on Windows and Linux whether the system title bar is used.

The workspace layout SHALL be: the active tab of the left panel (Textures, Layers or Resources), the width of the left panel, the width of the inspector, and whether the panels are hidden (Hide Panels). The active space (Project, Workshop or Brand) SHALL NOT be persisted: opening a project always shows the Project space.

A preferences file written by a build with the 3D preview, which also stores a view mode and the width of the 3D panel, SHALL load with all its other preferences. The view mode and the panel width are ignored.

A preferences file written by a build before the spaces, which stores a stack of panels (their order, visibility and collapsed state), a panel column width, and the Vehicles sidebar's visibility and width, SHALL load with all its other preferences. Those fields are ignored and the workspace layout takes its default values.

A preferences file that stores a list of the games the player plays SHALL load with all its other preferences. The list is ignored: a game is played when its installation folder is set.

#### Scenario: Preferences survive restart
- **WHEN** the user sets the UI scale to 125%, shows the Layers tab, quits and restarts the application, then opens a texture in the Workshop
- **THEN** the UI scale is 125% and the left panel shows the Layers tab

#### Scenario: Panel widths and hidden panels survive restart
- **WHEN** the user widens the inspector, presses Tab to hide the panels, quits and restarts, then opens a texture in the Workshop
- **THEN** the left panel and the inspector are hidden, and pressing Tab shows them with the inspector at the width it had

#### Scenario: Grid and snapping settings survive restart
- **WHEN** the user shows the grid, sets the grid spacing to 128, turns snapping off, quits and restarts
- **THEN** the grid is shown with a 128 px spacing and snapping is off

#### Scenario: First launch survives restart
- **WHEN** the player finishes or skips the first launch, quits and restarts
- **THEN** the first launch is not shown again

#### Scenario: Preferences without the first launch
- **WHEN** the application loads a preferences file with no first-launch field and no recent project
- **THEN** the first launch is shown, no backup of the file is made, and no message about the preferences appears

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

#### Scenario: Older preferences with a games list
- **WHEN** the application starts with a preferences file that lists the games played
- **THEN** all its other preferences load, the list is ignored, and no message about the preferences appears

### Requirement: Preferences dialog
The application SHALL provide a Preferences dialog (Edit > Preferences, shortcut `⌘,` on macOS and `Ctrl+,` elsewhere) to change UI scale and text size with a live preview, the grid spacing (texture pixels, 4 to 1024) and the interface language, and a "Reset to defaults" action that restores UI scale, text size and grid spacing. The Language setting SHALL list "System default" followed by English, Français, Español and Deutsch, each named in its own language. "System default" follows the operating system's language, and choosing a language applies it immediately. "Reset to defaults" SHALL NOT change the language. On Windows and Linux, the dialog SHALL also offer **Use the system title bar** (see the window-title-bar capability), applied at once; "Reset to defaults" SHALL NOT change it.

The dialog SHALL show its settings on three tabs, with the same segmented control as the Vehicle Library's tabs:
- **General:** the Interface settings (UI scale, text size, language, and Use the system title bar where offered) and the Canvas setting (grid spacing), under the headings Interface and Canvas;
- **Games:** one group per game with its game folder and its mod folder, under no heading of its own (see the requirements Game folders, Mod folders in Preferences and Games tab layout);
- **AI assistant:** the AI assistant (MCP) section (see the requirement AI assistant (MCP) setting).

Edit > Preferences (and Preferences on the home screen) SHALL open the dialog on General. A tab chosen by the player SHALL stay chosen until the dialog is closed. The dialog SHALL keep the same width and height whichever tab is shown; a tab's content SHALL scroll inside the dialog only when the window is too small to show it whole. The footer SHALL hold **Done** and, on the General tab only, **Reset to defaults**, so it stays in reach when the settings scroll. A setting changed on one tab SHALL keep its value when the player switches tabs.

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

#### Scenario: Opens on General
- **WHEN** the user chooses Edit > Preferences
- **THEN** the dialog shows the General tab with UI scale, text size, language and grid spacing, and Reset to defaults in its footer

#### Scenario: Switch tabs
- **WHEN** the user clicks the Games tab, then the AI assistant tab
- **THEN** the dialog shows the games' folders, then the AI assistant (MCP) section, and its size doesn't change

#### Scenario: Tab kept while open
- **WHEN** the user changes the grid spacing, opens the Games tab, then goes back to General
- **THEN** the grid spacing shows the new value

#### Scenario: Small window
- **WHEN** the window is at its smallest size and the General tab is shown
- **THEN** the tabs, Reset to defaults and Done stay visible and the General settings scroll between them

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

### Requirement: Game folders
The Games tab of Preferences (completed by the first-run change with the mod folders) SHALL show the installation folder of ETS2 and of ATS, each with: the folder (empty by default), the game's version when the folder is valid, **Choose…**, **Detect** while the folder is empty and **Clear** once it is set; a button that cannot be used SHALL NOT be shown disabled. Detection and validity are those of the vehicle-generation capability. A game is played when its installation folder is set: there is no separate setting. A folder SHALL be accepted for a game only when it holds that game: a folder of the other game SHALL be refused with a message that names it. A folder SHALL be checked when it is chosen and when the dialog opens, and an invalid or missing one SHALL show a message next to its field while other settings stay usable. When a first run has no folder set, the app SHALL detect the folders by itself once, without asking. The folders SHALL be persisted with the other preferences; a preferences file without them SHALL load with both empty. "Reset to defaults" SHALL NOT change them.

#### Scenario: Detect
- **WHEN** the player clicks Detect and ETS2 is in a Steam library
- **THEN** the ETS2 field shows that folder and the game's version

#### Scenario: Refused folder
- **WHEN** the player chooses a folder that is not a game
- **THEN** a message names the reason and the field keeps its previous folder

#### Scenario: Folder of the other game
- **WHEN** the player chooses an ATS installation for ETS2
- **THEN** a message says it is an ATS folder, and the ETS2 field keeps its previous folder

#### Scenario: Folder removed
- **WHEN** the saved ETS2 folder no longer exists and Preferences is opened
- **THEN** the field shows a message that the folder cannot be found, and Clear and Choose… still work

#### Scenario: Setting survives restart
- **WHEN** the player sets the ETS2 folder, quits and restarts
- **THEN** the same folder is set

#### Scenario: Older preferences file
- **WHEN** the application starts with a preferences file written before this setting existed
- **THEN** all its other preferences load, both folders are empty, and no message about the preferences appears

### Requirement: Mod folders in Preferences
Every setting made in the first launch SHALL stay changeable afterwards in Preferences. The Games tab of Preferences (whose installation folders come from the in-app-vehicle-generation change) SHALL show, for ETS2 and for ATS, and SHALL NOT have an "I play this game" setting:
- the installation folder (Choose…, and Detect or Clear as the Game folders requirement says);
- the **mod folder**, with the detected folder shown when none was chosen, **Choose…**, and **Clear** once one is chosen (Clear goes back to the detected folder). There is no Detect button for the mod folder: the detected folder is already shown and used. Export Mod's save dialog SHALL open in it.

New Project's initial game SHALL be the first game whose folder is set (ETS2 then ATS) that has installed vehicles, else the game of the installed vehicles when they are all of one game, else the only game whose folder is set, else ETS2. A change SHALL be saved like the other preferences. "Reset to defaults" SHALL NOT change this section.

#### Scenario: First launch choices shown in Preferences
- **WHEN** the player chose ETS2, an installation folder and a mod folder in the first launch, then opens Preferences
- **THEN** the Games tab shows the same ETS2 installation folder and the same mod folder

#### Scenario: Fix a wrong mod folder
- **WHEN** the player chooses another mod folder for ETS2 in Preferences and exports a mod
- **THEN** Export Mod's save dialog opens in the new folder

#### Scenario: Start playing ATS
- **WHEN** the player who set only the ETS2 folder clicks Detect for ATS and ATS is found
- **THEN** its installation folder is shown, and New Project still starts on ETS2

#### Scenario: Stop playing a game
- **WHEN** the player clicks Clear on the ATS installation folder
- **THEN** ATS is no longer played: the first launch and New Project no longer count it

#### Scenario: No setting to check
- **WHEN** the player opens the Games tab
- **THEN** it has no "I play this game" checkbox for either game

### Requirement: Games tab layout
The Games tab SHALL group its settings by game: each game has a heading with its name, and the game's version beside it when its folder is valid, then two rows labeled **Game folder** and **Mod folder**. The tab SHALL NOT repeat its own name as a heading. A one-sentence note that TruckPaint reads these folders for the player's own use and sends nothing anywhere SHALL come first. Both rows SHALL show a path in a field of the same look under their label; an empty one SHALL read **Not set** in a color that meets the contrast of secondary text and is visibly not a path. The game folder field SHALL accept a typed path as before; the mod folder field SHALL show its path and change through **Choose…**. Every button SHALL have an accessible name that includes the game and which folder it is for, such as "Choose…, Euro Truck Simulator 2 game folder", while its visible text stays the same.

#### Scenario: Grouped by game
- **WHEN** the player opens the Games tab
- **THEN** it shows Euro Truck Simulator 2 with its Game folder and Mod folder rows, then American Truck Simulator with its two rows, and no heading that repeats Games

#### Scenario: Empty folder
- **WHEN** no ATS folder is set
- **THEN** its Game folder field reads Not set, in a color distinct from a path and of at least 4.5:1 contrast

#### Scenario: No disabled buttons
- **WHEN** no folder is set for a game, then one is chosen
- **THEN** first the row shows Detect and Choose…, then Choose… and Clear, and no button is shown disabled

#### Scenario: Buttons named for a screen reader
- **WHEN** a screen reader reads the buttons of the Games tab
- **THEN** each says its action and its game and folder, so the four Choose… buttons are told apart

#### Scenario: German
- **WHEN** the language is German and the Games tab is shown at 100%
- **THEN** every label, field and button is shown whole, without truncation, in the dialog's width
