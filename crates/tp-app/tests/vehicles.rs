//! Headless tests for vehicles: the library dialog, the New Project vehicle
//! step, the fleet tree, the template overlay, Textures…, Update Template
//! and export of vehicle textures.

mod common;

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{By, NodeT, Queryable};
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::prefs::Prefs;
use tp_app::state::Modal;
use tp_app::vehicles::VehicleLibrary;
use tp_app::workspace::{SaveState, Workspace};
use tp_core::TemplateStatus;
use tp_core::document::{Frame, ShapeKind};
use tp_core::kurbo::{Point, Size};
use tp_vehicles::sample::{self, SampleTexture};

type H = Harness<'static, AppState>;

const ID: &str = "scs.sample.truck";

fn tex(id: &'static str, name: &'static str, size: u32, layout: u32) -> SampleTexture {
    SampleTexture {
        id,
        name,
        size: tp_vehicles::TextureSize::new(size, size),
        layout,
    }
}

/// Writes a package file of a truck whose first texture is its single main
/// texture and the others accessories; returns its path.
fn package_file(dir: &Path, version: &str, textures: &[SampleTexture]) -> PathBuf {
    let path = dir.join(format!("truck-{version}.tpv"));
    std::fs::write(
        &path,
        sample::package(ID, "Sample Truck", version, textures),
    )
    .unwrap();
    path
}

/// The app with a library in `dir/library`.
fn app(dir: &Path) -> H {
    let mut state = AppState::with_prefs(Prefs::default(), None);
    state.vehicles = VehicleLibrary::open(&dir.join("library"));
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 2000.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.run();
    h
}

fn script_packages(h: &mut H, files: &[PathBuf]) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        packages: [files.to_vec()].into(),
        ..Default::default()
    });
}

/// Lets a dialog that changed size settle (egui re-centers a modal over a
/// few frames) and background template renders finish.
fn settle(h: &mut H) {
    h.run_steps(8);
    for _ in 0..1000 {
        if h.try_run().is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the UI keeps repainting");
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

/// Creates a project for the installed sample truck through the wizard.
fn create_vehicle_project(h: &mut H) {
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::CheckBox, "Sample Truck")
        .click();
    settle(h);
    h.get_by_label("Create Project").click();
    h.run();
}

/// Opens the ⋯ menu of `vehicle` in the fleet tree and clicks `item`.
fn vehicle_action(h: &mut H, vehicle: &str, item: &str) {
    h.get_by_label(&format!("Actions for {vehicle}")).click();
    h.run();
    h.get_by_label(item).click();
    h.run();
}

/// Clicks a texture of the fleet tree (the Project space's, or the
/// Workshop's Textures tab).
fn pick_texture(h: &mut H, path: &str) {
    h.get_by_label_contains(&format!("Texture {path},")).click();
    h.run();
}

/// Whether the breadcrumb (inlaid in the canvas) shows `text`.
fn breadcrumb_shows(h: &H, text: &str) -> bool {
    h.query_all_by_label_contains(text).count() == 1
}

/// Shows the Workshop (New Project opens the Project space).
fn show_workshop(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num2);
    h.run();
}

fn install_v120(h: &mut H, dir: &Path) {
    let file = package_file(dir, "1.2.0", &sample::truck_textures());
    h.state_mut().vehicles.install_file(&file).unwrap();
    h.run();
}

#[test]
fn install_from_the_library_dialog_and_two_versions() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    let v12 = package_file(dir.path(), "1.2.0", &sample::truck_textures());
    let v13 = package_file(dir.path(), "1.3.0", &sample::truck_textures());
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    assert!(h.query_by_label("No vehicle installed").is_some());
    script_packages(&mut h, &[v12, v13]);
    h.get_by_label("Install…").click();
    h.run();
    assert!(
        h.query_by_label_contains("Installed Sample Truck 1.3.0")
            .is_some()
    );
    let v = h.state().vehicles.get(ID).unwrap();
    assert_eq!(v.versions.len(), 2);
    assert_eq!(v.newest().manifest.version.to_string(), "1.3.0");
    // Persisted across sessions.
    assert_eq!(
        VehicleLibrary::open(&dir.path().join("library"))
            .get(ID)
            .unwrap()
            .versions
            .len(),
        2
    );
}

#[test]
fn library_status_and_update_template() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    let v13 = package_file(dir.path(), "1.3.0", &sample::truck_textures());
    h.state_mut().vehicles.install_file(&v13).unwrap();
    // From the home screen: no project, so up to date.
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    settle(&mut h);
    assert!(h.query_by_label("Up to date").is_some());
    assert!(h.query_by_label("Installed · 1").is_some());
    assert!(h.query_by_label("My vehicles · 0").is_some());
    // Both versions are listed in the details, the newest in the list.
    assert!(h.query_by_label("Version 1.3.0").is_some());
    assert!(h.query_by_label("Version 1.2.0").is_some());
    assert!(h.query_by_label("Update Template…").is_none());
    h.key_press(Key::Escape);
    h.run();

    // A project made with 1.2.0: the vehicle has an update.
    let package = h
        .state()
        .vehicles
        .load(ID, &"1.2.0".parse().unwrap())
        .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let project = tp_app::vehicle_project::fleet_project("T", &[(&package, &textures)]).unwrap();
    h.state_mut().open_project(project);
    settle(&mut h);
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    settle(&mut h);
    assert!(h.query_by_label("Update").is_some());
    h.get_by_label("Update Template…").click();
    settle(&mut h);
    assert!(
        matches!(h.state().modal, Some(Modal::UpdateTemplate(_))),
        "Update Template opens for the vehicle"
    );
}

#[test]
fn invalid_package_shows_why() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    let bad = dir.path().join("bad.tpv");
    std::fs::write(&bad, b"not a package").unwrap();
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    script_packages(&mut h, &[bad]);
    h.get_by_label("Install…").click();
    h.run();
    assert!(
        h.query_by_label_contains("bad.tpv could not be installed: it is not a vehicle package.")
            .is_some()
    );
    assert!(h.state().vehicles.vehicles().is_empty());
}

#[test]
fn creating_a_project_for_a_vehicle() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::CheckBox, "Sample Truck")
        .click();
    settle(&mut h);
    // Your fleet: the textures that will be created, on the same page.
    assert!(h.query_by_label("Cabin (4096²)").is_some());
    assert!(h.query_by_label("1 main + 2 accessories").is_some());
    assert!(h.query_by_label("3 textures in total").is_some());
    // The name keeps the choice: there is nothing to go back to.
    h.get_by_role_and_label(Role::TextInput, "Project name")
        .click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Project name")
        .type_text("Fleet");
    h.run();
    assert!(
        toggled(&h, Role::CheckBox, "Chassis"),
        "the checked textures stay"
    );
    // A single main texture is always painted.
    let cabin = h.get_by_role_and_label(Role::CheckBox, "Cabin");
    assert!(cabin.accesskit_node().is_disabled());
    assert!(toggled(&h, Role::CheckBox, "Cabin"));
    h.get_by_label("Create Project").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.name, "Fleet");
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Cabin", "Chassis", "Accessories"]);
    assert_eq!(p.vehicles[0].version, "1.2.0");
}

#[test]
fn the_tree_and_shortcuts_switch_textures() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    pick_texture(&mut h, "Sample Truck › Chassis");
    assert_eq!(ws(&h).project.active_surface, 1);
    // The breadcrumb names the active texture with its vehicle and group.
    assert!(breadcrumb_shows(
        &h,
        "Sample Truck › Accessories › Chassis 2048²"
    ));
    let id = ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(100.0, 100.0), 0.0),
        1.0,
    );
    h.run();
    assert!(ws(&h).project.surfaces[1].get(id).is_some());
    assert!(ws(&h).project.surfaces[0].objects.is_empty());
    // Next and previous texture from the keyboard: Cmd/Ctrl+] and [, with
    // Page Down and Page Up as alternates. No object moves in the stacking
    // order.
    ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(700.0, 500.0), Size::new(100.0, 100.0), 0.0),
        2.0,
    );
    ws_mut(&mut h).selection = vec![id];
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::CloseBracket);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 2);
    assert_eq!(ws(&h).project.surfaces[1].index_of(id), Some(0));
    h.key_press_modifiers(Modifiers::COMMAND, Key::PageDown);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 0, "wraps around");
    h.key_press_modifiers(Modifiers::COMMAND, Key::OpenBracket);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 2);
    h.key_press_modifiers(Modifiers::COMMAND, Key::PageUp);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 1);
}

#[test]
fn hide_the_template_with_g() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    show_workshop(&mut h);
    let visible = |h: &H| ws(h).project.surface().template.as_ref().unwrap().visible;
    let toggle_on = |h: &H| {
        h.get_by_role_and_label(Role::CheckBox, "Template")
            .accesskit_node()
            .toggled()
            == Some(egui::accesskit::Toggled::True)
    };
    assert!(visible(&h) && toggle_on(&h));
    h.key_press(Key::G);
    h.run();
    assert!(!visible(&h));
    assert!(!toggle_on(&h), "the status bar shows the off state");
    assert!(!ws(&h).history.can_undo(), "not an undo step");
    assert_eq!(
        ws(&h).tool,
        tp_app::tool::Tool::Select,
        "not the Gradient tool"
    );
    h.key_press(Key::G);
    h.run();
    assert!(visible(&h));
    // The same setting in the status bar.
    h.get_by_role_and_label(Role::CheckBox, "Template").click();
    h.run();
    assert!(!visible(&h));
    assert!(!ws(&h).history.can_undo(), "not an undo step");
    assert!(
        h.query_by_role_and_label(Role::Slider, "Template opacity")
            .is_some()
    );
}

#[test]
fn update_from_the_vehicle_panel_and_undo() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    assert!(
        h.query_by_label("Update the template of Sample Truck")
            .is_none()
    );
    let v13 = package_file(
        dir.path(),
        "1.3.0",
        &[
            tex("cabin", "Cabin", 4096, 2),
            tex("chassis", "Chassis", 2048, 1),
            tex("accessories", "Accessories", 1024, 1),
        ],
    );
    h.state_mut().vehicles.install_file(&v13).unwrap();
    h.run();
    // The Project space's card says so, with Update Template….
    assert!(h.query_by_label("Update 1.3.0 available").is_some());
    h.get_by_label("Update the template of Sample Truck")
        .click();
    h.run();
    assert!(h.query_by_label_contains("Cabin: layout changed").is_some());
    h.get_by_label("Update").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.vehicles[0].version, "1.3.0");
    let status = |h: &H, i: usize| ws(h).project.surfaces[i].template.as_ref().unwrap().status;
    assert_eq!(status(&h, 0), TemplateStatus::LayoutChanged);
    assert_eq!(status(&h, 1), TemplateStatus::Current);
    // The Cabin's tile is To check, saying why; no update is left.
    assert_eq!(h.query_all_by_label("Layout changed in 1.3.0").count(), 1);
    assert!(
        h.query_by_label_contains("Sample Truck › Cabin, To check")
            .is_some()
    );
    assert!(h.query_by_label_contains("available").is_none());
    // The inspector (nothing selected) says so for the active Cabin.
    show_workshop(&mut h);
    assert!(
        h.query_by_label("To check: Layout changed in 1.3.0")
            .is_some()
    );
    h.get_by_label("Mark Sample Truck › Cabin as checked")
        .click();
    h.run();
    assert_eq!(status(&h, 0), TemplateStatus::Current);
    ws_mut(&mut h).undo();
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.vehicles[0].version, "1.2.0");
}

#[test]
fn removing_a_version_keeps_open_projects_and_files_work_without_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    h.state_mut()
        .vehicles
        .remove(ID, &"1.2.0".parse().unwrap())
        .unwrap();
    h.run();
    assert!(ws(&h).project.surfaces.iter().all(|s| s.template.is_some()));
    assert!(
        h.query_by_label("Truck · package 1.2.0").is_some(),
        "recorded version"
    );
    // Saved and reopened without the package: templates travel with it.
    let bytes = tp_file::to_bytes(&ws(&h).project).unwrap();
    let opened = tp_file::from_bytes(&bytes).unwrap().project;
    assert_eq!(opened.template_assets().count(), 3);
    assert_eq!(opened.vehicles[0].version, "1.2.0");
}

/// Opens `path` through File › Open… (no project open).
fn open_file(h: &mut H, path: &Path) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        open: [path.to_path_buf()].into(),
        ..Default::default()
    });
    h.key_press_modifiers(Modifiers::COMMAND, Key::O);
    settle(h);
}

#[test]
fn older_files_get_the_game_data_of_their_installed_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    let recorded = ws(&h).project.clone();
    assert!(recorded.vehicles[0].game_data.is_some());
    // A file written before game data was recorded.
    let mut old = recorded.clone();
    old.vehicles[0].game_data = None;
    for s in &mut old.surfaces {
        let t = s.template.as_mut().unwrap();
        t.game_ids.clear();
        t.main_index = None;
    }
    let path = dir.path().join("old.truckpaint");
    tp_file::write(&old, &path).unwrap();
    h.state_mut().close_project();
    open_file(&mut h, &path);
    let p = &ws(&h).project;
    assert_eq!(p.vehicles, recorded.vehicles);
    let templates = |p: &tp_core::Project| {
        p.surfaces
            .iter()
            .map(|s| s.template.clone().map(|t| (t.game_ids, t.main_index)))
            .collect::<Vec<_>>()
    };
    assert_eq!(templates(p), templates(&recorded));
    assert_eq!(ws(&h).save_state(), SaveState::Unsaved);

    // Without the package version, it opens as it is, editable and saved.
    h.state_mut().close_project();
    h.state_mut()
        .vehicles
        .remove(ID, &"1.2.0".parse().unwrap())
        .unwrap();
    open_file(&mut h, &path);
    assert!(ws(&h).project.vehicles[0].game_data.is_none());
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
    ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0), 0.0),
        1.0,
    );
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
}

#[test]
fn exporting_a_smaller_texture_without_its_template() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    pick_texture(&mut h, "Sample Truck › Accessories");
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::E);
    for _ in 0..4 {
        h.step();
    }
    assert!(h.query_by_label_contains("1024 × 1024 px · PNG").is_some());
    let out = dir.path().join("out.png");
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        export: [out.clone()].into(),
        ..Default::default()
    });
    common::last(&h, "Export…").click();
    for _ in 0..3000 {
        h.step();
        if h.state().modal.is_none() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        h.state().dialogs_suggested(),
        vec!["Sample Truck - Sample Truck - Accessories.png".to_owned()]
    );
    let img = image::open(&out).unwrap().to_rgba8();
    assert_eq!(img.dimensions(), (1024, 1024));
    // The template's dark lines are not in the export: all white.
    assert!(img.pixels().all(|p| p.0 == [255, 255, 255, 255]));
}

fn looks_like_id(text: &str) -> bool {
    text.len() > 3
        && text.contains('-')
        && text.starts_with(|c: char| c.is_ascii_lowercase())
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn assert_no_raw_ids(h: &H, screen: &str) {
    use egui_kittest::kittest::By;
    let leaks: Vec<String> = h
        .query_all(By::new().include_labels().predicate(|node| {
            [node.label(), node.value()]
                .into_iter()
                .flatten()
                .any(|t| looks_like_id(&t))
        }))
        .map(|n| format!("{:?}", n.accesskit_node().label()))
        .collect();
    assert!(
        leaks.is_empty(),
        "{screen}: raw message ids shown: {leaks:#?}"
    );
}

#[test]
fn vehicle_screens_show_no_raw_message_id() {
    for language in tp_i18n::Language::ALL {
        let dir = tempfile::tempdir().unwrap();
        let mut h = app(dir.path());
        h.state_mut().prefs.set_language(Some(language));
        install_v120(&mut h, dir.path());
        h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
        h.run();
        assert_no_raw_ids(&h, "library");
        h.state_mut().modal = Some(Modal::NewProject(Default::default()));
        h.run();
        assert_no_raw_ids(&h, "new project");
        if let Some(Modal::NewProject(d)) = &mut h.state_mut().modal {
            d.check(tp_app::ui::vehicle_dialogs::VehicleChoice {
                id: ID.into(),
                version: "1.2.0".parse().unwrap(),
                textures: vec!["chassis".into()],
            });
        }
        h.run();
        assert_no_raw_ids(&h, "new project with a vehicle");
        h.state_mut().modal = None;
        let package = h
            .state()
            .vehicles
            .load(ID, &"1.2.0".parse().unwrap())
            .unwrap();
        let project =
            tp_app::vehicle_project::fleet_project("T", &[(&package, &["chassis".to_owned()])])
                .unwrap();
        h.state_mut().open_project(project);
        let v13 = package_file(
            dir.path(),
            "1.3.0",
            &[tex("cabin", "Cabin", 4096, 2), tex("new", "New", 1024, 1)],
        );
        h.state_mut().vehicles.install_file(&v13).unwrap();
        h.run();
        assert_no_raw_ids(&h, "vehicle workspace");
        h.state_mut().open_update_dialog(None);
        h.run();
        assert_no_raw_ids(&h, "update dialog");
        h.state_mut().modal = Some(Modal::AddVehicle(Default::default()));
        h.run();
        assert_no_raw_ids(&h, "add vehicle dialog");
        let dialog = {
            let p = &ws(&h).project;
            tp_app::ui::vehicle_dialogs::TexturesDialog::new(p, &p.vehicles[0])
        };
        h.state_mut().modal = Some(Modal::Textures(dialog));
        h.run();
        assert_no_raw_ids(&h, "textures dialog");
    }
}

#[test]
fn install_the_sample_vehicles_from_new_project() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_label("Install the sample vehicles").click();
    settle(&mut h);
    assert!(
        h.query_by_label_contains("Installed TruckPaint Sample Truck 1.1.0")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("Installed TruckPaint Sample Trailer 1.0.0")
            .is_some()
    );
    // Installed, the truck selected, and the button is gone.
    assert!(h.query_by_label("Install the sample vehicles").is_none());
    let row = h.get_by_role_and_label(Role::CheckBox, "TruckPaint Sample Truck");
    assert_eq!(
        row.accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True),
        "the sample truck is selected"
    );
    h.get_by_label("Create Project").click();
    h.run();
    let project = &ws(&h).project;
    assert_eq!(project.name, "TruckPaint Sample Truck");
    let names: Vec<&str> = project.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        ["Standard cab", "Chassis", "Cab accessories", "Side skirts"]
    );
}

#[test]
fn install_the_sample_vehicles_from_the_library() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    h.get_by_label("Install the sample vehicles").click();
    settle(&mut h);
    for id in [
        tp_app::vehicles::SAMPLE_ID,
        tp_app::vehicles::SAMPLE_TRAILER_ID,
    ] {
        assert!(h.state().vehicles.get(id).is_some(), "{id}");
    }
    assert!(h.query_by_label("No vehicle installed").is_none());
    assert!(h.query_by_label("Install the sample vehicles").is_none());
    // The detail pane summarizes the chosen vehicle's paint job, the
    // first listed at first.
    assert!(
        h.query_by_label_contains("Main texture: Base · 3\u{a0}accessories")
            .is_some()
    );
    assert!(
        h.query_by_label("Template of TruckPaint Sample Trailer")
            .is_some()
    );
    h.get_by_role_and_label(Role::RadioButton, "TruckPaint Sample Truck")
        .click();
    settle(&mut h);
    assert!(
        h.query_by_label_contains("Main textures: Standard cab, High roof · 3\u{a0}accessories")
            .is_some()
    );
}

#[test]
fn no_sample_button_once_a_vehicle_is_installed() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    h.get_by_label("New Project").click();
    h.run();
    assert!(h.query_by_label("Install the sample vehicles").is_none());
    h.key_press(Key::Escape);
    h.run();
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    assert!(h.query_by_label("Install the sample vehicles").is_none());
}

const SAMPLE_ID: &str = "community.truckpaint.sample_truck";

/// Writes a package of `game` and `kind` with a single 1024 px main
/// texture "Body"; returns its path.
fn other_package_file(dir: &Path, id: &str, name: &str, game: &str, kind: &str) -> PathBuf {
    let mut m = sample::manifest_with(
        id,
        name,
        "1.0.0",
        kind,
        &[tex("body", "Body", 1024, 1)],
        &[],
    );
    m["game"]["id"] = game.into();
    let path = dir.join(format!("{id}.tpv"));
    std::fs::write(
        &path,
        sample::zip(&m, &[("templates/body.png".into(), sample::png(16))]),
    )
    .unwrap();
    path
}

fn toggled(h: &H, role: Role, label: &str) -> bool {
    h.get_by_role_and_label(role, label)
        .accesskit_node()
        .toggled()
        .is_some_and(|t| t == egui::accesskit::Toggled::True)
}

/// Sets checkbox `label` to `on`.
fn set_checkbox(h: &mut H, label: &str, on: bool) {
    if toggled(h, Role::CheckBox, label) != on {
        h.get_by_role_and_label(Role::CheckBox, label).click();
        h.run();
    }
}

/// A project on the sample truck (1.1.0) through the wizard, with the main
/// textures `main` checked and the accessories `left_out` unchecked.
fn create_sample_project(h: &mut H, main: &[&str], left_out: &[&str]) {
    h.state_mut().vehicles.install_samples().unwrap();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::CheckBox, "TruckPaint Sample Truck")
        .click();
    settle(h);
    assert!(
        toggled(h, Role::CheckBox, "Standard cab"),
        "first main texture checked"
    );
    // Check before unchecking: the last checked main texture is locked.
    for m in main {
        set_checkbox(h, m, true);
    }
    for m in ["Standard cab", "High roof"] {
        if !main.contains(&m) {
            set_checkbox(h, m, false);
        }
    }
    for a in left_out {
        set_checkbox(h, a, false);
    }
    h.get_by_label("Create Project").click();
    h.run();
}

#[test]
fn two_main_textures_and_the_tree_selects_textures() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab", "High roof"], &[]);
    let p = &ws(&h).project;
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Standard cab",
            "High roof",
            "Chassis",
            "Cab accessories",
            "Side skirts"
        ]
    );
    pick_texture(&mut h, "TruckPaint Sample Truck › High roof");
    assert_eq!(ws(&h).project.active_surface, 1);
    assert!(
        breadcrumb_shows(
            &h,
            "TruckPaint Sample Truck › Main textures › High roof 4096²"
        ),
        "breadcrumb"
    );
    // The next texture follows the project order.
    h.key_press_modifiers(Modifiers::COMMAND, Key::CloseBracket);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 2);
    assert!(breadcrumb_shows(
        &h,
        "TruckPaint Sample Truck › Accessories › Chassis"
    ));
}

#[test]
fn create_needs_a_vehicle_and_the_last_main_texture_stays_checked() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.get_by_label("New Project").click();
    h.run();
    let disabled = |h: &H| {
        h.get_by_label("Create Project")
            .accesskit_node()
            .is_disabled()
    };
    assert!(disabled(&h));
    assert!(h.query_by_label("Install the sample vehicles").is_some());
    assert!(h.query_by_label("Blank texture").is_none());
    h.get_by_label("Install the sample vehicles").click();
    settle(&mut h);
    assert!(!disabled(&h));
    // The main texture mode comes from the package, read only.
    assert!(h.query_by_label("One per cabin layout").is_some());
    // The only checked main texture can't be unchecked.
    let locked = |h: &H, label: &str| {
        h.get_by_role_and_label(Role::CheckBox, label)
            .accesskit_node()
            .is_disabled()
    };
    assert!(locked(&h, "Standard cab"));
    set_checkbox(&mut h, "High roof", true);
    assert!(!locked(&h, "Standard cab"));
    set_checkbox(&mut h, "Standard cab", false);
    assert!(locked(&h, "High roof"));
    assert!(!disabled(&h));
    // Accessories can all be left out.
    for a in ["Chassis", "Cab accessories", "Side skirts"] {
        set_checkbox(&mut h, a, false);
    }
    assert!(h.query_by_label("High roof (4096²)").is_some());
    assert!(h.query_by_label("Chassis (4096²)").is_none());
    assert!(h.query_by_label("1 main").is_some());
    assert!(h.query_by_label("1 texture in total").is_some());
}

#[test]
fn textures_that_will_be_created() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles.install_samples().unwrap();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::CheckBox, "TruckPaint Sample Truck")
        .click();
    settle(&mut h);
    assert!(h.query_by_label("1 main + 3 accessories").is_some());
    for texture in [
        "Standard cab (4096²)",
        "Chassis (4096²)",
        "Cab accessories (1024²)",
        "Side skirts (1024²)",
    ] {
        assert!(h.query_by_label(texture).is_some(), "{texture}");
    }
    assert!(h.query_by_label("High roof (4096²)").is_none());
    assert!(h.query_by_label("4 textures in total").is_some());
}

#[test]
fn a_trailer_needs_no_choice_of_main_texture() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles.install_samples().unwrap();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::CheckBox, "TruckPaint Sample Trailer")
        .click();
    settle(&mut h);
    let base = h.get_by_role_and_label(Role::CheckBox, "Base");
    assert!(base.accesskit_node().is_disabled());
    assert!(toggled(&h, Role::CheckBox, "Base"));
    assert!(toggled(&h, Role::CheckBox, "Mudflaps"));
    assert!(h.query_by_label("Single main texture").is_some());
    assert!(
        !h.get_by_label("Create Project")
            .accesskit_node()
            .is_disabled()
    );
    h.get_by_label("Create Project").click();
    h.run();
    let names: Vec<&str> = ws(&h)
        .project
        .surfaces
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "Base",
            "Curtain body 13.6 m",
            "Curtain body 10.5 m",
            "Mudflaps"
        ]
    );
}

const TRUCK: &str = "TruckPaint Sample Truck";
const TRAILER: &str = "TruckPaint Sample Trailer";

/// New Project with the sample vehicles installed, `names` checked in
/// order.
fn wizard_with(h: &mut H, names: &[&str]) {
    h.state_mut().vehicles.install_samples().unwrap();
    h.get_by_label("New Project").click();
    h.run();
    for name in names {
        h.get_by_role_and_label(Role::CheckBox, name).click();
        settle(h);
    }
}

fn create_disabled(h: &H) -> bool {
    h.get_by_label("Create Project")
        .accesskit_node()
        .is_disabled()
}

/// start-screen "A truck and a trailer at once".
#[test]
fn a_truck_and_a_trailer_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    wizard_with(&mut h, &[TRUCK, TRAILER]);
    assert!(toggled(&h, Role::CheckBox, TRUCK));
    assert!(toggled(&h, Role::CheckBox, TRAILER));
    // Textures created: the truck's, then the trailer's.
    assert!(h.query_by_label("Standard cab (4096²)").is_some());
    assert!(h.query_by_label("Mudflaps (512²)").is_some());
    assert!(h.query_by_label("8 textures in total").is_some());
    h.get_by_label("Create Project").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.name, TRUCK);
    let vehicles: Vec<&str> = p.vehicles.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(vehicles, [TRUCK, TRAILER]);
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Standard cab",
            "Chassis",
            "Cab accessories",
            "Side skirts",
            "Base",
            "Curtain body 13.6 m",
            "Curtain body 10.5 m",
            "Mudflaps"
        ]
    );
    assert_eq!(p.active_surface, 0, "the truck's Standard cab");
}

/// start-screen "One card open at a time".
#[test]
fn one_card_open_at_a_time() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    wizard_with(&mut h, &[TRUCK, TRAILER]);
    assert!(h.query_by_label("2 vehicles").is_some());
    // The trailer's card is open, the truck's folded.
    assert!(h.query_by_role_and_label(Role::CheckBox, "Base").is_some());
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Mudflaps")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Standard cab")
            .is_none()
    );
    assert!(
        h.query_by_label("Standard cab · 1 main + 3 accessories")
            .is_some()
    );
    assert!(h.query_by_role_and_label(Role::Button, TRAILER).is_none());
    h.get_by_role_and_label(Role::Button, TRUCK).click();
    settle(&mut h);
    // Now the truck's is open and the trailer's folded.
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Standard cab")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Mudflaps")
            .is_none()
    );
    assert!(h.query_by_label("Base · 1 main + 3 accessories").is_some());
    assert!(h.query_by_role_and_label(Role::Button, TRUCK).is_none());
    // The fleet keeps its order.
    let Some(Modal::NewProject(d)) = &h.state().modal else {
        panic!("wizard");
    };
    let order: Vec<&str> = d.vehicles.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        order,
        [
            tp_app::vehicles::SAMPLE_ID,
            tp_app::vehicles::SAMPLE_TRAILER_ID
        ]
    );
}

/// start-screen "Remove from the fleet".
#[test]
fn remove_from_the_fleet() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    wizard_with(&mut h, &[TRUCK, TRAILER]);
    h.query_all_by_role_and_label(Role::Button, "Remove")
        .next()
        .expect("Remove on the first card")
        .click();
    settle(&mut h);
    assert!(!toggled(&h, Role::CheckBox, TRUCK), "unchecked in the list");
    assert!(toggled(&h, Role::CheckBox, TRAILER));
    assert!(h.query_by_label("1 vehicle").is_some());
    assert!(h.query_by_label("Standard cab (4096²)").is_none());
    assert!(h.query_by_label("Base (2048²)").is_some());
    assert!(h.query_by_label("4 textures in total").is_some());
}

/// start-screen "Unchecking in the list".
#[test]
fn unchecking_in_the_list() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    wizard_with(&mut h, &[TRUCK]);
    assert!(!create_disabled(&h));
    h.get_by_role_and_label(Role::CheckBox, TRUCK).click();
    settle(&mut h);
    assert!(!toggled(&h, Role::CheckBox, TRUCK));
    assert!(h.query_by_label("Check vehicles in the list.").is_some());
    assert!(create_disabled(&h));
}

/// start-screen "Changing the game".
#[test]
fn changing_the_game_unchecks_the_vehicles() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    wizard_with(&mut h, &[TRUCK, TRAILER]);
    h.get_by_label("American Truck Simulator").click();
    settle(&mut h);
    let Some(Modal::NewProject(d)) = &h.state().modal else {
        panic!("wizard");
    };
    assert!(d.vehicles.is_empty(), "no vehicle is checked");
    assert!(create_disabled(&h));
    h.get_by_label("Euro Truck Simulator 2").click();
    settle(&mut h);
    assert!(!toggled(&h, Role::CheckBox, TRUCK));
    assert!(!toggled(&h, Role::CheckBox, TRAILER));
}

#[test]
fn add_vehicle_lists_only_the_project_game() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    for (id, name, game, kind) in [
        ("scs.krone.dry_liner", "Krone Dry Liner", "ets2", "trailer"),
        ("scs.peterbilt.579", "Peterbilt 579", "ats", "truck"),
    ] {
        let file = other_package_file(dir.path(), id, name, game, kind);
        h.state_mut().vehicles.install_file(&file).unwrap();
    }
    create_sample_project(&mut h, &["Standard cab"], &[]);
    h.get_by_label("Add Vehicle…").click();
    settle(&mut h);
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Krone Dry Liner")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Peterbilt 579")
            .is_none()
    );
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "TruckPaint Sample Truck")
            .is_none(),
        "vehicles already in the project are hidden"
    );
    h.get_by_role_and_label(Role::CheckBox, "Krone Dry Liner")
        .click();
    settle(&mut h);
    // The Add Vehicle dialog's, over the Game versions' + Add.
    common::last(&h, "Add").click();
    settle(&mut h);
    assert!(h.state().modal.is_none());
    let p = &ws(&h).project;
    assert_eq!(p.vehicles.len(), 2);
    assert_eq!(p.surfaces.len(), 5);
    assert_eq!(p.active_surface, 4);
    assert!(h.query_by_label("2 vehicles · 5 textures").is_some());
    // The trailer can be removed (no artwork: no question); the last
    // vehicle cannot.
    vehicle_action(&mut h, "Krone Dry Liner", "Remove from Project");
    assert_eq!(ws(&h).project.vehicles.len(), 1);
    h.get_by_label("Actions for TruckPaint Sample Truck")
        .click();
    h.run();
    assert!(
        h.get_by_label("Remove from Project")
            .accesskit_node()
            .is_disabled()
    );
    h.key_press(Key::Escape);
    h.run();
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.vehicles.len(), 2);
}

#[test]
fn the_tree_groups_main_textures_and_accessories() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab", "High roof"], &[]);
    h.get_by_label("Add Vehicle…").click();
    settle(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "TruckPaint Sample Trailer")
        .click();
    settle(&mut h);
    // The Add Vehicle dialog's, over the Game versions' + Add.
    common::last(&h, "Add").click();
    settle(&mut h);
    // One heading of each per vehicle: both vehicles have main textures and
    // accessories.
    assert_eq!(h.query_all_by_label("Main textures").count(), 2);
    assert_eq!(h.query_all_by_label("Accessories").count(), 2);
    for row in [
        "TruckPaint Sample Truck › Standard cab",
        "TruckPaint Sample Truck › High roof",
        "TruckPaint Sample Truck › Side skirts",
        "TruckPaint Sample Trailer › Base",
    ] {
        assert!(
            h.query_by_label_contains(&format!("Texture {row},"))
                .is_some(),
            "{row}"
        );
    }
    pick_texture(&mut h, "TruckPaint Sample Trailer › Mudflaps");
    let p = &ws(&h).project;
    assert_eq!(p.surfaces[p.active_surface].name, "Mudflaps");
    assert!(
        breadcrumb_shows(&h, "TruckPaint Sample Trailer › Accessories › Mudflaps"),
        "breadcrumb"
    );
}

#[test]
fn removing_a_texture_with_artwork_asks_and_undoes() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab", "High roof"], &[]);
    let id = ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(100.0, 100.0), 0.0),
        1.0,
    );
    h.run();
    vehicle_action(&mut h, "TruckPaint Sample Truck", "Textures…");
    set_checkbox(&mut h, "Standard cab", false);
    h.get_by_label("Apply").click();
    h.run();
    assert!(
        h.query_by_label_contains("The artwork of Standard cab")
            .is_some()
    );
    assert_eq!(ws(&h).project.surfaces.len(), 5, "nothing removed yet");
    h.get_by_label("Remove").click();
    h.run();
    assert!(h.state().modal.is_none());
    assert_eq!(ws(&h).project.surfaces.len(), 4);
    assert_eq!(ws(&h).project.surfaces[0].name, "High roof");
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.surfaces.len(), 5);
    assert!(ws(&h).project.surfaces[0].get(id).is_some());
}

#[test]
fn adding_an_accessory_later_keeps_the_package_order() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab"], &["Cab accessories"]);
    let names = |h: &H| -> Vec<String> {
        ws(h)
            .project
            .surfaces
            .iter()
            .map(|s| s.name.clone())
            .collect()
    };
    assert_eq!(names(&h), ["Standard cab", "Chassis", "Side skirts"]);
    vehicle_action(&mut h, "TruckPaint Sample Truck", "Textures…");
    // The only main texture painted can't be unchecked here either.
    assert!(
        h.get_by_role_and_label(Role::CheckBox, "Standard cab")
            .accesskit_node()
            .is_disabled()
    );
    set_checkbox(&mut h, "Cab accessories", true);
    h.get_by_label("Apply").click();
    h.run();
    assert_eq!(
        names(&h),
        ["Standard cab", "Chassis", "Cab accessories", "Side skirts"]
    );
}

#[test]
fn textures_need_the_recorded_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab"], &[]);
    h.state_mut()
        .vehicles
        .remove(SAMPLE_ID, &"1.1.0".parse().unwrap())
        .unwrap();
    h.run();
    vehicle_action(&mut h, "TruckPaint Sample Truck", "Textures…");
    assert!(
        h.query_by_label_contains("Version 1.1.0 of this vehicle is not installed")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "High roof")
            .is_none()
    );
}

#[test]
fn only_the_vehicle_with_a_newer_version_offers_an_update() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    let trailer = other_package_file(
        dir.path(),
        "scs.krone.cool_liner",
        "Krone Cool Liner",
        "ets2",
        "trailer",
    );
    h.state_mut().vehicles.install_file(&trailer).unwrap();
    let v100 = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/vehicles/community.truckpaint.sample_truck-1.0.0.tpv");
    h.state_mut().vehicles.install_file(&v100).unwrap();
    // A project on 1.0.0 (Standard cab, Chassis, Cab accessories) with the
    // trailer, then 1.1.0 installed.
    let package = h
        .state()
        .vehicles
        .load(SAMPLE_ID, &"1.0.0".parse().unwrap())
        .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let project = tp_app::vehicle_project::fleet_project("F", &[(&package, &textures)]).unwrap();
    h.state_mut().open_project(project);
    let krone = h
        .state()
        .vehicles
        .load("scs.krone.cool_liner", &"1.0.0".parse().unwrap())
        .unwrap();
    ws_mut(&mut h).add_vehicle(&krone, &[], 1.0).unwrap();
    h.state_mut().vehicles.install_samples().unwrap();
    h.run();
    assert!(
        h.query_by_label("Update the template of TruckPaint Sample Truck")
            .is_some()
    );
    assert!(
        h.query_by_label("Update the template of Krone Cool Liner")
            .is_none()
    );
    // The menu command applies to the active surface's vehicle: the
    // trailer, which has no update.
    assert_eq!(ws(&h).project.active_surface, 3);
    h.get_by_label("Update the template of TruckPaint Sample Truck")
        .click();
    h.run();
    // The new accessory is offered checked; the High roof (a main texture
    // not painted) unchecked.
    assert!(h.query_by_label("New in this version:").is_some());
    assert!(toggled(&h, Role::CheckBox, "Side skirts"));
    assert!(!toggled(&h, Role::CheckBox, "High roof"));
    h.get_by_label("Update").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.vehicles[0].version, "1.1.0");
    assert_eq!(p.vehicles[1].version, "1.0.0");
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Standard cab",
            "Chassis",
            "Cab accessories",
            "Side skirts",
            "Body"
        ]
    );
    assert_eq!(
        p.surfaces[0].template.as_ref().unwrap().status,
        TemplateStatus::LayoutChanged
    );
    assert_eq!(
        p.surfaces[1].size,
        Size::new(4096.0, 4096.0),
        "the chassis grew"
    );
}

/// Whether the row of texture `path` in the fleet tree is highlighted.
fn texture_selected(h: &H, path: &str) -> bool {
    h.get_by_label_contains(&format!("Texture {path},"))
        .accesskit_node()
        .toggled()
        .is_some_and(|t| t == egui::accesskit::Toggled::True)
}

#[test]
fn texture_rows_show_the_artwork_without_the_template() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab"], &[]);
    pick_texture(&mut h, "TruckPaint Sample Truck › Chassis");
    assert!(texture_selected(&h, "TruckPaint Sample Truck › Chassis"));
    assert!(!texture_selected(
        &h,
        "TruckPaint Sample Truck › Standard cab"
    ));
    let chassis = ws(&h).project.active_surface;
    assert!(
        ws(&h).project.surfaces[chassis]
            .template
            .as_ref()
            .is_some_and(|t| t.visible),
        "the template is shown on the canvas"
    );
    // Its marker is a hollow ring: Empty.
    let row = "Texture TruckPaint Sample Truck › Chassis, ";
    assert!(h.query_by_label(&format!("{row}Empty")).is_some());
    assert!(h.query_by_label("Modified").is_none());
    // A red rectangle covering the whole Chassis texture.
    let red = tp_core::document::Rgba::rgb(220, 0, 0);
    let side = ws(&h).project.surface().max_side();
    ws_mut(&mut h).style.fill = tp_core::document::Paint::Solid(red);
    ws_mut(&mut h).style.stroke_enabled = false;
    ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(
            Point::new(side / 2.0, side / 2.0),
            Size::new(side, side),
            0.0,
        ),
        1.0,
    );
    let shows_red = |h: &H| {
        let thumbnails = &ws(h).thumbnails;
        (0..8).all(|y| {
            (0..8).all(|x| {
                let uv = [(x as f32 + 0.5) / 8.0, (y as f32 + 0.5) / 8.0];
                thumbnails.color_at(chassis, uv) == Some(red)
            })
        })
    };
    for _ in 0..1000 {
        h.step();
        if shows_red(&h) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        shows_red(&h),
        "the Chassis thumbnail shows the red artwork and no template lines"
    );
    // Other textures were not rendered again.
    let renders = ws(&h).thumbnails.renders;
    settle(&mut h);
    assert_eq!(ws(&h).thumbnails.renders, renders, "nothing else changed");
    // Its marker is now a filled dot: Modified.
    assert!(h.query_by_label(&format!("{row}Modified")).is_some());
    assert_eq!(h.query_all_by_label("Modified").count(), 1);
}

#[test]
fn an_update_flags_the_texture_rows() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    let v100 = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/vehicles/community.truckpaint.sample_truck-1.0.0.tpv");
    h.state_mut().vehicles.install_file(&v100).unwrap();
    let package = h
        .state()
        .vehicles
        .load(SAMPLE_ID, &"1.0.0".parse().unwrap())
        .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let project = tp_app::vehicle_project::fleet_project("F", &[(&package, &textures)]).unwrap();
    h.state_mut().open_project(project);
    show_workshop(&mut h);
    assert!(h.query_by_label_contains("Layout changed").is_none());
    h.state_mut().vehicles.install_samples().unwrap();
    h.run();
    h.get_by_label("Update the template of TruckPaint Sample Truck")
        .click();
    h.run();
    h.get_by_label("Update").click();
    settle(&mut h);
    // The Standard cab's layout changed in 1.1.0: its row's marker is the
    // warning icon, saying To check and why.
    assert_eq!(
        h.query_all_by_label("To check\nLayout changed in 1.1.0")
            .count(),
        1
    );
    assert!(
        h.query_by_label_contains("Sample Truck › Standard cab, To check")
            .is_some()
    );
}

/// The search field is as tall as the filter combos beside it, in the
/// Vehicle Library and in Add Vehicle.
#[test]
fn search_fields_as_tall_as_the_filters() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    let height = |h: &H, label: &str| h.get_by_label(label).rect().height();
    h.state_mut().vehicles.install_samples().unwrap();
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    settle(&mut h);
    let search = height(&h, "Search vehicles");
    assert_eq!(search, height(&h, "Kind"), "library");
    assert_eq!(search, height(&h, "Game"), "library");
    h.state_mut().modal = None;
    create_sample_project(&mut h, &["Standard cab"], &[]);
    h.get_by_label("Add Vehicle…").click();
    settle(&mut h);
    assert_eq!(
        height(&h, "Search vehicles"),
        height(&h, "Kind"),
        "Add Vehicle"
    );
}

/// Installs version `version` of the sample truck with `changelog`.
fn install_with_changelog(h: &mut H, version: &str, textures: &[SampleTexture], changelog: &str) {
    let mut manifest = sample::manifest(ID, "Sample Truck", version, textures);
    manifest["changelog"] = serde_json::json!(changelog);
    let bytes = sample::zip(&manifest, &sample::templates(textures));
    h.state_mut().vehicles.install_bytes(&bytes).unwrap();
    h.run();
}

#[test]
fn update_template_counts_objects_to_reposition_and_shows_the_changelog() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    // A logo, a text and a group of two rectangles on the Cabin: 4 objects.
    let shape = |h: &mut H, x: f64| {
        ws_mut(h).create_shape(
            ShapeKind::rectangle(),
            Frame::new(Point::new(x, 500.0), Size::new(100.0, 100.0), 0.0),
            1.0,
        )
    };
    ws_mut(&mut h).set_active_surface(0);
    shape(&mut h, 300.0);
    shape(&mut h, 600.0);
    let a = shape(&mut h, 900.0);
    let b = shape(&mut h, 1200.0);
    ws_mut(&mut h).selection = vec![a, b];
    ws_mut(&mut h).group_selection(2.0);
    h.run();

    let v13 = [
        tex("cabin", "Cabin", 4096, 2),
        tex("chassis", "Chassis", 2048, 1),
        tex("accessories", "Accessories", 1024, 1),
    ];
    // Without a changelog: no changelog section.
    h.state_mut()
        .vehicles
        .install_file(&package_file(dir.path(), "1.3.0", &v13))
        .unwrap();
    h.run();
    h.get_by_label("Update the template of Sample Truck")
        .click();
    h.run();
    assert!(h.query_by_label_contains("What changed in").is_none());
    assert!(
        h.query_by_label("• Cabin: layout changed, 4 objects to reposition")
            .is_some()
    );
    assert!(
        h.query_by_label("• Chassis: template replaced").is_some(),
        "no count"
    );
    h.key_press(Key::Escape);
    h.run();

    install_with_changelog(&mut h, "1.4.0", &v13, "Door layout redrawn.");
    h.get_by_label("Update the template of Sample Truck")
        .click();
    h.run();
    assert!(h.query_by_label("What changed in 1.4.0:").is_some());
    assert!(h.query_by_label("Door layout redrawn.").is_some());
    h.get_by_label("Update").click();
    h.run();
    let t = ws(&h).project.surfaces[0].template.as_ref().unwrap();
    assert_eq!(t.flagged.len(), 4, "the count shown is what gets flagged");
}

/// Version 1.3.0 of the sample truck: the Cabin's layout raised and Side
/// skirts added.
fn v13_textures() -> Vec<SampleTexture> {
    vec![
        tex("cabin", "Cabin", 4096, 2),
        tex("chassis", "Chassis", 2048, 1),
        tex("accessories", "Accessories", 1024, 1),
        tex("side_skirts", "Side skirts", 1024, 1),
    ]
}

/// Opens the Vehicle Library and waits until the recent projects are read.
fn open_library(h: &mut H) {
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    for _ in 0..200 {
        settle(h);
        if !h.state().project_summaries.is_reading() {
            break;
        }
    }
    settle(h);
}

/// Writes a project made with version `version` holding `objects`
/// rectangles on its Cabin, and lists it among the recent projects.
fn recent_project(h: &mut H, dir: &Path, name: &str, version: &str, objects: usize) -> PathBuf {
    let package = h
        .state()
        .vehicles
        .load(ID, &version.parse().unwrap())
        .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let mut project =
        tp_app::vehicle_project::fleet_project(name, &[(&package, &textures)]).unwrap();
    project.active_surface = 0;
    for i in 0..objects {
        project.add(tp_core::document::Object::new(
            tp_core::document::ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(
                Point::new(100.0 + i as f64 * 200.0, 100.0),
                Size::new(50.0, 50.0),
                0.0,
            ),
        ));
    }
    let path = dir.join(format!("{name}.truckpaint"));
    tp_file::write(&project, &path).unwrap();
    h.state_mut()
        .prefs
        .recent
        .push(tp_app::prefs::RecentProject {
            name: name.into(),
            path: path.clone(),
            last_opened: 0,
        });
    path
}

#[test]
fn library_update_summary_and_projects_touched() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    // One version: no summary.
    open_library(&mut h);
    assert!(h.query_by_label_contains("Update summary").is_none());
    h.key_press(Key::Escape);
    h.run();

    let ardent = recent_project(&mut h, dir.path(), "Transports Ardent", "1.2.0", 3);
    let bytes = std::fs::read(&ardent).unwrap();
    let modified = std::fs::metadata(&ardent).unwrap().modified().unwrap();
    // A deleted file, a damaged one, and the same file under another path.
    recent_project(&mut h, dir.path(), "Gone", "1.2.0", 1);
    std::fs::remove_file(dir.path().join("Gone.truckpaint")).unwrap();
    std::fs::write(dir.path().join("Broken.truckpaint"), b"not a project").unwrap();
    h.state_mut()
        .prefs
        .recent
        .push(tp_app::prefs::RecentProject {
            name: "Broken".into(),
            path: dir.path().join("Broken.truckpaint"),
            last_opened: 0,
        });
    h.state_mut()
        .prefs
        .recent
        .push(tp_app::prefs::RecentProject {
            name: "Transports Ardent".into(),
            path: dir.path().join(".").join("Transports Ardent.truckpaint"),
            last_opened: 0,
        });
    h.state_mut()
        .vehicles
        .install_file(&package_file(dir.path(), "1.3.0", &v13_textures()))
        .unwrap();
    let up_to_date = recent_project(&mut h, dir.path(), "New fleet", "1.3.0", 2);

    // From the home screen: the dialog shows at once, the projects follow.
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.step();
    assert!(h.query_by_label("Reading recent projects…").is_some());
    open_library(&mut h);
    assert!(h.query_by_label("Update summary: 1.3.0").is_some());
    assert!(h.query_by_label("Layout changed: Cabin").is_some());
    assert!(h.query_by_label("New: Side skirts").is_some());
    assert_eq!(
        h.query_all_by_label("Transports Ardent: 1 texture, 3 objects to reposition")
            .count(),
        1,
        "listed once"
    );
    assert!(h.query_by_label("Broken: Can’t be read").is_some());
    assert!(h.query_by_label_contains("Gone:").is_none());
    assert!(
        h.query_by_label_contains("New fleet:").is_none(),
        "up to date"
    );
    assert!(
        h.query_by_label("Only recent projects are listed")
            .is_some()
    );
    // Nothing to update from the home screen.
    assert!(
        h.get_by_label("Update Sample Truck")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        h.query_by_label("Open a project that uses this vehicle to update it.")
            .is_some()
    );
    assert_eq!(std::fs::read(&ardent).unwrap(), bytes, "files untouched");
    assert_eq!(
        std::fs::metadata(&ardent).unwrap().modified().unwrap(),
        modified
    );
    assert!(up_to_date.is_file());

    // Later closes the library, changing nothing.
    h.get_by_label("Later").click();
    settle(&mut h);
    assert!(h.state().modal.is_none());
}

#[test]
fn library_update_runs_update_template_on_the_open_project() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    h.state_mut()
        .vehicles
        .install_file(&package_file(dir.path(), "1.3.0", &v13_textures()))
        .unwrap();
    open_library(&mut h);
    // The open project is counted from memory, unsaved.
    assert!(
        h.query_by_label_contains("1 texture, nothing to reposition")
            .is_some()
    );
    h.get_by_label("Update Sample Truck").click();
    settle(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::UpdateTemplate(_))));
}

/// At 960 × 600 in German, the update summary, a project line and the
/// Update Template counts stay inside the window.
#[test]
fn german_update_impact_fits_the_minimum_window() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    recent_project(&mut h, dir.path(), "Transports Ardent", "1.2.0", 3);
    create_vehicle_project(&mut h);
    ws_mut(&mut h).set_active_surface(0);
    ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(300.0, 300.0), Size::new(100.0, 100.0), 0.0),
        1.0,
    );
    install_with_changelog(
        &mut h,
        "1.3.0",
        &v13_textures(),
        "Türlayout nach dem Spielupdate neu gezeichnet.",
    );
    h.set_size(Vec2::new(960.0, 600.0));
    h.state_mut()
        .prefs
        .set_language(Some(tp_i18n::Language::German));
    open_library(&mut h);
    let inside = |h: &H, label: &str| {
        let r = h.get_by_label(label).rect();
        assert!(r.left() >= 0.0 && r.right() <= 960.0, "{label}: {r:?}");
    };
    inside(&h, "Update-Übersicht: 1.3.0");
    inside(
        &h,
        "Transports Ardent: 1 Textur, 3 Objekte neu zu positionieren",
    );
    h.key_press(Key::Escape);
    h.run();
    h.state_mut()
        .queue
        .push(tp_app::commands::CommandId::UpdateTemplate);
    settle(&mut h);
    // Wrapped lines: every node of the dialog naming the Cabin fits.
    let lines: Vec<_> = h
        .query_all(By::new().predicate(|n| {
            n.value()
                .is_some_and(|v| v.contains("Cabin: Layout geändert"))
        }))
        .map(|n| n.rect())
        .collect();
    assert!(!lines.is_empty());
    for r in lines {
        assert!(r.left() >= 0.0 && r.right() <= 960.0, "{r:?}");
    }
    inside(&h, "Türlayout nach dem Spielupdate neu gezeichnet.");
}
