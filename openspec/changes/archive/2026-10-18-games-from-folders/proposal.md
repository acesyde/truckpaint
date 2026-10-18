## Why

"I play this game" repeats what the game folder already says. A player who sets the ETS2 folder plays ETS2; asking them to also check a box means two settings that can disagree (a valid folder with the box unchecked, or the box checked with no folder), a rule ("keep at least one game checked") with its own hint, and a first step that blocks Next until a box is checked even though a game that is not found must not block the flow.

## What Changes

- **A game is played when its installation folder is set.** There is no "I play this game" setting any more: not in Preferences, not in the first launch.
- **First launch, Language and game step:** the checkboxes and the "at least one game" rule go. The step shows ETS2 and ATS, each with its installation folder (detected, or **Choose Folder…**) and its mod folder. Both games are detected. **Next** is never blocked.
- **Preferences, Games tab:** the "I play this game" checkbox and its hint go; for each game the folder and the mod folder stay. To stop playing a game, the player uses **Clear** on its folder.
- **A folder is only accepted for the game it holds.** The check that exists today (the game's own archives say ETS2 or ATS) is kept and becomes a rule of the spec: a folder of the other game is refused with a message, in the first launch as in Preferences, so that "folder set" can be trusted to mean "this game".
- **Vehicles to paint step:** it scans ETS2 when its folder is set and valid, says so when ATS is set (not supported yet), and no longer reports ETS2 as missing when it was never set; it then offers the sample vehicles and Custom Vehicle….
- **New Project's first game:** the first game, ETS2 then ATS, that has installed vehicles; else the only game whose folder is set; else ETS2. The first launch hands off to New Project the same way.
- **Preferences file:** the list of games played is no longer written. An older file that has it still loads and the list is ignored (the tool is not released yet, so nothing else is migrated).
- **Docs:** the first-launch page and the Preferences pages stop talking about a game checkbox; the first-launch and Preferences screenshots are rendered again.

Non-goals:
- ATS vehicle generation;
- changing how folders are detected or checked;
- a way to mark a game as "not played" while keeping its folder.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `first-run`: the Language and game step shows both games with their folders and no checkboxes; the Vehicles to paint and First project steps follow the folders; Run First Launch Again preselects the folders.
- `app-preferences`: no "I play this game" setting; a game is played when its folder is set; a folder of the other game is refused; the stored list of games is gone.

## Impact

- **App:** the games list in preferences and its helpers (`crates/tp-app/src/prefs.rs`), the first launch (`ui/first_run.rs`, including its hand-off to New Project), New Project's initial game (`state.rs`, `project_io.rs`), and the Games tab of Preferences (`ui/dialogs.rs`).
- **Strings:** `prefs-play`, `prefs-play-hint` and `first-run-need-game` are removed and `first-run-games` is reworded, in en, fr, es and de; German must fit.
- **Tests:** headless UI tests that check or uncheck a game, the first-launch tests, the preferences tests and the screenshot and doc media scenarios that show these screens.
- **Docs:** `getting-started/first-launch`, and the pages that mention the Games tab, in four languages; the `getting-started` media.
