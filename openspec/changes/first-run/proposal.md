## Why

A new player opens TruckPaint on an empty home screen: no vehicle installed, no idea that packages exist, and New Project stops at an empty Vehicle step. Once `marketplace` can browse and install community packages, the first launch can ask the three things the app needs (language, game, which vehicles) and land in a first project, so the player paints within a minute instead of hunting for `.tpv` files.

## What Changes

- **A first launch in three steps,** shown in place of the home screen the first time TruckPaint starts. A step indicator shows where the player is; Back returns to the previous step; **Skip** on every step goes to the home screen. Once finished or skipped, it is not shown again; the language stays in Preferences.
  1. **Language and game.** The interface language, preselected from the system (choosing another one saves it, as Preferences does; keeping the detected one doesn't). The game, ETS2 or ATS. The game's mod folder, detected where Export Mod already looks (`Documents/<game>/mod` on Windows, the user's data folder on macOS and Linux), shown as "Game folder found" with its path, or "not found"; **Change folder…** picks another one.
  2. **Which vehicles to paint.** The packages of the chosen game from the marketplace's catalog: search by brand or model, filters All / Trucks / Trailers with counts, checkboxes with Select all, each with its kind, content, version and download size, and the total of the selection. **Install and continue** installs the checked packages, with progress, then goes on. A note says that a vehicle without a package can be added later with Custom vehicle…. Installing nothing is allowed.
  3. **First project.** Goes straight into New Project, at the Vehicle step, filtered on the chosen game and showing the vehicles just installed.
- **Offline or without the catalog,** step 2 says the catalog can't be reached and offers **Install the sample vehicles**, which needs no network; the rest of the flow works the same.
- **Preferences remember** that the first launch is done, the chosen game (New Project's game filter starts on it), and the mod folder when the player changed it.
- **Export Mod uses the chosen mod folder:** the save dialog opens there when it was set, else in the detected folder as today.
- **No game version.** The mockup shows "game detected — version 1.53". The mod folder holds no version. The game writes its version in `game.log.txt`, next to `mod/`, but only for its last launch: the file is missing before the first launch, and stale after an update until the game runs again. The install folder (Steam libraries) would be needed for a reliable answer. This change shows no version; the project's Game versions stay typed by the player.

Non-goals:
- the marketplace itself (index, catalog format, download, install, updates): this change only uses what `marketplace` provides;
- detecting the game installation or its version;
- a tour of the workspace.

## Capabilities

### New Capabilities
- `first-run`: when the first launch is shown and how it ends (finish or skip, never again), its three steps, the mod folder detection and Change folder…, the package selection from the catalog, the offline path with the sample vehicles, and the hand-off to New Project.

### Modified Capabilities
- `start-screen`: "Home screen on launch" shows the first launch instead of the home screen the first time; "New Project dialog" starts its game filter on the chosen game.
- `app-preferences`: "Persisted preferences" adds the first launch being done, the chosen game and the chosen mod folder.
- `mod-export`: "Destination" opens the save dialog in the mod folder chosen at first launch when there is one.
- `vehicle-packages`: "Built-in sample vehicle" adds the first launch's step 2 to the places offering Install the sample vehicles.

## Impact

- **Depends on `marketplace`**: reuses its catalog client (`tp-app/src/marketplace.rs`: download, offline cache, size and SHA-256 checks, sequential install with Cancel) and its list model (`ui/marketplace_tab.rs`: `rows`, statuses, sizes), without the Vehicle Library dialog. The catalog's sample packages make "Install the sample vehicles" and the online list consistent. It also uses `new-design`'s look.
- **tp-app:**
  - `prefs.rs`: `first_run_done`, `game`, `mod_folder` (optional fields; files without them load as today, and an existing user with recent projects or installed vehicles is treated as having done the first launch);
  - `ui/home.rs` and a new `ui/first_run.rs`: the three steps;
  - `mod_export.rs`: `existing_mod_folder` honours the preferred folder;
  - the New Project dialog: an initial game filter.
- **tp-i18n:** the flow's strings in en/fr/es/de.
- **Dependencies:** none beyond what `marketplace` adds.

Open questions:
- whether the chosen game should matter beyond the first launch (a default for New Project only, or also for the marketplace's filters);
- where Change folder… lives afterwards (Preferences or the Export Mod dialog), so a wrong choice can be fixed.
