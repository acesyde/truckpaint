## ADDED Requirements

### Requirement: Mod folders in Preferences
Every setting made in the first launch SHALL stay changeable afterwards in Preferences. The **Games** section of Preferences (whose installation folders come from the in-app-vehicle-generation change) SHALL show, for ETS2 and for ATS, and SHALL NOT have an "I play this game" setting:
- the installation folder (Detect, Choose…, Clear);
- the **mod folder**, with the detected folder when none was chosen, **Detect**, **Choose…** and **Clear**. Export Mod's save dialog SHALL open in it.

New Project's initial game SHALL be the first game whose folder is set (ETS2 then ATS) that has installed vehicles, else the game of the installed vehicles when they are all of one game, else the only game whose folder is set, else ETS2. A change SHALL be saved like the other preferences. "Reset to defaults" SHALL NOT change this section.

#### Scenario: First launch choices shown in Preferences
- **WHEN** the player chose ETS2, an installation folder and a mod folder in the first launch, then opens Preferences
- **THEN** the Games section shows the same ETS2 installation folder and the same mod folder

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

## REMOVED Requirements

### Requirement: Games played and mod folders in Preferences
**Reason**: "I play this game" is gone: a game is played when its installation folder is set.
**Migration**: See "Mod folders in Preferences" and "Game folders".

## MODIFIED Requirements

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

### Requirement: Game folders
Preferences SHALL have a **Games** section (completed by the first-run change with the mod folders) with the installation folder of ETS2 and of ATS, each with: the folder (empty by default), the game's version when the folder is valid, **Detect**, **Choose…** and **Clear**. Detection and validity are those of the vehicle-generation capability. A game is played when its installation folder is set: there is no separate setting. A folder SHALL be accepted for a game only when it holds that game: a folder of the other game SHALL be refused with a message that names it. A folder SHALL be checked when it is chosen and when the dialog opens, and an invalid or missing one SHALL show a message next to its field while other settings stay usable. When a first run has no folder set, the app SHALL detect the folders by itself once, without asking. The folders SHALL be persisted with the other preferences; a preferences file without them SHALL load with both empty. "Reset to defaults" SHALL NOT change them.

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
