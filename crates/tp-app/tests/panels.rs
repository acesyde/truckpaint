//! Headless tests for editing-panels: the inspector (Layout, Appearance and
//! its color popover), the Layers tab, groups on the canvas and the
//! Eyedropper.

mod common;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::prefs::{Prefs, PrefsStore};
use tp_app::tool::Tool;
use tp_app::workspace::{ColorTarget, Workspace};
use tp_core::document::tree;
use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

/// Tall window, so nothing scrolls.
fn open() -> H {
    open_with_step(1.0 / 4.0)
}

fn open_with_step(step_dt: f32) -> H {
    let mut prefs = Prefs::default();
    prefs.layout.left_tab = tp_app::layout::LeftTab::Layers;
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
        .with_step_dt(step_dt)
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(prefs, None),
        );
    common::create_project(&mut h);
    h.run();
    h
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn add(
    h: &mut H,
    kind: ShapeKind,
    name: &str,
    center: (f64, f64),
    size: (f64, f64),
    rot: f64,
) -> ObjectId {
    let mut o = Object::new(
        ObjectId(0),
        kind,
        Frame::new(
            Point::new(center.0, center.1),
            Size::new(size.0, size.1),
            rot,
        ),
    );
    o.name = name.into();
    let id = ws_mut(h).project.add(o);
    h.run();
    id
}

fn rect(h: &mut H, name: &str, center: (f64, f64), size: (f64, f64)) -> ObjectId {
    add(h, ShapeKind::rectangle(), name, center, size, 0.0)
}

fn select(h: &mut H, ids: &[ObjectId]) {
    ws_mut(h).selection = ids.to_vec();
    h.run();
}

fn obj(h: &H, id: ObjectId) -> Object {
    (**ws(h).project.surface().get(id).expect("object")).clone()
}

fn field_value(h: &H, name: &str) -> String {
    h.get_by_role_and_label(Role::TextInput, name)
        .value()
        .unwrap_or_default()
}

/// Focuses a text field, replaces its content and confirms with Enter.
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

fn screen(h: &H, x: f64, y: f64) -> Pos2 {
    ws(h)
        .screen_map(1.0)
        .expect("canvas")
        .to_screen(Point::new(x, y))
}

fn click_at(h: &mut H, at: Pos2, mods: Modifiers) {
    h.event(Event::ModifiersChanged(mods));
    h.event(Event::PointerMoved(at));
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: mods,
        });
    }
    h.step();
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run();
}

fn drag(h: &mut H, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    for i in 1..=8 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 8.0)));
        h.step();
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.run();
}

/// Center of a Layers row.
fn row(h: &H, name: &str) -> egui::Rect {
    h.get_by_role_and_label(Role::Button, name).rect()
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.5
}

// --- Transform -----------------------------------------------------------------

#[test]
fn transform_shows_single_object_values() {
    let mut h = open();
    let a = add(
        &mut h,
        ShapeKind::rectangle(),
        "A",
        (300.0, 200.0),
        (400.0, 200.0),
        15.0,
    );
    select(&mut h, &[a]);
    assert_eq!(field_value(&h, "X position"), "300");
    assert_eq!(field_value(&h, "Y position"), "200");
    assert_eq!(field_value(&h, "Width"), "400");
    assert_eq!(field_value(&h, "Height"), "200");
    assert_eq!(field_value(&h, "Rotation"), "15");
    assert_eq!(field_value(&h, "Scale"), "100");
}

#[test]
fn no_layout_section_nor_empty_state_with_nothing_selected() {
    let h = open();
    assert!(h.query_by_label("Nothing to transform").is_none());
    assert!(h.query_by_label("Layout").is_none());
    assert!(h.query_by_label("X position").is_none());
    // The texture's properties instead.
    h.get_by_label("Standard cab");
    h.get_by_label("Select an object to set its layout, fill and stroke.");
}

#[test]
fn typed_width_resizes_around_center_and_undoes() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    type_into(&mut h, "Width", "600");
    let f = obj(&h, a).frame;
    assert!(
        near(f.size.width, 600.0) && near(f.size.height, 200.0),
        "{f:?}"
    );
    assert!(near(f.center.x, 300.0));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert!(near(obj(&h, a).frame.size.width, 400.0));
}

#[test]
fn escape_reverts_a_typed_value() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    h.get_by_role_and_label(Role::TextInput, "X position")
        .focus();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "X position")
        .type_text("999");
    h.run();
    h.key_press(Key::Escape);
    h.run();
    assert_eq!(obj(&h, a).frame.center.x, 300.0);
    assert_eq!(field_value(&h, "X position"), "300");
}

#[test]
fn scale_field_scales_and_resets() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    type_into(&mut h, "Scale", "50");
    let f = obj(&h, a).frame;
    assert!(
        near(f.size.width, 200.0) && near(f.size.height, 100.0),
        "{f:?}"
    );
    assert_eq!(field_value(&h, "Scale"), "100");
}

#[test]
fn locked_proportions() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    h.get_by_label("Lock proportions").click();
    h.run();
    type_into(&mut h, "Width", "800");
    assert!(near(obj(&h, a).frame.size.height, 400.0));
}

#[test]
fn mixed_rotation() {
    let mut h = open();
    let a = add(
        &mut h,
        ShapeKind::rectangle(),
        "A",
        (300.0, 200.0),
        (100.0, 100.0),
        0.0,
    );
    let b = add(
        &mut h,
        ShapeKind::rectangle(),
        "B",
        (900.0, 200.0),
        (100.0, 100.0),
        30.0,
    );
    select(&mut h, &[a, b]);
    assert_eq!(field_value(&h, "Rotation"), "");
}

// --- Properties ----------------------------------------------------------------

#[test]
fn multiple_selection_summary() {
    let mut h = open();
    let ids = [
        rect(&mut h, "A", (100.0, 100.0), (50.0, 50.0)),
        rect(&mut h, "B", (300.0, 100.0), (50.0, 50.0)),
        rect(&mut h, "C", (500.0, 100.0), (50.0, 50.0)),
    ];
    select(&mut h, &ids);
    h.get_by_label("3 objects");
}

#[test]
fn opacity_field_and_undo() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    type_into(&mut h, "Opacity", "40");
    assert!((obj(&h, a).opacity - 0.4).abs() < 1e-6);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(obj(&h, a).opacity, 1.0);
}

#[test]
fn corner_radius_is_clamped() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    type_into(&mut h, "Corner radius", "300");
    assert_eq!(
        obj(&h, a).kind,
        ShapeKind::Rectangle {
            corner_radius: 100.0
        }
    );
}

fn hexagon(h: &mut H) -> ObjectId {
    add(
        h,
        ShapeKind::Polygon {
            sides: 6,
            star: None,
        },
        "Hex",
        (800.0, 800.0),
        (400.0, 300.0),
        15.0,
    )
}

#[test]
fn make_a_five_point_star() {
    let mut h = open();
    let p = hexagon(&mut h);
    let frame = obj(&h, p).frame;
    select(&mut h, &[p]);
    type_into(&mut h, "Sides", "5");
    h.get_by_role_and_label(Role::CheckBox, "Star").click();
    h.run();
    let star = obj(&h, p);
    assert_eq!(
        star.kind,
        ShapeKind::Polygon {
            sides: 5,
            star: Some(0.5)
        }
    );
    assert_eq!(star.flattened(0.1).len(), 10);
    assert_eq!(star.frame, frame, "bounds, center and rotation are kept");
    assert_eq!(field_value(&h, "Inner radius"), "50");
    // One step per change: Undo turns the star back into a pentagon.
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(
        obj(&h, p).kind,
        ShapeKind::Polygon {
            sides: 5,
            star: None
        }
    );
    // New polygons use the last values set.
    assert_eq!(
        ws(&h).polygon_style.kind(),
        ShapeKind::Polygon {
            sides: 5,
            star: Some(0.5)
        }
    );
}

#[test]
fn sides_are_clamped() {
    let mut h = open();
    let p = hexagon(&mut h);
    select(&mut h, &[p]);
    type_into(&mut h, "Sides", "40");
    assert_eq!(
        obj(&h, p).kind,
        ShapeKind::Polygon {
            sides: 12,
            star: None
        }
    );
}

#[test]
fn polygon_settings_show_mixed_and_hide_for_other_kinds() {
    let mut h = open();
    let a = hexagon(&mut h);
    let b = add(
        &mut h,
        ShapeKind::Polygon {
            sides: 3,
            star: None,
        },
        "Tri",
        (1500.0, 800.0),
        (300.0, 300.0),
        0.0,
    );
    select(&mut h, &[a, b]);
    assert_eq!(field_value(&h, "Sides"), "");
    let r = rect(&mut h, "R", (300.0, 200.0), (100.0, 100.0));
    select(&mut h, &[a, r]);
    assert!(h.query_by_label("Sides").is_none());
}

fn add_line(h: &mut H) -> ObjectId {
    use tp_core::document::{Node, PathData, Subpath};
    let line = Object::from_path(
        ObjectId(0),
        PathData::new(vec![Subpath::new(
            vec![
                Node::corner(Point::new(100.0, 100.0)),
                Node::corner(Point::new(900.0, 400.0)),
            ],
            false,
        )]),
    );
    let id = ws_mut(h).project.add(line);
    h.run();
    id
}

fn line_width(h: &H, id: ObjectId) -> f64 {
    obj(h, id).path_data().unwrap().line_width
}

#[test]
fn thicker_line() {
    let mut h = open();
    let l = add_line(&mut h);
    select(&mut h, &[l]);
    assert_eq!(field_value(&h, "Line width"), "8");
    type_into(&mut h, "Line width", "40");
    assert_eq!(line_width(&h, l), 40.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(line_width(&h, l), 8.0);
}

#[test]
fn next_line_uses_the_last_width() {
    let mut h = open();
    let l = add_line(&mut h);
    select(&mut h, &[l]);
    type_into(&mut h, "Line width", "40");
    let from = screen(&h, 1000.0, 1000.0);
    let to = screen(&h, 1500.0, 1200.0);
    ws_mut(&mut h).tool = tp_app::tool::Tool::Line;
    h.run();
    drag(&mut h, from, to);
    let new = *ws(&h).selection.first().unwrap();
    assert_ne!(new, l);
    assert_eq!(line_width(&h, new), 40.0);
}

#[test]
fn nearly_horizontal_line() {
    let mut h = open();
    ws_mut(&mut h).tool = tp_app::tool::Tool::Line;
    h.run();
    let (a, b) = (screen(&h, 500.0, 1000.0), screen(&h, 3500.0, 1020.0));
    drag(&mut h, a, b);
    let id = ws(&h).selection[0];
    let line = obj(&h, id);
    let expected = (20.0f64 / 3000.0).atan().to_degrees();
    assert!(
        (line.frame.rotation_deg - expected).abs() < 0.05,
        "{}",
        line.frame.rotation_deg
    );
    assert!(
        (line.frame.size.width - 3000.0).abs() < 2.0,
        "the frame follows the line"
    );
    assert!(line.frame.size.height <= 1.0);
    type_into(&mut h, "Rotation", "0");
    let line = obj(&h, id);
    let ends: Vec<Point> = line.path_data().unwrap().subpaths[0]
        .nodes
        .iter()
        .map(|n| line.frame.affine() * n.point)
        .collect();
    assert!((ends[0].y - ends[1].y).abs() < 1e-6, "{ends:?}");
}

#[test]
fn line_width_hidden_for_closed_shapes() {
    let mut h = open();
    let l = add_line(&mut h);
    let r = rect(&mut h, "R", (300.0, 200.0), (100.0, 100.0));
    select(&mut h, &[l, r]);
    assert!(h.query_by_label("Line width").is_none());
}

#[test]
fn stroke_swatch_targets_the_stroke() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    h.get_by_label("Stroke color").click();
    h.run();
    assert_eq!(ws(&h).panels.color_target, ColorTarget::Stroke);
    // The color popover opens under the Stroke row.
    let row = h.get_by_label("Stroke color").rect();
    let hex = h.get_by_role_and_label(Role::TextInput, "Hex color").rect();
    assert!(hex.top() > row.bottom(), "{row:?} {hex:?}");
}

// --- Colors --------------------------------------------------------------------

#[test]
fn x_switches_target() {
    let mut h = open();
    h.key_press(Key::X);
    h.run();
    assert_eq!(ws(&h).panels.color_target, ColorTarget::Stroke);
    // The Stroke row is marked as the target, not by color alone.
    let marked = |label: &str| {
        h.get_by_label(label).accesskit_node().toggled() == Some(egui::accesskit::Toggled::True)
    };
    assert!(marked("Stroke color") && !marked("Fill color"));
}

#[test]
fn color_with_nothing_selected_applies_to_new_shapes() {
    let mut h = open();
    common::open_color_popover(&mut h, ColorTarget::Fill);
    type_into(&mut h, "Hex color", "#FF0000");
    ws_mut(&mut h).tool = Tool::Rectangle;
    h.run();
    let (a, b) = (screen(&h, 100.0, 100.0), screen(&h, 900.0, 600.0));
    drag(&mut h, a, b);
    let id = ws(&h).selection[0];
    assert_eq!(
        obj(&h, id).fill,
        tp_core::document::Paint::from(Rgba::rgb(255, 0, 0))
    );
}

#[test]
fn rgb_fields_update_color() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    common::open_color_popover(&mut h, ColorTarget::Fill);
    type_into(&mut h, "Red", "255");
    type_into(&mut h, "Green", "0");
    type_into(&mut h, "Blue", "0");
    assert_eq!(
        obj(&h, a).fill,
        tp_core::document::Paint::from(Rgba::rgb(255, 0, 0))
    );
    assert_eq!(field_value(&h, "Hex color"), "#FF0000");
}

#[test]
fn short_and_invalid_hex() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    common::open_color_popover(&mut h, ColorTarget::Fill);
    type_into(&mut h, "Hex color", "#f80");
    assert_eq!(
        obj(&h, a).fill,
        tp_core::document::Paint::from(Rgba::rgb(0xFF, 0x88, 0x00))
    );
    type_into(&mut h, "Hex color", "zz12");
    assert_eq!(
        obj(&h, a).fill,
        tp_core::document::Paint::from(Rgba::rgb(0xFF, 0x88, 0x00))
    );
}

#[test]
fn none_removes_stroke() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    let mut o = obj(&h, a);
    o.stroke = Some(StrokeStyle {
        paint: Rgba::rgb(0, 0, 0).into(),
        width: 8.0,
        ..Default::default()
    });
    ws_mut(&mut h).project.surface_mut().replace(&[o]);
    select(&mut h, &[a]);
    common::open_color_popover(&mut h, ColorTarget::Stroke);
    h.get_by_label("No stroke").click();
    h.run();
    assert_eq!(obj(&h, a).stroke, None);
}

#[test]
fn palette_add_and_apply() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    let b = rect(&mut h, "B", (900.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    common::open_color_popover(&mut h, ColorTarget::Fill);
    type_into(&mut h, "Hex color", "#123456");
    h.get_by_label("Add to Palette").click();
    h.run();
    let palette = &ws(&h).project.palette;
    assert_eq!(palette.len(), 1);
    assert_eq!(
        (palette[0].name.as_str(), palette[0].color),
        ("Color 1", Rgba::rgb(0x12, 0x34, 0x56))
    );
    let swatch = palette[0].id;
    assert_eq!(obj(&h, a).fill_swatch, Some(swatch), "the target is linked");
    select(&mut h, &[b]);
    h.get_by_label("Color 1").click();
    h.run();
    assert_eq!(
        obj(&h, b).fill,
        tp_core::document::Paint::from(Rgba::rgb(0x12, 0x34, 0x56))
    );
    assert_eq!(obj(&h, b).fill_swatch, Some(swatch), "linked to the swatch");
}

#[test]
fn picker_drag_is_one_named_undo_step() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    common::open_color_popover(&mut h, ColorTarget::Fill);
    let before = obj(&h, a).fill;
    let square = h.get_by_label("Saturation and value").rect();
    drag(
        &mut h,
        square.left_top() + Vec2::new(5.0, 5.0),
        square.right_top() + Vec2::new(-2.0, 2.0),
    );
    assert_ne!(obj(&h, a).fill, before);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-change-fill"));
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Undo Change Fill").click();
    h.run();
    assert_eq!(obj(&h, a).fill, before);
}

#[test]
fn recent_colors_persist_across_sessions() {
    let dir = tempfile::tempdir().unwrap();
    let store = || Some(PrefsStore::new(dir.path()));
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::new(store()),
        );
    common::create_project(&mut h);
    common::open_color_popover(&mut h, ColorTarget::Fill);
    for hex in ["#111111", "#222222", "#333333"] {
        type_into(&mut h, "Hex color", hex);
    }
    h.state_mut().persist_prefs(f64::MAX, true);
    let reopened = AppState::new(store());
    assert_eq!(
        &reopened.prefs.recent_colors[..3],
        &[
            [0x33, 0x33, 0x33, 255],
            [0x22, 0x22, 0x22, 255],
            [0x11, 0x11, 0x11, 255]
        ]
    );
}

// --- Stroke --------------------------------------------------------------------

#[test]
fn enable_stroke_and_set_width() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (400.0, 200.0));
    select(&mut h, &[a]);
    common::open_stroke_popover(&mut h);
    h.get_by_label("Stroke enabled").click();
    h.run();
    let stroke = obj(&h, a).stroke.expect("stroke added");
    assert_eq!(stroke.width, ws(&h).style.stroke.width);
    type_into(&mut h, "Stroke width", "12");
    assert_eq!(obj(&h, a).stroke.unwrap().width, 12.0);
}

// --- Layers --------------------------------------------------------------------

/// Background < Graphics[Burgundy base, White stripe, Grey stripe] < Branding[Logo, Company name] < Truck number
fn livery(h: &mut H) -> [ObjectId; 8] {
    let bg = rect(h, "Background", (2048.0, 2048.0), (4096.0, 4096.0));
    let base = rect(h, "Burgundy base", (2048.0, 1500.0), (4096.0, 1200.0));
    let white = rect(h, "White stripe", (2048.0, 2200.0), (4096.0, 160.0));
    let grey = rect(h, "Grey stripe", (2048.0, 2400.0), (4096.0, 120.0));
    let logo = add(
        h,
        ShapeKind::Ellipse,
        "Logo",
        (1200.0, 3200.0),
        (900.0, 900.0),
        0.0,
    );
    let name = rect(h, "Company name", (2700.0, 3200.0), (1600.0, 300.0));
    let number = rect(h, "Truck number", (3600.0, 3700.0), (400.0, 200.0));
    let ws = ws_mut(h);
    let graphics = ws.project.group(&[base, white, grey]).unwrap();
    let branding = ws.project.group(&[logo, name]).unwrap();
    for (id, label) in [(graphics, "Graphics"), (branding, "Branding")] {
        let mut g = (**ws.project.surface().get(id).unwrap()).clone();
        g.name = label.into();
        ws.project.surface_mut().replace(&[g]);
    }
    // Truck number back on top of the top level.
    ws.project.move_objects(&[number], tree::Placement::Top);
    ws.panels.expanded.extend([graphics, branding]);
    h.run();
    [bg, graphics, base, white, grey, branding, logo, number]
}

#[test]
fn tree_is_topmost_first_with_children_indented() {
    let mut h = open();
    livery(&mut h);
    let top = |name: &str| row(&h, name).top();
    assert!(top("Truck number") < top("Branding"));
    assert!(top("Branding") < top("Company name") && top("Company name") < top("Logo"));
    assert!(top("Logo") < top("Graphics") && top("Graphics") < top("Background"));
    assert!(row(&h, "Grey stripe").left() == row(&h, "Graphics").left());
}

#[test]
fn row_click_selects_object_inside_group() {
    let mut h = open();
    let ids = livery(&mut h);
    h.get_by_role_and_label(Role::Button, "Logo").click();
    h.run();
    assert_eq!(ws(&h).selection, vec![ids[6]]);
}

#[test]
fn rename_a_layer() {
    let mut h = open();
    // Not "Rectangle": the tool bar has a button with that name.
    let a = rect(&mut h, "Shape 1", (300.0, 200.0), (400.0, 200.0));
    let r = row(&h, "Shape 1").center();
    h.event(Event::PointerMoved(r));
    h.step();
    // Two clicks 0.1 s apart make a double click (time is set explicitly:
    // the harness does not advance the input clock).
    for k in 0..2 {
        h.input_mut().time = Some(100.0 + 0.1 * f64::from(k));
        for pressed in [true, false] {
            h.event(Event::PointerButton {
                pos: r,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        h.step();
    }
    h.run();
    assert!(ws(&h).panels.renaming.is_some());
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, "Layer name")
        .type_text("Burgundy base");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(obj(&h, a).name, "Burgundy base");
}

#[test]
fn hide_object_and_canvas_ignores_it() {
    let mut h = open();
    let ids = livery(&mut h);
    let white = ids[3];
    select(&mut h, &[white]);
    h.get_by_label("Hide White stripe").click();
    h.run();
    assert!(!obj(&h, white).visible);
    assert!(ws(&h).selection.is_empty());
    h.get_by_label("Show White stripe");
    let p = screen(&h, 2048.0, 2200.0);
    click_at(&mut h, p, Modifiers::COMMAND);
    assert_ne!(ws(&h).selection, vec![white]);
}

#[test]
fn hidden_group_hides_children_on_canvas() {
    let mut h = open();
    let ids = livery(&mut h);
    h.get_by_label("Hide Graphics").click();
    h.run();
    let drawn: Vec<ObjectId> = tree::draw_list(&ws(&h).project.surface().objects)
        .iter()
        .map(|(o, _)| o.id)
        .collect();
    for child in &ids[2..5] {
        assert!(!drawn.contains(child));
    }
}

#[test]
fn locked_object_ignores_canvas_clicks() {
    let mut h = open();
    let ids = livery(&mut h);
    h.get_by_label("Lock Background").click();
    h.run();
    let p = screen(&h, 100.0, 100.0);
    click_at(&mut h, p, Modifiers::NONE);
    assert!(!ws(&h).selection.contains(&ids[0]));
}

#[test]
fn drag_row_into_group_and_reorder() {
    let mut h = open();
    let ids = livery(&mut h);
    let (branding, number) = (ids[5], ids[7]);
    let (from, to) = (
        row(&h, "Truck number").center(),
        row(&h, "Branding").center(),
    );
    drag(&mut h, from, to);
    let group = obj(&h, branding);
    assert_eq!(group.children.last().unwrap().id, number);

    // Grey stripe is above White stripe; drag White stripe onto the upper part of Grey stripe.
    let grey = row(&h, "Grey stripe");
    let from = row(&h, "White stripe").center();
    drag(&mut h, from, Pos2::new(grey.center().x, grey.top() + 3.0));
    let children: Vec<u64> = obj(&h, ids[1]).children.iter().map(|c| c.id.0).collect();
    assert_eq!(children, vec![ids[2].0, ids[4].0, ids[3].0]);
}

#[test]
fn group_then_ungroup() {
    let mut h = open();
    let a = rect(&mut h, "A", (300.0, 200.0), (100.0, 100.0));
    let b = rect(&mut h, "B", (600.0, 200.0), (100.0, 100.0));
    select(&mut h, &[a, b]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::G);
    h.run();
    let group = ws(&h).selection[0];
    assert!(obj(&h, group).is_group());
    assert_eq!(obj(&h, group).children.len(), 2);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::G);
    h.run();
    assert_eq!(ws(&h).selection, vec![a, b]);
    let top: Vec<ObjectId> = ws(&h)
        .project
        .surface()
        .objects
        .iter()
        .map(|o| o.id)
        .collect();
    assert_eq!(top, vec![a, b]);
}

#[test]
fn new_layer_command() {
    let mut h = open();
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::N);
    h.run();
    let id = ws(&h).selection[0];
    assert_eq!(obj(&h, id).name, "Layer 1");
    h.get_by_role_and_label(Role::Button, "Layer 1");
}

// --- Canvas with groups, active layer, eyedropper ------------------------------------

#[test]
fn click_selects_group_and_cmd_click_selects_inner() {
    let mut h = open();
    let ids = livery(&mut h);
    let p = screen(&h, 1200.0, 3200.0);
    click_at(&mut h, p, Modifiers::NONE);
    assert_eq!(ws(&h).selection, vec![ids[5]]);
    click_at(&mut h, p, Modifiers::COMMAND);
    assert_eq!(ws(&h).selection, vec![ids[6]]);
}

#[test]
fn drawing_goes_into_selected_group_with_current_style() {
    let mut h = open();
    let ids = livery(&mut h);
    select(&mut h, &[ids[5]]);
    ws_mut(&mut h).style.fill = Rgba::rgb(255, 255, 255).into();
    ws_mut(&mut h).style.stroke_enabled = true;
    ws_mut(&mut h).style.stroke.width = 8.0;
    ws_mut(&mut h).tool = Tool::Ellipse;
    h.run();
    let (a, b) = (screen(&h, 500.0, 500.0), screen(&h, 900.0, 900.0));
    drag(&mut h, a, b);
    let new = ws(&h).selection[0];
    let group = obj(&h, ids[5]);
    assert_eq!(group.children.last().unwrap().id, new);
    let o = obj(&h, new);
    assert_eq!(
        o.fill,
        tp_core::document::Paint::from(Rgba::rgb(255, 255, 255))
    );
    assert_eq!(o.stroke.unwrap().width, 8.0);
}

#[test]
fn eyedropper_picks_fill_into_selection() {
    let mut h = open();
    let a = rect(&mut h, "A", (500.0, 500.0), (400.0, 400.0));
    let red = add(
        &mut h,
        ShapeKind::Ellipse,
        "Red",
        (2000.0, 2000.0),
        (400.0, 400.0),
        0.0,
    );
    let mut o = obj(&h, red);
    o.fill = Rgba::rgb(220, 20, 20).into();
    ws_mut(&mut h).project.surface_mut().replace(&[o]);
    select(&mut h, &[a]);
    ws_mut(&mut h).tool = Tool::Eyedropper;
    h.run();
    let p = screen(&h, 2000.0, 2000.0);
    click_at(&mut h, p, Modifiers::NONE);
    assert_eq!(
        obj(&h, a).fill,
        tp_core::document::Paint::from(Rgba::rgb(220, 20, 20))
    );
    assert_eq!(ws(&h).selection, vec![a]);
}

#[test]
fn layers_tab_header_counts_the_layers_and_adds_one() {
    let mut h = open();
    let side_skirts = ws(&h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == "Side skirts")
        .expect("Side skirts");
    ws_mut(&mut h).set_active_surface(side_skirts);
    h.run();
    for i in 0..5 {
        rect(
            &mut h,
            &format!("R{i}"),
            (100.0 + 60.0 * f64::from(i), 100.0),
            (40.0, 40.0),
        );
    }
    assert!(h.query_by_label("Side skirts · 5 layers").is_some());
    // The header's add button (before the footer's) runs New Layer.
    h.get_all_by_label("New Layer").next().unwrap().click();
    h.run();
    let top = ws(&h).project.surface().objects.last().cloned().unwrap();
    assert!(top.is_group() && top.children.is_empty());
    assert_eq!(top.name, "Layer 1");
    assert!(h.query_by_label("Side skirts · 6 layers").is_some());
}

#[test]
fn text_layers_are_named_by_their_text() {
    let mut h = open();
    for (i, content) in ["ACE LOGISTICS", "Since 1984"].into_iter().enumerate() {
        let at = Point::new(400.0, 400.0 + 300.0 * i as f64);
        let text = Object::text(
            ObjectId(0),
            tp_core::document::TextBlock::new(content, tp_core::document::CharStyle::default()),
            at,
        );
        assert_eq!(text.name, "Text");
        ws_mut(&mut h).project.add(text);
    }
    h.run();
    assert!(
        h.query_by_role_and_label(Role::Button, "ACE LOGISTICS")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::Button, "Since 1984")
            .is_some()
    );
}

/// How many nodes read `label` (a row, and its tooltip when shown).
fn count(h: &H, label: &str) -> usize {
    h.query_all_by_label(label).count()
}

#[test]
fn long_layer_name_ends_with_an_ellipsis_and_shows_in_full_on_hover() {
    let mut h = open();
    let long = "Grey stripe along the lower edge of the cabin";
    rect(&mut h, long, (400.0, 400.0), (200.0, 40.0));
    rect(&mut h, "Band", (400.0, 800.0), (200.0, 40.0));
    for name in [long, "Band"] {
        assert_eq!(count(&h, name), 1);
        h.get_by_role_and_label(Role::Button, name).hover();
        for _ in 0..8 {
            h.step();
        }
        h.run();
        // Only the shortened name has a tooltip.
        let tooltip = name == long;
        assert_eq!(count(&h, name), 1 + usize::from(tooltip), "{name}");
    }
}

/// Spec "Long vehicle name in the left panel": in the 40% longer text
/// mode, the vehicle's name and its version note stay on one line in the
/// Textures tab; a name too long ends with "…" and shows in full on hover.
#[test]
fn long_vehicle_name_stays_on_one_line_and_shows_in_full_on_hover() {
    tp_i18n::set_lengthening(40);
    let mut prefs = Prefs::default();
    prefs.layout.left_tab = tp_app::layout::LeftTab::Textures;
    let mut h = common::harness_with(prefs);
    common::create_project(&mut h);
    // A name longer than the panel, whatever its width.
    const NAME: &str = "TruckPaint Sample Truck, long-haul edition with a high roof";
    h.state_mut().workspace_mut().unwrap().project.vehicles[0].name = NAME.to_owned();
    h.run();
    let in_panel = |h: &H| {
        h.query_all_by_label(NAME)
            .map(|n| n.rect())
            .filter(|r| r.left() < 320.0)
            .collect::<Vec<_>>()
    };
    let name = in_panel(&h);
    assert_eq!(name.len(), 1, "{name:?}");
    let name = name[0];
    // One line of text, not two.
    assert!(name.height() < 24.0, "one line: {name:?}");
    let note = h
        .query_all_by_label_contains(" · 1.1.0")
        .map(|n| n.rect())
        .find(|r| r.left() < 320.0)
        .expect("version note");
    assert!(
        (note.center().y - name.center().y).abs() < 2.0,
        "{note:?} {name:?}"
    );
    assert!(name.right() <= note.left(), "{note:?} {name:?}");
    // Shortened with "…", never wrapped.
    let elided = h.output().shapes.iter().any(|c| match &c.shape {
        egui::Shape::Text(t) => {
            t.galley.text() == NAME && t.galley.elided && t.galley.rows.len() == 1
        }
        _ => false,
    });
    assert!(elided, "the name ends with an ellipsis");
    let before = h.query_all_by_label(NAME).count();
    h.query_all_by_label(NAME)
        .find(|n| n.rect().left() < 320.0)
        .unwrap()
        .hover();
    for _ in 0..60 {
        h.step();
    }
    h.run();
    // The tooltip with the whole name.
    assert_eq!(h.query_all_by_label(NAME).count(), before + 1);
    tp_i18n::set_lengthening(0);
}

/// Spec "Text that doesn't fit": a texture name longer than its row in the
/// Textures tab ends with "…" on one line and shows in full on hover.
#[test]
fn long_texture_name_ends_with_an_ellipsis_and_shows_in_full_on_hover() {
    let mut prefs = Prefs::default();
    prefs.layout.left_tab = tp_app::layout::LeftTab::Textures;
    let mut h = common::harness_with(prefs);
    common::create_project(&mut h);
    const NAME: &str = "Main texture of the cabin, with the long-haul side skirts and roof";
    h.state_mut().workspace_mut().unwrap().project.surfaces[0].name = NAME.to_owned();
    h.run();
    let elided = h.output().shapes.iter().any(|c| match &c.shape {
        egui::Shape::Text(t) => {
            t.galley.text() == NAME && t.galley.elided && t.galley.rows.len() == 1
        }
        _ => false,
    });
    assert!(elided, "the name ends with an ellipsis");
    // Elsewhere (the inspector's header) the name may be shown whole.
    let before = h.query_all_by_label(NAME).count();
    h.query_all_by_label_contains(NAME)
        .find(|n| n.rect().left() < 320.0)
        .expect("texture row")
        .hover();
    for _ in 0..60 {
        h.step();
    }
    h.run();
    // The tooltip with the whole name.
    assert_eq!(h.query_all_by_label(NAME).count(), before + 1);
}

// --- Objects to reposition ---------------------------------------------------

#[test]
fn layers_mark_objects_to_reposition() {
    let mut h = open();
    let logo = rect(&mut h, "Ace logo", (300.0, 300.0), (100.0, 50.0));
    rect(&mut h, "Stripe", (600.0, 300.0), (100.0, 50.0));
    let inner = rect(&mut h, "Inner", (300.0, 600.0), (50.0, 50.0));
    let other = rect(&mut h, "Other", (400.0, 600.0), (50.0, 50.0));
    select(&mut h, &[inner, other]);
    ws_mut(&mut h).group_selection(1.0);
    // As Update Template leaves them: the texture "Layout changed", the
    // logo and the group's first rectangle flagged.
    {
        let t = ws_mut(&mut h)
            .project
            .surface_mut()
            .template
            .as_mut()
            .unwrap();
        t.status = tp_core::TemplateStatus::LayoutChanged;
        t.flagged = [logo, inner].into();
    }
    select(&mut h, &[]);
    let marked = |h: &H, name: &str| {
        h.query_by_label(&format!("{name}, to reposition"))
            .is_some()
    };
    assert!(marked(&h, "Ace logo"));
    assert!(
        marked(&h, "Group"),
        "a collapsed group shows its child's mark"
    );
    assert!(!marked(&h, "Stripe") && h.query_all_by_label("Stripe").next().is_some());

    // Moving the logo clears its mark; Undo brings it back.
    select(&mut h, &[logo]);
    ws_mut(&mut h).nudge(10.0, 0.0, 2.0);
    h.run();
    assert!(!marked(&h, "Ace logo") && h.query_all_by_label("Ace logo").next().is_some());
    ws_mut(&mut h).undo();
    h.run();
    assert!(marked(&h, "Ace logo"));

    // An object drawn after the update has no mark.
    rect(&mut h, "Later", (800.0, 800.0), (50.0, 50.0));
    assert!(h.query_all_by_label("Later").next().is_some() && !marked(&h, "Later"));
}
