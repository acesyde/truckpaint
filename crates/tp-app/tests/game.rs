//! The Games section of Preferences and the My Game tab, on a made-up game folder.

mod common;

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use egui::Vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::prefs::Prefs;
use tp_app::state::Modal;
use tp_app::vehicles::VehicleLibrary;
use tp_pack::synthetic::{Vehicle, write_game};

type H = Harness<'static, AppState>;

fn vehicles() -> Vec<Vehicle<'static>> {
    vec![
        Vehicle::truck("example.alpha", "Alpha"),
        Vehicle::truck("example.beta", "Beta"),
        Vehicle::trailer("example.box", "Box"),
        Vehicle::trailer("example.flat", "Flat").in_archive("dlc_flat"),
    ]
}

/// The app with an empty library and `game` as the ETS2 folder.
fn app(dir: &Path, game: Option<&Path>) -> H {
    let mut prefs = Prefs::default();
    if let Some(game) = game {
        prefs.game_folders.insert("ets2".into(), game.to_path_buf());
    }
    let mut state = AppState::with_prefs(prefs, None);
    state.vehicles = VehicleLibrary::open(&dir.join("library"));
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 900.0))
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.run();
    h
}

#[test]
fn preferences_from_my_game_opens_the_games_tab() {
    let temp = tempfile::tempdir().unwrap();
    let mut h = app(temp.path(), None);
    open_my_game(&mut h);
    h.get_by_label("Preferences…").click();
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::Preferences)));
    assert_eq!(h.state().preferences_tab, tp_app::state::PrefsTab::Games);
    assert_eq!(count(&h, "I play this game"), 0);
}

fn open_my_game(h: &mut H) {
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    h.get_all_by_label("My Game").next().unwrap().click();
    h.run();
}

/// Frames until `done` holds (workers answer from other threads).
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

fn count(h: &H, label: &str) -> usize {
    h.query_all_by_label(label).count()
}

#[test]
fn preferences_show_the_first_launch_choices_without_a_play_checkbox() {
    let temp = tempfile::tempdir().unwrap();
    let mods = temp.path().join("mods");
    let prefs = Prefs {
        mod_folders: [("ets2".to_owned(), mods.clone())].into(),
        ..Prefs::default()
    };
    let mut state = AppState::with_prefs(prefs, None);
    state.vehicles = VehicleLibrary::open(&temp.path().join("library"));
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 900.0))
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.state_mut()
        .open_preferences(tp_app::state::PrefsTab::Games);
    h.run();
    assert!(shows(&h, &mods.display().to_string()));
    // A game is played when its folder is set: there is nothing to check.
    assert_eq!(count(&h, "I play this game"), 0);
}

#[test]
fn preferences_set_check_refuse_and_clear_the_game_folders() {
    let temp = tempfile::tempdir().unwrap();
    let game = temp.path().join("ets2");
    std::fs::create_dir(&game).unwrap();
    write_game(&game, "eut2", "1.61.1.9", &vehicles());
    let ats = temp.path().join("ats");
    std::fs::create_dir(&ats).unwrap();
    write_game(&ats, "ats", "1.55.0.1", &[]);
    let nothing = temp.path().join("empty");
    std::fs::create_dir(&nothing).unwrap();
    let mut h = app(temp.path(), None);
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        folders: [game.clone(), nothing.clone(), ats.clone()].into(),
        ..Default::default()
    });
    h.state_mut()
        .open_preferences(tp_app::state::PrefsTab::Games);
    h.run();
    assert!(count(&h, "Games") >= 1, "the Games tab and its section");
    // The first Choose… is Euro Truck Simulator 2's.
    h.get_all_by_label("Choose…").next().unwrap().click();
    h.run();
    assert_eq!(h.state().prefs.game_folders["ets2"], game);
    assert!(shows(&h, "Version 1.61.1.9"));
    // A folder that holds no game is refused with its reason; the folder stays.
    h.get_all_by_label("Choose…").next().unwrap().click();
    h.run();
    assert_eq!(h.state().prefs.game_folders["ets2"], game);
    assert!(shows(&h, "base.scs is missing in this folder."));
    // An ATS folder given as ETS2's.
    h.get_all_by_label("Choose…").next().unwrap().click();
    h.run();
    assert!(shows(&h, "This is an American Truck Simulator folder."));
    // Reset to defaults, on the General tab, keeps the folders.
    h.state_mut().preferences_tab = tp_app::state::PrefsTab::General;
    h.run();
    h.get_by_label("Reset to defaults").click();
    h.run();
    assert_eq!(h.state().prefs.game_folders.len(), 1);
    // The saved folder disappears: a message, and Clear still works.
    std::fs::remove_dir_all(&game).unwrap();
    h.state_mut()
        .open_preferences(tp_app::state::PrefsTab::Games);
    h.run();
    assert!(shows(&h, "This folder cannot be found."));
    h.get_all_by_label("Clear").next().unwrap().click();
    h.run();
    assert!(h.state().prefs.game_folders.is_empty());
    assert!(!shows(&h, "This folder cannot be found."));
}

#[test]
fn my_game_lists_generates_and_regenerates_after_a_game_update() {
    let temp = tempfile::tempdir().unwrap();
    let game = temp.path().join("game");
    std::fs::create_dir(&game).unwrap();
    write_game(&game, "eut2", "1.61.1.9", &vehicles());
    let mut h = app(temp.path(), Some(&game));
    open_my_game(&mut h);
    until(&mut h, "the scan", |h| count(h, "Not generated") == 4);
    assert!(shows(&h, "Euro Truck Simulator 2 1.61.1.9"));
    assert!(shows(&h, "Generate (0)"));
    // Check three and generate them.
    for name in ["Example Alpha", "Example Box", "Example Flat"] {
        h.get_by_label(name).click();
    }
    h.run();
    h.get_by_label("Generate (3)").click();
    until(&mut h, "the run", |h| h.state().game.run.is_none());
    h.run();
    assert!(shows(&h, "Installed: 3 · Up to date: 0 · Failed: 0"));
    assert_eq!(count(&h, "Generated 1.61.1"), 3);
    assert_eq!(count(&h, "Not generated"), 1);
    let ids = h.state().vehicles.vehicles().len();
    assert_eq!(ids, 3);
    // The game moves to the next series: the three are Game updated.
    write_game(&game, "eut2", "1.62.0.3", &vehicles());
    h.get_by_label("Refresh").click();
    until(&mut h, "the rescan", |h| count(h, "Game updated") == 3);
    h.get_by_label("Select Updates").click();
    h.run();
    h.get_by_label("Generate (3)").click();
    until(&mut h, "the run", |h| h.state().game.run.is_none());
    h.run();
    assert_eq!(count(&h, "Generated 1.62.0"), 3);
    let alpha = h
        .state()
        .vehicles
        .get("official.ets2.example.alpha")
        .unwrap();
    assert_eq!(alpha.versions.len(), 2);
}

#[test]
fn progress_names_the_vehicle_and_cancel_keeps_the_installed_ones() {
    let temp = tempfile::tempdir().unwrap();
    let game = temp.path().join("game");
    std::fs::create_dir(&game).unwrap();
    write_game(&game, "eut2", "1.61.1.9", &vehicles());
    let mut h = app(temp.path(), Some(&game));
    open_my_game(&mut h);
    until(&mut h, "the scan", |h| count(h, "Not generated") == 4);
    h.get_by_label("Select All").click();
    h.run();
    // The worker waits after each event until the test lets it go.
    let (release, gate) = mpsc::channel::<()>();
    {
        let s = h.state_mut();
        s.game.generate(&s.vehicles, move || {
            let _ = gate.recv();
        });
    }
    until(&mut h, "the first vehicle starts", |h| {
        shows(h, "In progress")
    });
    assert!(shows(&h, "0 of 4"));
    assert!(shows(&h, "Making Example Alpha…"));
    assert_eq!(count(&h, "Waiting"), 3);
    // Cancel while it is made: it ends, no other starts.
    h.get_by_label("Cancel").click();
    h.run();
    assert!(shows(&h, "Stopping after Example Alpha…"));
    drop(release);
    until(&mut h, "the end", |h| h.state().game.run.is_none());
    h.run();
    assert!(shows(
        &h,
        "Cancelled. Installed: 1 · Up to date: 0 · Failed: 0"
    ));
    assert_eq!(h.state().vehicles.vehicles().len(), 1);
}

#[test]
fn ats_is_checked_but_not_generated_and_an_unset_game_asks_for_a_folder() {
    let temp = tempfile::tempdir().unwrap();
    let ats = temp.path().join("ats");
    std::fs::create_dir(&ats).unwrap();
    write_game(&ats, "ats", "1.55.0.1", &[]);
    let mut h = app(temp.path(), None);
    open_my_game(&mut h);
    assert!(shows(&h, "No game folder set"));
    assert!(!shows(&h, "Generate (0)"));
    h.state_mut()
        .prefs
        .game_folders
        .insert("ats".into(), ats.clone());
    h.get_by_label("ATS").click();
    h.run();
    h.state_mut().game.folders.clear();
    h.run();
    assert!(shows(&h, "American Truck Simulator is not supported yet"));
    assert!(shows(&h, "American Truck Simulator 1.55.0.1"));
    assert!(h.state().game.scanned().is_none());
    assert!(!shows(&h, "Generate (0)"));
}

#[test]
fn an_empty_library_offers_my_game() {
    let temp = tempfile::tempdir().unwrap();
    let mut h = app(temp.path(), None);
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    assert!(shows(&h, "No vehicle installed"));
    // The tab and the button.
    assert_eq!(count(&h, "My Game"), 2);
}
