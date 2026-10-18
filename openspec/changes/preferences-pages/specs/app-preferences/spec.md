## MODIFIED Requirements

### Requirement: Preferences dialog
The application SHALL provide a Preferences dialog (Edit > Preferences, shortcut `⌘,` on macOS and `Ctrl+,` elsewhere) to change UI scale and text size with a live preview, the grid spacing (texture pixels, 4 to 1024) and the interface language, and a "Reset to defaults" action that restores UI scale, text size and grid spacing. The Language setting SHALL list "System default" followed by English, Français, Español and Deutsch, each named in its own language. "System default" follows the operating system's language, and choosing a language applies it immediately. "Reset to defaults" SHALL NOT change the language. On Windows and Linux, the dialog SHALL also offer **Use the system title bar** (see the window-title-bar capability), applied at once; "Reset to defaults" SHALL NOT change it.

The dialog SHALL show its settings on three tabs, with the same segmented control as the Vehicle Library's tabs:
- **General:** the Interface settings (UI scale, text size, language, and Use the system title bar where offered) and the Canvas setting (grid spacing), under the headings Interface and Canvas, followed by "Reset to defaults";
- **Games:** the Games section (see the requirements Game folders and Games played and mod folders in Preferences);
- **AI assistant:** the AI assistant (MCP) section (see the requirement AI assistant (MCP) setting).

Edit > Preferences (and Preferences on the home screen) SHALL open the dialog on General. A tab chosen by the player SHALL stay chosen until the dialog is closed. The dialog SHALL keep the same width and height whichever tab is shown; a tab's content SHALL scroll inside the dialog only when the window is too small to show it whole. The footer SHALL hold only **Done**. A setting changed on one tab SHALL keep its value when the player switches tabs.

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
- **THEN** the dialog shows the General tab with UI scale, text size, language and grid spacing, and Reset to defaults

#### Scenario: Switch tabs
- **WHEN** the user clicks the Games tab, then the AI assistant tab
- **THEN** the dialog shows the Games section, then the AI assistant (MCP) section, and its size doesn't change

#### Scenario: Tab kept while open
- **WHEN** the user changes the grid spacing, opens the Games tab, then goes back to General
- **THEN** the grid spacing shows the new value

#### Scenario: Small window
- **WHEN** the window is at its smallest size and the General tab is shown
- **THEN** the tabs and Done stay visible and the General settings scroll between them
