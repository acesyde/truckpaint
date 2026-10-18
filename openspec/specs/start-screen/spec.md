# start-screen Specification

## Purpose

Gives the user an Adobe-style home screen to start a new livery project, open an existing one or resume recent work, as the entry point of the application.

## Requirements

### Requirement: Home screen on launch
The first time the application starts, it SHALL show the first launch instead of the home screen (see the first-run capability). Otherwise, when no project is open, the application SHALL show a home screen with:
- the application name and tagline;
- **New Project** as the primary button and **Open…** (Open Project) as a secondary button, each with its shortcut in its tooltip;
- the recovered projects, when there are any (see Recovered projects on the home screen);
- the **Recent projects** area (see Recent projects list);
- an entry to Preferences.

The buttons SHALL have free widths that fit their translated labels.

#### Scenario: First launch
- **WHEN** the application starts for the first time
- **THEN** the first launch is shown instead of the home screen

#### Scenario: Launch after the first launch
- **WHEN** the application starts and the first launch was finished or skipped
- **THEN** the home screen is shown with New Project as the primary button, Open… as a secondary button, and a Recent projects area

#### Scenario: Returning to home
- **WHEN** the user closes the last open project
- **THEN** the home screen is shown again

#### Scenario: German labels
- **WHEN** the interface language is German
- **THEN** the New Project and Open… buttons show their whole German labels, without truncation

### Requirement: Recent projects list
The Recent projects area SHALL list previously opened projects, most recent first, as **cards**. Each card SHALL show a thumbnail of the project, its name, its file location and its last-opened date. The thumbnail is the artwork of the project's first main texture drawn over its template; while it is being read, and when the file can't be read, the card SHALL show a placeholder icon instead.

Clicking an available card SHALL open that project. When the list is empty, it SHALL show an explanatory empty state instead of a blank area: centered on the whole width of the list (from the left of the Recent projects heading to the right of Preferences), in a frame standing where the cards will be, with a **Create Your First Project** button that does what New Project does. Entries whose file no longer exists SHALL be shown as unavailable, SHALL NOT open, and SHALL be removable from the list.

#### Scenario: Empty recent list
- **WHEN** no project has ever been opened
- **THEN** the Recent projects area shows an empty state message inviting the user to create a new project

#### Scenario: Missing recent file
- **WHEN** a recent project's file has been deleted or moved
- **THEN** its card is displayed as unavailable with a non-color indicator, shows the placeholder instead of a thumbnail, and offers a "Remove from list" action

#### Scenario: Open a recent project
- **WHEN** the user clicks an available recent card
- **THEN** the editor opens that project

#### Scenario: Thumbnail of a recent project
- **WHEN** a recent project's first main texture is covered by a red rectangle
- **THEN** its card's thumbnail is red

#### Scenario: Empty list centered with a create button
- **WHEN** the home screen is shown with no recent project
- **THEN** the empty state is centered between the left edge of the Recent projects heading and the right edge of Preferences, and clicking Create Your First Project opens New Project

### Requirement: New Project dialog
Choosing New Project SHALL open a modal dialog on one page:

- **On top:** the **Project name** field and the **Game**, ETS2 or ATS, as a segmented control. The name defaults to the first checked vehicle's name when left empty. The game starts on the game chosen at first launch when there is one and its vehicles are installed, else on the game of the installed vehicles, or ETS2 when vehicles of both games or none are installed. Changing the game unchecks the vehicles of the other game. A note says that vehicles can be added to the project at any time.
- **On the left, the vehicle list**, each row with a checkbox, under the column headers Vehicle, Type and Package:
  - Only installed vehicles of the chosen game SHALL be listed, one row per vehicle with its name, kind, brand and newest installed version; no uninstalled catalog entries or automatic template pack installation SHALL be offered.
  - The list is searchable by name and brand, and filterable by kind with a segmented control **All / Trucks / Trailers**. Checked vehicles stay checked when search or filtering hides them.
  - Under the list, **Custom Vehicle…** explains creation from DDS, PNG or SVG templates and game data, and opens the Custom Vehicle dialog. **Install…** installs validated packages manually.
  - With no installed vehicle for the chosen game, the list offers Install…, Install the sample vehicles and Custom Vehicle….
  - Clicking a row or pressing Space checks or unchecks an installed vehicle. Several trucks and trailers can be checked together.
- **On the right, "Your fleet",** with the number of checked vehicles ("3 vehicles"): one card per checked vehicle, in the order they were checked, each with the vehicle's name and **Remove**, which unchecks it. One card is open at a time: the vehicle checked last, or the card the user clicks. The open card shows:
  - under **Cabins**, one chip per main texture, named after it, that toggles it (filled when it is checked, outlined otherwise; the cabins it covers in its tooltip). The first one is checked when the vehicle is checked; the last checked main texture (and so a single main texture) can't be unchecked;
  - the **main texture mode**, read only, from the package: "One per cabin layout" when the package has several main textures, "One for every cabin" for a truck with a single main texture, "Single main texture" for a trailer. The user can't change it;
  - under **Accessories**, one checkbox per accessory, all checked when the vehicle is checked;
  - the count of the textures that will be created for it ("2 main + 3 accessories").

  A folded card shows, on one line, the names of its checked main textures and its count. Under the cards, **Textures created** lists the textures that will be created for every vehicle, each with its size, and their total. With no vehicle checked, Your fleet SHALL say to check vehicles in the list.
- **Create Project**, the primary action, SHALL be enabled only when at least one vehicle is checked (each keeps at least one main texture); its tooltip says why when it is disabled. **Cancel** sits beside it.

A vehicle created with Custom Vehicle… SHALL be checked when the dialog is shown again, with its textures checked as for any checked vehicle; the vehicles checked before stay checked. The game, search and kind filter SHALL be changed if needed so it is listed; when the game changes, the vehicles of the other game are unchecked. Cancelling the Custom Vehicle dialog SHALL show the New Project dialog as it was.

Confirming SHALL open the editor workspace, in the Project space, with a new project for the checked vehicles, in fleet order, and their chosen textures (see the vehicle-projects capability); the first texture of the first vehicle is active. Cancelling SHALL return to the home screen without side effects. The dialog SHALL be fully operable by keyboard (Tab to move, Space to toggle a checkbox or a chip or check a vehicle, Enter to create when Create Project is enabled, Escape to cancel).

#### Scenario: Creating a project
- **WHEN** the user opens New Project, picks the sample truck, keeps "Standard cab" and the accessories checked, and clicks Create Project
- **THEN** the editor workspace opens in the Project space with a project named "TruckPaint Sample Truck" on its Standard cab and accessory textures, and the window title shows the project name

#### Scenario: Cancelling
- **WHEN** the user presses Escape in the New Project dialog
- **THEN** the dialog closes and the home screen is unchanged

#### Scenario: Game chosen at first launch
- **WHEN** the player chose ATS at first launch and vehicles of both games are installed
- **THEN** New Project opens with the Game set to ATS

#### Scenario: Empty name
- **WHEN** the user leaves the name empty and confirms
- **THEN** the project is created with the vehicle's name

#### Scenario: Creating a project for a vehicle
- **WHEN** the user opens New Project, picks "Volvo FH16 2012", checks its "Globetrotter XL" main texture, unchecks every accessory, and clicks Create Project
- **THEN** the workspace opens with a project named "Volvo FH16 2012" with one surface, the Globetrotter XL main texture

#### Scenario: Creating a trailer project
- **WHEN** the user picks the sample trailer
- **THEN** Your fleet shows Base checked and impossible to uncheck, the main texture mode "Single main texture", the accessories checked, and Create Project is enabled

#### Scenario: The last main texture stays checked
- **WHEN** the user picks the sample truck, checks "High roof" and unchecks "Standard cab"
- **THEN** "High roof" can't be unchecked, and Create Project stays enabled

#### Scenario: Main texture mode is read only
- **WHEN** the user picks the sample truck, whose package has two main textures
- **THEN** Your fleet shows the main texture mode "One per cabin layout" as text, with no control to change it

#### Scenario: Textures that will be created
- **WHEN** the user picks the sample truck 1.1.0 and keeps Standard cab and every accessory checked
- **THEN** Your fleet shows "1 main + 3 accessories" and lists Standard cab (4096), Chassis (4096), Cab accessories (1024) and Side skirts (1024), 4 textures in total

#### Scenario: Only the chosen game is listed
- **WHEN** ETS2 and ATS vehicles are installed and the user sets the Game to ATS
- **THEN** the vehicle list shows only the ATS vehicles

#### Scenario: No vehicle installed
- **WHEN** the user opens New Project with no vehicle installed and the Game set to ATS
- **THEN** Create Project is disabled, and Install…, Install the sample vehicles and Custom Vehicle… are offered

#### Scenario: Project from a custom vehicle
- **WHEN** the user, with no vehicle installed, clicks Custom Vehicle…, creates a truck from one template file, then clicks Create Project
- **THEN** the workspace opens with a project for that custom vehicle, with one surface showing the template

#### Scenario: Created vehicle hidden by a filter
- **WHEN** the New Project dialog's Game is ATS and the user creates an ETS2 custom vehicle
- **THEN** the dialog shows the Game ETS2 and the new vehicle chosen, with its textures checked

#### Scenario: Official vehicles first
- **WHEN** only the sample truck is installed and the user opens New Project with Game ETS2
- **THEN** only the sample truck is listed; uninstalled DAF and Krone vehicles are not offered

#### Scenario: Project from an official vehicle
- **WHEN** the user manually installs an extracted DAF NGD package, opens New Project, selects XF as its main texture and its accessories, and clicks Create Project
- **THEN** a DAF NGD project opens with the chosen textures and no template pack prompt

#### Scenario: A truck and a trailer at once
- **WHEN** the user checks the sample truck, then the sample trailer, and clicks Create Project with an empty name
- **THEN** the project, named "TruckPaint Sample Truck", holds both vehicles, the truck's textures then the trailer's, with the truck's Standard cab active

#### Scenario: One card open at a time
- **WHEN** the user checks the sample truck, then the sample trailer
- **THEN** Your fleet shows "2 vehicles", the trailer's card open with Base and its accessories, and the truck's card folded to "Standard cab · 1 main + 3 accessories"; clicking the truck's card opens it and folds the trailer's

#### Scenario: Remove from the fleet
- **WHEN** two vehicles are checked and the user clicks Remove on the first card
- **THEN** that vehicle is unchecked in the list, Your fleet shows "1 vehicle", and Textures created lists only the other vehicle's textures

#### Scenario: Unchecking in the list
- **WHEN** the sample truck is the only checked vehicle and the user clicks its row again
- **THEN** it is unchecked, Your fleet says to check vehicles in the list, and Create Project is disabled

#### Scenario: Changing the game
- **WHEN** an ETS2 truck and an ETS2 trailer are checked and the user sets the Game to ATS
- **THEN** no vehicle is checked

#### Scenario: Custom vehicle added to the fleet
- **WHEN** the sample truck is checked and the user creates a custom trailer with Custom Vehicle…
- **THEN** the sample truck and the custom trailer are both checked, with the trailer's card open

#### Scenario: Official vehicle filtered by kind
- **WHEN** an extracted official truck and a manual trailer package are installed and the kind filter is set to Trailers
- **THEN** only the installed trailer is shown, with no uninstalled official entries

### Requirement: Open Project action
The home screen's Open Project action SHALL open the same file dialog as File › Open… and open the chosen project.

#### Scenario: Open from the home screen
- **WHEN** the user clicks Open Project and selects a `.truckpaint` file
- **THEN** the editor opens that project

### Requirement: Recovered projects on the home screen
When recovery copies exist, the home screen SHALL show them above the recent projects, as described by crash recovery, until each one is restored or discarded.

#### Scenario: Recovery shown first
- **WHEN** the application starts with a recovered project
- **THEN** the home screen shows a "Recovered projects" section above Recent projects
