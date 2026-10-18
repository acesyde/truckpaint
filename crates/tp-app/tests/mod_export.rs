//! Headless tests for the Export Mod dialog: its summary, destination,
//! problems (each leading to where it is fixed) and warnings, and an export
//! that never changes the project.

mod common;

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tempfile::TempDir;
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::layout::Space;
use tp_app::mod_export::Folders;
use tp_app::state::{Modal, is_enabled};
use tp_app::ui::mod_export_dialog::ModExportDialog;
use tp_app::workspace::{SaveState, Workspace};
use tp_core::ModSettings;
use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

/// The sample truck project (Standard cab, Chassis, Cab accessories, Side
/// skirts) named "ACE Logistics", with an ellipse on the Standard cab,
/// shown in the Workshop. The user's folders are in `dir` (Documents,
/// data, home), with no game mod folder.
fn open(dir: &TempDir) -> H {
    let mut h = common::harness();
    common::create_project(&mut h);
    h.state_mut().mod_folders = folders(dir);
    {
        let ws = ws_mut(&mut h);
        ws.project.name = "ACE Logistics".into();
        ws.project.mod_settings = ModSettings::for_project("ACE Logistics");
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            Frame::new(Point::new(2048.0, 2048.0), Size::new(2000.0, 1000.0), 0.0),
        );
        o.fill = Rgba::rgb(200, 30, 40).into();
        ws.project.add(o);
    }
    h.run();
    h
}

fn folders(dir: &TempDir) -> Folders {
    let documents = dir.path().join("Documents");
    std::fs::create_dir_all(&documents).unwrap();
    Folders {
        documents: Some(documents),
        data: Some(dir.path().join("data")),
        home: Some(dir.path().to_path_buf()),
    }
}

/// Where the mods are written when the game has no mod folder.
fn documents(dir: &TempDir) -> PathBuf {
    dir.path().join("Documents")
}

/// Runs a few frames (background renders make `Harness::run` unusable
/// while the dialog is open), enough for the dialog to settle in place as
/// egui sizes and centers it.
fn settle(h: &mut H) {
    for _ in 0..8 {
        h.step();
    }
}

/// Steps until `done` holds (background work), with a time limit.
fn wait_until(h: &mut H, mut done: impl FnMut(&H) -> bool) {
    for _ in 0..6000 {
        h.step();
        if done(h) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.step();
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn dialog(h: &H) -> Option<&ModExportDialog> {
    match &h.state().modal {
        Some(Modal::ExportMod(d)) => Some(d),
        _ => None,
    }
}

fn open_dialog(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    settle(h);
    assert!(dialog(h).is_some(), "Export Mod dialog open");
}

fn script_save(h: &mut H, mod_save: &[PathBuf]) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        mod_save: mod_save.iter().cloned().collect(),
        ..Default::default()
    });
}

/// The dialog's Export button.
fn export_button(h: &H) -> egui_kittest::Node<'_> {
    common::last(h, "Export")
}

/// Clicks Export and waits for the export to end.
fn export(h: &mut H) {
    export_button(h).click();
    settle(h);
    wait_until(h, |h| dialog(h).is_none_or(|d| d.job.is_none()));
    settle(h);
}

/// The text of the mod's manifest at `path`.
fn manifest(path: &Path) -> String {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut out = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("manifest.sii").unwrap(), &mut out).unwrap();
    out
}

/// Marks the project saved.
fn mark_saved(h: &mut H) {
    let snapshot = ws(h).snapshot();
    ws_mut(h).saved = Some(snapshot);
}

#[test]
fn the_command_needs_a_texture() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    let enabled = |h: &H| is_enabled(CommandId::ExportMod, &h.state().edit_context());
    assert!(enabled(&h));
    let now = h.ctx.input(|i| i.time);
    let ws = ws_mut(&mut h);
    let id = ws.project.surface().objects[0].id;
    ws.selection = vec![id];
    let symbol = ws.convert_to_symbol(now).expect("symbol");
    ws.edit_symbol(symbol, now);
    h.run();
    assert!(!enabled(&h), "disabled while a symbol is edited");
}

#[test]
fn opening_the_dialog() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    open_dialog(&mut h);
    // Titled with the project's name and game.
    assert!(
        h.query_by_label("ACE Logistics · Euro Truck Simulator 2")
            .is_some()
    );
    for line in [
        "Standard cab (cabin: standard)",
        "High roof (not painted)",
        "Accessories: Chassis, Cab accessories, Side skirts",
    ] {
        assert!(h.query_by_label(line).is_some(), "{line}");
    }
    let destination = documents(&dir).join("ACE Logistics.scs");
    assert_eq!(dialog(&h).unwrap().destination, destination);
    assert!(
        h.query_by_label(&destination.display().to_string())
            .is_some()
    );
    assert!(h.query_by_label("Destination").is_some());
    assert!(h.query_by_label("Change…").is_some());
    assert!(!export_button(&h).accesskit_node().is_disabled());
}

/// A trailer's main texture is named alone in the summary: no cabin.
#[test]
fn trailer_summary_has_no_cabin_line() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    let trailer = tp_vehicles::Package::read(tp_app::vehicles::SAMPLES[1].bytes).unwrap();
    ws_mut(&mut h)
        .add_vehicle(&trailer, &["base".to_owned(), "mudflaps".to_owned()], 0.0)
        .unwrap();
    common::settle_renders(&mut h);
    let before = h.query_all_by_label("Base").count();
    open_dialog(&mut h);
    assert!(h.query_by_label("Standard cab (cabin: standard)").is_some());
    // The summary's line, besides the Textures tab's.
    assert_eq!(h.query_all_by_label("Base").count(), before + 1);
    assert!(h.query_by_label_contains("Base (").is_none());
    assert!(h.query_by_label_contains("Base:").is_none());
}

#[test]
fn nothing_to_edit_in_the_dialog() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    // From the Workshop: no Mod information column behind the dialog.
    open_dialog(&mut h);
    let fields = h
        .query_all(egui_kittest::kittest::By::new().predicate(|n| n.role() == Role::TextInput))
        .count();
    assert_eq!(fields, 0, "no field");
    assert_eq!(
        h.query_all(egui_kittest::kittest::By::new().predicate(|n| n.role() == Role::Image))
            .count(),
        0,
        "no picture"
    );
    // No Advanced section either.
    assert!(h.query_by_label("Advanced").is_none());
}

#[test]
fn export_from_the_brand_space() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    common::show_space(&mut h, Space::Brand);
    h.run();
    h.get_by_role_and_label(Role::Button, "Export…").click();
    settle(&mut h);
    assert!(dialog(&h).is_some(), "the Export Mod dialog opens");
}

#[test]
fn from_the_project_space() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    common::show_space(&mut h, Space::Project);
    common::settle_renders(&mut h);
    assert!(h.query_by_label("Edit in Export Mod…").is_none());
    h.get_by_role_and_label(Role::Button, "Export…").click();
    settle(&mut h);
    assert!(dialog(&h).is_some(), "the Export Mod dialog opens");
}

#[test]
fn an_empty_name_blocks_the_export() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    ws_mut(&mut h).project.mod_settings.name.clear();
    open_dialog(&mut h);
    assert!(h.query_by_label("The mod needs a name.").is_some());
    assert!(export_button(&h).accesskit_node().is_disabled());
}

#[test]
fn a_problem_leads_to_its_field() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    ws_mut(&mut h).project.mod_settings.price = 0;
    open_dialog(&mut h);
    assert!(export_button(&h).accesskit_node().is_disabled());
    h.get_by_label("Show Price").click();
    h.run();
    assert!(dialog(&h).is_none(), "closed");
    assert_eq!(ws(&h).space, Space::Project);
    assert!(
        h.get_by_role_and_label(Role::TextInput, "Price")
            .is_focused()
    );
}

#[test]
fn advanced_opens_on_an_internal_name_problem() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    // Set while the project held only a trailer (12 characters were
    // allowed); the truck, with two main textures, allows 10.
    ws_mut(&mut h).project.mod_settings.internal_name = Some("ace_logistic".into());
    open_dialog(&mut h);
    let problem = "The internal name can have at most 10 characters.";
    let problem_top = h.get_by_label(problem).rect().top();
    let export = export_button(&h);
    assert!(export.accesskit_node().is_disabled());
    // Listed just above the buttons.
    assert!(problem_top < export.rect().top());
    assert!(problem_top > h.get_by_label("In the mod").rect().top());
    h.get_by_label("Show Internal name").click();
    h.run();
    assert!(dialog(&h).is_none(), "closed");
    assert_eq!(ws(&h).space, Space::Project);
    let field = h.get_by_role_and_label(Role::TextInput, "Internal name");
    assert!(field.is_focused());
    assert_eq!(field.value().as_deref(), Some("ace_logistic"));
}

#[test]
fn cancel_keeps_the_settings() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    mark_saved(&mut h);
    let undo_steps = ws(&h).history.len();
    open_dialog(&mut h);
    h.key_press(Key::Escape);
    settle(&mut h);
    assert!(dialog(&h).is_none());
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
    assert_eq!(ws(&h).history.len(), undo_steps);
}

/// Puts a rectangle on texture `name` (it becomes Modified).
fn draw(h: &mut H, name: &str) {
    let ws = ws_mut(h);
    let i = ws
        .project
        .surfaces
        .iter()
        .position(|s| s.name == name)
        .unwrap();
    let active = ws.project.active_surface;
    ws.project.active_surface = i;
    ws.project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(200.0, 200.0), Size::new(100.0, 100.0), 0.0),
    ));
    ws.project.active_surface = active;
}

/// Flags texture `name` "Layout changed".
fn flag_layout_changed(h: &mut H, name: &str) {
    let ws = ws_mut(h);
    let surface = ws
        .project
        .surfaces
        .iter_mut()
        .find(|s| s.name == name)
        .unwrap();
    surface.template.as_mut().unwrap().status = tp_core::TemplateStatus::LayoutChanged;
}

#[test]
fn warnings_dont_block() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    flag_layout_changed(&mut h, "Chassis");
    open_dialog(&mut h);
    // Texture to check warned, then the empty ones.
    let check = h.get_by_label("Chassis: layout changed").rect();
    let empty = h
        .get_by_label("2 textures empty, exported with the game's color")
        .rect();
    assert!(check.top() < empty.top());
    assert!(!export_button(&h).accesskit_node().is_disabled());
    // A problem is listed before them, and alone disables Export.
    ws_mut(&mut h).project.mod_settings.name.clear();
    settle(&mut h);
    let problem = h.get_by_label("The mod needs a name.").rect();
    assert!(problem.top() < h.get_by_label("Chassis: layout changed").rect().top());
    assert!(export_button(&h).accesskit_node().is_disabled());
}

#[test]
fn exporting_into_the_game() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    let folders = h.state().mod_folders.clone();
    let game = tp_app::mod_export::mod_folder_in(
        "ets2",
        folders.documents.as_deref(),
        folders.data.as_deref(),
    )
    .unwrap();
    std::fs::create_dir_all(&game).unwrap();
    open_dialog(&mut h);
    let path = game.join("ACE Logistics.scs");
    assert_eq!(dialog(&h).unwrap().destination, path);
    export(&mut h);
    assert!(dialog(&h).is_none(), "closed once written");
    assert!(manifest(&path).contains("display_name: \"ACE Logistics\""));
    assert_eq!(h.state().last_mod_folder.as_deref(), Some(game.as_path()));
}

#[test]
fn exporting_into_the_mod_folder_chosen_in_preferences() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    let chosen = dir.path().join("MyMods");
    std::fs::create_dir_all(&chosen).unwrap();
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        folders: [chosen.clone()].into(),
        ..Default::default()
    });
    h.state_mut()
        .open_preferences(tp_app::state::PrefsTab::Games);
    h.run();
    h.get_by_label("Choose…, Euro Truck Simulator 2 mod folder")
        .click();
    h.run();
    assert_eq!(h.state().prefs.mod_folders["ets2"], chosen);
    h.state_mut().modal = None;
    h.run();
    open_dialog(&mut h);
    assert_eq!(
        dialog(&h).unwrap().destination,
        chosen.join("ACE Logistics.scs")
    );
}

#[test]
fn no_game_folder() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    // An ATS project, while only the ETS2 folder exists.
    let folders = h.state().mod_folders.clone();
    let ets2 = tp_app::mod_export::mod_folder_in(
        "ets2",
        folders.documents.as_deref(),
        folders.data.as_deref(),
    )
    .unwrap();
    std::fs::create_dir_all(&ets2).unwrap();
    {
        let ws = ws_mut(&mut h);
        ws.project.vehicles[0].game = "ats".into();
        ws.project.mod_settings.name = "Blue Line".into();
    }
    open_dialog(&mut h);
    assert_eq!(
        dialog(&h).unwrap().destination,
        documents(&dir).join("Blue Line.scs")
    );
}

#[test]
fn changing_the_destination() {
    let dir = tempfile::tempdir().unwrap();
    let other = dir.path().join("other");
    std::fs::create_dir_all(&other).unwrap();
    let mut h = open(&dir);
    open_dialog(&mut h);
    script_save(&mut h, &[other.join("ace")]);
    h.get_by_label("Change…").click();
    settle(&mut h);
    // Opened in the destination's folder, proposing its file name.
    assert_eq!(h.state().dialogs.suggested(), ["ACE Logistics.scs"]);
    let path = other.join("ace.scs");
    assert_eq!(dialog(&h).unwrap().destination, path);
    assert!(h.query_by_label(&path.display().to_string()).is_some());
    export(&mut h);
    assert!(path.is_file());
    assert!(!documents(&dir).join("ACE Logistics.scs").exists());
}

#[test]
fn cancelling_change() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    open_dialog(&mut h);
    let before = dialog(&h).unwrap().destination.clone();
    script_save(&mut h, &[]);
    h.get_by_label("Change…").click();
    settle(&mut h);
    assert_eq!(dialog(&h).unwrap().destination, before);
    assert!(!before.exists(), "nothing written");
    assert_eq!(std::fs::read_dir(documents(&dir)).unwrap().count(), 0);
}

#[test]
fn replacing_an_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    let path = documents(&dir).join("ACE Logistics.scs");
    std::fs::write(&path, b"earlier export").unwrap();
    open_dialog(&mut h);
    export_button(&h).click();
    settle(&mut h);
    let question = "ACE Logistics.scs already exists. Replace it?";
    assert!(h.query_by_label(question).is_some());
    // Cancel writes nothing and keeps the dialog.
    common::last(&h, "Cancel").click();
    settle(&mut h);
    assert!(dialog(&h).is_some());
    assert!(h.query_by_label(question).is_none());
    assert_eq!(std::fs::read(&path).unwrap(), b"earlier export");
    // Replace starts the export.
    export_button(&h).click();
    settle(&mut h);
    h.get_by_label("Replace").click();
    settle(&mut h);
    wait_until(&mut h, |h| dialog(h).is_none_or(|d| d.job.is_none()));
    settle(&mut h);
    assert!(dialog(&h).is_none(), "closed once written");
    assert!(manifest(&path).contains("ACE Logistics"));
}

#[test]
fn replacing_an_earlier_export() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    let path = documents(&dir).join("ACE Logistics.scs");
    open_dialog(&mut h);
    export(&mut h);
    assert!(manifest(&path).contains("package_version: \"1.0\""));
    ws_mut(&mut h).project.mod_settings.version = "1.1".into();
    open_dialog(&mut h);
    export_button(&h).click();
    settle(&mut h);
    h.get_by_label("Replace").click();
    settle(&mut h);
    wait_until(&mut h, |h| dialog(h).is_none_or(|d| d.job.is_none()));
    settle(&mut h);
    assert!(manifest(&path).contains("package_version: \"1.1\""));
    // Written whole: no temporary file left beside it.
    assert_eq!(std::fs::read_dir(documents(&dir)).unwrap().count(), 1);
}

#[test]
fn a_file_confirmed_by_the_save_dialog_isnt_asked_again() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    let path = dir.path().join("ace.scs");
    std::fs::write(&path, b"earlier export").unwrap();
    open_dialog(&mut h);
    script_save(&mut h, std::slice::from_ref(&path));
    h.get_by_label("Change…").click();
    settle(&mut h);
    export(&mut h);
    assert!(dialog(&h).is_none(), "written without asking");
    assert!(manifest(&path).contains("ACE Logistics"));
}

#[test]
fn export_of_a_saved_project_with_unchanged_settings() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    mark_saved(&mut h);
    let undo_steps = ws(&h).history.len();
    open_dialog(&mut h);
    export(&mut h);
    assert!(documents(&dir).join("ACE Logistics.scs").is_file());
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
    assert_eq!(ws(&h).history.len(), undo_steps);
}

/// Commits `version` in the Project space's Version field.
fn commit_version(h: &mut H, version: &str) {
    common::show_space(h, Space::Project);
    common::settle_renders(h);
    h.get_by_role_and_label(Role::TextInput, "Version").focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, "Version")
        .type_text(version);
    h.run();
    h.key_press(Key::Enter);
    h.run();
}

#[test]
fn exporting_records_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    commit_version(&mut h, "1.1");
    let steps = ws(&h).history.len();
    open_dialog(&mut h);
    export(&mut h);
    assert_eq!(ws(&h).history.len(), steps);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-mod-settings"));
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.mod_settings.version, "1.0");
}

#[test]
fn undo_the_settings() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    commit_version(&mut h, "1.1");
    open_dialog(&mut h);
    export(&mut h);
    let path = documents(&dir).join("ACE Logistics.scs");
    let written = std::fs::read(&path).unwrap();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(ws(&h).project.mod_settings.version, "1.0");
    assert_eq!(
        h.get_by_role_and_label(Role::TextInput, "Version")
            .value()
            .as_deref(),
        Some("1.0")
    );
    assert_eq!(std::fs::read(&path).unwrap(), written, "file unchanged");
    assert!(manifest(&path).contains("package_version: \"1.1\""));
}

#[test]
fn empty_textures_warned_and_exported_transparent() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    draw(&mut h, "Chassis");
    draw(&mut h, "Cab accessories");
    h.run();
    open_dialog(&mut h);
    assert!(
        h.query_by_label("Side skirts is empty, exported with the game's color")
            .is_some()
    );
    export(&mut h);
    assert!(dialog(&h).is_none(), "exported");
    let path = documents(&dir).join("ACE Logistics.scs");
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let name = archive
        .file_names()
        .find(|n| n.contains("side_skirts") && n.ends_with(".dds"))
        .expect("the Side skirts texture")
        .to_owned();
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut archive.by_name(&name).unwrap(), &mut bytes).unwrap();
    // Its first block (BC3, after the 128-byte header) is fully transparent.
    let mut pixels = [0u8; 64];
    texpresso::Format::Bc3.decompress(&bytes[128..144], 4, 4, &mut pixels);
    assert!(pixels.chunks(4).all(|p| p[3] == 0), "{pixels:?}");
}

/// mod-export: File › Export Mod… opens the dialog.
#[test]
fn opened_from_the_file_menu() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open(&dir);
    h.get_by_label("File").click();
    h.run();
    h.get_by_label("Export Mod…").click();
    settle(&mut h);
    assert!(dialog(&h).is_some(), "Export Mod dialog open");
}
