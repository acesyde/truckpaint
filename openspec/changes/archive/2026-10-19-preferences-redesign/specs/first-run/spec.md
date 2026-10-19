## MODIFIED Requirements

### Requirement: Language and game folders step
The first step SHALL ask:
- the **interface language** under a group title, preselected from the system language. Choosing another language SHALL apply it at once and save it as Preferences does; keeping the detected one SHALL NOT save a language;
- for each game, ETS2 and ATS, a card that is the same as the Games tab of Preferences (see the Games tab layout requirement of app-preferences): the game's name with its state as a dot and words (**Game found**, **Not found** or **Folder problem**, never its version), its **installation folder**, detected as the Preferences of the app do (see the in-app-vehicle-generation change), and its **mod folder**, detected where Export Mod looks, each in a field showing the path with **Choose…** inside it and **Clear** once something is chosen, and **Detect automatically** while the installation folder is empty. Choosing a folder SHALL store it in the same preference as the game installation folder of Preferences.

The step SHALL use the same hierarchy of text as Preferences: group titles (Language, Your games) small, bold and in capitals in the secondary color, setting labels at regular size in the primary color.

The step SHALL NOT show a game version and SHALL NOT ask which games the player plays: a game is played when its installation folder is set. A game that is not found SHALL NOT block the flow, and Next SHALL always be available. A folder SHALL be accepted only for the game it holds (ETS2 or ATS, as the vehicle-generation capability checks it); a folder of the other game SHALL be refused with a message naming that game, and the previous folder SHALL be kept.

#### Scenario: Language from the system
- **WHEN** the system language is German and the first launch opens
- **THEN** the interface is in German and Deutsch is selected, and nothing is saved until the player picks a language

#### Scenario: Game found
- **WHEN** the ETS2 installation folder is detected
- **THEN** the ETS2 card shows the state "Game found" and the path in its game folder field

#### Scenario: Game not found
- **WHEN** no ATS installation folder is detected
- **THEN** the ATS card shows the state "Not found" with Choose… and Detect automatically, and Next stays available

#### Scenario: Choosing a folder
- **WHEN** the player clicks Choose… in the ETS2 game folder field and picks a folder
- **THEN** the step shows that path and Preferences shows the same folder

#### Scenario: Mod folder
- **WHEN** the player picks another mod folder for ETS2
- **THEN** the step shows it and it is remembered

#### Scenario: Both games detected
- **WHEN** ETS2 and ATS are both in a Steam library
- **THEN** the step shows both cards as "Game found" with their paths, with no checkbox to choose

#### Scenario: No game found
- **WHEN** neither game is detected and the player sets no folder
- **THEN** Next is available and leads to the Vehicles to paint step

#### Scenario: Folder of the other game
- **WHEN** the player clicks Choose… in the ATS game folder field and picks the ETS2 installation
- **THEN** a message says it is an ETS2 folder, the ATS card is unchanged, and no folder is saved for ATS

#### Scenario: Same look as Preferences
- **WHEN** the player compares the first step with the Games tab of Preferences
- **THEN** the game cards, their fields and buttons, and the titles, labels and hints look the same
