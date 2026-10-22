//! Headless tests for the Side View panel of the Workshop.

mod common;

use egui::accesskit::{Role, Toggled};
use egui::{Event, Key, Modifiers, PointerButton, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_i18n::tr;
use tp_ui::tokens::size;
use tp_vehicles::sample::{self, SampleTexture};
use tp_vehicles::{Package, TextureSize};

type H = Harness<'static, AppState>;

const STANDARD: &str = "standard";

/// The sample truck with both cabins, the chassis and the side skirts, in the Workshop on the Standard cab.
fn open() -> H {
    let mut h = common::harness();
    let package = Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
    let textures = ["standard", "high_roof", "chassis", "side_skirts"].map(String::from);
    let mut project =
        tp_app::vehicle_project::fleet_project(common::SAMPLE, &[(&package, &textures)]).unwrap();
    for surface in &mut project.surfaces {
        if let Some(t) = surface.template.as_mut() {
            t.visible = false;
        }
    }
    common::open_project(&mut h, project);
    h.run();
    common::settle_renders(&mut h);
    h
}

fn ws(h: &H) -> &tp_app::workspace::Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut tp_app::workspace::Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn surface(h: &H, texture: &str) -> usize {
    ws(h)
        .project
        .surfaces
        .iter()
        .position(|s| s.template.as_ref().is_some_and(|t| t.texture_id == texture))
        .unwrap()
}

fn show_side_view(h: &mut H) {
    h.state_mut().prefs.layout.side_view = true;
    common::settle_renders(h);
}

fn activate(h: &mut H, texture: &str) {
    let i = surface(h, texture);
    h.state_mut().workspace_mut().unwrap().set_active_surface(i);
    common::settle_renders(h);
}

fn drag(h: &mut H, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.step();
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    for i in 1..=4 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 4.0)));
        h.step();
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run();
}

#[test]
fn the_view_menu_shows_and_hides_the_side_view() {
    let mut h = open();
    assert!(!h.state().prefs.layout.side_view, "hidden on first launch");
    assert!(h.query_by_label("Side View").is_none());
    let view = {
        let ws = h.state_mut().workspace_mut().unwrap();
        let mut view = ws.viewport.unwrap();
        view.zoom = 2.0;
        view.fitted = false;
        ws.viewport = Some(view);
        view
    };
    h.run();
    let before = ws(&h).canvas_rect.unwrap();
    h.get_by_label("View").click();
    h.run();
    common::last(&h, "Show Side View").click();
    common::settle_renders(&mut h);
    assert!(h.state().prefs.layout.side_view);
    assert!(h.query_by_label("Side View").is_some());
    let after = ws(&h).canvas_rect.unwrap();
    assert!(after.height() < before.height(), "the canvas gives it room");
    assert_eq!(after.width(), before.width());
    assert_eq!(ws(&h).viewport, Some(view));
    let center = ws(&h).screen_map(1.0).unwrap().to_doc(after.center());
    assert!((center - view.center).hypot() < 1e-3);
    // The item is checked while the panel is shown.
    h.get_by_label("View").click();
    h.run();
    let checked = common::last(&h, "Show Side View")
        .accesskit_node()
        .toggled();
    assert_eq!(checked, Some(Toggled::True));
}

#[test]
fn dragging_the_top_edge_resizes_within_the_limits() {
    let mut h = open();
    show_side_view(&mut h);
    let layout = |h: &H| h.state().prefs.layout.clone();
    assert_eq!(layout(&h).side_height, size::SIDE_VIEW_DEFAULT);
    let x = common::SIZE.x / 2.0;
    let bottom = common::SIZE.y - size::STATUS_BAR_HEIGHT;
    let canvas_before = ws(&h).canvas_rect.unwrap().height();
    drag(
        &mut h,
        Pos2::new(x, bottom - size::SIDE_VIEW_DEFAULT),
        Pos2::new(x, bottom - size::SIDE_VIEW_DEFAULT - 60.0),
    );
    let grown = layout(&h).side_height;
    assert!(grown > size::SIDE_VIEW_DEFAULT + 30.0, "{grown}");
    assert!(ws(&h).canvas_rect.unwrap().height() < canvas_before);
    // As tall as allowed, then as short as allowed.
    drag(&mut h, Pos2::new(x, bottom - grown), Pos2::new(x, 0.0));
    let max = layout(&h).side_height;
    assert!(
        max > grown && max <= common::SIZE.y * size::SIDE_VIEW_MAX_SHARE,
        "{max}"
    );
    drag(
        &mut h,
        Pos2::new(x, bottom - max),
        Pos2::new(x, bottom + 100.0),
    );
    assert_eq!(layout(&h).side_height, size::SIDE_VIEW_MIN);
}

#[test]
fn hide_panels_keeps_the_side_view() {
    let mut h = open();
    show_side_view(&mut h);
    h.key_press(Key::Tab);
    h.run();
    assert!(h.state().prefs.layout.panels_hidden);
    assert!(h.state().prefs.layout.side_view);
    assert!(h.query_by_label("Side View").is_some());
}

#[test]
fn the_directions_are_chosen_in_the_header() {
    let mut h = open();
    show_side_view(&mut h);
    for (label, direction) in [
        ("Front", tp_app::side_view::Direction::Front),
        ("Back", tp_app::side_view::Direction::Back),
        ("Top", tp_app::side_view::Direction::Top),
        ("Right", tp_app::side_view::Direction::Right),
        ("Left", tp_app::side_view::Direction::Left),
    ] {
        h.get_by_role_and_label(Role::RadioButton, label).click();
        common::settle_renders(&mut h);
        assert_eq!(h.state().prefs.layout.side_direction, direction);
    }
}

#[test]
fn the_drawn_texture_is_the_open_main_texture_or_the_last_one() {
    let mut h = open();
    show_side_view(&mut h);
    assert_eq!(ws(&h).side_view.drawn(), Some(surface(&h, STANDARD)));
    assert!(ws(&h).side_view.texture(surface(&h, STANDARD)).is_some());
    activate(&mut h, "high_roof");
    assert_eq!(ws(&h).side_view.drawn(), Some(surface(&h, "high_roof")));
    assert!(ws(&h).side_view.texture(surface(&h, "high_roof")).is_some());
    // An accessory texture: the High roof stays, and the Chassis is never rendered for it.
    activate(&mut h, "chassis");
    assert_eq!(ws(&h).side_view.drawn(), Some(surface(&h, "high_roof")));
    assert!(ws(&h).side_view.texture(surface(&h, "chassis")).is_none());
}

#[test]
fn a_recolored_object_shows_in_the_side_view() {
    use tp_core::document::{Frame, Paint, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    let mut h = open();
    show_side_view(&mut h);
    let standard = surface(&h, STANDARD);
    // Covering the LEFT SIDE of the template (120..2020 × 420..1620) in red.
    let frame = Frame::new(Point::new(1070.0, 1020.0), Size::new(1900.0, 1200.0), 0.0);
    let id =
        h.state_mut()
            .workspace_mut()
            .unwrap()
            .create_shape(ShapeKind::rectangle(), frame, 1.0);
    let paint = |h: &mut H, color: Rgba| {
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.edit_object(id, "undo-change-fill", 0.0, |o| {
            o.fill = Paint::Solid(color)
        });
        common::settle_renders(h);
    };
    paint(&mut h, Rgba::rgb(255, 0, 0));
    let color_at = |h: &H| ws(h).side_view.color_at(standard, [0.26, 0.25]);
    assert_eq!(color_at(&h), Some(Rgba::rgb(255, 0, 0)));
    paint(&mut h, Rgba::rgb(0, 0, 255));
    assert_eq!(color_at(&h), Some(Rgba::rgb(0, 0, 255)));
}

/// A project of one vehicle made from a package `id` (not installed unless asked).
fn open_vehicle(id: &str, kind: &str, main: SampleTexture, install: bool) -> H {
    let mut h = common::harness();
    let bytes = sample::package_with(id, "Test", "1.0.0", kind, &[main], &[]);
    if install {
        h.state_mut().vehicles.install_bytes(&bytes).unwrap();
    }
    let package = Package::read(&bytes).unwrap();
    let textures = vec![package.manifest.paint_job.main[0].id.clone()];
    let project = tp_app::vehicle_project::fleet_project("Test", &[(&package, &textures)]).unwrap();
    common::open_project(&mut h, project);
    h.run();
    h.state_mut().prefs.layout.side_view = true;
    common::settle_renders(&mut h);
    h
}

fn tex(id: &'static str, side: u32) -> SampleTexture {
    SampleTexture {
        id,
        name: "Main",
        size: TextureSize::new(side, side),
        layout: 1,
    }
}

fn says(h: &H, message: &str) {
    let text = tr(message);
    assert!(h.query_by_label(&text).is_some(), "{message}: {text}");
    assert_eq!(ws(h).side_view.drawn(), None);
}

#[test]
fn a_custom_vehicle_says_the_side_view_is_not_available() {
    let h = open_vehicle("custom.mine", "truck", tex("cabin", 4096), true);
    says(&h, "side-view-custom");
}

#[test]
fn an_official_package_without_a_mesh_says_to_generate_it_again() {
    let h = open_vehicle(
        "official.ets2.test.truck",
        "truck",
        tex("cabin", 4096),
        true,
    );
    says(&h, "side-view-regenerate");
}

#[test]
fn a_blank_official_trailer_without_a_mesh_says_to_generate_it_again() {
    // Trailers made before accessories were drawn have no mesh for their blank main texture.
    let h = open_vehicle(
        "official.ets2.test.box",
        "trailer",
        tex("trailer", 64),
        true,
    );
    says(&h, "side-view-regenerate");
}

#[test]
fn a_package_that_is_not_installed_is_needed() {
    let h = open_vehicle("community.test.gone", "truck", tex("cabin", 4096), false);
    says(&h, "side-view-package");
}

#[test]
fn another_package_without_a_mesh_says_it_is_not_available() {
    let h = open_vehicle("community.test.plain", "truck", tex("cabin", 4096), true);
    says(&h, "side-view-none");
}

#[test]
fn the_sample_trailer_is_drawn_with_its_curtain_body() {
    let mut h = common::harness();
    let package = Package::read(tp_app::vehicles::SAMPLES[1].bytes).unwrap();
    let textures = vec!["base".to_owned(), "body_13_6".to_owned()];
    let project =
        tp_app::vehicle_project::fleet_project("Trailer", &[(&package, &textures)]).unwrap();
    common::open_project(&mut h, project);
    h.state_mut().prefs.layout.side_view = true;
    common::settle_renders(&mut h);
    assert_eq!(ws(&h).side_view.drawn(), Some(0));
    // The chassis and the long curtain body, each with its own texture; the body is rendered too.
    assert_eq!(ws(&h).side_view.drawn_textures(), ["base", "body_13_6"]);
    assert!(ws(&h).side_view.texture(1).is_some());
}

/// The header fits the narrowest canvas (the smallest window with both panels at their minimum)
/// in every language.
#[test]
fn the_header_fits_the_narrowest_canvas_in_every_language() {
    use tp_i18n::Language;
    for language in [
        Language::English,
        Language::French,
        Language::Spanish,
        Language::German,
    ] {
        let mut prefs = tp_app::prefs::Prefs::default();
        prefs.set_language(Some(language));
        prefs.layout.side_view = true;
        prefs.layout.left_width = size::LEFT_PANEL_MIN;
        prefs.layout.inspector_width = size::INSPECTOR_MIN;
        let mut state = AppState::with_prefs(prefs, None);
        state.vehicles = common::sample_library();
        let mut h = common::builder()
            .with_size(egui::Vec2::new(960.0, 600.0))
            .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
        let package = Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
        let textures = vec![STANDARD.to_owned()];
        let project =
            tp_app::vehicle_project::fleet_project("Fit", &[(&package, &textures)]).unwrap();
        common::open_project(&mut h, project);
        common::settle_renders(&mut h);
        let canvas = ws(&h).canvas_rect.unwrap();
        let button = h
            .get_by_role_and_label(Role::Button, &tr("side-view-accessories"))
            .rect();
        // The title gives way to the controls when the canvas is too narrow for both.
        if let Some(title) = h.query_by_label(&tr("side-view-title")) {
            assert!(
                title.rect().right() <= button.left(),
                "{language:?}: title overlaps"
            );
        }
        let mut previous = button.right();
        for direction in tp_app::side_view::Direction::ALL {
            let rect = h
                .get_by_role_and_label(Role::RadioButton, &tr(direction.label()))
                .rect();
            assert!(
                rect.left() >= previous,
                "{language:?} {direction:?} overlaps: {} < {previous}",
                rect.left()
            );
            previous = rect.right();
        }
        assert!(
            previous <= canvas.right(),
            "{language:?}: {previous} > {}",
            canvas.right()
        );
        assert!(
            button.left() >= canvas.left(),
            "{language:?}: button leaves the canvas"
        );
    }
}

fn drawn(h: &H) -> Vec<String> {
    ws(h).side_view.drawn_textures().to_vec()
}

/// Opens the Accessories list and picks `option` in the row of `slot`.
fn pick(h: &mut H, slot: &str, option: &str) {
    // The list stays open between picks.
    if h.query_by_role_and_label(Role::ComboBox, slot).is_none() {
        h.get_by_role_and_label(Role::Button, "Accessories").click();
        h.run();
    }
    h.get_by_role_and_label(Role::ComboBox, slot).click();
    h.run();
    common::last(h, option).click();
    common::settle_renders(h);
}

#[test]
fn the_default_configuration_draws_the_cab_with_its_side_skirts() {
    let mut h = open();
    show_side_view(&mut h);
    assert_eq!(drawn(&h), ["standard", "side_skirts"]);
    // Each texture the view draws is rendered, the other accessories are not.
    assert!(
        ws(&h)
            .side_view
            .texture(surface(&h, "side_skirts"))
            .is_some()
    );
    assert!(ws(&h).side_view.texture(surface(&h, "chassis")).is_none());
    // Switching cabins keeps the skirts; opening the accessory texture keeps the cab.
    activate(&mut h, "high_roof");
    assert_eq!(drawn(&h), ["high_roof", "side_skirts"]);
    activate(&mut h, "side_skirts");
    assert_eq!(ws(&h).side_view.drawn(), Some(surface(&h, "high_roof")));
    assert_eq!(drawn(&h), ["high_roof", "side_skirts"]);
}

#[test]
fn picking_none_removes_a_piece_and_marks_the_project_changed() {
    let mut h = open();
    show_side_view(&mut h);
    ws_mut(&mut h).mark_saved(std::path::PathBuf::from("x.truckpaint"));
    assert!(!ws(&h).has_unsaved_changes());
    pick(&mut h, "Side skirts", "None");
    assert_eq!(drawn(&h), ["standard"]);
    assert!(ws(&h).has_unsaved_changes());
    // The choice belongs to the vehicle: the other cabin shows no skirts either.
    activate(&mut h, "high_roof");
    assert_eq!(drawn(&h), ["high_roof"]);
    activate(&mut h, "standard");
    // Back to the only option: the default again, nothing recorded.
    pick(&mut h, "Side skirts", "Side skirts");
    assert_eq!(drawn(&h), ["standard", "side_skirts"]);
    assert!(ws(&h).project.side_view.is_empty());
}

#[test]
fn a_choice_is_not_an_undo_step() {
    use tp_core::document::{Frame, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    let mut h = open();
    show_side_view(&mut h);
    let frame = Frame::new(Point::new(500.0, 500.0), Size::new(300.0, 300.0), 0.0);
    ws_mut(&mut h).create_shape(ShapeKind::rectangle(), frame, 1.0);
    pick(&mut h, "Side skirts", "None");
    ws_mut(&mut h).undo();
    common::settle_renders(&mut h);
    assert!(
        ws(&h).project.surface().objects.is_empty(),
        "the shape is undone"
    );
    assert_eq!(drawn(&h), ["standard"], "the skirts stay at None");
}

#[test]
fn an_edit_of_the_accessory_texture_shows_on_the_vehicle() {
    use tp_core::document::{Frame, Paint, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    let mut h = open();
    show_side_view(&mut h);
    activate(&mut h, "side_skirts");
    let skirts = surface(&h, "side_skirts");
    let frame = Frame::new(Point::new(512.0, 512.0), Size::new(1024.0, 1024.0), 0.0);
    let id = ws_mut(&mut h).create_shape(ShapeKind::rectangle(), frame, 1.0);
    ws_mut(&mut h).edit_object(id, "undo-change-fill", 0.0, |o| {
        o.fill = Paint::Solid(Rgba::rgb(0, 128, 0));
    });
    common::settle_renders(&mut h);
    assert_eq!(
        ws(&h).side_view.color_at(skirts, [0.5, 0.2]),
        Some(Rgba::rgb(0, 128, 0))
    );
}

#[test]
fn a_package_made_before_accessories_is_drawn_with_an_empty_list() {
    let mut h = common::harness();
    let old = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/vehicles/community.truckpaint.sample_truck-1.0.0.tpv"
    );
    let bytes = std::fs::read(old).unwrap();
    h.state_mut().vehicles.install_bytes(&bytes).unwrap();
    let package = Package::read(&bytes).unwrap();
    let textures = vec!["standard".to_owned()];
    let project =
        tp_app::vehicle_project::fleet_project(common::SAMPLE, &[(&package, &textures)]).unwrap();
    common::open_project(&mut h, project);
    h.state_mut().prefs.layout.side_view = true;
    common::settle_renders(&mut h);
    assert_eq!(drawn(&h), ["standard"], "the cab alone, as before");
    h.get_by_role_and_label(Role::Button, "Accessories").click();
    h.run();
    assert!(
        h.query_by_label(&tr("side-view-accessories-empty"))
            .is_some()
    );
}

#[test]
fn the_button_is_disabled_when_there_is_no_vehicle_to_draw() {
    let h = open_vehicle("custom.mine", "truck", tex("cabin", 4096), true);
    let button = h.get_by_role_and_label(Role::Button, "Accessories");
    assert!(button.accesskit_node().is_disabled());
}

/// A truck with a roof deflector slot (two options) on its cabin and a chassis of two lengths.
fn fitted_truck() -> H {
    use serde_json::json;
    let mut h = common::harness();
    let textures = sample::truck_textures();
    let mut manifest = sample::manifest("official.ets2.test.fit", "Fit", "1.0.0", &textures);
    manifest["paint_job"]["main"][0]["preview_mesh"] = json!("preview/cabin.json");
    let piece = |x: f64| {
        json!({"texture": "cabin", "positions": [[x, 0, 0], [x + 1.0, 0, 0], [x, 1, 0]],
               "uv": [[0, 0], [1, 0], [0, 1]], "triangles": [[0, 1, 2]]})
    };
    let at = |x: f64| json!([{"position": [x, 0, 0], "rotation": [1, 0, 0, 0], "scale": 1}]);
    let mesh = json!({
    "format": 1, "pieces": [piece(0.0)], "locators": {"r_deflector": at(0.5)},
    "slots": [
        {"id": "chassis", "options": [
            {"id": "short", "name": "Short", "default": true, "pieces": [piece(10.0)]},
            {"id": "long", "name": "Long", "pieces": [piece(20.0)]}]},
        {"id": "r_deflector", "parent": "cabin", "options": [
            {"id": "stock", "name": "Stock", "default": true, "pieces": [piece(30.0)]},
            {"id": "long", "name": "Long", "pieces": [piece(40.0)]}]},
    ]});
    let mut files = sample::templates(&textures);
    files.push(("preview/cabin.json".into(), mesh.to_string().into_bytes()));
    let bytes = sample::zip(&manifest, &files);
    h.state_mut().vehicles.install_bytes(&bytes).unwrap();
    let package = Package::read(&bytes).unwrap();
    let project =
        tp_app::vehicle_project::fleet_project("Fit", &[(&package, &["cabin".to_owned()])])
            .unwrap();
    common::open_project(&mut h, project);
    h.state_mut().prefs.layout.side_view = true;
    common::settle_renders(&mut h);
    h
}

#[test]
fn each_slot_shows_one_option_and_the_chassis_has_no_none() {
    let mut h = fitted_truck();
    let choices = |h: &H| {
        ws(h)
            .project
            .side_view
            .get("official.ets2.test.fit")
            .cloned()
    };
    assert_eq!(
        choices(&h),
        None,
        "the game's configuration records nothing"
    );
    // The rows are in the order of the preview mesh, with the slots' names.
    h.get_by_role_and_label(Role::Button, "Accessories").click();
    h.run();
    let chassis = h.get_by_role_and_label(Role::ComboBox, "Chassis").rect();
    let deflector = h
        .get_by_role_and_label(Role::ComboBox, "Roof deflector")
        .rect();
    assert!(chassis.top() < deflector.top());
    // The chassis offers its options only; the deflector offers None as well.
    h.get_by_role_and_label(Role::ComboBox, "Chassis").click();
    h.run();
    assert!(h.query_by_label("None").is_none());
    assert!(h.query_all_by_label("Long").count() > 0);
    common::last(&h, "Long").click();
    h.run();
    h.get_by_role_and_label(Role::ComboBox, "Roof deflector")
        .click();
    h.run();
    assert!(h.query_by_label("None").is_some());
    common::last(&h, "Long").click();
    h.run();
    let chosen = choices(&h).unwrap();
    assert_eq!(chosen["chassis"].as_deref(), Some("long"));
    assert_eq!(chosen["r_deflector"].as_deref(), Some("long"));
    assert_eq!(chosen.len(), 2);
}
