## MODIFIED Requirements

### Requirement: Persisted preferences
The application SHALL persist, in the operating system's standard per-user configuration directory, at least: UI scale, text size, the workspace layout, window size/position/maximized state, the recent projects list, the interface language when the user has chosen one, whether the first launch is done, the games the player chose at first launch, the mod folder chosen for each game, whether the grid and guides are shown, whether snapping is on, the grid spacing, on Windows and Linux whether the system title bar is used.

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

## ADDED Requirements

### Requirement: Games played and mod folders in Preferences
Every setting made in the first launch SHALL stay changeable afterwards in Preferences. The **Games** section of Preferences (whose installation folders come from the in-app-vehicle-generation change) SHALL show, for ETS2 and for ATS:
- **I play this game**, the games chosen in the first launch; at least one SHALL stay checked. The first game checked is New Project's initial game;
- the installation folder (Detect, Choose…, Clear);
- the **mod folder**, with the detected folder when none was chosen, **Detect**, **Choose…** and **Clear**. Export Mod's save dialog SHALL open in it.

A change SHALL be saved like the other preferences. "Reset to defaults" SHALL NOT change this section.

#### Scenario: First launch choices shown in Preferences
- **WHEN** the player chose ETS2, an installation folder and a mod folder in the first launch, then opens Preferences
- **THEN** the Games section shows ETS2 checked, the same installation folder and the same mod folder

#### Scenario: Fix a wrong mod folder
- **WHEN** the player chooses another mod folder for ETS2 in Preferences and exports a mod
- **THEN** Export Mod's save dialog opens in the new folder

#### Scenario: Start playing ATS
- **WHEN** the player who chose only ETS2 checks I play this game for ATS
- **THEN** its installation and mod folders are detected and shown, and New Project still starts on ETS2

#### Scenario: Last game unchecked
- **WHEN** only ETS2 is checked and the player unchecks it
- **THEN** it stays checked and a hint says at least one game is needed

