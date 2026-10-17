# template-update-impact Specification

## Purpose
Tells the player what a new version of a vehicle changes before they update: the version's changelog, the changes detected between versions, and how many textures and objects of their known projects are touched. After an update, it keeps the affected objects marked until they are dealt with.

## Requirements

### Requirement: Update summary in the Vehicle Library
The detail pane of a vehicle with several installed versions SHALL show an **Update summary** for its newest version, compared with the installed version just before it:
- the **changelog** of the newest version, when it has one;
- the **detected changes**: textures whose layout version was raised, textures whose size changed, and main textures and accessories added or removed. Textures are named by their names.

The summary SHALL be left out when only one version is installed. When nothing is detected and there is no changelog, it SHALL say "No changes listed". Detected changes SHALL be shown whether or not the package has a changelog, and SHALL be computed from the installed manifests only.

#### Scenario: Layout raised and texture added
- **WHEN** the sample truck 1.0.0 and 1.1.0 are installed, 1.1.0 raises Standard cab's layout version and adds Side skirts
- **THEN** the summary lists "Layout changed: Standard cab" and "New: Side skirts"

#### Scenario: Changelog shown
- **WHEN** 1.1.0 has the changelog "Door layout redrawn."
- **THEN** the summary shows that text above the detected changes

#### Scenario: Resized and removed
- **WHEN** 1.1.0 resizes Chassis from 2048² to 4096² and drops the accessory Mudflaps
- **THEN** the summary lists Chassis as resized and Mudflaps as removed

#### Scenario: One version only
- **WHEN** only version 1.0.0 of a vehicle is installed
- **THEN** its detail pane shows no update summary

### Requirement: Projects touched by an update
Under the summary, the Vehicle Library SHALL list the **known projects** that use an older version of the vehicle than its newest installed one. The known projects are the recent projects (see the start-screen capability) whose file still exists, plus the open project. They SHALL be read without being opened and without being changed, and the open project SHALL be counted from its current content, unsaved edits included. The list SHALL say "Only recent projects are listed" below it.

Each line SHALL read like "Transports Ardent: 2 textures, 3 objects to reposition". It compares what the project records with the newest version:
- **textures** counts the textures the project paints whose layout version in the newest version differs from the recorded one;
- **objects to reposition** counts the objects on those textures, groups' children included, groups themselves and the template left out; a symbol instance counts as one object. TruckPaint can't tell which areas of a template moved, so every object of such a texture counts, hidden ones too;
- a project texture missing from the newest version is added as "N no longer in this version", without objects;
- a project with no layout change, resized texture or removed texture SHALL read "Nothing to reposition".

A project is listed once, even when the recent list holds it under two paths. A project already using the newest version, and one that doesn't use the vehicle, SHALL NOT be listed. A file that can't be read SHALL be listed by its name with "Can't be read". A project whose recorded version is no longer installed SHALL be listed normally, because its counts only need what the project records. The list SHALL NOT delay opening the dialog.

#### Scenario: Project with layout changes
- **WHEN** "Transports Ardent" is in the recent list, uses the sample truck 1.0.0, paints Standard cab and Chassis, 1.1.0 changed only Standard cab's layout, and Standard cab holds 3 objects
- **THEN** the sample truck's detail pane lists "Transports Ardent: 1 texture, 3 objects to reposition"

#### Scenario: Not a recent project
- **WHEN** a project using the sample truck 1.0.0 is not among the recent projects
- **THEN** it is not listed, and "Only recent projects are listed" is shown

#### Scenario: Missing file
- **WHEN** a recent project's file was deleted
- **THEN** it is not listed

#### Scenario: Up to date project
- **WHEN** a recent project already uses the sample truck 1.1.0
- **THEN** it is not listed for that vehicle

#### Scenario: Old version no longer installed
- **WHEN** a recent project uses 1.0.0 of the sample truck, and only 1.1.0 is installed
- **THEN** it is listed with the counts from its recorded layout versions, and textures it paints that 1.1.0 no longer has appear as "no longer in this version"

#### Scenario: Files untouched
- **WHEN** the library lists three projects
- **THEN** none of the three files is modified or marked as open, and no recent entry moves

### Requirement: Later and Update
The update summary SHALL offer **Later** and **Update**:
- **Later** SHALL close the panel without changing anything; the update button of the vehicle in the project's Textures tab stays.
- **Update** SHALL be enabled when the open project uses the vehicle at an older version, and SHALL run Update Template for that vehicle of the open project (see the vehicle-projects capability). It SHALL be disabled, with a hint, otherwise.

The other listed projects SHALL be updated by their owners when they open them, with their own update button. No project file SHALL be changed without being opened.

#### Scenario: Later
- **WHEN** the user clicks Later with a project open that uses the sample truck 1.0.0
- **THEN** the panel closes, the project is unchanged and its Textures tab still shows the update button for 1.1.0

#### Scenario: Update the open project
- **WHEN** the open project uses the sample truck 1.0.0 and the user clicks Update
- **THEN** the library closes and Update Template opens for the sample truck

#### Scenario: No open project
- **WHEN** the library is opened from the home screen
- **THEN** Update is disabled with a hint that a project using the vehicle must be open, and the projects are still listed

### Requirement: Objects to reposition after an update
When Update Template is applied and flags a texture "Layout changed", every object on that texture at that time SHALL be flagged **To reposition** (see the layers-panel capability). A flag SHALL be removed when its object is moved or transformed (position, size or rotation, not its look: colors, strokes, text content, opacity and name keep it), when the object is deleted, or when the texture is marked as checked. Objects added after the update SHALL NOT be flagged. A texture that stays "Layout changed" through a later update keeps its flags and gains those of objects present at that update. Undoing the update SHALL remove the flags it added, and undoing a move SHALL restore its flag.

Flags SHALL be kept with the project (see the project-files capability). They SHALL NOT change a texture's state, the warnings under "Before exporting" or the export.

#### Scenario: Flagged after update
- **WHEN** Update Template flags Standard cab "Layout changed" while it holds 3 objects
- **THEN** the 3 objects are flagged and the Layers tab marks them

#### Scenario: Editing the look keeps the flag
- **WHEN** the user changes the fill of a flagged rectangle
- **THEN** it is still flagged

#### Scenario: Moving clears the flag
- **WHEN** the user drags a flagged logo
- **THEN** its flag is cleared, and Undo restores it

#### Scenario: Mark as Checked clears all
- **WHEN** the user clicks Mark as Checked on the texture with 2 flagged objects left
- **THEN** no object of the texture is flagged

#### Scenario: Undo the update
- **WHEN** the user presses Undo right after the update
- **THEN** the texture is no longer flagged and neither are its objects

### Requirement: Impact in Update Template
Update Template SHALL show, for each texture it will flag "Layout changed", the number of objects on it to reposition, counted as in the projects list ("Standard cab: layout changed, 3 objects to reposition"), and the new version's changelog when it has one. The numbers SHALL equal the number of objects flagged by confirming. A texture without a layout change SHALL show no count.

#### Scenario: Counts match
- **WHEN** Update Template shows "Standard cab: layout changed, 3 objects to reposition" and the user confirms
- **THEN** exactly 3 objects of Standard cab are flagged
