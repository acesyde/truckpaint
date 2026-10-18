## Why

Official truck and trailer templates belong to SCS Software, and parts of them come from DLC, so TruckPaint stops sharing packages made from the game (see `remove-marketplace`). Each player must be able to make them from their own game, for their own use, without a command line or extra tools. Today that needs the `tpv extract` command and ConverterPIX.

## What Changes

- **Game folders in Preferences:** one folder for ETS2 and one for ATS. The app finds them by itself from the Steam libraries (Windows, Linux including Flatpak Steam, macOS), and the player can pick a folder by hand (for GOG, Epic or any other place). A folder is checked: the game's main files must be there, and the game and its version are read from them.
- **A "My Game" tab in the Vehicle Library,** next to Installed and My vehicles. It reads the chosen game and lists the official trucks and trailers the player owns (only installed DLC show up). Each has a status: **Not generated**, **Generated** (with its version) or **Game updated** (the game changed since).
- **Check the vehicles, then Generate:** a list of checkboxes (Select All, Select Updates), and one **Generate (n)** button that generates new vehicles and regenerates outdated ones. They are made **one after another**, with a progress bar ("3 of 12"), the vehicle being made, and each row's state (waiting, in progress, installed, failed), and **Cancel**. It runs in the background, so the app stays usable. Each result is installed like any package.
- **Versions follow the game.** A package gets its version from the game's version (game 1.61.1.9 gives 1.61.1). When the game changes a template, the new package gets a higher template version, so projects see "template updated" (`template-update-impact`); templates that did not change keep theirs.
- **ATS can be set, but not generated yet.** The folder is accepted and checked, and the tab says that ATS generation is not supported yet.
- **Everything stays on the computer.** Packages are never uploaded and carry the license "Personal modding only, not for sale. Game content belongs to SCS Software.".
- **Reusable pieces** (find the games, scan one, generate one vehicle) have no dialog of their own, so the first-launch flow (`first-run`) can use them.

Non-goals:
- ATS generation (extraction only understands ETS2 today);
- reading the game (`native-game-reader` does it) and removing the Marketplace (`remove-marketplace`);
- the first-launch flow (`first-run`);
- generating in the background without the player asking, or watching the game folder for updates;
- exporting or sharing generated packages (the existing Export keeps its reminder).

## Capabilities

### New Capabilities
- `vehicle-generation`: finding and checking a game folder, scanning it, the My Game tab, generating packages (progress, cancel, errors), versions and template versions, and what stays local.

### Modified Capabilities
- `app-preferences`: the ETS2 and ATS game folders.
- `vehicle-packages`: the Vehicle Library dialog gets the My Game tab.

## Impact

- **tp-pack:** `extract` becomes a library API that does not print or create folders: scan a game (version, vehicles with kind and name), generate one vehicle's package in memory from the previous installed package. The `tpv extract` command keeps working on top of it. Depends on `native-game-reader`.
- **tp-app:**
  - `prefs.rs`: two optional folders;
  - a new `game.rs`: Steam library detection, folder check, scan and generation worker (mpsc + thread, like `custom_vehicles.rs`);
  - `ui/dialogs.rs`: Preferences section; `ui/vehicle_dialogs.rs`: the My Game tab;
  - installation reuses `VehicleLibrary::install_bytes`.
- **tp-i18n:** strings in en/fr/es/de.
- **Documentation:** `vehicles/library.mdx`, `vehicles/official-vehicles.mdx`, `getting-started/first-project.mdx` in four languages; new doc media.
- **No compatibility work:** the app is not released.
