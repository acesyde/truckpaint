# first-run Specification

## Purpose

Guides a new player, on the first launch only, from language and game to a first project, generating the vehicles they own from their own game so they can paint within minutes.

## Requirements

### Requirement: When the first launch is shown
The application SHALL show the first launch, in place of the home screen, the first time it starts. It SHALL NOT be shown again once finished or skipped. A preferences file that records recent projects or installed vehicles but no first-launch field SHALL be treated as having done the first launch.

The first launch SHALL have three steps (Language and game, Vehicles to paint, First project), a step indicator showing the current step, a **Back** button from the second step on, and **Skip** on every step. Skip SHALL end the first launch and show the home screen, keeping what was already chosen.

#### Scenario: Shown once
- **WHEN** a player with no preferences file starts the application, finishes the first launch, quits and restarts
- **THEN** the first launch is shown the first time and the home screen the second time

#### Scenario: Skip
- **WHEN** the player clicks Skip on the Vehicles to paint step
- **THEN** the home screen is shown, and the first launch is not shown at the next start

#### Scenario: Existing user
- **WHEN** the application starts with preferences that list recent projects and no first-launch field
- **THEN** the home screen is shown

#### Scenario: Back
- **WHEN** the player is on the Vehicles to paint step and clicks Back
- **THEN** the Language and game step is shown with the choices unchanged

### Requirement: Language and game folders step
The first step SHALL ask:
- the **interface language**, preselected from the system language. Choosing another language SHALL apply it at once and save it as Preferences does; keeping the detected one SHALL NOT save a language;
- for each game, ETS2 and ATS, its **installation folder**, detected as the Preferences of the app do (see the in-app-vehicle-generation change), shown as "Game found" with its path, or "Not found" with **Choose folder…**. Choosing a folder SHALL store it in the same preference as the game installation folder of Preferences;
- for each game, its **mod folder**, detected where Export Mod looks, shown as "Mod folder found" with its path or "Not found"; **Choose folder…** picks another one.

The step SHALL NOT show a game version and SHALL NOT ask which games the player plays: a game is played when its installation folder is set. A game that is not found SHALL NOT block the flow, and Next SHALL always be available. A folder SHALL be accepted only for the game it holds (ETS2 or ATS, as the vehicle-generation capability checks it); a folder of the other game SHALL be refused with a message naming that game, and the previous folder SHALL be kept.

#### Scenario: Language from the system
- **WHEN** the system language is German and the first launch opens
- **THEN** the interface is in German and Deutsch is selected, and nothing is saved until the player picks a language

#### Scenario: Game found
- **WHEN** the ETS2 installation folder is detected
- **THEN** the step shows "Game found" with the path

#### Scenario: Game not found
- **WHEN** no ATS installation folder is detected
- **THEN** the step shows "Not found" with Choose folder…, and Next stays available

#### Scenario: Choosing a folder
- **WHEN** the player clicks Choose folder… for the ETS2 installation and picks a folder
- **THEN** the step shows that path and Preferences shows the same folder

#### Scenario: Mod folder
- **WHEN** the player picks another mod folder for ETS2
- **THEN** the step shows it and it is remembered

#### Scenario: Both games detected
- **WHEN** ETS2 and ATS are both in a Steam library
- **THEN** the step shows both as "Game found" with their paths, with no checkbox to choose

#### Scenario: No game found
- **WHEN** neither game is detected and the player sets no folder
- **THEN** Next is available and leads to the Vehicles to paint step

#### Scenario: Folder of the other game
- **WHEN** the player clicks Choose folder… for ATS and picks the ETS2 installation
- **THEN** a message says it is an ETS2 folder, the ATS row is unchanged, and no folder is saved for ATS

### Requirement: Vehicles to paint step
The second step SHALL scan the ETS2 installation when its folder is set (see the in-app-vehicle-generation change) and list the trucks and trailers the player owns, with a search by brand or model, filters **All / Trucks / Trailers** with counts, one checkbox per vehicle, **Select all**, and the number of vehicles checked. The scan SHALL run in the background, with a way to cancel it, and SHALL NOT freeze the window. When both games' folders are set, the vehicles of both SHALL be listed with their game.

**Generate and continue** SHALL generate the checked vehicles as installed packages, with progress and **Cancel**, then go to the next step when every vehicle was generated; after a Cancel or a failure the player stays on the step. Cancel SHALL stop after the current vehicle and keep those already generated. A vehicle that fails SHALL be reported by name and SHALL NOT stop the others. Generating nothing SHALL be allowed: with nothing checked, the button reads **Continue**.

When the ETS2 folder is set but cannot be read, or the ATS folder is set (ATS is not supported for generation yet), the step SHALL say so; when no folder is set, it SHALL say nothing about a missing game. In these cases and when no game is scannable, the step SHALL offer **Install the sample vehicles** (no network), and say that a vehicle can be added later with Custom Vehicle…. When the other game can be scanned, the step SHALL still list its vehicles.

#### Scenario: Owned vehicles listed
- **WHEN** the ETS2 installation has two trucks and one trailer the player owns
- **THEN** the step lists those three, the filters read All 3, Trucks 2, Trailers 1

#### Scenario: Generate
- **WHEN** the player checks two trucks and clicks Generate and continue
- **THEN** progress is shown, both are installed, and the First project step follows

#### Scenario: Cancel
- **WHEN** the player clicks Cancel while the second of three vehicles is being generated
- **THEN** generation stops, the first vehicle stays installed, and the player stays on the step

#### Scenario: Generate nothing
- **WHEN** the player unchecks every vehicle
- **THEN** the button reads Continue and goes on without generating

#### Scenario: No game folder
- **WHEN** no game folder is set
- **THEN** the step offers Install the sample vehicles and mentions Custom Vehicle…, shows no error about a missing game, and no scan runs

#### Scenario: Game not found
- **WHEN** the ETS2 folder is set but no longer exists
- **THEN** the step says ETS2 cannot be read, offers Install the sample vehicles and mentions Custom Vehicle…, and no scan runs

#### Scenario: ATS
- **WHEN** only the ATS folder is set
- **THEN** the step says generation is not supported yet for ATS and offers Install the sample vehicles

#### Scenario: A failed vehicle
- **WHEN** one of three vehicles fails to generate
- **THEN** the other two are installed, a message names the one that failed, and the player stays on the step

### Requirement: First project step
The third step SHALL close the first launch and open New Project at its Vehicle step, with the Game set to the first game whose folder is set (ETS2 then ATS) that has installed vehicles, else to the game of the installed vehicles when they are all of one game, else to the only game whose folder is set, else to ETS2, and the vehicles just generated or installed listed. With no vehicle installed, New Project SHALL open as usual with its own offers (Install the sample vehicles, Custom Vehicle…). Cancelling New Project SHALL show the home screen.

#### Scenario: Landing in New Project
- **WHEN** the player generated two ETS2 trucks and continues
- **THEN** New Project opens on ETS2, listing those two trucks

#### Scenario: Nothing installed
- **WHEN** the player generated nothing and continues
- **THEN** New Project opens with no vehicle listed and offers Install the sample vehicles and Custom Vehicle…

### Requirement: Language and keyboard
Every string of the first launch SHALL be translated in the four languages of the application and fit in German without truncation. The first launch SHALL be fully operable by keyboard (Tab to move, Space to toggle, Enter for the primary button, Escape for Skip).

#### Scenario: German labels
- **WHEN** the language is German
- **THEN** the buttons, the step names and the folder rows show their whole labels

### Requirement: Run the first launch again
Help › **Run First Launch Again…** SHALL show the first launch again, from its first step, with the current settings preselected (language and folders) and the vehicles already generated shown as such. It SHALL be available from the home screen and from a project; from a project, it SHALL first close the project as Close Project does (asking to save unsaved changes). Finishing or skipping it SHALL keep the settings as changed and return to the home screen.

#### Scenario: Run again from the home screen
- **WHEN** the player chooses Help › Run First Launch Again… on the home screen
- **THEN** the first step is shown with the current language and folders preselected

#### Scenario: Run again with unsaved changes
- **WHEN** a project with unsaved changes is open and the player chooses Run First Launch Again…
- **THEN** the save question of Close Project is asked first, and Cancel keeps the project open

#### Scenario: Skip after running again
- **WHEN** the player runs the first launch again and presses Skip on the first step
- **THEN** nothing is changed and the home screen is shown
