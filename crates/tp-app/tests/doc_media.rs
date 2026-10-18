//! Renders the screenshots and animations of the player documentation
//! (`website/`), in every language of the app.
//!
//! Ignored by default (needs a GPU). Run with `mise run doc:media`, which
//! also encodes the results into `website/public/media/`; one section only
//! with `mise run doc:media <section>` (the test names are `section_*`).
//! Raw images are written to `target/doc-media/<language>/`: a screenshot
//! is `<name>.png`, an animation a folder of numbered frames `<name>/`.

mod common;

use std::path::PathBuf;

use common::scenes::*;
use egui::{Event, Modifiers, Pos2, Rect, Vec2};
use egui_kittest::kittest::Queryable;
use image::{Rgba, RgbaImage};
use tp_app::AppState;
use tp_app::prefs::Prefs;
use tp_i18n::Language;

type H = egui_kittest::Harness<'static, AppState>;

/// Window size in points: large enough for the Workshop's three columns,
/// small enough to read on a documentation page.
const WINDOW: Vec2 = Vec2::new(1280.0, 800.0);
/// Rendering density: 1920 × 1200 px images.
const PPP: f32 = 1.5;
/// Animation frame rate.
const FPS: f32 = 30.0;

fn out_dir(language: Language) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/doc-media")
        .join(language.code());
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The app in `language`, with the drawn title bar of Windows and Linux.
fn harness(language: Language) -> H {
    harness_with(language, Prefs::default())
}

/// Same, from `prefs`.
fn harness_with(language: Language, mut prefs: Prefs) -> H {
    prefs.set_language(Some(language));
    let mut h = common::wgpu_harness_hidpi(prefs, WINDOW, PPP, 1.0 / FPS);
    h.state_mut().macos = false;
    h.state_mut().native_menu_installed = false;
    h.run();
    h
}

/// Runs `scenario` once per language.
fn for_each_language(scenario: impl Fn(Language)) {
    for language in Language::ALL {
        scenario(language);
    }
}

/// Saves the settled window as the screenshot `name` (may contain `/`).
fn shot(h: &mut H, language: Language, name: &str) {
    // No pointer: no hover state and no stray cursor in a still.
    h.event(Event::PointerGone);
    h.run();
    wait_for_thumbnails(h);
    let image = h.render().expect("render");
    let path = out_dir(language).join(format!("{name}.png"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    image.save(path).unwrap();
}

/// The screen position of a document point of the open texture.
fn doc_to_screen(h: &H, x: f64, y: f64) -> Pos2 {
    h.state()
        .workspace()
        .unwrap()
        .screen_map(PPP)
        .unwrap()
        .to_screen(tp_core::kurbo::Point::new(x, y))
}

/// The center of the widget labeled with the text of `key`.
fn widget(h: &H, key: &str) -> Pos2 {
    h.get_by_label(&tp_i18n::tr(key)).rect().center()
}

/// An animation being recorded: every call advances the app by whole
/// frames, renders them, and draws the pointer on top.
struct Clip {
    dir: PathBuf,
    frame: u32,
    pointer: Pos2,
    pressed: bool,
    /// Frames since the last press, for the press ring.
    since_press: Option<u32>,
    /// Region kept from each frame, in points.
    crop: Option<Rect>,
}

impl Clip {
    /// Starts recording the animation `name`, cropped to `crop` (points)
    /// when given, with the pointer at `pointer`.
    fn start(h: &mut H, language: Language, name: &str, crop: Option<Rect>, pointer: Pos2) -> Self {
        let dir = out_dir(language).join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        settle(h);
        h.event(Event::PointerMoved(pointer));
        h.run();
        Self {
            dir,
            frame: 0,
            pointer,
            pressed: false,
            since_press: None,
            crop,
        }
    }

    fn record(&mut self, h: &mut H) {
        h.step();
        let mut image = h.render().expect("render");
        draw_pointer(&mut image, self.pointer * PPP, self.since_press);
        if let Some(crop) = self.crop {
            let x = (crop.min.x * PPP) as u32;
            let y = (crop.min.y * PPP) as u32;
            // Even sizes: H.264 with 4:2:0 chroma needs them.
            let w = ((crop.width() * PPP) as u32) & !1;
            let hgt = ((crop.height() * PPP) as u32) & !1;
            image = image::imageops::crop_imm(&image, x, y, w, hgt).to_image();
        }
        self.frame += 1;
        image
            .save(self.dir.join(format!("frame_{:05}.png", self.frame)))
            .unwrap();
        if let Some(n) = self.since_press.as_mut() {
            *n += 1;
        }
    }

    /// Holds still for `ms` milliseconds (the app keeps running).
    fn wait(&mut self, h: &mut H, ms: u32) -> &mut Self {
        for _ in 0..frames(ms) {
            self.record(h);
        }
        self
    }

    /// Moves the pointer to `to` in `ms` milliseconds, eased, dragging if
    /// the button is down.
    fn move_to(&mut self, h: &mut H, to: Pos2, ms: u32) -> &mut Self {
        let from = self.pointer;
        let n = frames(ms).max(1);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let eased = t * t * (3.0 - 2.0 * t);
            self.pointer = from + (to - from) * eased;
            h.event(Event::PointerMoved(self.pointer));
            self.record(h);
        }
        self
    }

    fn button(&mut self, h: &mut H, pressed: bool) {
        h.event(Event::PointerButton {
            pos: self.pointer,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        self.pressed = pressed;
        if pressed {
            self.since_press = Some(0);
        }
    }

    fn press(&mut self, h: &mut H) -> &mut Self {
        self.button(h, true);
        self.record(h);
        self
    }

    fn release(&mut self, h: &mut H) -> &mut Self {
        self.button(h, false);
        self.record(h);
        self
    }

    /// Moves to `to`, clicks, and lets the click show for a moment.
    fn click_at(&mut self, h: &mut H, to: Pos2) -> &mut Self {
        self.move_to(h, to, 450);
        self.wait(h, 120);
        self.press(h);
        self.wait(h, 60);
        self.release(h);
        self.wait(h, 250)
    }

    /// Clicks the first widget labeled `label` (a literal text).
    fn click_label(&mut self, h: &mut H, label: &str) -> &mut Self {
        let at = h
            .get_all_by_label(label)
            .next()
            .unwrap_or_else(|| panic!("no widget labeled {label:?}"))
            .rect()
            .center();
        self.click_at(h, at)
    }

    /// Clicks the widget labeled with the text of `key`.
    fn click(&mut self, h: &mut H, key: &str) -> &mut Self {
        let at = widget(h, key);
        self.click_at(h, at)
    }

    /// Drags from the pointer's position to `to` in `ms` milliseconds.
    fn drag_to(&mut self, h: &mut H, to: Pos2, ms: u32) -> &mut Self {
        self.press(h);
        self.wait(h, 80);
        self.move_to(h, to, ms);
        self.wait(h, 80);
        self.release(h)
    }

    /// Types `text` like a keyboard, one character every 70 ms.
    fn type_text(&mut self, h: &mut H, text: &str) -> &mut Self {
        for c in text.chars() {
            let key = egui::Key::from_name(&c.to_ascii_uppercase().to_string());
            let send_key = |h: &mut H, pressed| {
                if let Some(key) = key {
                    h.event(Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Modifiers::NONE,
                    });
                }
            };
            send_key(h, true);
            h.event(Event::Text(c.to_string()));
            send_key(h, false);
            self.wait(h, 70);
        }
        self
    }

    /// Presses `key` (with `modifiers`).
    fn key(&mut self, h: &mut H, key: egui::Key, modifiers: Modifiers) -> &mut Self {
        for pressed in [true, false] {
            h.event(Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            });
        }
        self.wait(h, 200)
    }

    /// Ends with a one-second hold on the result.
    fn finish(&mut self, h: &mut H) {
        self.wait(h, 1000);
    }
}

fn frames(ms: u32) -> u32 {
    (ms as f32 * FPS / 1000.0).round() as u32
}

/// Waits for background renders, so an animation starts on a still screen.
fn settle(h: &mut H) {
    h.run();
    wait_for_thumbnails(h);
    if h.state().workspace().is_some() {
        common::settle_renders(h);
    }
}

/// The arrow pointer, tip at the origin, in points.
const ARROW: [(f32, f32); 7] = [
    (0.0, 0.0),
    (0.0, 17.0),
    (4.2, 13.2),
    (7.2, 19.8),
    (9.8, 18.6),
    (6.9, 12.2),
    (12.2, 12.2),
];

/// Draws the pointer with its tip at `tip` (pixels), and the press ring
/// for a few frames after a press.
fn draw_pointer(image: &mut RgbaImage, tip: Pos2, since_press: Option<u32>) {
    if let Some(n) = since_press.filter(|n| *n < 10) {
        let t = n as f32 / 10.0;
        let radius = (8.0 + 10.0 * t) * PPP;
        let alpha = 1.0 - t;
        ring(image, tip, radius, 2.5 * PPP, [20, 20, 22], alpha * 0.6);
        ring(image, tip, radius, 1.5 * PPP, [255, 255, 255], alpha);
    }
    let poly: Vec<Pos2> = ARROW
        .iter()
        .map(|&(x, y)| tip + Vec2::new(x, y) * PPP)
        .collect();
    let outline = 1.1 * PPP;
    let min = poly
        .iter()
        .fold(Pos2::new(f32::MAX, f32::MAX), |a, p| a.min(*p))
        - Vec2::splat(outline);
    let max = poly
        .iter()
        .fold(Pos2::new(f32::MIN, f32::MIN), |a, p| a.max(*p))
        + Vec2::splat(outline);
    const SS: u32 = 4;
    for py in min.y.floor() as i32..=max.y.ceil() as i32 {
        for px in min.x.floor() as i32..=max.x.ceil() as i32 {
            let (mut white, mut black) = (0u32, 0u32);
            for sy in 0..SS {
                for sx in 0..SS {
                    let p = Pos2::new(
                        px as f32 + (sx as f32 + 0.5) / SS as f32,
                        py as f32 + (sy as f32 + 0.5) / SS as f32,
                    );
                    let inside = contains(&poly, p);
                    let d = edge_distance(&poly, p);
                    if inside && d > outline {
                        white += 1;
                    } else if inside || d <= outline * 0.5 {
                        black += 1;
                    }
                }
            }
            let total = (SS * SS) as f32;
            blend(image, px, py, [255, 255, 255], white as f32 / total);
            blend(image, px, py, [16, 16, 18], black as f32 / total);
        }
    }
}

fn ring(image: &mut RgbaImage, center: Pos2, radius: f32, width: f32, rgb: [u8; 3], alpha: f32) {
    let r = radius + width;
    for py in (center.y - r).floor() as i32..=(center.y + r).ceil() as i32 {
        for px in (center.x - r).floor() as i32..=(center.x + r).ceil() as i32 {
            let d = (Pos2::new(px as f32 + 0.5, py as f32 + 0.5) - center).length();
            let coverage = (width / 2.0 + 0.5 - (d - radius).abs()).clamp(0.0, 1.0);
            blend(image, px, py, rgb, coverage * alpha);
        }
    }
}

fn blend(image: &mut RgbaImage, x: i32, y: i32, rgb: [u8; 3], alpha: f32) {
    if alpha <= 0.0 || x < 0 || y < 0 || x as u32 >= image.width() || y as u32 >= image.height() {
        return;
    }
    let Rgba([r, g, b, a]) = *image.get_pixel(x as u32, y as u32);
    let mix = |c: u8, n: u8| (f32::from(c) * (1.0 - alpha) + f32::from(n) * alpha).round() as u8;
    image.put_pixel(
        x as u32,
        y as u32,
        Rgba([mix(r, rgb[0]), mix(g, rgb[1]), mix(b, rgb[2]), a]),
    );
}

fn contains(poly: &[Pos2], p: Pos2) -> bool {
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn edge_distance(poly: &[Pos2], p: Pos2) -> f32 {
    let mut best = f32::MAX;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let ab = b - a;
        let t = ((p - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
        best = best.min((a + ab * t - p).length());
    }
    best
}

/// Shows `space` and lets it settle.
fn space(h: &mut H, space: tp_app::layout::Space) {
    common::show_space(h, space);
    h.run();
}

fn left_tab(h: &mut H, tab: tp_app::layout::LeftTab) {
    h.state_mut().prefs.layout.left_tab = tab;
    h.run();
}

fn modal(h: &mut H, modal: tp_app::state::Modal) {
    h.state_mut().modal = Some(modal);
    settle_dialog(h);
}

/// ETS2 played, and a chosen mod folder per game so the detected ones of the
/// machine rendering the media never show.
fn played_games(prefs: &mut Prefs) {
    prefs.games = vec!["ets2".into()];
    for (game, name) in [
        ("ets2", "Euro Truck Simulator 2"),
        ("ats", "American Truck Simulator"),
    ] {
        prefs
            .mod_folders
            .insert(game.into(), format!("~/Documents/{name}/mod").into());
    }
}

/// The sample truck with both main textures and the sample trailer.
fn load_sample(h: &H, id: &str, version: &str) -> tp_vehicles::Package {
    h.state()
        .vehicles
        .load(id, &version.parse().unwrap())
        .unwrap()
}

/// The sample truck alone, with both main textures.
fn sample_truck_project(h: &mut H) {
    let truck = load_sample(h, tp_app::vehicles::SAMPLE_ID, "1.1.0");
    let mut textures = tp_app::vehicle_project::default_textures(&truck.manifest);
    textures.insert(1, "high_roof".into());
    let project =
        tp_app::vehicle_project::fleet_project("ACE Logistics", &[(&truck, &textures)]).unwrap();
    common::open_project(h, project);
}

/// The sample truck 1.0.0 with the demo livery on its Standard cab, the
/// sample truck 1.1.0 (which redraws that cab's layout) installed too.
fn outdated_truck_project(h: &mut H) {
    let older = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/vehicles/community.truckpaint.sample_truck-1.0.0.tpv"
    );
    h.state_mut()
        .vehicles
        .install_file(std::path::Path::new(older))
        .unwrap();
    let truck = load_sample(h, tp_app::vehicles::SAMPLE_ID, "1.0.0");
    let textures = tp_app::vehicle_project::default_textures(&truck.manifest);
    let project =
        tp_app::vehicle_project::fleet_project("ACE Logistics", &[(&truck, &textures)]).unwrap();
    common::open_project(h, project);
    demo_scene(h);
}

/// Update Template for the sample truck of [`outdated_truck_project`].
fn update_template_dialog(h: &H) -> tp_app::ui::vehicle_dialogs::UpdateDialog {
    let newer = load_sample(h, tp_app::vehicles::SAMPLE_ID, "1.1.0");
    let plan =
        tp_app::vehicle_project::plan(&h.state().workspace().unwrap().project, &newer).unwrap();
    tp_app::ui::vehicle_dialogs::UpdateDialog::new(newer, plan)
}

/// The sample truck and the sample trailer.
fn sample_fleet(h: &mut H) {
    sample_truck_project(h);
    let trailer = load_sample(h, tp_app::vehicles::SAMPLE_TRAILER_ID, "1.0.0");
    let all = tp_app::vehicle_project::default_textures(&trailer.manifest);
    h.state_mut()
        .workspace_mut()
        .unwrap()
        .add_vehicle(&trailer, &all, 1.0)
        .unwrap();
    common::settle_renders(h);
}

/// Mod export writes into a temporary Documents folder.
fn temporary_mod_folders(h: &mut H) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let documents = dir.path().join("Documents");
    std::fs::create_dir_all(&documents).unwrap();
    h.state_mut().mod_folders = tp_app::mod_export::Folders {
        documents: Some(documents),
        data: Some(dir.path().join("data")),
        home: Some(dir.path().to_path_buf()),
    };
    dir
}

// --- Getting started ----------------------------------------------------------

#[test]
#[ignore = "needs a GPU; run with `mise run doc:media`"]
fn section_getting_started() {
    use egui::accesskit::Role;
    // A recent project with artwork, for the home screen's card.
    let dir = tempfile::tempdir().unwrap();
    let ace = dir.path().join("ACE Logistics.truckpaint");
    {
        let mut h = common::harness();
        common::create_project(&mut h);
        demo_scene(&mut h);
        tp_file::write(&h.state().workspace().unwrap().project, &ace).unwrap();
    }
    for_each_language(|language| {
        let prefs = Prefs {
            recent: vec![tp_app::prefs::RecentProject {
                name: "ACE Logistics".into(),
                path: ace.clone(),
                last_opened: tp_app::prefs::now_unix() - 7_200,
            }],
            ..Prefs::default()
        };
        let mut h = harness_with(language, prefs);
        shot(&mut h, language, "getting-started/home");

        // Creating a project: New Project, the sample truck, Create.
        let start = widget(&h, "home-new-project") + Vec2::new(160.0, 140.0);
        let mut clip = Clip::start(
            &mut h,
            language,
            "getting-started/create-project",
            None,
            start,
        );
        clip.wait(&mut h, 300).click(&mut h, "home-new-project");
        clip.wait(&mut h, 300);
        let sample = h
            .get_by_role_and_label(Role::CheckBox, common::SAMPLE)
            .rect()
            .center();
        clip.click_at(&mut h, sample).wait(&mut h, 500);
        shot(&mut h, language, "getting-started/new-project");
        clip.click(&mut h, "new-project-create");
        clip.wait(&mut h, 600);
        clip.finish(&mut h);
        settle(&mut h);
        shot(&mut h, language, "getting-started/first-project");
    });

    // The first launch, on a made-up game (nothing else is installed).
    let steam = std::env::temp_dir().join("truckpaint-doc-steam");
    let game = steam.join("steamapps/common/Euro Truck Simulator 2");
    let _ = std::fs::remove_dir_all(&steam);
    std::fs::create_dir_all(&game).unwrap();
    {
        use tp_pack::synthetic::{Vehicle, write_game};
        let vehicles = [
            Vehicle::truck("example.alpha", "Alpha"),
            Vehicle::truck("example.beta", "Beta"),
            Vehicle::truck("example.gamma", "Gamma"),
            Vehicle::trailer("example.box", "Box"),
            Vehicle::trailer("example.flat", "Flat"),
        ];
        write_game(&game, "eut2", "1.61.1.9", &vehicles);
    }
    let library = tempfile::tempdir().unwrap();
    for_each_language(|language| {
        let mut prefs = Prefs::default();
        prefs.game_folders.insert("ets2".into(), game.clone());
        prefs.mod_folders.insert(
            "ets2".into(),
            "~/Documents/Euro Truck Simulator 2/mod".into(),
        );
        let mut h = harness_with(language, prefs);
        let _ = std::fs::remove_dir_all(library.path().join("vehicles"));
        h.state_mut().vehicles =
            tp_app::vehicles::VehicleLibrary::open(&library.path().join("vehicles"));
        h.state_mut()
            .game
            .detected
            .insert(tp_vehicles::Game::Ets2, None);
        h.state_mut()
            .game
            .detected
            .insert(tp_vehicles::Game::Ats, None);
        h.state_mut().begin_first_run_if_needed();
        settle(&mut h);
        shot(&mut h, language, "getting-started/first-launch");
        click_key(&mut h, "first-run-next");
        for _ in 0..400 {
            h.run_steps(1);
            if h.state().game.scanned().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        settle(&mut h);
        for name in ["Example Alpha", "Example Box"] {
            h.get_by_role_and_label(Role::CheckBox, name).click();
        }
        settle(&mut h);
        shot(&mut h, language, "getting-started/first-launch-vehicles");
    });
}

/// Clicks the widget labeled with the text of `key`.
fn click_key(h: &mut H, key: &str) {
    h.get_by_label(&tp_i18n::tr(key)).click();
    h.run();
}

// --- Vehicles -----------------------------------------------------------------

#[test]
#[ignore = "needs a GPU; run with `mise run doc:media`"]
fn section_vehicles() {
    use tp_app::state::Modal;
    use tp_app::ui::custom_vehicle::{CustomVehicleDialog, Origin};
    for_each_language(|language| {
        let mut h = harness(language);
        modal(&mut h, Modal::VehicleLibrary(Default::default()));
        shot(&mut h, language, "vehicles/library");
        h.state_mut().modal = None;
        h.run();

        // Custom Vehicle, filled in.
        let mut dialog = CustomVehicleDialog::new(Origin::NewProject(Default::default()));
        dialog.form.name = "R 2024".into();
        dialog.form.brand = "Scania".into();
        dialog.form.path = "scania.r_2024".into();
        dialog.add_files(
            vec![
                ("cabin.png".into(), Ok(tp_vehicles::sample::png(64))),
                ("side_skirts.png".into(), Ok(tp_vehicles::sample::png(32))),
            ],
            None,
        );
        dialog.form.rows[1].game_ids = "sideskirt.a".into();
        modal(&mut h, Modal::CustomVehicle(Box::new(dialog)));
        shot(&mut h, language, "vehicles/custom-vehicle");
        h.state_mut().modal = None;
        h.run();

        // The update summary of the sample truck, with the open project.
        let mut h = harness(language);
        outdated_truck_project(&mut h);
        let library = tp_app::ui::vehicle_dialogs::LibraryDialog {
            selected: Some(tp_app::vehicles::SAMPLE_ID.into()),
            ..Default::default()
        };
        modal(&mut h, Modal::VehicleLibrary(library));
        shot(&mut h, language, "vehicles/update-summary");

        my_game_shots(language);
    });
}

/// A made-up game folder: three trucks and two trailers, one of them from a DLC.
fn game_folder(version: &str) -> PathBuf {
    use tp_pack::synthetic::{Vehicle, write_game};
    // A path that looks like a Steam library, since the window shows it.
    let dir = std::env::temp_dir().join("steamapps/common/Euro Truck Simulator 2");
    std::fs::create_dir_all(&dir).unwrap();
    write_game(
        &dir,
        "eut2",
        version,
        &[
            Vehicle::truck("example.arctic", "Arctic"),
            Vehicle::truck("example.meridian", "Meridian"),
            Vehicle::truck("example.voyager", "Voyager"),
            Vehicle::trailer("example.curtain", "Curtain"),
            Vehicle::trailer("example.tanker", "Tanker").in_archive("dlc_tanker"),
        ],
    );
    dir
}

/// Reads the game folder and makes `ids` now, one after another.
fn generate_now(h: &mut H, folder: &std::path::Path, ids: &[&str]) {
    let s = h.state_mut();
    s.game.refresh_scan();
    s.game.ensure_scan(folder, || {});
    while s.game.scanned().is_none() {
        s.game.poll(&mut s.vehicles);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    s.game.checked = ids.iter().map(|id| (*id).to_owned()).collect();
    s.game.generate(&s.vehicles, || {});
    while s.game.run.is_some() {
        s.game.poll(&mut s.vehicles);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

/// The My Game tab (before and in the middle of a run) and the Games section of Preferences.
fn my_game_shots(language: Language) {
    use tp_app::state::Modal;
    use tp_app::ui::vehicle_dialogs::{LibraryDialog, LibraryTab};
    {
        let mut h = harness(language);
        let folder = game_folder("1.61.1.9");
        h.state_mut()
            .prefs
            .game_folders
            .insert("ets2".into(), folder.clone());
        // Two vehicles made on the first version of the game, one after its update.
        generate_now(&mut h, &folder, &["example.arctic", "example.meridian"]);
        game_folder("1.62.0.3");
        generate_now(&mut h, &folder, &["example.voyager"]);
        let tab = LibraryDialog {
            tab: LibraryTab::Game,
            ..Default::default()
        };
        h.state_mut().game.last = None;
        modal(&mut h, Modal::VehicleLibrary(tab.clone()));
        h.get_by_label(&tp_i18n::tr("game-select-updates")).click();
        settle(&mut h);
        shot(&mut h, language, "vehicles/my-game");

        // Mid-run: the worker waits after each event until it is let go.
        let mut h = harness(language);
        let folder = game_folder("1.61.1.9");
        h.state_mut()
            .prefs
            .game_folders
            .insert("ets2".into(), folder.clone());
        modal(&mut h, Modal::VehicleLibrary(tab));
        for _ in 0..500 {
            h.run_steps(1);
            if h.state().game.scanned().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        h.get_by_label(&tp_i18n::tr("game-select-all")).click();
        h.run();
        let (release, gate) = std::sync::mpsc::channel::<()>();
        {
            let s = h.state_mut();
            s.game.generate(&s.vehicles, move || {
                let _ = gate.recv();
            });
        }
        // Started(0), Made(0), Started(1): the second vehicle is in progress.
        release.send(()).unwrap();
        release.send(()).unwrap();
        for _ in 0..500 {
            h.run_steps(1);
            if h.state().game.run.as_ref().is_some_and(|r| r.done() == 1) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        settle(&mut h);
        shot(&mut h, language, "vehicles/my-game-progress");
        drop(release);

        // The Games section of Preferences, in a window tall enough to show it whole.
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let mut h = common::wgpu_harness_hidpi(prefs, Vec2::new(1280.0, 1300.0), PPP, 1.0 / FPS);
        h.state_mut().macos = false;
        h.state_mut().native_menu_installed = false;
        h.run();
        let folder = game_folder("1.61.1.9");
        h.state_mut()
            .prefs
            .game_folders
            .insert("ets2".into(), folder);
        played_games(&mut h.state_mut().prefs);
        h.state_mut()
            .open_preferences(tp_app::state::PrefsTab::Games);
        settle_dialog(&mut h);
        shot(&mut h, language, "vehicles/game-folders");
    }
}

// --- Project and fleet --------------------------------------------------------

#[test]
#[ignore = "needs a GPU; run with `mise run doc:media`"]
fn section_fleet() {
    use tp_app::layout::{LeftTab, Space};
    use tp_app::state::Modal;
    use tp_app::ui::vehicle_dialogs::{AddVehicleDialog, TexturesDialog, VehicleChoice};
    for_each_language(|language| {
        let mut h = harness(language);
        sample_fleet(&mut h);
        demo_scene(&mut h);
        space(&mut h, Space::Project);
        common::settle_renders(&mut h);
        shot(&mut h, language, "fleet/project");
        space(&mut h, Space::Workshop);
        left_tab(&mut h, LeftTab::Textures);
        shot(&mut h, language, "fleet/textures-tab");
        // Add Vehicle lists installed vehicles only: shown with the truck
        // alone, so the installed sample trailer is offered, checked.
        let mut add = harness(language);
        sample_truck_project(&mut add);
        demo_scene(&mut add);
        let mut dialog = AddVehicleDialog::for_game(Some("ets2"));
        let trailer = add
            .state()
            .vehicles
            .get(tp_app::vehicles::SAMPLE_TRAILER_ID);
        dialog.choices.push(VehicleChoice::of(trailer.unwrap()));
        modal(&mut add, Modal::AddVehicle(dialog));
        shot(&mut add, language, "fleet/add-vehicle");
        let dialog = {
            let p = &h.state().workspace().unwrap().project;
            TexturesDialog::new(p, &p.vehicles[0])
        };
        modal(&mut h, Modal::Textures(dialog));
        shot(&mut h, language, "fleet/textures-dialog");

        // Update Template, with the objects to reposition.
        let mut h = harness(language);
        outdated_truck_project(&mut h);
        let dialog = update_template_dialog(&h);
        modal(&mut h, Modal::UpdateTemplate(Box::new(dialog)));
        shot(&mut h, language, "fleet/update-template");
    });
}

// --- Workshop -----------------------------------------------------------------

#[test]
#[ignore = "needs a GPU; run with `mise run doc:media`"]
fn section_workshop() {
    use tp_app::layout::LeftTab;
    use tp_core::document::{CharStyle, Rgba};
    for_each_language(|language| {
        // The Workshop with a livery, the lettering selected.
        let mut h = harness(language);
        common::create_project(&mut h);
        demo_scene(&mut h);
        let text = {
            let ws = h.state_mut().workspace_mut().unwrap();
            let style = CharStyle {
                size: 260.0,
                ..CharStyle::default()
            };
            add_text(
                ws,
                "ACE Logistics",
                style,
                (300.0, 3950.0),
                Rgba::rgb(0x1B, 0x23, 0x40),
                None,
            )
        };
        shot(&mut h, language, "workshop/workshop");
        h.state_mut().workspace_mut().unwrap().selection = vec![text];
        shot(&mut h, language, "workshop/text-selected");
        h.state_mut().workspace_mut().unwrap().selection.clear();
        left_tab(&mut h, LeftTab::Layers);
        let ids = livery_tree(&mut h);
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[3]];
        shot(&mut h, language, "workshop/layers");

        // After Update Template: the objects to reposition are marked.
        let mut h = harness(language);
        outdated_truck_project(&mut h);
        let dialog = update_template_dialog(&h);
        h.state_mut()
            .workspace_mut()
            .unwrap()
            .apply_update(&dialog.package, &dialog.added, 1.0);
        left_tab(&mut h, LeftTab::Layers);
        shot(&mut h, language, "workshop/layers-to-reposition");

        // Drawing a rectangle: pick the tool, drag on the texture.
        let mut h = harness(language);
        common::create_project(&mut h);
        let start = doc_to_screen(&h, 900.0, 1500.0);
        let mut clip = Clip::start(
            &mut h,
            language,
            "workshop/draw-rectangle",
            None,
            start + Vec2::new(-60.0, 120.0),
        );
        clip.wait(&mut h, 300).click(&mut h, "tool-rectangle");
        clip.move_to(&mut h, start, 600);
        let end = doc_to_screen(&h, 3100.0, 2600.0);
        clip.drag_to(&mut h, end, 900);
        clip.finish(&mut h);

        // Moving and resizing: drag the ellipse, then its corner handle.
        let mut h = harness(language);
        common::create_project(&mut h);
        let ellipse = demo_scene(&mut h)[2];
        let center = doc_to_screen(&h, 2900.0, 2600.0);
        let mut clip = Clip::start(
            &mut h,
            language,
            "workshop/move-resize",
            None,
            center + Vec2::new(80.0, 90.0),
        );
        clip.wait(&mut h, 300).click_at(&mut h, center);
        let moved = doc_to_screen(&h, 2500.0, 2000.0);
        clip.drag_to(&mut h, moved, 800).wait(&mut h, 300);
        // The bottom right handle, where the move (and snapping) left it.
        let frame = h
            .state()
            .workspace()
            .unwrap()
            .project
            .surface()
            .get(ellipse)
            .unwrap()
            .frame;
        let (x, y) = (
            frame.center.x + frame.size.width / 2.0,
            frame.center.y + frame.size.height / 2.0,
        );
        let corner = doc_to_screen(&h, x, y);
        clip.move_to(&mut h, corner, 500).wait(&mut h, 150);
        let bigger = doc_to_screen(&h, x + 450.0, y + 450.0);
        clip.drag_to(&mut h, bigger, 700);
        clip.finish(&mut h);

        // Writing a text: the Text tool, a click, typing.
        let mut h = harness(language);
        common::create_project(&mut h);
        demo_scene(&mut h);
        let at = doc_to_screen(&h, 1300.0, 3350.0);
        let mut clip = Clip::start(
            &mut h,
            language,
            "workshop/write-text",
            None,
            at + Vec2::new(-80.0, -100.0),
        );
        clip.wait(&mut h, 300).click(&mut h, "tool-text");
        clip.click_at(&mut h, at);
        clip.type_text(&mut h, "ACE Logistics");
        clip.wait(&mut h, 400)
            .key(&mut h, egui::Key::Escape, Modifiers::NONE);
        clip.finish(&mut h);

        // Drawing a path with the Pen: four clicks, back to the first point.
        let mut h = harness(language);
        common::create_project(&mut h);
        let points = [
            (800.0, 3400.0),
            (2000.0, 1800.0),
            (3300.0, 3400.0),
            (2050.0, 2900.0),
        ]
        .map(|(x, y)| doc_to_screen(&h, x, y));
        let mut clip = Clip::start(
            &mut h,
            language,
            "workshop/draw-path",
            None,
            points[0] + Vec2::new(-60.0, -120.0),
        );
        clip.wait(&mut h, 300).click(&mut h, "tool-pen");
        for p in points {
            clip.click_at(&mut h, p);
        }
        clip.click_at(&mut h, points[0]);
        clip.finish(&mut h);

        // Texts and images; shapes and paths; gradients.
        let mut h = harness(language);
        common::create_project(&mut h);
        lettering_scene(&mut h);
        wait_for_images(&mut h);
        shot(&mut h, language, "workshop/text-images");
        let mut h = harness(language);
        common::create_project(&mut h);
        vector_scene(&mut h);
        shot(&mut h, language, "workshop/shapes-paths");
        let mut h = harness(language);
        common::create_project(&mut h);
        let (text, _) = gradient_scene(&mut h);
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.selection = vec![text];
            ws.panels.color_target = tp_app::workspace::ColorTarget::Fill;
        }
        shot(&mut h, language, "workshop/gradients");
    });
}

// --- Brand --------------------------------------------------------------------

#[test]
#[ignore = "needs a GPU; run with `mise run doc:media`"]
fn section_brand() {
    use tp_app::layout::{LeftTab, Space};
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    for_each_language(|language| {
        // The Resources tab with a palette, styles, a symbol and an image.
        let mut h = harness(language);
        common::create_project(&mut h);
        let ids = demo_scene(&mut h);
        demo_brand(&mut h, &ids);
        left_tab(&mut h, LeftTab::Resources);
        shot(&mut h, language, "brand/resources");

        // The Brand space, editing a swatch with its impact on the fleet.
        let mut h = harness(language);
        editor_fleet(&mut h);
        space(&mut h, Space::Brand);
        open_editor(&mut h, EditorTarget::Swatch);
        h.run();
        common::settle_renders(&mut h);
        shot(&mut h, language, "brand/brand-space");

        // Applying a swatch, then a style, to a new panel.
        let mut h = harness(language);
        brand_fleet(&mut h);
        left_tab(&mut h, LeftTab::Resources);
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            let mut o = Object::new(
                ObjectId(0),
                ShapeKind::Rectangle {
                    corner_radius: 80.0,
                },
                Frame::new(Point::new(2048.0, 900.0), Size::new(2600.0, 900.0), 0.0),
            );
            o.fill = Rgba::rgb(0xD8, 0xDA, 0xDF).into();
            let id = ws.project.add(o);
            ws.selection = vec![id];
            ws.panels.color_target = tp_app::workspace::ColorTarget::Fill;
        }
        let start = doc_to_screen(&h, 2048.0, 900.0);
        let mut clip = Clip::start(&mut h, language, "brand/apply-swatch", None, start);
        clip.wait(&mut h, 400).click_label(&mut h, "Company red");
        clip.wait(&mut h, 500).click_label(&mut h, "Company grey");
        clip.wait(&mut h, 500).click_label(&mut h, "Company red");
        clip.finish(&mut h);

        let style = tp_i18n::tr_args(
            "styles-graphic-item",
            &[("name", tp_i18n::value("Band"))].into_iter().collect(),
        );
        let start = clip.pointer;
        let mut clip = Clip::start(&mut h, language, "brand/apply-style", None, start);
        clip.wait(&mut h, 400).click_label(&mut h, &style);
        clip.finish(&mut h);
    });
}

// --- Export -------------------------------------------------------------------

#[test]
#[ignore = "needs a GPU; run with `mise run doc:media`"]
fn section_export() {
    use tp_app::layout::Space;
    use tp_app::state::Modal;
    for_each_language(|language| {
        // Export Texture, with its preview.
        let mut h = harness(language);
        common::create_project(&mut h);
        lettering_scene(&mut h);
        wait_for_images(&mut h);
        h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, egui::Key::E);
        for _ in 0..400 {
            h.step();
            if matches!(&h.state().modal, Some(Modal::Export(d)) if d.preview_ready()) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        shot(&mut h, language, "export/export-texture");
        h.state_mut().modal = None;
        h.run();

        // The Project space's mod information.
        let _folders = temporary_mod_folders(&mut h);
        {
            let s = &mut h.state_mut().workspace_mut().unwrap().project.mod_settings;
            s.author = "Jane Doe".into();
            s.description = "ACE Logistics fleet colors".into();
        }
        space(&mut h, Space::Project);
        common::settle_renders(&mut h);
        shot(&mut h, language, "export/mod-information");

        // Exporting the mod from the top bar.
        space(&mut h, Space::Workshop);
        let start = widget(&h, "top-bar-export") + Vec2::new(-200.0, 260.0);
        let mut clip = Clip::start(&mut h, language, "export/export-mod", None, start);
        clip.wait(&mut h, 300).click(&mut h, "top-bar-export");
        clip.wait(&mut h, 500);
        shot(&mut h, language, "export/export-mod-dialog");
        let export = common::last(&h, &tp_i18n::tr("mod-export-start"))
            .rect()
            .center();
        clip.click_at(&mut h, export);
        clip.wait(&mut h, 1200);
        clip.finish(&mut h);
    });
}

// --- AI assistant -------------------------------------------------------------

#[test]
#[ignore = "needs a GPU; run with `mise run doc:media`"]
fn section_ai_assistant() {
    use tp_app::state::Modal;
    for_each_language(|language| {
        let mut h = harness(language);
        common::create_project(&mut h);
        demo_scene(&mut h);
        h.state_mut().prefs.mcp_port = std::net::TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        h.state_mut().prefs.mcp_enabled = true;
        played_games(&mut h.state_mut().prefs);
        h.state_mut().preferences_tab = tp_app::state::PrefsTab::Ai;
        modal(&mut h, Modal::Preferences);
        shot(&mut h, language, "ai-assistant/preferences");
        h.state_mut().prefs.mcp_enabled = false;
        h.state_mut().modal = None;
        h.run();
    });
}
