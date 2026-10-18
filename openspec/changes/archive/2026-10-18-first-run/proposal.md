## Why

A new player opens TruckPaint on an empty home screen: no vehicle, and no idea that vehicles come from their own game. Now that the app generates vehicles from the player's installed game (`in-app-vehicle-generation`), the first launch can ask what the app needs (language, game, which vehicles) and land in a first project, so the player paints within minutes.

## What Changes

- **A first launch in three steps,** shown in place of the home screen the first time TruckPaint starts. A step indicator shows where the player is; Back goes to the previous step; **Skip** on every step goes to the home screen. Once finished or skipped, it is not shown again.
  1. **Language and game.** The language, preselected from the system (choosing another one saves it; keeping the detected one doesn't). The game or games the player plays (ETS2, ATS, or both). For each game, the installation folder, found automatically: "Game found" with its path, or "Not found" with **Choose folder…**. The mod folder, found where Export Mod already looks, with **Choose folder…**.
  2. **Vehicles to paint.** The trucks and trailers the player owns, read from their game: search, filters All / Trucks / Trailers with counts, checkboxes and Select all. **Generate and continue** makes the checked vehicles on the player's computer, with progress and Cancel. Generating nothing is allowed. If the game is not found, or is ATS (not supported yet), the step offers **Install the sample vehicles**, and says Custom Vehicle… can add one later.
  3. **First project.** Opens New Project at the Vehicle step, on the chosen game, with the new vehicles listed.
- **Preferences remember** that the first launch is done, the chosen games, and the mod folder of each game. The game folders are the ones Preferences already has.
- **Export Mod uses the chosen mod folder** when there is one, else the detected folder as today. New Project starts on the chosen game.
- Someone with recent projects or installed vehicles is treated as having done the first launch.
- No game version is shown.

Non-goals:
- detecting or generating itself (see `in-app-vehicle-generation` and `native-game-reader`);
- a tour of the workspace.

## Capabilities

### New Capabilities
- `first-run`: when the first launch is shown and how it ends, its three steps, folder choices, the vehicle selection and generation, the sample path, and the hand-off to New Project.

### Modified Capabilities
- `start-screen`: "Home screen on launch" shows the first launch the first time; "New Project dialog" starts on the chosen game.
- `app-preferences`: "Persisted preferences" adds the first launch being done, the chosen games and mod folders.
- `mod-export`: "Destination" prefers the chosen mod folder.
- `vehicle-packages`: "Built-in sample vehicle" adds the first launch as a place offering Install the sample vehicles.

## Impact

- **Depends on `in-app-vehicle-generation`** (and so `native-game-reader`): reuses its folder detection, scan and generation (with progress and Cancel) outside the dialog. `remove-marketplace` removes the catalog this change used to rely on; the Marketplace is not used here.
- **tp-app:** `prefs.rs` (`first_run_done`, `games`, `mod_folders`, optional with defaults); a new `ui/first_run.rs`, shown by `ui/home.rs`/`state.rs`; `mod_export.rs` (preferred folder); New Project (initial game).
- **tp-i18n:** strings in en/fr/es/de. **Docs:** getting-started pages in four languages and media.
- **Conflicts to watch:** `app-preferences`, `vehicle-packages` and `start-screen` are also touched by the parallel changes; merge their deltas when archiving.
