//! Headless tests for the example livery of the documentation (Blue Wave
//! Biarritz, `tests/common/blue_wave.rs`).

mod common;

use common::blue_wave::{self, LAYOUT, Livery};
use tp_core::document::{Object, ShapeKind};

type H = egui_kittest::Harness<'static, tp_app::AppState>;

fn open() -> (H, Livery) {
    let mut h = common::harness();
    let livery = blue_wave::blue_wave_fleet(&mut h);
    (h, livery)
}

/// Every object of a tree, groups and instances included.
fn all(objects: &[std::sync::Arc<Object>], out: &mut Vec<std::sync::Arc<Object>>) {
    for o in objects {
        out.push(o.clone());
        all(&o.children, out);
    }
}

#[test]
fn dump_for_review() {
    // `BLUE_WAVE_DUMP=<folder> cargo test … dump_for_review` writes every
    // texture as a PNG to look at; without it, nothing is written.
    let Some(dir) = std::env::var_os("BLUE_WAVE_DUMP") else {
        return;
    };
    let (h, _) = open();
    let ws = h.state().workspace().unwrap();
    for (i, s) in ws.project.surfaces.iter().enumerate() {
        let name = s
            .template
            .as_ref()
            .map_or("none", |t| t.texture_id.as_str());
        blue_wave::render(ws, i, 1024)
            .save(std::path::Path::new(&dir).join(format!("{i}-{name}.png")))
            .unwrap();
    }
}

#[test]
fn the_layout_boxes_stay_inside_their_zone() {
    for r in LAYOUT {
        let [x, y, w, h] = r.at;
        assert!(
            x >= 0.0
                && y >= 0.0
                && w > 0.0
                && h > 0.0
                && x + w <= 1.0 + 1e-9
                && y + h <= 1.0 + 1e-9,
            "{} {} #{}: box {:?} leaves the zone",
            r.texture,
            r.zone,
            r.nth,
            r.at
        );
    }
}

#[test]
fn every_row_names_a_zone_of_a_texture_of_the_fleet() {
    let (h, _) = open();
    let ws = h.state().workspace().unwrap();
    for r in LAYOUT {
        let surface = blue_wave::surface_of(ws, r.texture)
            .unwrap_or_else(|| panic!("no texture {}", r.texture));
        assert!(
            blue_wave::zone(ws, surface, r.zone, r.nth).is_some(),
            "{} has no zone {} #{}",
            r.texture,
            r.zone,
            r.nth
        );
    }
}

#[test]
fn the_palette_holds_the_three_colors() {
    let (h, livery) = open();
    let p = &h.state().workspace().unwrap().project;
    let names: Vec<_> = livery
        .swatches
        .iter()
        .map(|id| p.swatch(*id).unwrap().name.as_str())
        .collect();
    assert_eq!(names, ["White", "Light blue", "Blue"]);
    assert_eq!(p.palette.len(), 3);
}

#[test]
fn every_color_is_a_swatch() {
    let (h, _) = open();
    let p = &h.state().workspace().unwrap().project;
    for s in &p.surfaces {
        let mut objects = Vec::new();
        all(&s.objects, &mut objects);
        for o in objects {
            if matches!(
                o.kind,
                ShapeKind::Group | ShapeKind::Image { .. } | ShapeKind::Instance { .. }
            ) {
                continue;
            }
            assert!(
                o.fill_swatch.is_some(),
                "{} on {} has an off-palette fill",
                o.name,
                s.name
            );
        }
    }
}

#[test]
fn the_logo_is_on_the_truck_and_on_the_trailer() {
    let (h, livery) = open();
    let p = &h.state().workspace().unwrap().project;
    let on = |texture: &str| {
        let i = p
            .surfaces
            .iter()
            .position(|s| s.template.as_ref().is_some_and(|t| t.texture_id == texture))
            .unwrap();
        let mut objects = Vec::new();
        all(&p.surfaces[i].objects, &mut objects);
        objects
            .iter()
            .filter(
                |o| matches!(o.kind, ShapeKind::Instance { symbol, .. } if symbol == livery.logo),
            )
            .count()
    };
    assert_eq!(on("standard"), 3, "two doors and the rear wall");
    assert_eq!(on("body_13_6"), 3, "two sides and the rear doors");
}

#[test]
fn the_styles_are_followed() {
    let (h, livery) = open();
    let p = &h.state().workspace().unwrap().project;
    for id in [livery.white_base, livery.blue_panel] {
        assert!(p.graphic_style(id).is_some());
    }
    assert!(p.text_style(livery.tagline.unwrap()).is_some());
}

#[test]
fn painted_textures_are_not_blank_and_the_others_are() {
    let (h, _) = open();
    let ws = h.state().workspace().unwrap();
    let blank = |i: usize| blue_wave::render(ws, i, 128).pixels().all(|p| p.0[3] == 0);
    for (i, s) in ws.project.surfaces.iter().enumerate() {
        let id = s.template.as_ref().unwrap().texture_id.as_str();
        let listed = LAYOUT.iter().any(|r| r.texture == id);
        assert_eq!(blank(i), !listed, "texture {id}");
    }
}

#[test]
fn every_listed_zone_is_painted() {
    const SIDE: u32 = 512;
    let (h, _) = open();
    let ws = h.state().workspace().unwrap();
    let mut done = std::collections::HashSet::new();
    for r in LAYOUT {
        if !done.insert((r.texture, r.zone, r.nth)) {
            continue;
        }
        let surface = blue_wave::surface_of(ws, r.texture).unwrap();
        let zone = blue_wave::zone(ws, surface, r.zone, r.nth).unwrap();
        let k = f64::from(SIDE) / ws.project.surfaces[surface].size.width;
        let image = blue_wave::render(ws, surface, SIDE);
        let painted = (zone.x0 * k).ceil() as u32..(zone.x1 * k).floor() as u32;
        let rows = (zone.y0 * k).ceil() as u32..(zone.y1 * k).floor() as u32;
        let any = rows
            .clone()
            .any(|y| painted.clone().any(|x| image.get_pixel(x, y).0[3] > 0));
        assert!(any, "{} {} #{} is not painted", r.texture, r.zone, r.nth);
    }
}

#[test]
fn the_side_view_draws_the_cab_with_the_livery() {
    use tp_core::document::Rgba;
    let (mut h, _) = open();
    let standard = {
        let ws = h.state_mut().workspace_mut().unwrap();
        let standard = blue_wave::surface_of(ws, "standard").unwrap();
        ws.set_active_surface(standard);
        standard
    };
    h.state_mut().prefs.layout.side_view = true;
    common::settle_renders(&mut h);
    let ws = h.state().workspace().unwrap();
    assert_eq!(ws.side_view.drawn(), Some(standard), "the cab is drawn");
    // The left door is white (the livery), the rest of the side the plain surface.
    let door = blue_wave::zone(ws, standard, "DOOR", 0).unwrap();
    let at = |x: f64, y: f64| {
        ws.side_view
            .color_at(standard, [(x / 4096.0) as f32, (y / 4096.0) as f32])
    };
    assert_eq!(
        at(door.center().x, door.y0 + 20.0),
        Some(Rgba::rgb(255, 255, 255))
    );
    assert_ne!(at(200.0, 1500.0), Some(Rgba::rgb(255, 255, 255)));
    assert!(at(200.0, 1500.0).is_some_and(|c| c.a == 255), "never blank");
}
