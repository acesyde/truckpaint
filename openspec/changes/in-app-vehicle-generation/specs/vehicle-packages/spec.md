## MODIFIED Requirements

### Requirement: Vehicle Library dialog
Vehicle › Vehicle Library… SHALL open a dialog with three tabs, a list and a detail pane:
- **Tabs:** **Installed**, every installed vehicle, **My vehicles**, the custom vehicles (id starting with `custom.`), and **My Game**, the official vehicles of the player's own game (see the vehicle-generation capability). Installed and My vehicles show their count.
- **The list**, under a search field (name and brand) and the game and kind filters. Each row shows the vehicle's name, its brand, kind and game, its newest installed version, and a **status**, as text with an icon, never by color alone:
  - **Custom** for a custom vehicle, in the link color;
  - **Update** when the open project holds that vehicle at an older version than the newest installed one, in the signal color;
  - **Up to date** otherwise.
- **The detail pane** shows the vehicle chosen in the list (the first one when the dialog opens):
  - a preview of its template: the package's preview image when it has one, otherwise the template of its first main texture;
  - its name, brand, kind and game;
  - the newest installed version and its supported game versions, or "any version" when the package sets no limit;
  - its paint job: the names of its main textures and the number of accessories;
  - every installed version, each with **Export…** and **Remove**;
  - for a vehicle with several installed versions, what its newest version changes and which known projects it touches (see the template-update-impact capability).

It SHALL offer:
- **Install…**;
- **Custom Vehicle…**, which opens the Custom Vehicle dialog (see the custom-vehicles capability);
- **New Version…** for a custom vehicle, in its detail pane, which opens the Custom Vehicle dialog filled in from its newest version;
- **Update Template…** in the detail pane of a vehicle whose status is Update. It closes the library and opens Update Template for that vehicle of the open project (see the vehicle-projects capability);
- **Export…** for a vehicle version. It saves that version's package, unchanged, as a `.tpv` file where the player chooses, proposing `<id>-<version>.tpv`. The save dialog SHALL remind the player that templates from the base games belong to SCS Software, and to check that they may share them;
- **Remove** for a vehicle version, after confirmation. Removing never changes projects, which embed their templates.

With no vehicle installed, the dialog SHALL show an empty state explaining what packages are and offering Install…, Custom Vehicle… and a way to open the My Game tab. With no custom vehicle, the My vehicles tab SHALL explain that custom vehicles are made from template files and offer Custom Vehicle…. The My Game tab has its own list, status and actions, and the Install… and Custom Vehicle… actions stay available from every tab.

#### Scenario: Remove a version
- **WHEN** the user removes version 1.2.0 of a vehicle that also has 1.3.0 installed
- **THEN** only 1.3.0 remains in the library, and an open project made with 1.2.0 still shows its templates

#### Scenario: Paint job summary
- **WHEN** the sample truck 1.1.0 is installed, the Vehicle Library is open and the user chooses it in the list
- **THEN** its detail pane lists the main textures "Standard cab" and "High roof" and three accessories, with a preview of its template

#### Scenario: Export a version
- **WHEN** the user clicks Export… on version 1.0.0 of `custom.scania.r_2024` and chooses a folder
- **THEN** `custom.scania.r_2024-1.0.0.tpv` is written there with the same bytes as the installed package

#### Scenario: Any game version
- **WHEN** a custom vehicle was created with no game versions
- **THEN** its detail pane shows "any version" as its supported game versions

#### Scenario: Custom vehicle from the library
- **WHEN** the user clicks Custom Vehicle… in the Vehicle Library and creates a vehicle
- **THEN** the Vehicle Library is shown again and lists the new vehicle

#### Scenario: My vehicles
- **WHEN** the sample truck, the sample trailer and the custom vehicle `custom.scania.r_2024` are installed
- **THEN** the Installed tab shows 3 vehicles, the My vehicles tab shows 1, and the custom vehicle's status is Custom

#### Scenario: Update status
- **WHEN** a project made with the sample truck 1.0.0 is open and the sample truck 1.1.0 is installed
- **THEN** the sample truck's status in the Vehicle Library is Update, and its detail pane offers Update Template…, which opens Update Template for that vehicle

#### Scenario: Up to date without a project
- **WHEN** the Vehicle Library is opened from the home screen with the sample truck 1.0.0 and 1.1.0 installed
- **THEN** the sample truck's status is Up to date

#### Scenario: My Game tab
- **WHEN** the user opens the Vehicle Library and clicks the My Game tab
- **THEN** the official vehicles of the game folder set in Preferences are listed with their generation status

#### Scenario: Empty library points to My Game
- **WHEN** no vehicle is installed and the Vehicle Library is opened
- **THEN** the empty state offers Install…, Custom Vehicle… and My Game
