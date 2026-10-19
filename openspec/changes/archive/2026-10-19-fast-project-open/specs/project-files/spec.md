## MODIFIED Requirements

### Requirement: Versioned format
Every project file SHALL record the version of the file format it was written with, where it can be read without reading the project itself. The application SHALL open files of every format version released so far, converting them to the current format when opening; such a file SHALL open marked as having unsaved changes and SHALL be written in the current format at the next save (the original file is only replaced when the user saves). Files written with a newer format version than the application supports SHALL be refused (see Files that cannot be opened). Files from builds that were never released SHALL NOT be recognized; they are refused like any file that cannot be read.

#### Scenario: Opening an older project
- **WHEN** a project saved by an earlier version of TruckPaint, using an older file format, is opened
- **THEN** it opens with all its content, the status bar shows "Unsaved changes", and saving writes it in the current format

#### Scenario: Version recorded
- **WHEN** a project is saved
- **THEN** the file records the current format version

#### Scenario: Large project
- **WHEN** a project holding about 1,300 objects, saved as a file of several megabytes, is opened
- **THEN** reading the file takes a fraction of a second, not minutes

### Requirement: Open a project
File › Open… (Cmd/Ctrl+O), a recent project and every other way to open a project file SHALL read the file without freezing the window. If the file takes longer than a quarter of a second to read, a small box "Opening <name>…" SHALL appear over the window with a moving indicator, and disappear when the project is shown or the file is refused; a quick open SHALL show no box. While a file is being read, the window SHALL keep answering, the project already open (if any) SHALL stay as it is, and no other project SHALL be opened.

Once read, the project SHALL open in the editor with nothing selected, the view fitted to the surface and an empty undo history, marked as saved. Opening the file of the project already open SHALL keep it as it is. A file that cannot be opened SHALL be reported as described in Files that cannot be opened.

#### Scenario: Open from the menu
- **WHEN** the user chooses File › Open… and selects "ace.truckpaint"
- **THEN** the editor shows that project, the window title shows its name and the status bar shows "Saved"

#### Scenario: Slow open
- **WHEN** the user opens a project file that takes a few seconds to read
- **THEN** a box "Opening ace.truckpaint…" is shown, the window keeps answering, and the editor shows the project as soon as it is read

#### Scenario: Quick open
- **WHEN** the user opens a small project that reads in a few milliseconds
- **THEN** no box flashes, and the editor shows the project

#### Scenario: Second open while reading
- **WHEN** the box is shown and the user chooses File › Open… again or clicks a recent project
- **THEN** nothing starts, and the first project opens when read

#### Scenario: Unreadable file
- **WHEN** the file being read turns out to be damaged
- **THEN** the box disappears, a message says the file is damaged, and the project that was open is still open and unchanged

## REMOVED Requirements

### Requirement: Invalid files
**Reason**: It told the files of unreleased development builds apart from damaged files, which needed a read of the whole document and slowed the opening of large projects. TruckPaint is not released, so no player has such a file; they are now refused like any file that cannot be read.
**Migration**: See "Files that cannot be opened", which keeps the other refusals (not a project, damaged, newer version, mixed games) and adds the file without a recorded version.

## ADDED Requirements

### Requirement: Files that cannot be opened
Files that are not TruckPaint projects, are damaged, were written by a newer version of TruckPaint, or have no recorded format version SHALL NOT be opened. The user SHALL see a message naming the file and the reason (for example "ace.truckpaint was created with a newer version of TruckPaint"). The currently open project, if any, SHALL stay open and unchanged.

Files mixing games SHALL be refused as damaged.

#### Scenario: Damaged file
- **WHEN** the user opens a truncated `.truckpaint` file
- **THEN** a message says the file is damaged and cannot be opened, and nothing else changes

#### Scenario: Newer format
- **WHEN** the user opens a file whose format version is newer than the application supports
- **THEN** a message says it was created with a newer version of TruckPaint

#### Scenario: No recorded version
- **WHEN** the user opens a `.truckpaint` file that records no format version, such as one written by a build that was never released
- **THEN** a message says the file is damaged and cannot be opened, and nothing else changes
