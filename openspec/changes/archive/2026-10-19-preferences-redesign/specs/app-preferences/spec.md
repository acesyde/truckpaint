## MODIFIED Requirements

### Requirement: Preferences dialog
The application SHALL provide a Preferences dialog (Edit > Preferences, shortcut `⌘,` on macOS and `Ctrl+,` elsewhere) to change UI scale and text size with a live preview, the grid spacing (texture pixels, 4 to 1024) and the interface language, and a "Reset to defaults" action that restores UI scale, text size and grid spacing. The Language setting SHALL list "System default" followed by English, Français, Español and Deutsch, each named in its own language. "System default" follows the operating system's language, and choosing a language applies it immediately. "Reset to defaults" SHALL NOT change the language. On Windows and Linux, the dialog SHALL also offer **Use the system title bar** (see the window-title-bar capability), applied at once; "Reset to defaults" SHALL NOT change it.

The dialog SHALL show its settings on three tabs, with the same segmented control as the Vehicle Library's tabs:
- **General:** the Interface settings (language first, then UI scale, text size, and Use the system title bar where offered) and the Canvas setting (grid spacing), under the headings Interface and Canvas, with a rule between the two;
- **Games:** one card per game with its state, its game folder and its mod folder, under no heading of its own (see the requirements Game folders, Mod folders in Preferences and Games tab layout);
- **AI assistant:** the AI assistant (MCP) section (see the requirement AI assistant (MCP) setting).

Edit > Preferences (and Preferences on the home screen) SHALL open the dialog on General. A tab chosen by the player SHALL stay chosen until the dialog is closed. The dialog SHALL keep the same width and height whichever tab is shown; a tab's content SHALL scroll inside the dialog only when the window is too small to show it whole. The footer SHALL hold **Done** and, on the General tab only, **Reset to defaults** as a quiet text button on the left (**Done** on the right), so it stays in reach when the settings scroll. On every tab, text SHALL read in levels: the title of a group of settings (Interface, Canvas, what the assistant can do) small, bold and in capitals in the secondary color; the label of a setting at regular size in the primary color; a hint smaller and dimmer than a label. Assistive technologies SHALL get a group title as written, not in capitals. A setting changed on one tab SHALL keep its value when the player switches tabs.

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

#### Scenario: Titles, labels and hints differ
- **WHEN** the user opens the General tab
- **THEN** INTERFACE and CANVAS are small bold capitals in the secondary color, Language and UI scale are regular size in the primary color, and the note under the sliders is smaller and dimmer, and a screen reader reads the titles as Interface and Canvas

#### Scenario: Language first
- **WHEN** the user opens the General tab
- **THEN** Language is the first setting, above UI scale and text size, and a rule separates Interface from Canvas

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

### Requirement: AI assistant (MCP) setting
The Preferences dialog SHALL have an **AI assistant (MCP)** section with:
- a switch, announced as a switch, with its title and a one-line subtitle, that turns the local MCP server on or off (see the mcp-server capability), applied at once, off by default;
- the server's state as a dot and words (off, on and waiting for a client, a client is connected, or "Port <port> is in use"), never by color alone;
- the server's URL (`http://127.0.0.1:<port>/mcp`) in a read-only field with a **Copy** button inside it, which says **Copied** with a check for a moment after the URL is copied;
- the port (1024 to 65535, default 7373), editable while the switch is off;
- a list of what the assistant can and cannot do, each line with a mark and words: it can read the open project; it can draw on the textures and every change can be undone; it can import an image from a file path it names; it cannot save or export files;
- a line saying that an MCP client (such as Claude Desktop or Claude Code) can connect to this URL.

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

#### Scenario: What the assistant can do
- **WHEN** the user opens the AI assistant tab
- **THEN** a list says the assistant can read the open project, draw on the textures with every change undoable, and import an image from a file path, and cannot save or export files, each line with a mark and words

#### Scenario: Copy says Copied
- **WHEN** the user clicks Copy next to the address
- **THEN** the URL is on the clipboard, the button reads Copied for about a second and a half, then Copy again

#### Scenario: The switch is a switch
- **WHEN** a screen reader reads the control that turns the server on
- **THEN** it is announced as a switch with its title and its on or off state

### Requirement: Game folders
The Games tab of Preferences (completed by the first-run change with the mod folders) SHALL show the installation folder of ETS2 and of ATS, each in the game's card with: the folder (empty by default) in one field with **Choose…** inside it at the right and, once the folder is set, **Clear** inside it; **Detect**, a button at the foot of the card, while the folder is empty; and the game's state beside its name (its version when the folder is valid). A control that cannot be used SHALL NOT be shown disabled. Detection and validity are those of the vehicle-generation capability. A game is played when its installation folder is set: there is no separate setting. A folder SHALL be accepted for a game only when it holds that game: a folder of the other game SHALL be refused with a message that names it. A folder SHALL be checked when it is chosen and when the dialog opens, and an invalid or missing one SHALL show a message next to its field while other settings stay usable. When a first run has no folder set, the app SHALL detect the folders by itself once, without asking. The folders SHALL be persisted with the other preferences; a preferences file without them SHALL load with both empty. "Reset to defaults" SHALL NOT change them.

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
- the installation folder (Choose…, Clear and Detect as the Game folders requirement says);
- the **mod folder**, with the detected folder shown when none was chosen, **Choose…** inside its field at the right, and **Clear** inside the field once one is chosen (Clear goes back to the detected folder). There is no Detect button for the mod folder: the detected folder is already shown and used. Export Mod's save dialog SHALL open in it.

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
The Games tab SHALL show one card per game, a bordered surface. A card has the game's name as its heading and, at the right of it, the game's state as a dot and words: **Version 1.61.1.9** when its folder is valid, **Not set** when it is empty, **Folder problem** when it is set but cannot be used, so the state never rests on color alone. Inside the card, two rows labeled **Game folder** and **Mod folder**, each one field showing a path, with **Choose…** inside it at the right and **Clear** inside it once something is chosen; **Detect** is a button at the foot of the card while the game folder is empty. The tab SHALL NOT repeat its own name as a heading. A one-sentence note that TruckPaint reads these folders for the player's own use and sends nothing anywhere SHALL come first. An empty field SHALL read **Not set** in a color that meets the contrast of secondary text and is visibly not a path. The game folder field SHALL accept a typed path as before; the mod folder field SHALL show its path and change through **Choose…**. Every button and every control inside a field SHALL have an accessible name that includes the game and which folder it is for, such as "Choose…, Euro Truck Simulator 2 game folder", while its visible text stays the same.

#### Scenario: Grouped by game
- **WHEN** the player opens the Games tab
- **THEN** it shows a card for Euro Truck Simulator 2 with its Game folder and Mod folder rows, then a card for American Truck Simulator with its two rows, and no heading that repeats Games

#### Scenario: Empty folder
- **WHEN** no ATS folder is set
- **THEN** its Game folder field reads Not set, in a color distinct from a path and of at least 4.5:1 contrast

#### Scenario: No disabled buttons
- **WHEN** no folder is set for a game, then one is chosen
- **THEN** first the card shows Detect and Choose… with the state Not set, then Choose… and Clear in the field with the state Version, and no control is shown disabled

#### Scenario: State beside the name
- **WHEN** a game's folder is valid, then is deleted from disk and Preferences is reopened
- **THEN** its card first shows Version with its number, then Folder problem, each as a dot and words

#### Scenario: Buttons named for a screen reader
- **WHEN** a screen reader reads the buttons of the Games tab
- **THEN** each says its action and its game and folder, so the four Choose… buttons are told apart

#### Scenario: German
- **WHEN** the language is German and the Games tab is shown at 100%
- **THEN** every label, field and button is shown whole, without truncation, in the dialog's width
