# project-files Specification

## Purpose

Lets users keep their liveries: save a project to a single `.truckpaint` file, reopen it later on any computer, and never lose work silently when closing.

## Requirements

### Requirement: Self-contained project file
A project SHALL be saved as one `.truckpaint` file containing everything needed to reopen it identically. The file SHALL contain:

- the project's name, game versions and surfaces, with each surface's name, width and height;
- the project's vehicles, each with its package id, version, name, brand, kind and game, and its game data: game path, supported game versions, whether it uses the alternate UV set and the colour picker, and the mods it requires;
- each surface's template: the vehicle and texture it belongs to, whether that texture is a main texture or an accessory, its game ids, its image, opacity, visibility, layout version and status;
- the full object tree, with every object's identity, name, kind, geometry (including polygon settings, the points and handles of every path subpath with its open or closed state, and the path's line width), fill, stroke, opacity, shadow (color, opacity, offset, blur), visibility and lock flags;
- each text's content and character style;
- image references;
- whether each text and image is mirrored;
- the guides of each surface;
- the project palette: each swatch's identity, name and color;
- the shared styles: each graphic style's identity, name, fill, stroke, opacity and shadow, and each text style's identity, name, character style, fill, stroke, opacity and shadow;
- the links: for every solid fill, solid stroke, gradient stop and shadow, the swatch it is linked to; for every object, the graphic style and the text style it follows;
- the symbols: each symbol's identity, name, artboard size, guides and content (an object tree, as for a surface);
- the library origin of each swatch, graphic style, text style and symbol that came from the library or was added to it;
- each instance's symbol, placement, opacity, visibility, lock and name. An instance's content is not stored: it is rebuilt from its symbol when the file opens;
- the mod settings: Name, Version, Author, Description, Price, Unlock level, internal name and whether it was edited, and the chosen shop icon and Mod Manager image, if any;
- the bytes of every imported asset, chosen mod images included.

Font families SHALL be stored by name only; font files SHALL NOT be embedded. Editor state that is not part of the document (selection, point selection, zoom, panel layout, grid and snapping settings, whether guides are shown, undo history) SHALL NOT be stored.

A file written before swatches had names SHALL open with each palette color as a swatch named "Color 1", "Color 2"… in palette order, and nothing linked. A file written before texts and images could be mirrored SHALL open with none of them mirrored. A file written before mod settings existed SHALL open with the default mod settings. A file written before game versions existed SHALL open with none. A file written before the library existed SHALL open with no library origins.

#### Scenario: Round trip
- **WHEN** the user saves a project with grouped shapes, an outlined text, a placed SVG logo, a star, a curved path with a hole and two guides, closes it and opens the file again
- **THEN** every object, group, text, image, path point and handle, guide, palette color and asset is restored with the same properties and stacking order

#### Scenario: Opening on another computer
- **WHEN** a project using a system font that is not installed is opened on another computer
- **THEN** it opens, the text is drawn with Inter and the font control shows the "Font not found" warning with the original family name

#### Scenario: Vehicle project round trip
- **WHEN** the user saves a project with a truck painting two main textures and two accessories and a trailer painting its Base texture, with one template hidden and one flagged "Layout changed", closes it and opens the file again
- **THEN** the vehicles and their recorded versions and game data, the surfaces with their main texture or accessory role and game ids, their templates, opacities, visibilities and flags are restored

#### Scenario: Brand kit round trip
- **WHEN** the user saves a project with the swatches "Company red" and "Company grey", a graphic style whose gradient has a stop linked to "Company red", a text style, a rectangle following the graphic style and a text following the text style with its fill linked to "Company grey", then closes and reopens it
- **THEN** the swatches, the styles and every link are restored, and editing "Company red" still recolors the style and the rectangle

#### Scenario: Symbols round trip
- **WHEN** the user saves a project with the symbol "Logo", holding a circle and a text, with an instance on two textures, one rotated and flipped, closes it and opens the file again
- **THEN** "Logo" and both instances are restored with their placements, and editing "Logo" updates both

#### Scenario: Palette of an earlier file
- **WHEN** a file whose palette holds three colors without names is opened
- **THEN** the palette shows the swatches "Color 1", "Color 2" and "Color 3" with those colors, and no object is linked

#### Scenario: Mirrored objects round trip
- **WHEN** the user saves a project with a mirrored image, a text flipped vertically and an image that is not mirrored, closes it and opens the file again
- **THEN** the first image is still mirrored, the text is still drawn upside down and mirrored, and the other image is not mirrored

#### Scenario: Mod settings round trip
- **WHEN** the user saves a project whose mod settings have the Version "2.0", the Author "Jane", the Price 7500, the edited internal name "acelog" and a chosen Mod Manager image, closes it and opens the file again
- **THEN** Export Mod… shows the same settings and image, and changing the Name leaves the internal name "acelog"

#### Scenario: File without mod settings
- **WHEN** a project named "ACE" saved before mod settings existed is opened
- **THEN** Export Mod… shows the Name "ACE", the Version "1.0", the Price 5000 and generated images

#### Scenario: Game versions round trip
- **WHEN** the user saves a project whose Game versions are `1.56.*, 1.57.*`, closes it and opens the file again
- **THEN** the Game versions field shows "1.56.*, 1.57.*"

#### Scenario: Library origins round trip
- **WHEN** the user imports "Logo Ardent" and "Vert Ardent" from the library, saves the project, closes it and opens it again
- **THEN** Import from Library… shows both as "In this project"

#### Scenario: Shadow saved and reopened
- **WHEN** a project with a text whose shadow is linked to "Night" (60%, offset 4 / 6, blur 12) is saved and opened again
- **THEN** the text has the same shadow, still linked to "Night"

#### Scenario: File without shadows
- **WHEN** a project file saved before shadows existed is opened
- **THEN** it opens unchanged, with no object or style having a shadow

### Requirement: Game data of older files
When a file without game data for a vehicle is opened, the game data of that vehicle and of its textures SHALL be filled in from the installed package version the vehicle records, if that version is installed. The file SHALL then open marked as having unsaved changes, so the next save stores the data. When that version isn't installed, the project SHALL open as it is and stay fully editable. Only Export Mod reports the missing data (see mod-export).

#### Scenario: Older file with the package installed
- **WHEN** a file saved before game data was recorded, holding the sample truck 1.1.0, is opened with that version installed
- **THEN** it opens with the game path "truckpaint.sample" and the game ids of its textures, and the status bar shows "Unsaved changes"

#### Scenario: Older file without the package
- **WHEN** the same file is opened on a computer where the sample truck 1.1.0 is not installed
- **THEN** it opens with its artwork and templates, the status bar shows "Saved", and Export Mod… reports the missing data

### Requirement: Versioned format
Every project file SHALL record the version of the file format it was written with. The application SHALL open files of every format version it has ever written, converting them to the current format when opening; such a file SHALL open marked as having unsaved changes and SHALL be written in the current format at the next save (the original file is only replaced when the user saves). Files written with a newer format version than the application supports SHALL be refused (see Invalid files).

#### Scenario: Opening an older project
- **WHEN** a project saved by an earlier version of TruckPaint, using an older file format, is opened
- **THEN** it opens with all its content, the status bar shows "Unsaved changes", and saving writes it in the current format

#### Scenario: Version recorded
- **WHEN** a project is saved
- **THEN** the file records the current format version

### Requirement: Save and Save As
File › Save (Cmd/Ctrl+S) SHALL write the project to its file, or behave as Save As when the project has never been saved. File › Save As… (Cmd/Ctrl+Shift+S) SHALL ask for a location with a native dialog, proposing "<project name>.truckpaint", and from then on Save SHALL write to that new file. Save SHALL be available while typing in a field. Saving SHALL NOT freeze the editor; while a save is in progress the status bar SHALL show "Saving…".

#### Scenario: First save
- **WHEN** the user presses Cmd/Ctrl+S on a new project named "ACE Logistics"
- **THEN** a save dialog proposes "ACE Logistics.truckpaint", and after confirming the status bar shows "Saved"

#### Scenario: Save after changes
- **WHEN** a saved project is modified and the user presses Cmd/Ctrl+S
- **THEN** the same file is updated without a dialog and the status bar shows "Saved"

### Requirement: Safe writes
Saving SHALL never leave a damaged project file: the new content SHALL be written completely before it replaces the existing file, so that a failed or interrupted save keeps the previous version intact. When saving fails, the user SHALL see a message naming the file and the reason, and the project SHALL remain marked as having unsaved changes.

#### Scenario: Save to a read-only location
- **WHEN** saving fails because the destination is not writable
- **THEN** a message explains that the file could not be saved and why, the previous file is unchanged and the status bar still shows "Unsaved changes"

### Requirement: Open a project
File › Open… (Cmd/Ctrl+O) SHALL show a native dialog filtered to `.truckpaint` files and open the chosen project in the editor, with nothing selected, the view fitted to the surface and an empty undo history, marked as saved. Opening the file of the project already open SHALL keep it as it is.

#### Scenario: Open from the menu
- **WHEN** the user chooses File › Open… and selects "ace.truckpaint"
- **THEN** the editor shows that project, the window title shows its name and the status bar shows "Saved"

### Requirement: Invalid files
Files that are not TruckPaint projects, are damaged, were written by a newer version of TruckPaint, or use a format from unreleased development builds SHALL NOT be opened. The user SHALL see a message naming the file and the reason (for example "ace.truckpaint was created with a newer version of TruckPaint"). The currently open project, if any, SHALL stay open and unchanged.

Development builds wrote project files that this version refuses as a development format:
- files without a vehicle, written before every project needed one;
- files whose vehicles record variants, written before projects followed the game's paint job structure;
- files whose surfaces have a single size, written before textures had a width and a height.

Files mixing games SHALL be refused as damaged.

#### Scenario: Damaged file
- **WHEN** the user opens a truncated `.truckpaint` file
- **THEN** a message says the file is damaged and cannot be opened, and nothing else changes

#### Scenario: Newer format
- **WHEN** the user opens a file whose format version is newer than the application supports
- **THEN** a message says it was created with a newer version of TruckPaint

#### Scenario: Development build file
- **WHEN** the user opens a file saved by an unreleased development build with format 3
- **THEN** a message says the file uses a development format that this version cannot open, and nothing else changes

#### Scenario: Blank-texture file from a development build
- **WHEN** the user opens a file saved by a development build for a blank texture, with no vehicle
- **THEN** a message says the file uses a development format that this version cannot open, and nothing else changes

#### Scenario: Variant file from a development build
- **WHEN** the user opens a file saved by a development build whose vehicle records the variants "Standard cab" and "High roof"
- **THEN** a message says the file uses a development format that this version cannot open, and nothing else changes

#### Scenario: File from before rectangular textures
- **WHEN** the user opens a project file whose surfaces have a single size
- **THEN** a message says the file uses a development format and cannot be opened, and nothing else changes

### Requirement: Unsaved changes prompt
When the project has unsaved changes, closing it, quitting the application, closing the window, creating a new project or opening another project SHALL first ask "Save changes to “<name>” before closing?" with the choices Save, Don't Save and Cancel. Save SHALL save (asking for a location if needed) and then continue only if saving succeeded; Don't Save SHALL continue without saving; Cancel (or Escape) SHALL return to the editor with nothing changed. Without unsaved changes, these actions SHALL proceed without asking.

#### Scenario: Quit with unsaved changes
- **WHEN** the user modifies a project and closes the window
- **THEN** the save prompt appears and the window stays open until a choice is made

#### Scenario: Cancel keeps working
- **WHEN** the prompt is shown and the user presses Escape
- **THEN** the prompt closes and the project stays open with its unsaved changes

#### Scenario: No prompt when saved
- **WHEN** the user closes a project right after saving it
- **THEN** the home screen is shown without a prompt

### Requirement: Recent projects follow files
Opening or saving a project SHALL add its file to the top of the recent projects list (without duplicates, keeping its name and the current date).

#### Scenario: Saved project appears in recent list
- **WHEN** the user saves a new project as "ace.truckpaint" and closes it
- **THEN** the home screen lists "ace.truckpaint" first among recent projects

### Requirement: Objects flagged by a template update
A project file SHALL be able to record, with a surface's template, the objects that an update flagged to reposition (see the template-update-impact capability). The record is optional: files without it, and files of a project with no flagged object, SHALL be written and read as before, in the same format version. Opening a file SHALL ignore a recorded object that no longer exists on the surface, and a record on a texture that is not flagged "Layout changed". An older TruckPaint that doesn't know the record SHALL open the file and ignore it; the texture stays To check.

#### Scenario: Flags survive closing
- **WHEN** a project with 3 flagged objects on Standard cab is saved, closed and opened again
- **THEN** the same 3 objects are marked in the Layers tab

#### Scenario: Older file
- **WHEN** a project saved before this record existed is opened
- **THEN** it opens with its To check textures and no marked object, and is not marked as having unsaved changes

#### Scenario: Stale record
- **WHEN** a file records an object id that is not on its surface
- **THEN** the project opens and the id is ignored

#### Scenario: Reading without opening
- **WHEN** the application needs only a project's vehicles, textures and object counts
- **THEN** it can read them from the file without loading its images and without changing the file
