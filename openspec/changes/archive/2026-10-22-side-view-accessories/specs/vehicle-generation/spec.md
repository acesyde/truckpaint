## MODIFIED Requirements

### Requirement: Package versions follow the game
A generated package's version SHALL be set from the game's version without any input from the player: the first three numbers of the game's version (game 1.61.1.9 gives 1.61.1). The package's supported game versions SHALL be the game's minor series (1.61 gives from 1.61 up to, not including, 1.62). When the vehicle already has an installed package, the new package SHALL keep the template version of every texture whose template did not change, and give the next template version to each one that changed. When the installed newest version is not lower than the new one and any template changed, the new package SHALL take the version after the installed newest one (the next patch number), so a package that was installed is never replaced by a different one under the same version. A package whose preview meshes differ from the installed package's (a preview mesh gained, or accessory slots added) SHALL count as a change for the package version, but not for its template version. When no template changed, the preview meshes are the same and the installed package already covers the game's version, nothing SHALL be installed and the vehicle SHALL be reported as skipped (up to date).

#### Scenario: Version from the game
- **WHEN** the game's version is 1.61.1.9 and a truck is generated for the first time
- **THEN** its package has version 1.61.1, and supports game versions from 1.61 up to, not including, 1.62

#### Scenario: Game updated, templates unchanged
- **WHEN** the game moves to 1.62.0.3, and a truck's templates are the same as the installed package's
- **THEN** a package 1.62.0 is installed with the same template versions, and projects using the truck are not flagged

#### Scenario: Template changed by a patch
- **WHEN** the game moves to 1.61.2.0 and one cabin's template changed
- **THEN** the package is 1.61.2, that texture's template version is one higher, and the update is shown for projects using the truck (see template-update-impact)

#### Scenario: Same version, changed template
- **WHEN** the installed truck is 1.61.1, the game reports 1.61.1.4, and a template changed
- **THEN** the new package is 1.61.2 and 1.61.1 stays installed beside it

#### Scenario: Nothing to do
- **WHEN** the player regenerates a truck whose templates are unchanged and the installed package covers the game's version
- **THEN** nothing is installed and the truck is reported as up to date

#### Scenario: Package made before the Side View
- **WHEN** the player regenerates a truck whose installed package covers the game's version, has the same templates and no preview mesh
- **THEN** a package with the next patch version is installed with the same template versions and a preview mesh per main texture, and projects using the truck are not flagged

#### Scenario: Package made before accessories were drawn
- **WHEN** the player regenerates a truck whose installed package covers the game's version, has the same templates and preview meshes without accessory slots
- **THEN** a package with the next patch version is installed with the same template versions and preview meshes with their accessory slots, and projects using the truck are not flagged
