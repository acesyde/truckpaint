//! The first launch: when it is shown, its three steps on a made-up game
//! folder, and Run First Launch Again.

mod common;

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use egui::Vec2;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::mod_export::Folders;
use tp_app::prefs::{Prefs, RecentProject};
use tp_app::state::Modal;
use tp_app::vehicles::VehicleLibrary;
use tp_pack::synthetic::{Vehicle, write_game};
use tp_vehicles::Game;

type H = Harness<'static, AppState>;

fn vehicles() -> Vec<Vehicle<'static>> {
    vec![
        Vehicle::truck("example.alpha", "Alpha"),
        Vehicle::truck("example.beta", "Beta"),
        Vehicle::trailer("example.box", "Box"),
    ]
}

fn game_folder(dir: &Path) -> std::path::PathBuf {
    let game = dir.join("game");
    std::fs::create_dir(&game).unwrap();
    write_game(&game, "eut2", "1.61.1.9", &vehicles());
    game
}

/// The app on its first launch, with an empty library, no game in Steam
/// and `game` as the ETS2 folder.
fn app(dir: &Path, game: Option<&Path>) -> H {
    let mut prefs = Prefs::default();
    if let Some(game) = game {
        prefs.game_folders.insert("ets2".into(), game.to_path_buf());
    }
    let mut state = AppState::with_prefs(prefs, None);
    state.vehicles = VehicleLibrary::open(&dir.join("library"));
    state.mod_folders = Folders {
        documents: Some(dir.join("Documents")),
        data: Some(dir.join("data")),
        home: Some(dir.to_path_buf()),
    };
    // Nothing is found in the Steam of the machine running the tests.
    for game in Game::ALL {
        state.game.detected.insert(game, None);
    }
    state.begin_first_run_if_needed();
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 900.0))
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.run();
    h
}

fn until(h: &mut H, what: &str, done: impl Fn(&H) -> bool) {
    for _ in 0..2000 {
        h.run_steps(1);
        if done(h) {
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("never: {what}");
}

fn shows(h: &H, label: &str) -> bool {
    h.query_by_label(label).is_some()
}

/// Clicks the Choose… of a folder field, named by its game and folder.
fn choose(h: &mut H, name: &str) {
    h.get_by_label(&format!("Choose…, {name}")).click();
    h.run();
}

/// What a folder field shows ("Euro Truck Simulator 2 game folder").
fn field(h: &H, name: &str) -> String {
    h.get_by_role_and_label(Role::TextInput, name)
        .value()
        .unwrap_or_default()
}

fn click(h: &mut H, label: &str) {
    h.get_by_label(label).click();
    // A click can start the scan, whose spinner repaints until it ends.
    let _ = h.run_ok();
}

/// Next, then the scan of the made-up game.
fn to_vehicles(h: &mut H) {
    click(h, "Next");
    until(h, "the scan", |h| shows(h, "All 3"));
}

#[test]
fn shown_once_and_skipped_for_existing_users() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path(), None);
    assert!(shows(&h, "Welcome to TruckPaint"));
    assert!(!shows(&h, "Recent projects"));
    // Skip: the home screen, and not again.
    click(&mut h, "Skip");
    assert!(shows(&h, "Recent projects"));
    assert!(h.state().prefs.first_run_done);
    assert!(h.state().first_run.is_none());
    h.state_mut().begin_first_run_if_needed();
    assert!(h.state().first_run.is_none());

    // Someone with recent projects has done it.
    let mut state = AppState::with_prefs(
        Prefs {
            recent: vec![RecentProject {
                name: "ACE".into(),
                path: "/tmp/ace.truckpaint".into(),
                last_opened: 1,
            }],
            ..Prefs::default()
        },
        None,
    );
    state.vehicles = VehicleLibrary::open(&dir.path().join("other"));
    state.begin_first_run_if_needed();
    assert!(state.first_run.is_none() && state.prefs.first_run_done);
    // Someone with installed vehicles too.
    let mut state = AppState::with_prefs(Prefs::default(), None);
    state.vehicles = common::sample_library();
    state.begin_first_run_if_needed();
    assert!(state.first_run.is_none());
}

#[test]
fn the_first_step_shows_the_language_the_games_and_their_folders() {
    let dir = tempfile::tempdir().unwrap();
    let game = game_folder(dir.path());
    let mods = dir.path().join("MyMods");
    std::fs::create_dir(&mods).unwrap();
    let mut h = app(dir.path(), Some(&game));
    // The language is the system's; nothing is saved.
    h.get_by_role_and_label(Role::ComboBox, "Language");
    assert_eq!(h.state().prefs.language, None);
    assert_eq!(h.state().prefs.played(), ["ets2"]);
    assert!(shows(&h, "Game found"));
    assert_eq!(
        field(&h, "Euro Truck Simulator 2 game folder"),
        game.display().to_string()
    );
    // Both games show a mod folder field, none found yet.
    assert_eq!(field(&h, "Euro Truck Simulator 2 mod folder"), "");
    assert_eq!(field(&h, "American Truck Simulator mod folder"), "");
    // The mod folder is chosen, and remembered.
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        folders: [mods.clone()].into(),
        ..Default::default()
    });
    choose(&mut h, "Euro Truck Simulator 2 mod folder");
    assert_eq!(h.state().prefs.mod_folders["ets2"], mods);
    assert_eq!(
        field(&h, "Euro Truck Simulator 2 mod folder"),
        mods.display().to_string()
    );
    // A game that is not found asks for its folder, which Preferences shares.
    assert!(shows(&h, "American Truck Simulator") && shows(&h, "Not found"));
    // The ETS2 folder is refused for ATS: nothing is stored for it.
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        folders: [game.clone()].into(),
        ..Default::default()
    });
    choose(&mut h, "American Truck Simulator game folder");
    assert!(shows(&h, "This is a Euro Truck Simulator 2 folder."));
    assert!(!h.state().prefs.game_folders.contains_key("ats"));
    let ats = dir.path().join("ats");
    std::fs::create_dir(&ats).unwrap();
    write_game(&ats, "ats", "1.55.0.1", &[]);
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        folders: [ats.clone()].into(),
        ..Default::default()
    });
    choose(&mut h, "American Truck Simulator game folder");
    assert_eq!(h.state().prefs.game_folders["ats"], ats);
    assert_eq!(h.state().prefs.played(), ["ets2", "ats"]);
    // No version is shown.
    assert!(!shows(&h, "1.61.1.9"));
}

#[test]
fn no_game_found_does_not_block_next() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path(), None);
    assert!(h.state().prefs.played().is_empty());
    assert!(!h.get_by_label("Next").accesskit_node().is_disabled());
    click(&mut h, "Next");
    // No game was set: no error about a missing game, the samples are offered.
    assert!(!shows(
        &h,
        "Euro Truck Simulator 2 was not found, so its vehicles cannot be made."
    ));
    assert!(shows(&h, "Install the sample vehicles"));
}

#[test]
fn a_saved_ets2_folder_that_is_gone_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("gone");
    let mut h = app(dir.path(), Some(&gone));
    assert_eq!(h.state().prefs.played(), ["ets2"]);
    click(&mut h, "Next");
    assert!(shows(
        &h,
        "Euro Truck Simulator 2 was not found, so its vehicles cannot be made."
    ));
    assert!(shows(&h, "Install the sample vehicles"));
}

#[test]
fn the_other_games_folder_is_refused_for_ets2() {
    let dir = tempfile::tempdir().unwrap();
    let ats = dir.path().join("ats");
    std::fs::create_dir(&ats).unwrap();
    write_game(&ats, "ats", "1.55.0.1", &[]);
    let mut h = app(dir.path(), None);
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        folders: [ats].into(),
        ..Default::default()
    });
    choose(&mut h, "Euro Truck Simulator 2 game folder");
    assert!(shows(&h, "This is an American Truck Simulator folder."));
    assert!(h.state().prefs.played().is_empty());
}

#[test]
fn another_language_applies_and_is_saved() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path(), None);
    h.get_by_role_and_label(Role::ComboBox, "Language").click();
    h.run();
    h.get_by_label("Deutsch").click();
    h.run();
    assert_eq!(
        h.state().prefs.language,
        Some("de".to_owned()),
        "chosen language is saved"
    );
    assert!(shows(&h, "Willkommen bei TruckPaint"));
}

#[test]
fn back_goes_to_the_first_step_with_the_choices_kept() {
    let dir = tempfile::tempdir().unwrap();
    let game = game_folder(dir.path());
    let mut h = app(dir.path(), Some(&game));
    to_vehicles(&mut h);
    click(&mut h, "Back");
    assert!(shows(&h, "Welcome to TruckPaint"));
    assert!(shows(&h, "Game found"));
    assert_eq!(h.state().prefs.played(), ["ets2"]);
}

#[test]
fn the_owned_vehicles_are_listed_filtered_and_generated() {
    let dir = tempfile::tempdir().unwrap();
    let game = game_folder(dir.path());
    let mut h = app(dir.path(), Some(&game));
    to_vehicles(&mut h);
    assert!(shows(&h, "Trucks 2") && shows(&h, "Trailers 1"));
    assert!(shows(&h, "Example Alpha") && shows(&h, "Example Box"));
    click(&mut h, "Trailers 1");
    assert!(!shows(&h, "Example Alpha") && shows(&h, "Example Box"));
    click(&mut h, "All 3");
    // Nothing checked: Continue.
    assert!(shows(&h, "Continue") && !shows(&h, "Generate and Continue"));
    click(&mut h, "Select All");
    assert!(shows(&h, "3 checked"));
    h.get_by_label("Example Beta").click();
    h.run();
    assert!(shows(&h, "2 checked"));
    click(&mut h, "Generate and Continue");
    until(&mut h, "the hand-off", |h| h.state().first_run.is_none());
    h.run();
    assert_eq!(h.state().vehicles.vehicles().len(), 2);
    assert!(h.state().prefs.first_run_done);
    // New Project opens on ETS2.
    let Some(Modal::NewProject(draft)) = &h.state().modal else {
        panic!("New Project is open");
    };
    assert_eq!(draft.game(), Game::Ets2);
    assert!(shows(&h, "Example Alpha") && shows(&h, "Example Box"));
}

#[test]
fn generating_nothing_goes_on() {
    let dir = tempfile::tempdir().unwrap();
    let game = game_folder(dir.path());
    let mut h = app(dir.path(), Some(&game));
    to_vehicles(&mut h);
    click(&mut h, "Continue");
    assert!(h.state().first_run.is_none());
    assert!(h.state().vehicles.vehicles().is_empty());
    assert!(matches!(h.state().modal, Some(Modal::NewProject(_))));
    assert!(shows(&h, "Install the sample vehicles"));
}

#[test]
fn cancel_keeps_the_generated_vehicles_and_stays_on_the_step() {
    let dir = tempfile::tempdir().unwrap();
    let game = game_folder(dir.path());
    let mut h = app(dir.path(), Some(&game));
    to_vehicles(&mut h);
    click(&mut h, "Select All");
    // The worker waits after each event until the test lets it go.
    let (release, gate) = mpsc::channel::<()>();
    {
        let s = h.state_mut();
        s.game.generate(&s.vehicles, move || {
            let _ = gate.recv();
        });
        s.first_run.as_mut().unwrap().generating = true;
    }
    until(&mut h, "the first vehicle starts", |h| {
        shows(h, "In progress")
    });
    assert!(shows(&h, "0 of 3"));
    click(&mut h, "Cancel");
    drop(release);
    until(&mut h, "the end", |h| h.state().game.run.is_none());
    h.run();
    assert!(h.state().first_run.is_some(), "stays on the step");
    assert_eq!(h.state().vehicles.vehicles().len(), 1);
    assert!(shows(
        &h,
        "Cancelled. Installed: 1 · Up to date: 0 · Failed: 0"
    ));
}

#[test]
fn a_failed_vehicle_is_named_and_the_others_are_installed() {
    let dir = tempfile::tempdir().unwrap();
    let game = game_folder(dir.path());
    let mut h = app(dir.path(), Some(&game));
    to_vehicles(&mut h);
    // Alpha is installed, then its file is damaged: making it again fails.
    h.get_by_label("Example Alpha").click();
    h.run();
    click(&mut h, "Generate and Continue");
    until(&mut h, "the hand-off", |h| h.state().first_run.is_none());
    let path = h
        .state()
        .vehicles
        .get("official.ets2.example.alpha")
        .unwrap()
        .newest()
        .path
        .clone();
    std::fs::write(&path, b"damaged").unwrap();
    h.state_mut().modal = None;
    h.state_mut().start_first_run(true);
    h.run();
    to_vehicles(&mut h);
    h.get_by_label("Example Alpha").click();
    h.get_by_label("Example Beta").click();
    h.run();
    click(&mut h, "Generate and Continue");
    until(&mut h, "the run", |h| h.state().game.run.is_none());
    h.run();
    assert!(h.state().first_run.is_some(), "stays to show the failure");
    assert!(shows(&h, "Installed: 1 · Up to date: 0 · Failed: 1"));
    assert!(h.query_by_label_contains("Example Alpha: ").is_some());
    assert!(
        h.state()
            .vehicles
            .get("official.ets2.example.beta")
            .is_some()
    );
}

#[test]
fn without_the_game_the_samples_are_offered() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path(), None);
    click(&mut h, "Next");
    assert!(shows(&h, "Install the sample vehicles"));
    assert!(h.state().game.scanned().is_none());
    click(&mut h, "Install the sample vehicles");
    assert_eq!(h.state().vehicles.vehicles().len(), 2);
    assert!(!shows(&h, "Install the sample vehicles"));
    click(&mut h, "Continue");
    assert!(matches!(h.state().modal, Some(Modal::NewProject(_))));
    assert!(shows(&h, "TruckPaint Sample Truck"));
}

#[test]
fn ats_is_not_generated_yet() {
    let dir = tempfile::tempdir().unwrap();
    let ats = dir.path().join("ats");
    std::fs::create_dir(&ats).unwrap();
    write_game(&ats, "ats", "1.55.0.1", &[]);
    let mut h = app(dir.path(), None);
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        folders: [ats].into(),
        ..Default::default()
    });
    choose(&mut h, "American Truck Simulator game folder");
    assert_eq!(h.state().prefs.played(), ["ats"]);
    click(&mut h, "Next");
    assert!(shows(
        &h,
        "Generating American Truck Simulator vehicles is not supported yet."
    ));
    assert!(shows(&h, "Install the sample vehicles"));
    click(&mut h, "Continue");
    // Nothing of ETS2 was asked for, and New Project opens on ATS.
    let Some(Modal::NewProject(draft)) = &h.state().modal else {
        panic!("New Project is open");
    };
    assert_eq!(draft.game(), Game::Ats);
}

#[test]
fn run_again_shows_the_current_settings_and_ends_on_the_home_screen() {
    let dir = tempfile::tempdir().unwrap();
    let game = game_folder(dir.path());
    let mut h = app(dir.path(), Some(&game));
    click(&mut h, "Skip");
    h.state_mut().queue.push(CommandId::RunFirstLaunch);
    h.run_steps(3);
    h.run();
    assert!(shows(&h, "Welcome to TruckPaint"));
    assert!(shows(&h, "Game found"));
    // Skip changes nothing and shows the home screen.
    let before = h.state().prefs.clone();
    click(&mut h, "Skip");
    assert_eq!(h.state().prefs, before);
    assert!(shows(&h, "Recent projects"));
    // Finishing returns to the home screen too.
    h.state_mut().queue.push(CommandId::RunFirstLaunch);
    h.run_steps(3);
    h.run();
    to_vehicles(&mut h);
    click(&mut h, "Continue");
    assert!(h.state().modal.is_none());
    assert!(shows(&h, "Recent projects"));
}

#[test]
fn run_again_from_a_project_asks_about_unsaved_changes_first() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path(), None);
    click(&mut h, "Skip");
    common::create_project(&mut h);
    h.state_mut().queue.push(CommandId::RunFirstLaunch);
    h.run_steps(3);
    assert!(matches!(h.state().modal, Some(Modal::UnsavedChanges(_))));
    click(&mut h, "Cancel");
    assert!(h.state().has_project());
    assert!(h.state().first_run.is_none());
    h.state_mut().queue.push(CommandId::RunFirstLaunch);
    h.run_steps(3);
    click(&mut h, "Don't Save");
    assert!(!h.state().has_project());
    assert!(shows(&h, "Welcome to TruckPaint"));
}
