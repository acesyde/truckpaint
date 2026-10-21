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

/// The sample truck with both cabins and the chassis, in the Workshop on the Standard cab.
fn open() -> H {
    let mut h = common::harness();
    let package = Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
    let textures = ["standard", "high_roof", "chassis"].map(String::from);
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
fn a_blank_main_texture_says_its_parts_have_their_own_textures() {
    let h = open_vehicle(
        "official.ets2.test.box",
        "trailer",
        tex("trailer", 64),
        true,
    );
    says(&h, "side-view-blank");
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
fn the_sample_trailer_is_drawn() {
    let mut h = common::harness();
    let package = Package::read(tp_app::vehicles::SAMPLES[1].bytes).unwrap();
    let textures = vec!["base".to_owned()];
    let project =
        tp_app::vehicle_project::fleet_project("Trailer", &[(&package, &textures)]).unwrap();
    common::open_project(&mut h, project);
    h.state_mut().prefs.layout.side_view = true;
    common::settle_renders(&mut h);
    assert_eq!(ws(&h).side_view.drawn(), Some(0));
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
        let title = h.get_by_label(&tr("side-view-title")).rect();
        let mut previous = title.right();
        for direction in tp_app::side_view::Direction::ALL {
            let rect = h
                .get_by_role_and_label(Role::RadioButton, &tr(direction.label()))
                .rect();
            assert!(
                rect.left() >= previous,
                "{language:?} {direction:?} overlaps"
            );
            previous = rect.right();
        }
        assert!(
            previous <= canvas.right(),
            "{language:?}: {previous} > {}",
            canvas.right()
        );
    }
}
