//! The local MCP server in the app: the tools on the open project, one undo
//! step per change, no file written, calls waiting for the user's gestures
//! and dialogs, and the server following its setting.

mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use base64::Engine;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::{Value, json};
use tp_app::AppState;
use tp_app::gesture::Gesture;
use tp_app::prefs::Prefs;
use tp_app::state::Modal;
use tp_core::document::{ObjectId, Paint, Rgba, ShapeKind};
use tp_core::kurbo::Shape;
use tp_mcp::{Outcome, ServerState};

type H = Harness<'static, AppState>;

fn open() -> H {
    let mut h = common::harness();
    common::create_project(&mut h);
    h
}

/// Runs a tool directly (no HTTP), finishing deferred work.
fn call(h: &mut H, name: &str, args: Value) -> Result<Outcome, String> {
    let call = tp_mcp::tools::parse(name, &args)?;
    let now = h.ctx.input(|i| i.time);
    match h.state_mut().run_mcp_call(&call, now)? {
        Outcome::Deferred(work) => work(),
        other => Ok(other),
    }
}

fn ok(h: &mut H, name: &str, args: Value) -> Value {
    match call(h, name, args) {
        Ok(Outcome::Json(v)) => v,
        other => panic!("{name}: {other:?}"),
    }
}

fn err(h: &mut H, name: &str, args: Value) -> String {
    call(h, name, args).expect_err(name)
}

/// The id of the first created object.
fn id(v: &Value) -> ObjectId {
    ObjectId(v["ids"][0].as_u64().expect("an id"))
}

/// The textures of the project, in order.
fn textures(h: &mut H) -> Vec<String> {
    let p = ok(h, "get_project", json!({}));
    p["vehicles"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|v| v["textures"].as_array().unwrap().clone())
        .map(|t| t["id"].as_str().unwrap().to_owned())
        .collect()
}

fn main_texture(h: &mut H) -> String {
    textures(h).remove(0)
}

fn objects(h: &H, index: usize) -> usize {
    h.state().workspace().unwrap().project.surfaces[index]
        .objects
        .len()
}

fn rect(h: &mut H, texture: &str, x: f64, fill: &str) -> ObjectId {
    let v = ok(
        h,
        "add_shape",
        json!({"texture": texture, "shape": "rectangle", "x": x, "y": 100, "width": 400, "height": 300, "fill": fill}),
    );
    id(&v)
}

fn decode(png: &[u8]) -> image::RgbaImage {
    image::load_from_memory(png).unwrap().to_rgba8()
}

#[test]
fn reading_a_fleet() {
    let mut h = common::harness();
    let truck = tp_vehicles::Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
    let trailer = tp_vehicles::Package::read(tp_app::vehicles::SAMPLES[1].bytes).unwrap();
    let fleet = [
        (
            &truck,
            tp_app::vehicle_project::default_textures(&truck.manifest),
        ),
        (
            &trailer,
            tp_app::vehicle_project::default_textures(&trailer.manifest),
        ),
    ];
    let project = tp_app::vehicle_project::fleet_project("Fleet", &fleet).unwrap();
    let surfaces = project.surfaces.len();
    common::open_project(&mut h, project);
    let p = ok(&mut h, "get_project", json!({}));
    let vehicles = p["vehicles"].as_array().unwrap();
    assert_eq!(vehicles.len(), 2);
    assert_eq!(vehicles[0]["id"], tp_app::vehicles::SAMPLE_ID);
    assert_eq!(vehicles[1]["id"], tp_app::vehicles::SAMPLE_TRAILER_ID);
    let listed: usize = vehicles
        .iter()
        .map(|v| v["textures"].as_array().unwrap().len())
        .sum();
    assert_eq!(listed, surfaces);
    let first = &vehicles[0]["textures"][0];
    assert!(
        first["id"]
            .as_str()
            .unwrap()
            .starts_with("community.truckpaint.sample_truck/")
    );
    assert!(first["width"].as_f64().unwrap() > 0.0);
    assert_eq!(first["part"], "main");
}

#[test]
fn no_project_open() {
    let mut h = common::harness();
    h.run();
    assert_eq!(
        err(&mut h, "get_project", json!({})),
        tp_app::mcp::NO_PROJECT
    );
}

#[test]
fn reading_a_path_round_trips() {
    let mut h = open();
    let t = main_texture(&mut h);
    let d = "M 100 100 C 400 0 700 200 1000 100 L 1000 300 L 100 300 Z";
    let first = id(&ok(
        &mut h,
        "add_path",
        json!({"texture": t, "d": d, "fill": "#d4202c"}),
    ));
    let read = ok(&mut h, "get_object", json!({"texture": t, "id": first.0}));
    assert_eq!(read["kind"], "path");
    assert_eq!(read["fill"], "#D4202C");
    let again = id(&ok(
        &mut h,
        "add_path",
        json!({"texture": t, "d": read["d"]}),
    ));
    let ws = h.state().workspace().unwrap();
    let s = ws.project.surface();
    let (a, b) = (
        s.get(first).unwrap().bounding_box(),
        s.get(again).unwrap().bounding_box(),
    );
    assert!(
        (a.x0 - b.x0).abs() < 1e-6 && (a.y1 - b.y1).abs() < 1e-6,
        "{a:?} {b:?}"
    );
    assert!((a.x0 - 100.0).abs() < 1e-6 && (a.x1 - 1000.0).abs() < 1e-6);
}

#[test]
fn drawing_a_stripe() {
    let mut h = open();
    let t = main_texture(&mut h);
    let before = objects(&h, 0);
    let v = ok(
        &mut h,
        "add_path",
        json!({"texture": t, "d": "M 0 1800 C 1200 1500 2800 2100 4096 1800 L 4096 2000 C 2800 2300 1200 1700 0 2000 Z", "fill": "#d4202c"}),
    );
    let ws = h.state().workspace().unwrap();
    let s = &ws.project.surfaces[0];
    assert_eq!(s.objects.len(), before + 1);
    let top = s.objects.last().unwrap();
    assert_eq!(top.id, id(&v));
    assert_eq!(top.kind, ShapeKind::Path);
    assert_eq!(top.fill, Paint::Solid(Rgba::rgb(0xd4, 0x20, 0x2c)));
    let b = top.bounding_box();
    assert!(b.x0.abs() < 1e-6 && (b.x1 - 4096.0).abs() < 1e-6);
    assert_eq!(ws.selection, vec![top.id]);
}

#[test]
fn ring_with_a_hole() {
    let mut h = open();
    let t = main_texture(&mut h);
    let circle = |r: f64| {
        format!(
            "M {} 1000 A {r} {r} 0 1 0 {} 1000 A {r} {r} 0 1 0 {} 1000 Z",
            1000.0 - r,
            1000.0 + r,
            1000.0 - r
        )
    };
    let d = format!("{} {}", circle(400.0), circle(200.0));
    let ring = id(&ok(
        &mut h,
        "add_path",
        json!({"texture": t, "d": d, "fill_rule": "evenodd"}),
    ));
    let ws = h.state().workspace().unwrap();
    let o = ws.project.surface().get(ring).unwrap();
    let fill = o.fill_path();
    assert_eq!(
        fill.winding(tp_core::kurbo::Point::new(1000.0, 1000.0)),
        0,
        "the hole"
    );
    assert_ne!(
        fill.winding(tp_core::kurbo::Point::new(1300.0, 1000.0)),
        0,
        "the ring"
    );
}

#[test]
fn seeing_the_result() {
    let mut h = open();
    let t = main_texture(&mut h);
    let side = h.state().workspace().unwrap().project.surfaces[0]
        .size
        .width;
    assert_eq!(side, 4096.0);
    ok(
        &mut h,
        "add_shape",
        json!({"texture": t, "shape": "rectangle", "x": 1024, "y": 1024, "width": 2048, "height": 2048, "fill": "#ff0000"}),
    );
    let Ok(Outcome::Image { png, text }) = call(&mut h, "render_texture", json!({"texture": t}))
    else {
        panic!("an image");
    };
    let img = decode(&png);
    assert_eq!(img.dimensions(), (1024, 1024));
    assert!(text.contains("1 image px = 4 texture px"), "{text}");
    let center = img.get_pixel(512, 512);
    assert!(
        center[0] > 240 && center[1] < 20 && center[2] < 20,
        "{center:?}"
    );
    let corner = img.get_pixel(10, 10);
    assert!(corner[0] < 240, "artboard, not red: {corner:?}");
}

#[test]
fn size_capped() {
    let mut h = open();
    let t = main_texture(&mut h);
    let Ok(Outcome::Image { png, .. }) = call(
        &mut h,
        "render_texture",
        json!({"texture": t, "max_size": 8000, "with_template": false}),
    ) else {
        panic!("an image");
    };
    assert_eq!(decode(&png).dimensions(), (2048, 2048));
}

#[test]
fn template_of_the_texture() {
    let mut h = open();
    let t = main_texture(&mut h);
    let Ok(Outcome::Image { png, text }) = call(
        &mut h,
        "get_template",
        json!({"texture": t, "max_size": 512}),
    ) else {
        panic!("an image");
    };
    assert_eq!(decode(&png).dimensions(), (512, 512));
    assert!(text.starts_with("Template"), "{text}");
}

#[test]
fn texture_without_template() {
    let mut h = common::harness();
    common::open_project(&mut h, tp_core::Project::new("Bare", Default::default()));
    let e = err(&mut h, "get_template", json!({"texture": "0"}));
    assert!(e.contains("has no template"), "{e}");
    let e = err(&mut h, "list_objects", json!({"texture": "nope"}));
    assert!(e.contains("unknown texture"), "{e}");
}

#[test]
fn undoing_the_assistants_change() {
    let mut h = open();
    let t = main_texture(&mut h);
    let before = objects(&h, 0);
    ok(
        &mut h,
        "add_path",
        json!({"texture": t, "d": "M 0 0 L 100 0 L 100 100 Z"}),
    );
    let label = h.state().workspace().unwrap().history.undo_label();
    assert_eq!(label, Some("undo-ai-add-path"));
    assert_eq!(tp_i18n::tr("undo-ai-add-path"), "AI: Add Path");
    h.state_mut().workspace_mut().unwrap().undo();
    assert_eq!(objects(&h, 0), before);
}

#[test]
fn two_quick_calls_are_two_steps() {
    let mut h = open();
    let t = main_texture(&mut h);
    let before = objects(&h, 0);
    let first = rect(&mut h, &t, 0.0, "#ff0000");
    rect(&mut h, &t, 500.0, "#00ff00");
    let ws = h.state_mut().workspace_mut().unwrap();
    ws.undo();
    assert_eq!(ws.project.surfaces[0].objects.len(), before + 1);
    assert!(ws.project.surfaces[0].get(first).is_some());
}

#[test]
fn interleaved_with_the_user() {
    let mut h = open();
    let t = main_texture(&mut h);
    let r = rect(&mut h, &t, 0.0, "#ff0000");
    let x0 = |h: &H| {
        h.state()
            .workspace()
            .unwrap()
            .project
            .surface()
            .get(r)
            .unwrap()
            .frame
            .center
            .x
    };
    let start = x0(&h);
    // The user moves it.
    h.state_mut()
        .workspace_mut()
        .unwrap()
        .edit("undo-move-guide", 10.0, false, |p, _| {
            let moved = tp_core::document::translate(
                &p.selected_objects(&[r]),
                tp_core::kurbo::Vec2::new(50.0, 0.0),
                false,
            );
            p.surface_mut().replace(&moved);
        });
    let ellipse = id(&ok(
        &mut h,
        "add_shape",
        json!({"texture": t, "shape": "ellipse", "x": 0, "y": 0, "width": 10, "height": 10}),
    ));
    let ws = h.state_mut().workspace_mut().unwrap();
    ws.undo();
    assert!(ws.project.surface().get(ellipse).is_none());
    ws.undo();
    assert_eq!(x0(&h), start);
}

#[test]
fn drawing_on_another_texture() {
    let mut h = open();
    let all = textures(&mut h);
    assert!(all.len() > 1);
    let shown_selection = vec![rect(&mut h, &all[0], 0.0, "#ff0000")];
    let other = all.last().unwrap().clone();
    let last = all.len() - 1;
    let before = objects(&h, last);
    rect(&mut h, &other, 0.0, "#0000ff");
    let ws = h.state().workspace().unwrap();
    assert_eq!(ws.project.active_surface, 0);
    assert_eq!(ws.selection, shown_selection);
    assert_eq!(ws.project.surfaces[last].objects.len(), before + 1);
}

#[test]
fn linking_to_the_palette() {
    let mut h = open();
    let t = main_texture(&mut h);
    let s = ok(
        &mut h,
        "add_swatch",
        json!({"name": "Company red", "color": "#d4202c"}),
    );
    let swatch = tp_core::document::SwatchId(s["id"].as_u64().unwrap());
    let shape = id(&ok(
        &mut h,
        "add_shape",
        json!({"texture": t, "shape": "ellipse", "x": 0, "y": 0, "width": 100, "height": 100, "fill": {"swatch": "company red"}}),
    ));
    let dark = Rgba::rgb(0x80, 0, 0);
    let ws = h.state_mut().workspace_mut().unwrap();
    ws.project.set_swatch_color(swatch, dark);
    assert_eq!(
        ws.project.surface().get(shape).unwrap().fill,
        Paint::Solid(dark)
    );
    let e = err(
        &mut h,
        "add_shape",
        json!({"texture": t, "shape": "ellipse", "x": 0, "y": 0, "width": 1, "height": 1, "fill": {"swatch": "Blue"}}),
    );
    assert!(e.contains("no swatch \"Blue\""), "{e}");
}

#[test]
fn partial_failure_changes_nothing() {
    let mut h = open();
    let t = main_texture(&mut h);
    let a = rect(&mut h, &t, 0.0, "#ff0000");
    let b = rect(&mut h, &t, 500.0, "#00ff00");
    let steps = h.state().workspace().unwrap().history.len();
    let e = err(
        &mut h,
        "delete_objects",
        json!({"texture": t, "ids": [a.0, b.0, 999_999]}),
    );
    assert!(e.contains("999999"), "{e}");
    let ws = h.state().workspace().unwrap();
    assert!(ws.project.surface().get(a).is_some() && ws.project.surface().get(b).is_some());
    assert_eq!(ws.history.len(), steps, "no step recorded");
}

#[test]
fn changing_the_fill_detaches_the_style() {
    let mut h = open();
    let t = main_texture(&mut h);
    let r = rect(&mut h, &t, 0.0, "#ff0000");
    let style = h
        .state_mut()
        .workspace_mut()
        .unwrap()
        .project
        .new_graphic_style(r, "Style")
        .unwrap();
    let other = rect(&mut h, &t, 500.0, "#00ff00");
    ok(
        &mut h,
        "apply_style",
        json!({"texture": t, "style": style.0.to_string(), "ids": [other.0]}),
    );
    let get = |h: &H| {
        (**h.state()
            .workspace()
            .unwrap()
            .project
            .surface()
            .get(other)
            .unwrap())
        .clone()
    };
    assert_eq!(get(&h).style, Some(style));
    assert_eq!(get(&h).fill, Paint::Solid(Rgba::rgb(255, 0, 0)));
    ok(
        &mut h,
        "update_object",
        json!({"texture": t, "id": other.0, "fill": "#0000ff"}),
    );
    assert_eq!(get(&h).style, None);
}

#[test]
fn update_moves_and_resizes() {
    let mut h = open();
    let t = main_texture(&mut h);
    let r = rect(&mut h, &t, 0.0, "#ff0000");
    ok(
        &mut h,
        "update_object",
        json!({"texture": t, "id": r.0, "x": 1000, "width": 800, "rotation": 30, "opacity": 0.5, "name": "Band"}),
    );
    let ws = h.state().workspace().unwrap();
    let o = ws.project.surface().get(r).unwrap();
    assert!((o.frame.size.width - 800.0).abs() < 1e-6);
    assert!((o.frame.size.height - 300.0).abs() < 1e-6);
    assert!((o.frame.center.x - 1400.0).abs() < 1e-6, "{:?}", o.frame);
    assert!((o.frame.center.y - 250.0).abs() < 1e-6);
    assert!((o.frame.rotation_deg - 30.0).abs() < 1e-6);
    assert_eq!(o.opacity, 0.5);
    assert_eq!(o.name, "Band");
}

#[test]
fn boolean_union_makes_one_path() {
    let mut h = open();
    let t = main_texture(&mut h);
    let a = rect(&mut h, &t, 0.0, "#ff0000");
    let b = rect(&mut h, &t, 200.0, "#00ff00");
    let before = objects(&h, 0);
    let v = ok(
        &mut h,
        "arrange_objects",
        json!({"texture": t, "ids": [a.0, b.0], "action": "combine", "operation": "unite"}),
    );
    let ws = h.state().workspace().unwrap();
    assert_eq!(ws.project.surfaces[0].objects.len(), before - 1);
    let o = ws.project.surface().get(id(&v)).unwrap();
    assert_eq!(o.kind, ShapeKind::Path);
    assert!((o.bounding_box().width() - 600.0).abs() < 0.5);
    assert_eq!(ws.history.undo_label(), Some("undo-ai-arrange"));
}

#[test]
fn arrange_group_order_align() {
    let mut h = open();
    let t = main_texture(&mut h);
    let a = rect(&mut h, &t, 0.0, "#ff0000");
    let b = rect(&mut h, &t, 700.0, "#00ff00");
    ok(
        &mut h,
        "arrange_objects",
        json!({"texture": t, "ids": [b.0], "action": "back"}),
    );
    let first = h.state().workspace().unwrap().project.surface().objects[0].id;
    assert_eq!(first, b);
    ok(
        &mut h,
        "arrange_objects",
        json!({"texture": t, "ids": [a.0], "action": "align", "edge": "right", "to": "texture"}),
    );
    let o = h
        .state()
        .workspace()
        .unwrap()
        .project
        .surface()
        .get(a)
        .unwrap()
        .bounding_box();
    assert!((o.x1 - 4096.0).abs() < 1e-6);
    let g = id(&ok(
        &mut h,
        "arrange_objects",
        json!({"texture": t, "ids": [a.0, b.0], "action": "group"}),
    ));
    assert!(
        h.state()
            .workspace()
            .unwrap()
            .project
            .surface()
            .get(g)
            .unwrap()
            .is_group()
    );
    let r = ok(&mut h, "list_objects", json!({"texture": t}));
    let group = r["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == g.0)
        .unwrap()
        .clone();
    assert_eq!(group["children"].as_array().unwrap().len(), 2);
    let inner = id(&ok(
        &mut h,
        "add_shape",
        json!({"texture": t, "shape": "star", "x": 0, "y": 0, "width": 50, "height": 50, "parent": g.0}),
    ));
    let group = h
        .state()
        .workspace()
        .unwrap()
        .project
        .surface()
        .get(g)
        .unwrap()
        .clone();
    assert_eq!(group.children.last().unwrap().id, inner);
    let e = err(
        &mut h,
        "add_shape",
        json!({"texture": t, "shape": "star", "x": 0, "y": 0, "width": 50, "height": 50, "parent": inner.0}),
    );
    assert!(e.contains("not a group"), "{e}");
}

#[test]
fn placed_symbol_instance() {
    let mut h = open();
    let t = main_texture(&mut h);
    let r = rect(&mut h, &t, 0.0, "#ff0000");
    let (symbol, _) = h
        .state_mut()
        .workspace_mut()
        .unwrap()
        .project
        .convert_to_symbol(&[r], "Logo")
        .unwrap();
    let name = h
        .state()
        .workspace()
        .unwrap()
        .project
        .symbol(symbol)
        .unwrap()
        .name
        .clone();
    let v = ok(
        &mut h,
        "place_symbol",
        json!({"texture": t, "symbol": name, "x": 2000, "y": 2000, "width": 800, "height": 600}),
    );
    let ws = h.state().workspace().unwrap();
    let o = ws.project.surface().get(id(&v)).unwrap();
    assert!(matches!(o.kind, ShapeKind::Instance { symbol: s, .. } if s == symbol));
    assert!(!o.children.is_empty(), "expanded from the symbol");
    let b = o.bounding_box();
    assert!(b.x0 >= 2000.0 - 1e-6 && b.x1 <= 2800.0 + 1e-6, "{b:?}");
    // Editing the symbol changes the instance.
    let instance = id(&v);
    let ws = h.state_mut().workspace_mut().unwrap();
    let content = &mut ws.project.symbols[0].surface.objects[0];
    std::sync::Arc::make_mut(content).fill = Paint::Solid(Rgba::rgb(0, 0, 255));
    ws.project.refresh_instances();
    let o = ws.project.surface().get(instance).unwrap();
    assert_eq!(o.children[0].fill, Paint::Solid(Rgba::rgb(0, 0, 255)));
}

#[test]
fn texts() {
    let mut h = open();
    let t = main_texture(&mut h);
    let v = ok(
        &mut h,
        "add_text",
        json!({"texture": t, "text": "ACME LOGISTICS", "x": 500, "y": 600, "size": 300, "fill": "#ffffff", "font": "oswald"}),
    );
    let ws = h.state().workspace().unwrap();
    let o = ws.project.surface().get(id(&v)).unwrap();
    assert_eq!(o.kind, ShapeKind::Text);
    let block = o.text.as_ref().unwrap();
    assert_eq!(block.style.family, "Oswald");
    assert_eq!(block.style.size, 300.0);
    let f = o.frame;
    assert!((f.center.x - f.size.width / 2.0 - 500.0).abs() < 1e-6);
    assert!((f.center.y - f.size.height / 2.0 - 600.0).abs() < 1e-6);
    let e = err(
        &mut h,
        "add_text",
        json!({"texture": t, "text": "X", "x": 0, "y": 0, "font": "No Such Font"}),
    );
    assert!(e.contains("not available"), "{e}");
}

fn png(rgba: [u8; 4]) -> Vec<u8> {
    tp_app::import::solid_png(64, 32, rgba)
}

#[test]
fn logo_pasted_in_the_chat() {
    let mut h = open();
    let t = main_texture(&mut h);
    let data = base64::engine::general_purpose::STANDARD.encode(png([0, 128, 255, 255]));
    let v = ok(
        &mut h,
        "import_image",
        json!({"texture": t, "data": data, "name": "logo.png", "x": 100, "y": 100, "width": 640, "height": 320}),
    );
    let ws = h.state().workspace().unwrap();
    let o = ws.project.surface().get(id(&v)).unwrap();
    let ShapeKind::Image { asset } = o.kind else {
        panic!("an image");
    };
    assert_eq!(ws.project.assets[&asset].name, "logo");
    assert!((o.frame.size.width - 640.0).abs() < 1e-6);
    assert_eq!(ws.history.undo_label(), Some("undo-ai-import-image"));
}

#[test]
fn logo_by_path_and_twice() {
    let mut h = open();
    let t = main_texture(&mut h);
    let dir = tempfile::tempdir().unwrap();
    let svg = dir.path().join("acme.svg");
    std::fs::write(&svg, r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><rect width="200" height="100" fill="red"/></svg>"#).unwrap();
    let v = ok(
        &mut h,
        "import_image",
        json!({"texture": t, "path": svg.to_str().unwrap()}),
    );
    let center = h
        .state()
        .workspace()
        .unwrap()
        .project
        .surface()
        .get(id(&v))
        .unwrap()
        .frame
        .center;
    assert_eq!(center, tp_core::kurbo::Point::new(2048.0, 2048.0));
    let file = dir.path().join("logo.png");
    std::fs::write(&file, png([1, 2, 3, 255])).unwrap();
    let assets = |h: &H| h.state().workspace().unwrap().project.assets.len();
    let before = assets(&h);
    ok(
        &mut h,
        "import_image",
        json!({"texture": t, "path": file.to_str().unwrap()}),
    );
    let data = base64::engine::general_purpose::STANDARD.encode(png([1, 2, 3, 255]));
    ok(
        &mut h,
        "import_image",
        json!({"texture": t, "data": data, "name": "again.png"}),
    );
    assert_eq!(assets(&h), before + 1, "one asset for the same image");
    let junk = dir.path().join("junk.png");
    std::fs::write(&junk, b"not a png").unwrap();
    let e = err(
        &mut h,
        "import_image",
        json!({"texture": t, "path": junk.to_str().unwrap()}),
    );
    assert!(e.starts_with("junk.png:"), "{e}");
}

#[test]
fn not_an_image() {
    let mut h = open();
    let t = main_texture(&mut h);
    let steps = h.state().workspace().unwrap().history.len();
    let e = err(
        &mut h,
        "import_image",
        json!({"texture": t, "path": "/Users/me/.ssh/id_rsa"}),
    );
    assert!(e.contains("only PNG, JPEG and SVG"), "{e}");
    assert_eq!(h.state().workspace().unwrap().history.len(), steps);
}

#[test]
fn symbol_being_edited() {
    let mut h = open();
    let t = main_texture(&mut h);
    let r = rect(&mut h, &t, 0.0, "#ff0000");
    let ws = h.state_mut().workspace_mut().unwrap();
    let (symbol, _) = ws.project.convert_to_symbol(&[r], "Logo").unwrap();
    ws.edit_symbol(symbol, 0.0);
    let e = err(
        &mut h,
        "add_shape",
        json!({"texture": t, "shape": "ellipse", "x": 0, "y": 0, "width": 1, "height": 1}),
    );
    assert_eq!(e, tp_app::mcp::EDITING_SYMBOL);
    ok(&mut h, "list_objects", json!({"texture": t}));
}

#[test]
fn saving_stays_with_the_user() {
    let mut h = open();
    let t = main_texture(&mut h);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fleet.tpp");
    std::fs::write(&path, b"saved project").unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    h.state_mut()
        .workspace_mut()
        .unwrap()
        .mark_saved(path.clone());
    assert_eq!(
        h.state().workspace().unwrap().save_state(),
        tp_app::workspace::SaveState::Saved
    );
    let a = rect(&mut h, &t, 0.0, "#ff0000");
    ok(
        &mut h,
        "update_object",
        json!({"texture": t, "id": a.0, "x": 10}),
    );
    ok(
        &mut h,
        "add_swatch",
        json!({"name": "Blue", "color": "#0000ff"}),
    );
    ok(
        &mut h,
        "delete_objects",
        json!({"texture": t, "ids": [a.0]}),
    );
    h.run();
    assert_eq!(std::fs::read(&path).unwrap(), b"saved project");
    assert_eq!(
        std::fs::metadata(&path).unwrap().modified().unwrap(),
        modified
    );
    assert_eq!(dir.path().read_dir().unwrap().count(), 1, "no other file");
    assert_eq!(
        h.state().workspace().unwrap().save_state(),
        tp_app::workspace::SaveState::Unsaved
    );
}

// --- Through the server ---------------------------------------------------

fn free_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// The app with the server on at a free port, a project open.
fn serving(timeout: Duration) -> (H, u16) {
    let port = free_port();
    let prefs = Prefs {
        mcp_port: port,
        ..Prefs::default()
    };
    let mut h = common::harness_with(prefs);
    // Turned on once the timeout is set (it applies when the server starts).
    h.state_mut().mcp.timeout = Some(timeout);
    h.state_mut().prefs.mcp_enabled = true;
    common::create_project(&mut h);
    h.run();
    assert_eq!(h.state().mcp.view().state, ServerState::Listening);
    (h, port)
}

/// Posts a tools/call; returns the result object.
fn post_tool(port: u16, name: &str, arguments: Value) -> Value {
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": name, "arguments": arguments}}).to_string();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("server listening");
    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .unwrap();
    write!(
        stream,
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut reader = BufReader::new(stream);
    let mut status = String::new();
    reader.read_line(&mut status).unwrap();
    assert!(status.contains("200"), "{status}");
    let mut rest = String::new();
    reader.read_to_string(&mut rest).unwrap();
    let (_, body) = rest.split_once("\r\n\r\n").unwrap();
    let v: Value = serde_json::from_str(body).unwrap();
    v["result"].clone()
}

/// Steps the app until `done` or a few seconds passed.
fn step_until(h: &mut H, done: impl Fn(&H) -> bool) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !done(h) && std::time::Instant::now() < deadline {
        h.step();
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn call_during_a_drag_waits_for_the_release() {
    let (mut h, port) = serving(Duration::from_secs(30));
    let t = main_texture(&mut h);
    let before = objects(&h, 0);
    h.state_mut().workspace_mut().unwrap().gesture = Gesture::Panning;
    let client = std::thread::spawn(move || {
        post_tool(
            port,
            "add_shape",
            json!({"texture": t, "shape": "ellipse", "x": 0, "y": 0, "width": 100, "height": 100}),
        )
    });
    for _ in 0..20 {
        h.step();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(objects(&h, 0), before, "waits during the gesture");
    h.state_mut().workspace_mut().unwrap().gesture = Gesture::Idle;
    step_until(&mut h, |h| objects(h, 0) == before + 1);
    assert_eq!(objects(&h, 0), before + 1);
    let result = client.join().unwrap();
    assert_eq!(result["isError"], false, "{result}");
    let view = h.state().mcp.view();
    assert!(view.connected);
    assert_eq!(view.recent_writes, 1);
}

#[test]
fn dialog_left_open() {
    let (mut h, port) = serving(Duration::from_millis(300));
    let t = main_texture(&mut h);
    let before = objects(&h, 0);
    h.state_mut().modal = Some(Modal::RemoveVehicle {
        package_id: tp_app::vehicles::SAMPLE_ID.into(),
        name: "Sample".into(),
    });
    let client = std::thread::spawn(move || {
        post_tool(
            port,
            "add_shape",
            json!({"texture": t, "shape": "ellipse", "x": 0, "y": 0, "width": 100, "height": 100}),
        )
    });
    let result = loop {
        h.step();
        if client.is_finished() {
            break client.join().unwrap();
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(result["isError"], true);
    assert!(
        result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("busy")
    );
    h.state_mut().modal = None;
    for _ in 0..10 {
        h.step();
    }
    assert_eq!(objects(&h, 0), before, "never added later");
}

#[test]
fn preferences_dialog_does_not_block_calls() {
    let (mut h, port) = serving(Duration::from_secs(30));
    h.state_mut().modal = Some(Modal::Preferences);
    let client = std::thread::spawn(move || post_tool(port, "get_project", json!({})));
    step_until(&mut h, |_| client.is_finished());
    let result = client.join().unwrap();
    assert_eq!(result["isError"], false, "{result}");
    assert_eq!(
        result["structuredContent"]["vehicles"][0]["id"],
        tp_app::vehicles::SAMPLE_ID
    );
}

#[test]
fn render_over_http_is_an_image() {
    let (mut h, port) = serving(Duration::from_secs(30));
    let t = main_texture(&mut h);
    let client = std::thread::spawn(move || {
        post_tool(
            port,
            "render_texture",
            json!({"texture": t, "max_size": 256}),
        )
    });
    step_until(&mut h, |_| client.is_finished());
    let result = client.join().unwrap();
    assert_eq!(result["content"][0]["type"], "image");
    assert_eq!(result["content"][0]["mimeType"], "image/png");
    let png = base64::engine::general_purpose::STANDARD
        .decode(result["content"][0]["data"].as_str().unwrap())
        .unwrap();
    assert_eq!(decode(&png).dimensions(), (256, 256));
}

#[test]
fn setting_starts_and_stops_the_server() {
    let (mut h, port) = serving(Duration::from_secs(30));
    assert!(TcpStream::connect(("127.0.0.1", port)).is_ok());
    h.state_mut().prefs.mcp_enabled = false;
    h.run();
    assert!(!h.state().mcp.is_running());
    assert_eq!(h.state().mcp.view().state, ServerState::Off);
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
    // The document keeps what was done.
    assert!(h.state().has_project());
}

#[test]
fn off_on_first_launch() {
    let mut h = common::harness();
    h.run();
    assert!(!h.state().mcp.is_running());
    assert_eq!(h.state().mcp.view().state, ServerState::Off);
    assert!(!Prefs::default().mcp_enabled);
    assert_eq!(Prefs::default().mcp_port, 7373);
}

#[test]
fn port_in_use() {
    let taken = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = taken.local_addr().unwrap().port();
    let mut h = common::harness_with(Prefs {
        mcp_enabled: true,
        mcp_port: port,
        ..Prefs::default()
    });
    h.run();
    let view = h.state().mcp.view();
    assert_eq!(view.state, ServerState::PortInUse);
    assert_eq!(view.port, port);
    assert!(!h.state().mcp.is_running());
}

// --- Preferences and status bar -------------------------------------------

fn copied(h: &H) -> Vec<String> {
    h.output()
        .platform_output
        .commands
        .iter()
        .filter_map(|c| match c {
            egui::OutputCommand::CopyText(text) => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn turning_the_server_on() {
    let port = free_port();
    let mut h = common::harness_with(Prefs {
        mcp_port: port,
        ..Prefs::default()
    });
    h.state_mut().modal = Some(Modal::Preferences);
    // Shown on its tab, as when opened from the status bar.
    h.state_mut().preferences_tab = tp_app::state::PrefsTab::Ai;
    h.run();
    assert!(h.query_by_label("Off").is_some());
    h.get_by_label("Let an AI assistant draw on the open project")
        .click();
    h.run();
    assert!(h.state().prefs.mcp_enabled);
    assert!(h.state().mcp.is_running());
    assert!(h.query_by_label("On, waiting for a client").is_some());
    h.get_by_label("Copy").click();
    h.step();
    assert_eq!(copied(&h), [format!("http://127.0.0.1:{port}/mcp")]);
    // The port can't change while the server is on.
    h.get_by_label("Let an AI assistant draw on the open project")
        .click();
    h.run();
    assert!(!h.state().mcp.is_running());
}

#[test]
fn port_in_use_in_preferences() {
    let taken = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = taken.local_addr().unwrap().port();
    let mut h = common::harness_with(Prefs {
        mcp_enabled: true,
        mcp_port: port,
        ..Prefs::default()
    });
    h.state_mut().modal = Some(Modal::Preferences);
    h.state_mut().preferences_tab = tp_app::state::PrefsTab::Ai;
    h.run();
    let text = format!("Port {port} is in use by another program.");
    assert!(
        h.query_by_label_contains(&text).is_some(),
        "the refusal is shown"
    );
}

#[test]
fn setting_survives_restart() {
    use tp_app::prefs::PrefsStore;
    let dir = tempfile::tempdir().unwrap();
    let store = PrefsStore::new(dir.path());
    let mut prefs = Prefs {
        mcp_enabled: true,
        mcp_port: 8123,
        ..Prefs::default()
    };
    store.save(&prefs).unwrap();
    let loaded = store.load();
    assert!(loaded.issue.is_none());
    assert!(loaded.prefs.mcp_enabled);
    assert_eq!(loaded.prefs.mcp_port, 8123);
    // Reset to defaults keeps them.
    prefs.reset_scaling();
    assert!(prefs.mcp_enabled && prefs.mcp_port == 8123);
}

#[test]
fn older_preferences_file() {
    use tp_app::prefs::PrefsStore;
    let dir = tempfile::tempdir().unwrap();
    let store = PrefsStore::new(dir.path());
    std::fs::write(
        store.path(),
        include_str!("fixtures/prefs-before-spaces.ron"),
    )
    .unwrap();
    let loaded = store.load();
    assert!(loaded.issue.is_none(), "{:?}", loaded.issue);
    assert!(!loaded.prefs.mcp_enabled);
    assert_eq!(loaded.prefs.mcp_port, tp_mcp::DEFAULT_PORT);
    assert_eq!(loaded.prefs.ui_scale, 1.25, "the rest is kept");
}

#[test]
fn status_bar_follows_the_server() {
    let (mut h, port) = serving(Duration::from_secs(30));
    assert!(h.query_by_label("MCP").is_some(), "on, idle");
    let t = main_texture(&mut h);
    for x in [0, 500, 1000] {
        let t = t.clone();
        let client = std::thread::spawn(move || {
            post_tool(
                port,
                "add_shape",
                json!({"texture": t, "shape": "ellipse", "x": x, "y": 0, "width": 100, "height": 100}),
            )
        });
        step_until(&mut h, |_| client.is_finished());
        client.join().unwrap();
    }
    h.run();
    assert!(
        h.query_by_label("MCP · 3 changes").is_some(),
        "client drawing"
    );
    h.get_by_label("MCP · 3 changes").click();
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::Preferences)));
    assert_eq!(h.state().preferences_tab, tp_app::state::PrefsTab::Ai);
    h.state_mut().modal = None;
    h.state_mut().prefs.mcp_enabled = false;
    h.run();
    assert!(h.query_by_label("MCP").is_none(), "off");
    assert!(h.query_by_label_contains("MCP ·").is_none());
}

#[test]
fn undo_labels_in_french() {
    tp_i18n::set_language(tp_i18n::Language::French);
    let label = tp_i18n::tr!("cmd-undo-action", action = tp_i18n::tr("undo-ai-add-path"));
    tp_i18n::set_language(tp_i18n::Language::English);
    assert_eq!(label, "Annuler IA : Ajouter un tracé");
}
