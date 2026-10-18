## Purpose

Lets a player make the official truck and trailer packages from their own installed game, inside the app, for their own use. Packages are never shared by TruckPaint.

## ADDED Requirements

### Requirement: Game folder detection and check
The application SHALL find the installation folders of ETS2 and ATS in the player's Steam libraries on Windows, Linux (including the Flatpak version of Steam) and macOS, by reading Steam's library list. A folder SHALL be accepted only when the game's main archives (`base.scs`, `def.scs` and `version.scs`) are readable in it; the game (ETS2 or ATS) and its version SHALL be read from those archives, never from the folder name. A folder chosen by hand SHALL go through the same check. A check that fails SHALL say why (not a folder, a file missing, not a game, unreadable).

#### Scenario: Steam library found
- **WHEN** Steam's library list names a library that holds `Euro Truck Simulator 2` with the main archives
- **THEN** that folder is offered as the ETS2 folder, with the game's version

#### Scenario: Several libraries
- **WHEN** ETS2 is installed in a second Steam library
- **THEN** the folder in that library is found

#### Scenario: Wrong folder
- **WHEN** the player picks a folder with no `base.scs`
- **THEN** it is refused with a message naming the missing file, and the previous folder stays set

#### Scenario: ATS folder given for ETS2
- **WHEN** the player picks an ATS folder as the ETS2 folder
- **THEN** it is refused with a message saying it is an ATS folder

#### Scenario: No Steam
- **WHEN** no Steam library is found
- **THEN** nothing is set, no error is shown, and the folder can be chosen by hand

### Requirement: Scanning the game
The application SHALL scan a valid game folder without writing anything, and list the official trucks and the trailers the player owns, each with its name, brand and kind. Only vehicles that exist in the installed game appear, so a DLC the player does not own is not listed. Vehicles that cannot be painted are not listed. A scan SHALL run off the user interface thread, show that it is busy, and be repeated on demand (**Refresh**) and when the game folder changes.

#### Scenario: Owned DLC only
- **WHEN** the game folder holds the base game and one trailer DLC
- **THEN** the base trucks and trailers and the DLC's trailers are listed, and no other DLC's vehicle is

#### Scenario: Scanning does not block
- **WHEN** a scan is running
- **THEN** the application still responds, and the rest of the Vehicle Library stays usable

#### Scenario: Game unreadable after a move
- **WHEN** the folder was deleted or moved since it was saved
- **THEN** the tab says the folder cannot be read and offers Preferences

### Requirement: Generating packages
In the My Game tab the player SHALL check the vehicles to make, in a list of checkboxes (with **Select All** and **Select None**), then start one run with **Generate (n)**, n being the number checked. The run SHALL generate Not generated vehicles and regenerate the others (Generated or Game updated), with no separate command. Each package SHALL be installed in the Vehicle Library like any other, with the id `official.<game>.<vehicle>`, as soon as it is made. Generation SHALL run off the user interface thread, **strictly one vehicle after another**, and SHALL show its progress while it runs:
- a progress bar of the vehicles finished out of the vehicles checked, with the count ("3 of 12");
- the name of the vehicle being made;
- in the list, each checked vehicle's state: waiting, in progress, installed, up to date (skipped) or failed.

It SHALL offer **Cancel**: a cancel stops before the next vehicle, and keeps those already installed. A vehicle that fails SHALL not stop the others, and SHALL be reported at the end with a reason in the player's words; nothing is installed for it. The result SHALL name how many were installed, skipped and failed. Generation SHALL NOT need a network, a command-line tool or a large cache folder.

#### Scenario: Generate one vehicle
- **WHEN** the player checks a truck that is Not generated and presses Generate (1)
- **THEN** its package is installed, the truck shows Generated with its version, and it is available in New Project

#### Scenario: Checked list with progress
- **WHEN** the player checks 3 vehicles, two Not generated and one Game updated, and presses Generate (3)
- **THEN** they are made one after another, the bar goes 0 of 3, 1 of 3, 2 of 3, 3 of 3, the vehicle being made is named, each row shows waiting, in progress then installed, and the Game updated one is regenerated

#### Scenario: Cancel
- **WHEN** the player generates 20 vehicles and presses Cancel after 5
- **THEN** the vehicle being made is finished or dropped, no more start, the 5 or 6 done stay installed, and the dialog says it was cancelled

#### Scenario: One vehicle fails
- **WHEN** one of three vehicles cannot be read
- **THEN** the other two are installed, and the result lists the failed one with its reason

#### Scenario: Closing the dialog
- **WHEN** the player closes the Vehicle Library during generation
- **THEN** generation continues, and its end is shown as a message in the application

#### Scenario: Only one run
- **WHEN** a generation is running
- **THEN** Generate and the checkboxes are unavailable until it ends or is cancelled

### Requirement: Package versions follow the game
A generated package's version SHALL be set from the game's version without any input from the player: the first three numbers of the game's version (game 1.61.1.9 gives 1.61.1). The package's supported game versions SHALL be the game's minor series (1.61 gives from 1.61 up to, not including, 1.62). When the vehicle already has an installed package, the new package SHALL keep the template version of every texture whose template did not change, and give the next template version to each one that changed. When the installed newest version is not lower than the new one and any template changed, the new package SHALL take the version after the installed newest one (the next patch number), so a package that was installed is never replaced by a different one under the same version. When no template changed and the installed package already covers the game's version, nothing SHALL be installed and the vehicle SHALL be reported as skipped (up to date).

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

### Requirement: My Game status
The My Game tab SHALL show, for each vehicle found, its name, brand, kind and a status, as text with an icon and never by color alone:
- **Not generated**, when no package for it is installed;
- **Generated**, with the installed version, when the installed package covers the game's version;
- **Game updated**, when the game's version is outside the installed package's supported versions; checking it and generating regenerates it.

Each vehicle SHALL have a checkbox. **Select Updates** SHALL check the Game updated vehicles. The tab SHALL offer **Generate (n)** for the checked vehicles, Refresh and **Preferences…** (to change the game folder), and SHALL show the game, its version and the folder. It SHALL have filters for kind and a search field, and an empty state for each case: no folder set (with the detected one proposed), ATS folder (see below), nothing found.

#### Scenario: Status after generation
- **WHEN** the player generates a truck on game 1.61.1.9
- **THEN** its status changes from Not generated to Generated 1.61.1

#### Scenario: Game updated status
- **WHEN** the installed truck supports 1.61 only and the game folder is now 1.62
- **THEN** its status is Game updated, and Select Updates checks it

#### Scenario: No folder set
- **WHEN** the tab is opened with no ETS2 folder set and one is detected
- **THEN** the tab proposes the detected folder with a button to use it, and does not scan until the player does

### Requirement: ATS is not generated yet
A valid ATS folder SHALL be accepted in Preferences. The My Game tab SHALL say that generating ATS vehicles is not supported yet, and SHALL NOT scan or list its vehicles. ETS2 SHALL be unaffected.

#### Scenario: ATS folder set
- **WHEN** an ATS folder is set and the tab is opened with ATS chosen
- **THEN** the tab shows the ATS game and version and the message that generation is not supported yet, with no Generate action

### Requirement: Generated packages stay local
Generated packages SHALL be written only to the Vehicle Library on the player's computer. The application SHALL send nothing to any server while scanning or generating. Each generated manifest SHALL carry the license "Personal modding only, not for sale. Game content belongs to SCS Software.", and the My Game tab SHALL say that the game's content belongs to SCS Software, that the packages are for the player's own use, and that TruckPaint is not endorsed by SCS.

#### Scenario: License and notice
- **WHEN** a truck is generated and its details are opened in the Installed tab
- **THEN** its license is the one above, and the My Game tab shows the notice

#### Scenario: No traffic
- **WHEN** the player scans and generates with no network
- **THEN** everything works
