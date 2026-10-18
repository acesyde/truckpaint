## Context

`Prefs.games` (an ordered list of game ids) is the only place "plays" lives. Readers: `Prefs::plays` / `set_plays` and the Preferences checkbox (`ui/dialogs.rs`), the first launch's checkboxes, the folders it shows, its Next guard and its Vehicles step (`ui/first_run.rs`), its hand-off and `start_first_run` (which seeds `games` from the folders found, else ETS2), and New Project's initial game (`NewProjectDraft::new` in `state.rs`, called from `project_io.rs`). The installation folders live in `Prefs.game_folders`, and `game::set_folder` only stores a folder that `game::check` accepted, so a stored folder always held that game when it was chosen. See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**
- One source of truth: a game is played when `game_folders` has an entry for it.
- Keep the existing folder check as the guarantee behind that rule, and say so in the spec.

**Non-Goals:**
- Changing detection or `check`, or reading the executable.
- A "not played, keep the folder" state.

## Decisions

**`plays(game)` becomes `game_folders.contains_key(game)`; `games` is removed from `Prefs`.** Callers that took `&prefs.games` take the folder map or a small `played()` helper that returns the codes in `Game::ALL` order (ETS2, ATS). Alternative: keep `games` and derive it from the folders on load — it would keep a second field that can go stale, which is the problem being removed.

**No migration.** The tool is not released, so no preferences file in the wild has `games` that matters. Loading must still not fail on a file that has the field: the existing tolerant loading ignores unknown fields; a test pins it (spec scenario "Older preferences with a games list").

**The folder check stays as is.** `check` reads `/version.sii` from the game's own archives (`application` is `eut2` or `ats`) and refuses the other game (`games-is-ets2` / `games-is-ats`). This works the same on Windows, Linux and macOS, unlike an executable name, and gives the version too. The change only adds spec text and a first-launch test for the other-game case, because "folder set means this game" now depends on it.

**First launch: no checkboxes, both games always shown.** Step 1 runs the one-time Steam detection for both games (today only for checked ones) and shows each game's installation row and mod row. `Next` has no guard, so `first-run-need-game` and its tooltip go; `start_first_run` no longer seeds a games list. Alternative: show a game's rows only once its folder is set — a player with no detected folder could not pick one, so the rows stay.

**Vehicles step keys off folders.** ETS2 is scannable when its folder is stored and its status is valid; a stored folder that no longer reads gives the existing "cannot be made" message; no folder gives no message and the samples hint. ATS stored gives the "not supported yet" line. This replaces `plays_ets2` / `plays_ats` with folder checks.

**New Project's initial game:** keep the current order — a played game with installed vehicles (ETS2 then ATS), else the game of the installed vehicles when they are all of one game — then the only game whose folder is set, then ETS2. The hand-off from the first launch uses the same function instead of `games.first()`.

**Strings:** remove `prefs-play`, `prefs-play-hint` and `first-run-need-game` in the four locales; reword `first-run-games` (it heads the folders now, for example "Your games"); German is checked for width.

## Risks / Trade-offs

- [A player who owns both games on Steam but plays one gets both detected and "played"] → Accepted; **Clear** removes one. ATS only shows a "not supported yet" line in the Vehicles step.
- [A stored folder that was moved still counts as played] → The error already shows next to the field and in the Vehicles step; the player can Clear or Choose again.
- [Removing `games` touches many tests] → They are listed in the tasks; each is changed to set a folder instead of toggling a box.
