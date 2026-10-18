## MODIFIED Requirements

### Requirement: My Game status
The My Game tab SHALL show, for each vehicle found, its name, brand, kind and a status, as text with an icon and never by color alone:
- **Not generated**, when no package for it is installed;
- **Generated**, with the installed version, when the installed package covers the game's version;
- **Game updated**, when the game's version is outside the installed package's supported versions; checking it and generating regenerates it.

Each vehicle SHALL have a checkbox. **Select Updates** SHALL check the Game updated vehicles. The tab SHALL offer **Generate (n)** for the checked vehicles, Refresh and **Preferences…**, which opens Preferences on the Games tab (to change the game folder), and SHALL show the game, its version and the folder. It SHALL have filters for kind and a search field, and an empty state for each case: no folder set (with the detected one proposed), ATS folder (see below), nothing found.

#### Scenario: Status after generation
- **WHEN** the player generates a truck on game 1.61.1.9
- **THEN** its status changes from Not generated to Generated 1.61.1

#### Scenario: Game updated status
- **WHEN** the installed truck supports 1.61 only and the game folder is now 1.62
- **THEN** its status is Game updated, and Select Updates checks it

#### Scenario: No folder set
- **WHEN** the tab is opened with no ETS2 folder set and one is detected
- **THEN** the tab proposes the detected folder with a button to use it, and does not scan until the player does

#### Scenario: Preferences from My Game
- **WHEN** the player clicks Preferences… in the My Game tab
- **THEN** Preferences opens on the Games tab
