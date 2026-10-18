## ADDED Requirements

### Requirement: Game folders
Preferences SHALL have a **Games** section (completed by the first-run change with the games played and their mod folders) with the installation folder of ETS2 and of ATS, each with: the folder (empty by default), the game's version when the folder is valid, **Detect**, **Choose…** and **Clear**. Detection and validity are those of the vehicle-generation capability. A folder SHALL be checked when it is chosen and when the dialog opens, and an invalid or missing one SHALL show a message next to its field while other settings stay usable. When a first run has no folder set, the app SHALL detect the folders by itself once, without asking. The folders SHALL be persisted with the other preferences; a preferences file without them SHALL load with both empty. "Reset to defaults" SHALL NOT change them.

#### Scenario: Detect
- **WHEN** the player clicks Detect and ETS2 is in a Steam library
- **THEN** the ETS2 field shows that folder and the game's version

#### Scenario: Refused folder
- **WHEN** the player chooses a folder that is not a game
- **THEN** a message names the reason and the field keeps its previous folder

#### Scenario: Folder removed
- **WHEN** the saved ETS2 folder no longer exists and Preferences is opened
- **THEN** the field shows a message that the folder cannot be found, and Clear and Choose… still work

#### Scenario: Setting survives restart
- **WHEN** the player sets the ETS2 folder, quits and restarts
- **THEN** the same folder is set

#### Scenario: Older preferences file
- **WHEN** the application starts with a preferences file written before this setting existed
- **THEN** all its other preferences load, both folders are empty, and no message about the preferences appears
