## Context

See proposal.md. Today: `ui/home.rs` shows the home screen (`Screen::Home`); `prefs.rs` holds `Prefs` (RON, serde defaults); `mod_export.rs` has `mod_folder_in`, `existing_mod_folder` and `destination` over `Folders`; New Project is `Modal::NewProject(NewProjectDraft)` with a game filter; `vehicles.rs` has `SAMPLES` and `install_samples`. Folder detection, scan and generation come from `in-app-vehicle-generation` as reusable pieces (detect, scan with cancel, generate with progress/Cancel).

## Goals / Non-Goals

**Goals:** a skippable, once-only three-step flow built on C's pieces, with testable pure logic.
**Non-Goals:** any detection, scan or generation logic; a game version.

## Decisions

- **Flow state in `AppState`, not a modal.** A `FirstRun` state (step, chosen games, search/filter/checks, running job) is shown by `home.rs` while `Screen::Home` and `!prefs.first_run_done`. Alternative, a `Modal`: rejected, it must replace the home screen, not float over it.
- **Preferences.** `first_run_done: bool`, `games: Vec<String>` (ids `ets2`/`ats`, ordered; the first is New Project's initial game), `mod_folders: BTreeMap<String, PathBuf>`. The installation folders are C's `game_folders: BTreeMap<String, PathBuf>` (same keys), not a field of this change. All `#[serde(default)]`; no migration (app unreleased). "Existing user" rule: at load, if the field is absent and `recent` or installed vehicles exist, set done. Alternative, one `game` and one `mod_folder`: rejected, players of both games would lose a folder.
- **Mod folder.** `Folders` gets `preferred: BTreeMap`-like lookup: `destination` tries the preferred folder (if it exists), then `existing_mod_folder` as today. Detection in the step reuses `existing_mod_folder` unchanged.
- **Language.** Reuse the Preferences language setter; the preselection is the already-detected system language and writes nothing until changed.
- **Vehicles step** reuses C's scan result type for the list and the filter/count model of the New Project list. Generation runs on C's background job; the UI only reads progress and sends Cancel. Cancel keeps vehicles already installed (C's generation installs one by one). With several games, scan each supported chosen game; ATS yields the "not supported yet" note, an unfound game the "not found" note; samples are offered when no game can be scanned.
- **Hand-off.** Finishing sets `first_run_done`, saves prefs, and opens `Modal::NewProject` with the draft's game filter set from `games[0]`. New Project's default game: first chosen game with installed vehicles, else the current rule.
- **Skip** sets done on any step, keeping saved choices (language, folders).

- **Everything stays changeable in Preferences.** The Games section (C adds the installation folders) gets "I play this game" (`games`) and the mod folder (`mod_folders`) per game, so each first-launch choice has a home afterwards. One section per game, not a section per setting, so the player sees a game's three settings together.
- **Run First Launch Again** is a Help command (command system, menus, palette). It resets nothing: it shows the flow with the current prefs preselected; from a project it goes through Close Project first, so the flow keeps its home-screen-only layout.

## Risks / Trade-offs

- A scan of a large game folder is slow → it runs in the background with Cancel and a progress indicator, never on the UI thread.
- Generation fails for some vehicles → per-vehicle error, others continue.
- Parallel changes edit the same specs → deltas copy the current text; re-sync on archive.
- German labels are long → check the step at the smallest window width.

## Open Questions

