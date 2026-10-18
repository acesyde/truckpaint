# vehicle-packages Specification

## Purpose

Defines vehicle packages, the versioned files that describe one truck or trailer and its texture templates, and the local library where they are installed. Painters can then work on any official or community vehicle without TruckPaint shipping game assets.

## Requirements

### Requirement: Package format
A vehicle package SHALL be a ZIP file with the `.tpv` extension containing a `vehicle.json` manifest at its root and the template images it references. The manifest SHALL contain:

- `format`: the package format version (1);
- `id`: lowercase words separated by dots, at least two (`scs.volvo.fh16_2012`, `community.jdoe.mighty_hauler`), each word made of a–z, 0–9, `_` and `-`;
- `version`: a semantic version (`1.2.0`);
- `name`, `brand` and `kind` (`truck` or `trailer`);
- optionally `authors`, `license`, `homepage`, `description` and a `preview` image;
- `game`, what the package targets in the game:
  - `id`: `ets2` or `ats`;
  - `versions`: a version range of the game the package is made for (`>=1.50, <1.54`);
  - `path`: the vehicle's path in the game's definitions (`scania.r_2016`), as words of a–z, 0–9 and `_` separated by dots;
  - optionally `alt_uv` (the paint job uses the vehicle's alternate UV set, some ATS trucks) and `colour_picker` (the paint job lets the player pick a base color), both false by default;
  - optionally `requires`: mods the vehicle depends on, each with a name and an optional version range;
- `paint_job`, the vehicle's paint job as the game structures it, made of **parts**:
  - `main`: one or more main textures. A trailer, or a truck whose cabins share one layout, has one; a truck whose cabins have different layouts has one per layout;
  - optionally `accessories`: the accessory groups, shared by the whole vehicle.

Every part, main texture or accessory, SHALL have the same fields:
- an `id`, unique within the vehicle, and a `name`;
- `game_ids`: the names the game uses for what the part covers. For a main texture, the internal names of the cabins that share its layout (`["highline", "highline_8x4"]`); a main texture without game ids covers every cabin. For an accessory, the accessory ids (`["mirror.painted", "s_mirror.painted"]`);
- for a main texture only, optionally `also_paints`: accessory ids the game paints with this same texture (`["body.reefer", "body.reef_13ct"]`), for a vehicle whose template covers both, like a trailer body. Empty by default;
- `texture`: its `size` in pixels as `[width, height]`, each a power of two from 64 to 8192 (`[4096, 4096]`, `[8192, 1024]`), its `template` image path inside the package (a PNG or SVG file), and its `layout_version`, a positive integer that the author increases whenever the texture's layout changes.

Unknown fields SHALL be ignored, so packages can carry data for later TruckPaint versions. A package SHALL contain only data: JSON and images.

#### Scenario: Minimal valid package
- **WHEN** a truck package declares one main texture "Cabin" without game ids, with a 4096 px template PNG in the package
- **THEN** it is accepted and listed as one vehicle with one main texture and no accessory, painted on every cabin

#### Scenario: Truck with cabins of different layouts
- **WHEN** a truck package declares the main textures "Standard cab" (game id `standard`) and "High roof" (game id `high_roof`)
- **THEN** it is accepted and listed with two main textures

#### Scenario: Trailer package
- **WHEN** a trailer package declares one main texture "Base" and two accessory groups "Body 13.6 m" and "Mudflaps"
- **THEN** it is accepted and listed as one trailer with one main texture and two accessories

#### Scenario: Main texture that also paints accessories
- **WHEN** a trailer package's main texture "Body" lists `also_paints: ["body.reefer", "body.reef_13ct"]`
- **THEN** it is accepted, and a project made from it records those ids with its Body

#### Scenario: Data for later versions
- **WHEN** a package's manifest contains a field TruckPaint does not know, such as `mirror`
- **THEN** the package is accepted and the field is ignored

### Requirement: Package validation
Installing a package SHALL validate it completely and refuse it, with a message naming the first problem found, when:

- it is not a ZIP, has no `vehicle.json`, or the manifest is not valid JSON;
- a required field is missing or invalid: the id, version, game id, game-version range or game path, or an unknown kind;
- it declares a format version newer than this version of TruckPaint supports;
- the paint job has no main texture;
- two parts of the vehicle share an id;
- the paint job has two or more main textures and one of them has no game id, or an accessory has no game id;
- a game id is not words of a–z, 0–9 and `_` separated by dots, or appears twice among the main textures or twice among the accessories;
- an accessory has `also_paints`, or an id of `also_paints` is not words of a–z, 0–9 and `_` separated by dots, appears twice in the package's `also_paints`, or is also an accessory's game id;
- a texture size is not a `[width, height]` pair of allowed sides (a single number is refused), or a template file is missing, is not a PNG or SVG, or cannot be read;
- an entry path is absolute or contains `..`;
- the package is larger than 512 MB uncompressed, or an image larger than 16384 px on a side.

Nothing SHALL be installed when validation fails.

#### Scenario: Missing template
- **WHEN** the user installs a package whose accessory "Chassis" references `templates/chassis.png` but the file is absent
- **THEN** installation is refused with a message saying the template of "Chassis" is missing, and the library is unchanged

#### Scenario: Unsafe path
- **WHEN** a package contains an entry named `../../evil.png`
- **THEN** installation is refused and no file is written outside the library

#### Scenario: Several main textures without game ids
- **WHEN** a package declares the main textures "Standard cab" and "High roof", and "High roof" has no game id
- **THEN** installation is refused with a message saying "High roof" has no game id

#### Scenario: Accessory without game id
- **WHEN** an accessory group "Mirrors" has an empty `game_ids` list
- **THEN** installation is refused with a message saying "Mirrors" has no game id

#### Scenario: Rectangular texture
- **WHEN** the user installs a package whose trailer base texture has the size `[8192, 1024]`
- **THEN** it is installed, and a project made from it has an 8192 × 1024 surface

#### Scenario: Single size refused
- **WHEN** a package gives a texture the size `4096`
- **THEN** installation is refused with a message saying the texture's size must be a width and a height

#### Scenario: Accessory id painted twice
- **WHEN** a main texture lists `also_paints: ["body.reefer"]` and an accessory has the game id `body.reefer`
- **THEN** installation is refused with a message saying `body.reefer` appears twice

### Requirement: Installing packages
The user SHALL be able to install a package:
- from Vehicle › Vehicle Library… › Install…, choosing one or more `.tpv` files;
- from the vehicle list of New Project › Install…;
- by dropping `.tpv` files on the home screen or the workspace.

Installed packages SHALL persist across sessions in the application's data folder.

Installing the same version again SHALL replace it. A different version of an installed vehicle SHALL be installed alongside the others. A confirmation SHALL name each vehicle and version installed.

#### Scenario: Install from the Vehicle Library
- **WHEN** the user installs `volvo_fh16-1.2.0.tpv` from the Vehicle Library and restarts the application
- **THEN** "Volvo FH16 2012" version 1.2.0 is listed in the library

#### Scenario: Two versions side by side
- **WHEN** versions 1.2.0 and 1.3.0 of the same vehicle are installed
- **THEN** the library lists the vehicle once, as version 1.3.0, and its details show 1.2.0 as an older installed version

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

### Requirement: Built-in sample vehicle
TruckPaint SHALL include the newest versions of the sample truck and the sample trailer (see the vehicle-authoring capability). When no vehicle is installed, an **Install the sample vehicles** button SHALL appear in two places:
- the empty state of the Vehicle Library;
- the vehicle list of New Project.

The button SHALL install both samples like any other package, with the same confirmation and the same errors. Once a vehicle is installed, the button SHALL no longer be shown. Installing the samples SHALL need no network access.

#### Scenario: First vehicle project
- **WHEN** a painter with no vehicle installed opens New Project and clicks Install the sample vehicles
- **THEN** "TruckPaint Sample Truck" and "TruckPaint Sample Trailer" appear in the list with their newest versions, a confirmation names them, and either can be picked to create a project

#### Scenario: Library with vehicles
- **WHEN** at least one vehicle is installed
- **THEN** neither the Vehicle Library nor New Project shows Install the sample vehicles

### Requirement: Package changelog
A package's manifest MAY have an optional `changelog`: plain text of at most 1000 characters saying what changed in that version ("Door layout redrawn, islands moved. Low cabin added."). A package without it SHALL be valid and behave as before. A `changelog` over the limit, or that is not text, SHALL refuse the package with a message naming the field. An application that doesn't know the field SHALL ignore it. The changelog describes the version it is in and SHALL NOT change which version is newer, the texture layouts or the export.

#### Scenario: Package with a changelog
- **WHEN** a package whose manifest has `"changelog": "Door layout redrawn."` is installed
- **THEN** it is installed like any other package and its newest version carries that text

#### Scenario: Package without a changelog
- **WHEN** a package whose manifest has no `changelog` is installed
- **THEN** it is installed and listed as before

#### Scenario: Changelog too long
- **WHEN** a package has a `changelog` of 1001 characters
- **THEN** installation is refused with a message saying the changelog is longer than 1000 characters
