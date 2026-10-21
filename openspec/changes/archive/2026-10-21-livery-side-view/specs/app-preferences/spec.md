## MODIFIED Requirements

### Requirement: Persisted preferences
The application SHALL persist, in the operating system's standard per-user configuration directory, at least: UI scale, text size, the workspace layout, window size/position/maximized state, the recent projects list, the interface language when the user has chosen one, whether the first launch is done, the installation folder and the mod folder of each game, whether the grid and guides are shown, whether snapping is on, the grid spacing, on Windows and Linux whether the system title bar is used.

The workspace layout SHALL be: the active tab of the left panel (Textures, Layers or Resources), the width of the left panel, the width of the inspector, whether the panels are hidden (Hide Panels), and the Side View panel's visibility, direction and height (see the side-view capability). The active space (Project, Workshop or Brand) SHALL NOT be persisted: opening a project always shows the Project space.

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

#### Scenario: Side View survives restart
- **WHEN** the user shows the Side View, picks the Front direction, makes the panel taller, quits and restarts, then opens a texture in the Workshop
- **THEN** the Side View is shown, from the front, at the height it had

#### Scenario: Preferences without the Side View
- **WHEN** the application loads a preferences file written before the Side View, with the UI scale 125%
- **THEN** the UI scale is 125%, the Side View is hidden with its default direction (Left) and height, and no message about the preferences appears
