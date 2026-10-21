//! The preview meshes of the committed sample vehicles stay on the panels of their templates.

mod common;

use common::blue_wave::zones;
use tp_core::kurbo::{Rect, Size};
use tp_vehicles::{Package, PreviewMesh};

const EXAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/vehicles");

/// The labeled panels of a template: its rectangles, and the bounding box of its polygons
/// (a cab side is a polygon).
fn panels(svg: &str, size: Size) -> Vec<Rect> {
    let mut out: Vec<Rect> = zones(svg, size).into_iter().map(|z| z.rect).collect();
    for tag in svg.split("<polygon ").skip(1) {
        let points = tag
            .split("points=\"")
            .nth(1)
            .and_then(|t| t.split('"').next());
        let numbers: Vec<f64> = points
            .into_iter()
            .flat_map(|p| p.split([' ', ',']))
            .filter_map(|n| n.parse().ok())
            .collect();
        let (xs, ys): (Vec<_>, Vec<_>) = numbers.chunks(2).map(|p| (p[0], p[1])).unzip();
        let fold = |v: &[f64], f: fn(f64, f64) -> f64, init: f64| v.iter().copied().fold(init, f);
        out.push(Rect::new(
            fold(&xs, f64::min, f64::MAX),
            fold(&ys, f64::min, f64::MAX),
            fold(&xs, f64::max, f64::MIN),
            fold(&ys, f64::max, f64::MIN),
        ));
    }
    out
}

/// The triangles of `mesh` whose three corners do not lie in one panel of `svg`.
fn off_panel(mesh: &PreviewMesh, svg: &str, size: Size) -> usize {
    let panels = panels(svg, size);
    let inside = |panel: &Rect, uv: [f32; 2]| {
        let (x, y) = (
            f64::from(uv[0]) * size.width,
            f64::from(uv[1]) * size.height,
        );
        (panel.x0 - 0.5..=panel.x1 + 0.5).contains(&x)
            && (panel.y0 - 0.5..=panel.y1 + 0.5).contains(&y)
    };
    mesh.pieces
        .iter()
        .flat_map(|piece| piece.triangles.iter().map(move |t| (piece, t)))
        .filter(|(piece, t)| {
            !panels
                .iter()
                .any(|panel| t.iter().all(|i| inside(panel, piece.uv[*i as usize])))
        })
        .count()
}

fn samples() -> Vec<(String, Package)> {
    let mut out = Vec::new();
    for file in std::fs::read_dir(EXAMPLES).unwrap().flatten() {
        let path = file.path();
        if path.extension().is_some_and(|e| e == "tpv") {
            let package = Package::read(&std::fs::read(&path).unwrap()).unwrap();
            out.push((file.file_name().to_string_lossy().into_owned(), package));
        }
    }
    out
}

#[test]
fn every_face_of_every_sample_mesh_lies_on_a_panel_of_its_template() {
    let samples = samples();
    assert_eq!(samples.len(), 3);
    for (name, package) in &samples {
        for part in &package.manifest.paint_job.main {
            let mesh = package
                .preview(&part.id)
                .unwrap_or_else(|| panic!("{name}: {} has no preview mesh", part.name));
            let svg = std::str::from_utf8(&package.template(&part.id).unwrap().bytes).unwrap();
            let size = Size::new(
                f64::from(part.texture.size.width),
                f64::from(part.texture.size.height),
            );
            assert_eq!(
                off_panel(mesh, svg, size),
                0,
                "{name}: the preview mesh of {} leaves the panels of its template",
                part.name
            );
        }
    }
}

#[test]
fn moving_a_panel_of_the_standard_cab_is_noticed() {
    let (_, package) = samples()
        .into_iter()
        .find(|(name, _)| name.contains("sample_truck-1.1.0"))
        .unwrap();
    let svg = std::str::from_utf8(&package.template("standard").unwrap().bytes)
        .unwrap()
        .to_owned();
    let mesh = package.preview("standard").unwrap();
    let size = Size::new(4096.0, 4096.0);
    assert_eq!(off_panel(mesh, &svg, size), 0);
    // The FRONT panel slides down by 100 px.
    let moved = svg.replace(
        "<rect x=\"120.0\" y=\"1760.0\" width=\"1900.0\" height=\"1500.0\"",
        "<rect x=\"120.0\" y=\"1860.0\" width=\"1900.0\" height=\"1500.0\"",
    );
    assert_ne!(moved, svg);
    assert!(off_panel(mesh, &moved, size) > 0);
}
