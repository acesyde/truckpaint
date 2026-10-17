## ADDED Requirements

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
