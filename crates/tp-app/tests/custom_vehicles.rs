//! Headless tests for custom vehicles: the Custom Vehicle dialog from New
//! Project, Add Vehicle… and the Vehicle Library, its template rows and
//! checks, building and installing the package, New Version… and Export….

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::prefs::Prefs;
use tp_app::state::Modal;
use tp_app::ui::custom_vehicle::CustomVehicleDialog;
use tp_app::vehicles::VehicleLibrary;
use tp_app::workspace::Workspace;
use tp_core::TemplateStatus;
use tp_pack::custom::CustomVehicle;
use tp_pack::dds::build as dds;
use tp_vehicles::{Game, Kind, Package, Role as PartRole};

type H = Harness<'static, AppState>;

/// The app with an empty library in `dir/library`.
fn app(dir: &Path) -> H {
    let mut state = AppState::with_prefs(Prefs::default(), None);
    state.vehicles = VehicleLibrary::open(&dir.join("library"));
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 1400.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.run();
    h
}

/// Lets the UI settle (modals re-center over a few frames, templates
/// render in the background).
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

fn dialog(h: &H) -> &CustomVehicleDialog {
    match &h.state().modal {
        Some(Modal::CustomVehicle(d)) => d,
        _ => panic!("the Custom Vehicle dialog should be open"),
    }
}

fn form(h: &mut H) -> &mut CustomVehicle {
    match &mut h.state_mut().modal {
        Some(Modal::CustomVehicle(d)) => &mut d.form,
        _ => panic!("the Custom Vehicle dialog should be open"),
    }
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

/// A DDS file of `side`² in `format` (FourCC `cc`), its blocks zeroed.
fn dds_file(side: u32, format: texpresso::Format, cc: &[u8; 4]) -> Vec<u8> {
    let mut bytes = dds::header(side, side, 0x4, cc, 0, [0; 4]);
    let data = format.compressed_size(side as usize, side as usize);
    bytes.resize(bytes.len() + data, 0);
    bytes
}

fn png(side: u32) -> Vec<u8> {
    tp_vehicles::sample::png(side)
}

/// A file dropped from the operating system.
#[derive(Debug)]
struct TestFile {
    path: PathBuf,
    bytes: Vec<u8>,
}

impl egui::DroppedFile for TestFile {
    fn path(&self) -> &Path {
        &self.path
    }

    fn bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.bytes.clone())
    }
}

/// Drops `files` with the pointer at `at`.
fn drop_at(h: &mut H, at: Pos2, files: Vec<(&str, Vec<u8>)>) {
    h.event(Event::PointerMoved(at));
    h.step();
    h.input_mut().dropped_files = files
        .into_iter()
        .map(|(name, bytes)| {
            Arc::new(TestFile {
                path: PathBuf::from("/tmp").join(name),
                bytes,
            }) as Arc<dyn egui::DroppedFile + Send + Sync>
        })
        .collect();
    h.step();
    h.input_mut().dropped_files.clear();
    settle(h);
}

fn drop_files(h: &mut H, files: Vec<(&str, Vec<u8>)>) {
    drop_at(h, Pos2::new(5.0, 5.0), files);
}

/// Types `text` in the text field `name`, replacing its content, and
/// leaves the field with Enter.
fn type_into(h: &mut H, name: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, name).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, name)
        .type_text(text);
    h.run();
    h.key_press(Key::Enter);
    h.run();
}

/// Fills in a valid truck: "R 2024" of "Scania", game path
/// `scania.r_2024`, one main texture from `cabin.png`.
fn fill_truck(h: &mut H) {
    let f = form(h);
    f.name = "R 2024".into();
    f.brand = "Scania".into();
    f.path = "scania.r_2024".into();
    drop_files(h, vec![("cabin.png", png(64))]);
}

fn create_enabled(h: &H) -> bool {
    !h.get_by_role_and_label(Role::Button, "Create")
        .accesskit_node()
        .is_disabled()
}

/// Clicks Create and waits for the package to be built and installed.
fn build(h: &mut H) {
    assert!(create_enabled(h), "Create should be enabled");
    h.get_by_role_and_label(Role::Button, "Create").click();
    h.step();
    for _ in 0..2000 {
        h.step();
        let building = matches!(&h.state().modal, Some(Modal::CustomVehicle(d)) if d.is_building());
        if !building {
            settle(h);
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the package was not built");
}

fn open_wizard(h: &mut H) {
    h.get_by_label("New Project").click();
    settle(h);
}

fn wizard_choice(h: &H) -> Option<String> {
    match &h.state().modal {
        Some(Modal::NewProject(draft)) => draft.vehicles.last().map(|c| c.id.clone()),
        _ => panic!("the New Project wizard should be open"),
    }
}

/// Installs the custom truck `custom.scania.r_2024` 1.0.0 (a main texture
/// "cabin" and an accessory "Mirrors") without the dialog.
fn install_custom_truck(h: &mut H, game: Game) {
    let mut v = CustomVehicle::new(game);
    v.name = "R 2024".into();
    v.brand = "Scania".into();
    v.path = "scania.r_2024".into();
    v.add(tp_pack::custom::probe("cabin.png", png(64).into()).unwrap());
    v.add(tp_pack::custom::probe("mirrors.png", png(16).into()).unwrap());
    v.rows[1].name = "Mirrors".into();
    v.rows[1].game_ids = "mirror.painted".into();
    let packed = v.pack(&mut || {}).unwrap();
    h.state_mut().vehicles.install_bytes(&packed.bytes).unwrap();
    h.run();
}

#[test]
fn no_vehicle_installed_offers_custom_vehicle() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    open_wizard(&mut h);
    assert!(
        h.get_by_label("Create Project")
            .accesskit_node()
            .is_disabled(),
        "Create Project is disabled"
    );
    for label in ["Install…", "Install the sample vehicles", "Custom Vehicle…"] {
        assert!(h.query_by_label(label).is_some(), "{label} is offered");
    }
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    assert!(h.query_by_label("Custom Vehicle").is_some(), "dialog title");
    let d = dialog(&h);
    assert_eq!((d.form.kind, d.form.game), (Kind::Truck, Game::Ets2));
    assert!(d.form.rows.is_empty() && d.form.name.is_empty());
}

#[test]
fn escape_returns_to_the_wizard_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles.install_samples().unwrap();
    open_wizard(&mut h);
    h.get_by_role_and_label(Role::CheckBox, common::SAMPLE)
        .click();
    settle(&mut h);
    let before = wizard_choice(&h);
    assert!(before.is_some());
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    form(&mut h).name = "Half done".into();
    h.key_press(Key::Escape);
    settle(&mut h);
    assert_eq!(wizard_choice(&h), before, "the wizard keeps its choice");
    assert_eq!(h.state().vehicles.vehicles().len(), 2, "nothing installed");
}

#[test]
fn dropping_the_games_templates() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    open_wizard(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    drop_files(
        &mut h,
        vec![
            ("cabin.dds", dds_file(4096, texpresso::Format::Bc3, b"DXT5")),
            (
                "mirrors.dds",
                dds_file(1024, texpresso::Format::Bc1, b"DXT1"),
            ),
        ],
    );
    let rows = &dialog(&h).form.rows;
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (rows[0].name.as_str(), rows[0].role, rows[0].size.label()),
        ("cabin", PartRole::Main, "4096²".to_owned())
    );
    assert_eq!(
        (rows[1].name.as_str(), rows[1].role, rows[1].size.label()),
        ("mirrors", PartRole::Accessory, "1024²".to_owned())
    );
    assert!(h.query_all_by_label("Width of texture 1").next().is_some());
    assert!(h.query_all_by_label("Height of texture 1").next().is_some());
    assert!(h.query_by_label_contains("will be scaled").is_none());
    assert!(
        h.query_by_label_contains("cabin.dds · 4096 × 4096 px")
            .is_some()
    );

    // An unsupported DDS is reported; the PNG of the same drop is added.
    let bc7 = dds::dx10(&dds::quadrants(), texpresso::Format::Bc3, 98);
    drop_files(&mut h, vec![("cabin2.png", png(32)), ("glass.dds", bc7)]);
    assert_eq!(dialog(&h).form.rows.len(), 3);
    assert!(
        h.query_by_label_contains("glass.dds can't be read: its DDS compression (BC7)")
            .is_some()
    );

    // A rectangular image keeps its shape, without a warning.
    let image = |width, height| {
        let mut out = Vec::new();
        image::RgbaImage::new(width, height)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    };
    drop_files(&mut h, vec![("wide.png", image(4096, 2048))]);
    assert_eq!(
        dialog(&h).form.rows[3].size,
        tp_vehicles::TextureSize::new(4096, 2048)
    );
    assert!(h.query_by_label_contains("scaled to 4096 × 2048").is_none());

    // Sides that aren't powers of two are rounded, with a warning.
    drop_files(&mut h, vec![("odd.png", image(4000, 1000))]);
    assert_eq!(
        dialog(&h).form.rows[4].size,
        tp_vehicles::TextureSize::new(4096, 1024)
    );
    assert!(
        h.query_by_label("The template will be scaled to 4096 × 1024.")
            .is_some()
    );
}

#[test]
fn replace_keeps_the_rows_choices() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    open_wizard(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    drop_files(
        &mut h,
        vec![("cabin.png", png(64)), ("mirrors.png", png(16))],
    );
    {
        let row = &mut form(&mut h).rows[1];
        row.name = "Mirrors".into();
        row.game_ids = "mirror.painted".into();
    }
    let v2 = dir.path().join("cabin_v2.png");
    std::fs::write(&v2, png(128)).unwrap();
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        templates: [vec![v2]].into(),
        ..Default::default()
    });
    h.get_by_label("Replace the template of texture 2").click();
    settle(&mut h);
    let row = &dialog(&h).form.rows[1];
    assert_eq!(row.template.file_name, "cabin_v2.png");
    assert_eq!(
        (row.name.as_str(), row.role, row.game_ids.as_str()),
        ("Mirrors", PartRole::Accessory, "mirror.painted")
    );

    // Dropping one file on a row replaces it too.
    let rect = dialog(&h).form.rows.len();
    assert_eq!(rect, 2);
    let first = h.get_by_label("Name of texture 1").rect().center();
    drop_at(&mut h, first, vec![("cabin_v3.png", png(32))]);
    let d = dialog(&h);
    assert_eq!(d.form.rows.len(), 2);
    assert_eq!(d.form.rows[0].template.file_name, "cabin_v3.png");
    assert_eq!(d.form.rows[0].name, "cabin");

    // Remove deletes a row.
    h.get_by_label("Remove texture 2").click();
    settle(&mut h);
    assert_eq!(dialog(&h).form.rows.len(), 1);
}

#[test]
fn dropped_templates_do_not_reach_the_canvas() {
    let mut h = common::harness();
    common::create_project(&mut h);
    let objects = |h: &H| -> usize { ws(h).project.surfaces.iter().map(|s| s.objects.len()).sum() };
    let before = objects(&h);
    h.state_mut()
        .queue
        .push(tp_app::commands::CommandId::AddVehicle);
    settle(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    let center = h.get_by_label("Custom Vehicle").rect().center();
    drop_at(&mut h, center, vec![("cabin.png", png(64))]);
    assert_eq!(dialog(&h).form.rows.len(), 1);
    assert_eq!(objects(&h), before, "nothing placed on the canvas");
}

#[test]
fn checks_are_shown_next_to_their_field() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    open_wizard(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    fill_truck(&mut h);
    drop_files(&mut h, vec![("mirrors.png", png(16))]);
    form(&mut h).rows[1].game_ids = "mirror.painted".into();
    settle(&mut h);
    // One cabin layout: accepted.
    assert!(create_enabled(&h));

    // Two cabin layouts need cabin names.
    {
        let f = form(&mut h);
        f.rows[0].name = "Highline".into();
        f.rows[0].game_ids = "highline".into();
        f.rows[1].name = "Topline".into();
        f.rows[1].role = PartRole::Main;
        f.rows[1].game_ids.clear();
    }
    settle(&mut h);
    assert!(!create_enabled(&h));
    assert!(
        h.query_by_label("Enter the internal names of the cabins that use this layout.")
            .is_some()
    );

    // An accessory without ids.
    {
        let f = form(&mut h);
        f.rows[1].name = "Side skirts".into();
        f.rows[1].role = PartRole::Accessory;
    }
    settle(&mut h);
    assert!(!create_enabled(&h));
    assert!(
        h.query_by_label("Enter the accessory ids this texture covers.")
            .is_some()
    );

    // A trailer with two main textures.
    {
        let f = form(&mut h);
        f.kind = Kind::Trailer;
        f.rows[1].role = PartRole::Main;
        f.rows[1].game_ids = "topline".into();
    }
    settle(&mut h);
    assert!(!create_enabled(&h));
    assert!(
        h.query_by_label_contains("A trailer has one main texture")
            .is_some()
    );
    {
        let f = form(&mut h);
        f.kind = Kind::Truck;
    }

    // An invalid game path, shown once the field is left.
    type_into(&mut h, "Game path", "Scania R");
    assert!(!create_enabled(&h));
    assert!(h.query_by_label_contains("e.g. scania.r_2016.").is_some());
    type_into(&mut h, "Game path", "scania.r_2024");
    assert!(create_enabled(&h));
}

#[test]
fn same_id_points_to_new_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_custom_truck(&mut h, Game::Ets2);
    open_wizard(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    fill_truck(&mut h);
    assert!(!create_enabled(&h));
    assert!(
        h.query_by_label_contains("custom.scania.r_2024 is already installed")
            .is_some()
    );
    assert_eq!(h.state().vehicles.vehicles().len(), 1);
}

#[test]
fn project_from_a_custom_vehicle() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    open_wizard(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    fill_truck(&mut h);
    build(&mut h);
    assert_eq!(
        wizard_choice(&h).as_deref(),
        Some("custom.scania.r_2024"),
        "the new vehicle is selected"
    );
    assert!(
        h.query_by_label_contains("Installed R 2024 1.0.0")
            .is_some()
    );
    h.get_by_label("Create Project").click();
    settle(&mut h);
    let p = &ws(&h).project;
    assert_eq!(p.name, "R 2024");
    assert_eq!(p.surfaces.len(), 1);
    assert_eq!(p.surfaces[0].name, "cabin");
    assert!(p.surfaces[0].template.is_some(), "the template is shown");
}

#[test]
fn dds_is_installed_as_a_png_of_the_same_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    open_wizard(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    {
        let f = form(&mut h);
        f.name = "Hauler".into();
        f.brand = "JDoe".into();
        f.path = "jdoe.hauler".into();
    }
    let image = dds::quadrants();
    drop_files(&mut h, vec![("cabin.dds", dds::bgra(&image))]);
    build(&mut h);
    let package = h
        .state()
        .vehicles
        .load("custom.jdoe.hauler", &semver::Version::new(1, 0, 0))
        .unwrap();
    let m = &package.manifest;
    assert_eq!(m.paint_job.main[0].texture.template, "templates/cabin.png");
    let png = image::load_from_memory(&package.template("cabin").unwrap().bytes)
        .unwrap()
        .to_rgba8();
    assert_eq!(png, image);
}

#[test]
fn custom_trailer_added_to_a_fleet() {
    let mut h = common::harness();
    common::create_project(&mut h);
    h.state_mut()
        .queue
        .push(tp_app::commands::CommandId::AddVehicle);
    settle(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    {
        let d = dialog(&h);
        assert!(d.game_locked, "the project's game can't be changed");
        assert_eq!(d.form.game, Game::Ets2);
    }
    assert!(
        h.get_by_role_and_label(Role::ComboBox, "Game")
            .accesskit_node()
            .is_disabled()
    );
    {
        let f = form(&mut h);
        f.name = "Box".into();
        f.brand = "JDoe".into();
        f.kind = Kind::Trailer;
        f.path = "jdoe.box".into();
    }
    drop_files(&mut h, vec![("base.png", png(64))]);
    build(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::AddVehicle(_))));
    // The Add Vehicle dialog's, over the Game versions' + Add.
    common::last(&h, "Add").click();
    settle(&mut h);
    let p = &ws(&h).project;
    assert_eq!(p.vehicles.len(), 2);
    let active = &p.surfaces[p.active_surface];
    assert_eq!(active.name, "base");
}

/// start-screen "Custom vehicle added to the fleet".
#[test]
fn custom_vehicle_added_to_the_fleet() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles.install_samples().unwrap();
    open_wizard(&mut h);
    h.get_by_role_and_label(Role::CheckBox, common::SAMPLE)
        .click();
    settle(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    {
        let f = form(&mut h);
        f.name = "Box".into();
        f.brand = "JDoe".into();
        f.kind = Kind::Trailer;
        f.path = "jdoe.box".into();
    }
    drop_files(&mut h, vec![("base.png", png(64))]);
    build(&mut h);
    let Some(Modal::NewProject(draft)) = &h.state().modal else {
        panic!("wizard");
    };
    let checked: Vec<&str> = draft.vehicles.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(checked.len(), 2);
    assert_eq!(checked[0], tp_app::vehicles::SAMPLE_ID);
    assert_eq!(
        draft.open.as_deref(),
        Some(checked[1]),
        "the trailer's card"
    );
    for name in [common::SAMPLE, "Box"] {
        assert_eq!(
            h.get_by_role_and_label(Role::CheckBox, name)
                .accesskit_node()
                .toggled(),
            Some(egui::accesskit::Toggled::True),
            "{name}"
        );
    }
    assert!(h.query_by_label("2 vehicles").is_some());
    // The truck's card is folded, the trailer's open.
    assert!(
        h.query_by_role_and_label(Role::Button, common::SAMPLE)
            .is_some()
    );
    assert!(h.query_by_role_and_label(Role::Button, "Box").is_none());
}

#[test]
fn custom_vehicle_from_an_ats_project_is_ats() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_custom_truck(&mut h, Game::Ats);
    let package = h
        .state()
        .vehicles
        .load("custom.scania.r_2024", &semver::Version::new(1, 0, 0))
        .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let project = tp_app::vehicle_project::fleet_project("ATS", &[(&package, &textures)]).unwrap();
    h.state_mut().open_project(project);
    settle(&mut h);
    h.state_mut()
        .queue
        .push(tp_app::commands::CommandId::AddVehicle);
    settle(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    let d = dialog(&h);
    assert_eq!(d.form.game, Game::Ats);
    assert!(d.game_locked);
}

#[test]
fn only_the_chosen_games_vehicles_are_listed() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles.install_samples().unwrap();
    install_custom_truck(&mut h, Game::Ats);
    open_wizard(&mut h);
    // Vehicles of both games: the game starts on ETS2.
    let chosen = |h: &H, label: &str| {
        h.get_by_role_and_label(Role::RadioButton, label)
            .accesskit_node()
            .toggled()
            .is_some_and(|t| t == egui::accesskit::Toggled::True)
    };
    assert!(chosen(&h, "Euro Truck Simulator 2"));
    let listed = |h: &H, name: &str| h.query_by_role_and_label(Role::CheckBox, name).is_some();
    assert!(listed(&h, "TruckPaint Sample Truck"));
    assert!(!listed(&h, "R 2024"));
    h.get_by_role_and_label(Role::RadioButton, "American Truck Simulator")
        .click();
    settle(&mut h);
    assert!(listed(&h, "R 2024"));
    assert!(!listed(&h, "TruckPaint Sample Truck"));
    assert!(!listed(&h, "TruckPaint Sample Trailer"));
}

#[test]
fn the_game_starts_on_the_installed_vehicles_game() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_custom_truck(&mut h, Game::Ats);
    open_wizard(&mut h);
    let Some(Modal::NewProject(draft)) = &h.state().modal else {
        panic!("New Project open");
    };
    assert_eq!(draft.game(), Game::Ats);
}

#[test]
fn created_vehicle_hidden_by_a_filter() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles.install_samples().unwrap();
    open_wizard(&mut h);
    if let Some(Modal::NewProject(draft)) = &mut h.state_mut().modal {
        draft.filter.game = Some(Game::Ats);
    }
    settle(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    // The dialog's game is proposed; the player picks ETS2.
    assert_eq!(dialog(&h).form.game, Game::Ats);
    form(&mut h).game = Game::Ets2;
    fill_truck(&mut h);
    build(&mut h);
    assert_eq!(wizard_choice(&h).as_deref(), Some("custom.scania.r_2024"));
    let Some(Modal::NewProject(draft)) = &h.state().modal else {
        panic!("wizard");
    };
    assert_eq!(draft.filter.game, Some(Game::Ets2), "the game is now ETS2");
    assert!(
        h.get_by_role_and_label(Role::CheckBox, "R 2024")
            .accesskit_node()
            .toggled()
            .is_some_and(|t| t == egui::accesskit::Toggled::True)
    );
}

#[test]
fn library_custom_vehicle_new_version_and_export() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    settle(&mut h);
    // From the empty library.
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    fill_truck(&mut h);
    build(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::VehicleLibrary(_))));
    assert!(
        h.query_by_role_and_label(Role::RadioButton, "R 2024")
            .is_some(),
        "listed"
    );
    assert!(
        h.query_by_label_contains("Any game version · Main texture: cabin · No accessories")
            .is_some()
    );
    h.state_mut().vehicles.install_samples().unwrap();
    settle(&mut h);
    assert!(
        h.query_by_label("New version of TruckPaint Sample Truck")
            .is_none(),
        "only custom vehicles offer New Version…"
    );

    // Export writes the installed bytes.
    let to = dir.path().join("out.tpv");
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        package_save: [to.clone()].into(),
        ..Default::default()
    });
    h.get_by_label("Export R 2024 1.0.0").click();
    settle(&mut h);
    assert_eq!(
        h.state().dialogs.suggested(),
        ["custom.scania.r_2024-1.0.0.tpv"]
    );
    let installed = &h
        .state()
        .vehicles
        .get("custom.scania.r_2024")
        .unwrap()
        .versions[0];
    assert_eq!(
        std::fs::read(&to).unwrap(),
        std::fs::read(&installed.path).unwrap()
    );

    // New Version…: the version must be higher.
    h.get_by_label("New version of R 2024").click();
    settle(&mut h);
    let d = dialog(&h);
    assert!(d.is_new_version() && d.game_locked);
    assert_eq!(d.form.version, "1.1.0");
    assert!(create_enabled(&h));
    type_into(&mut h, "Version", "1.0.0");
    assert!(!create_enabled(&h));
    assert!(
        h.query_by_label("The version must be higher than 1.0.0.")
            .is_some()
    );
    assert!(
        h.get_by_role_and_label(Role::ComboBox, "Kind")
            .accesskit_node()
            .is_disabled()
    );
    h.key_press(Key::Escape);
    settle(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::VehicleLibrary(_))));
}

#[test]
fn library_tabs_and_custom_status() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles.install_samples().unwrap();
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    settle(&mut h);
    // No custom vehicle: My vehicles says how to make one.
    h.get_by_label("My vehicles · 0").click();
    settle(&mut h);
    assert!(
        h.query_by_label_contains("Custom vehicles are made from the game's template files")
            .is_some()
    );
    install_custom_truck(&mut h, Game::Ets2);
    settle(&mut h);
    assert!(h.query_by_label("Installed · 3").is_some());
    h.get_by_label("My vehicles · 1").click();
    settle(&mut h);
    let listed = |h: &H, name: &str| h.query_by_role_and_label(Role::RadioButton, name).is_some();
    assert!(listed(&h, "R 2024"));
    assert!(!listed(&h, "TruckPaint Sample Truck"));
    assert!(h.query_by_label("Custom").is_some(), "its status");
    assert!(h.query_by_label("New version of R 2024").is_some());
}

#[test]
fn update_template_on_a_custom_vehicle() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_custom_truck(&mut h, Game::Ets2);
    let package = h
        .state()
        .vehicles
        .load("custom.scania.r_2024", &semver::Version::new(1, 0, 0))
        .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let project =
        tp_app::vehicle_project::fleet_project("Fleet", &[(&package, &textures)]).unwrap();
    h.state_mut().open_project(project);
    settle(&mut h);

    // New Version… from the library, replacing the cabin's template.
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    settle(&mut h);
    h.get_by_label("New version of R 2024").click();
    settle(&mut h);
    let v2 = dir.path().join("cabin_v2.png");
    std::fs::write(&v2, png(96)).unwrap();
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        templates: [vec![v2]].into(),
        ..Default::default()
    });
    h.get_by_label("Replace the template of texture 1").click();
    settle(&mut h);
    build(&mut h);
    let versions: Vec<String> = h
        .state()
        .vehicles
        .get("custom.scania.r_2024")
        .unwrap()
        .versions
        .iter()
        .map(|v| v.manifest.version.to_string())
        .collect();
    assert_eq!(versions, ["1.1.0", "1.0.0"]);
    h.key_press(Key::Escape);
    settle(&mut h);

    // Update Template flags the cabin only.
    let new = h
        .state()
        .vehicles
        .load("custom.scania.r_2024", &semver::Version::new(1, 1, 0))
        .unwrap();
    assert_eq!(new.manifest.paint_job.main[0].texture.layout_version, 2);
    let _: &Package = &new;
    h.state_mut()
        .queue
        .push(tp_app::commands::CommandId::UpdateTemplate);
    settle(&mut h);
    h.get_by_label("Update").click();
    settle(&mut h);
    let p = &ws(&h).project;
    assert_eq!(p.vehicles[0].version, "1.1.0");
    let status = |name: &str| {
        p.surfaces
            .iter()
            .find(|s| s.name == name)
            .and_then(|s| s.template.as_ref())
            .map(|t| t.status)
    };
    assert_eq!(status("cabin"), Some(TemplateStatus::LayoutChanged));
    assert_eq!(status("Mirrors"), Some(TemplateStatus::Current));
}

/// With a main and an accessory texture, the rows' fixed grid keeps the
/// whole dialog, its title and its Create button inside a 1440 × 1000
/// window; the rows show short labels above their combos.
#[test]
fn custom_vehicle_dialog_stays_in_the_window() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.set_size(Vec2::new(1440.0, 1000.0));
    open_wizard(&mut h);
    h.get_by_label("Custom Vehicle…").click();
    settle(&mut h);
    drop_files(
        &mut h,
        vec![
            ("cabin.png", png(64)),
            ("side_skirts_and_mirrors.png", png(32)),
        ],
    );
    assert_eq!(dialog(&h).form.rows[1].role, PartRole::Accessory);
    settle(&mut h);
    let window = egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(1440.0, 1000.0));
    let modal = h
        .ctx
        .memory(|m| m.area_rect(egui::Id::new("custom_vehicle_modal")))
        .expect("the dialog is shown");
    assert!(
        window.contains_rect(modal),
        "{modal:?} is outside the window"
    );
    for name in ["Custom Vehicle", "Create", "Role of texture 2"] {
        let rect = h.get_by_label(name).rect();
        assert!(window.contains_rect(rect), "{name} at {rect:?}");
    }
    assert!(h.query_all_by_label("Role").next().is_some());
    assert!(h.query_all_by_label("Height").next().is_some());
}

#[test]
fn new_version_records_what_changed() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_custom_truck(&mut h, Game::Ets2);
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    settle(&mut h);
    let changelog = |h: &H, version: (u64, u64, u64)| {
        h.state()
            .vehicles
            .load(
                "custom.scania.r_2024",
                &semver::Version::new(version.0, version.1, version.2),
            )
            .unwrap()
            .manifest
            .changelog
    };

    h.get_by_label("New version of R 2024").click();
    settle(&mut h);
    assert_eq!(form(&mut h).changelog, "", "starts empty");
    assert!(
        h.query_by_role_and_label(Role::TextInput, "What changed")
            .is_some()
    );
    form(&mut h).changelog = "x".repeat(tp_vehicles::MAX_CHANGELOG + 1);
    settle(&mut h);
    assert!(!create_enabled(&h));
    assert!(
        h.query_by_label("“What changed” is longer than 1000 characters.")
            .is_some()
    );
    assert!(h.query_by_label("1001 / 1000 characters").is_some());
    form(&mut h).changelog = "Cabin template redrawn after game update 1.57".into();
    settle(&mut h);
    build(&mut h);
    assert_eq!(
        changelog(&h, (1, 1, 0)).as_deref(),
        Some("Cabin template redrawn after game update 1.57")
    );

    // Left empty: the next version has no changelog.
    h.get_by_label("New version of R 2024").click();
    settle(&mut h);
    assert_eq!(form(&mut h).changelog, "", "not copied from 1.1.0");
    build(&mut h);
    assert_eq!(changelog(&h, (1, 2, 0)), None);
}
