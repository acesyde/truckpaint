//! The preview meshes of the committed sample vehicles stay on the panels of their templates.

mod common;

use common::blue_wave::zones;
use tp_core::kurbo::{Rect, Size};
use tp_vehicles::{Package, PreviewMesh, PreviewPiece};

const EXAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/vehicles");

/// The labeled panels of a template: its rectangles, and the bounding box of its polygons
/// (a cab side is a polygon).
fn panels(svg: &str, size: Size) -> Vec<Rect> {
    zones(svg, size).into_iter().map(|z| z.rect).collect()
}

/// The triangles of `pieces` whose three corners do not lie in one panel of the template of the
/// texture that paints them.
fn off_panel(pieces: &[&PreviewPiece], package: &Package) -> usize {
    pieces
        .iter()
        .map(|piece| {
            let (_, part) = package.manifest.paint_job.part(&piece.texture).unwrap();
            let size = Size::new(
                f64::from(part.texture.size.width),
                f64::from(part.texture.size.height),
            );
            let svg = std::str::from_utf8(&package.template(&piece.texture).unwrap().bytes)
                .unwrap()
                .to_owned();
            off_panel_in(piece, &svg, size)
        })
        .sum()
}

/// Every piece of a preview mesh: the always drawn ones and those of every option of every slot.
fn all_pieces(mesh: &PreviewMesh) -> Vec<&PreviewPiece> {
    mesh.pieces
        .iter()
        .chain(
            mesh.slots
                .iter()
                .flat_map(|s| &s.options)
                .flat_map(|o| &o.pieces),
        )
        .collect()
}

/// The triangles of `piece` that lie in no panel of `svg`.
fn off_panel_in(piece: &PreviewPiece, svg: &str, size: Size) -> usize {
    let panels = panels(svg, size);
    let inside = |panel: &Rect, uv: [f32; 2]| {
        let (x, y) = (
            f64::from(uv[0]) * size.width,
            f64::from(uv[1]) * size.height,
        );
        (panel.x0 - 0.5..=panel.x1 + 0.5).contains(&x)
            && (panel.y0 - 0.5..=panel.y1 + 0.5).contains(&y)
    };
    piece
        .triangles
        .iter()
        .filter(|t| {
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
            assert_eq!(
                off_panel(&all_pieces(mesh), package),
                0,
                "{name}: the preview mesh of {} leaves the panels of the template of the texture that paints it",
                part.name
            );
        }
    }
}

#[test]
fn the_samples_carry_their_accessories() {
    let samples = samples();
    let find = |name: &str| samples.iter().find(|(n, _)| n.contains(name)).unwrap();
    // The side skirts are the sample truck's 1.1.0 slot, on both cabins, fitted by default.
    for cab in ["standard", "high_roof"] {
        let mesh = find("sample_truck-1.1.0").1.preview(cab).unwrap();
        let skirts = mesh.slot("sideskirt").unwrap();
        assert!(skirts.options[0].default);
        assert_eq!(skirts.options[0].pieces[0].texture, "side_skirts");
    }
    assert!(
        find("sample_truck-1.0.0")
            .1
            .preview("standard")
            .unwrap()
            .slots
            .is_empty()
    );
    // The trailer's curtain bodies hang on its body locator; the long one is the default.
    let mesh = find("sample_trailer").1.preview("base").unwrap();
    let body = mesh.slot("body").unwrap();
    let names: Vec<_> = body
        .options
        .iter()
        .map(|o| (o.name.as_str(), o.default))
        .collect();
    assert_eq!(
        names,
        [
            ("Curtain body 13.6 m", true),
            ("Curtain body 10.5 m", false)
        ]
    );
    let length = |k: usize| {
        let z: Vec<f32> = body.options[k]
            .pieces
            .iter()
            .flat_map(|p| p.positions.iter().map(|p| p[2]))
            .collect();
        z.iter().copied().fold(f32::MIN, f32::max) - z.iter().copied().fold(f32::MAX, f32::min)
    };
    assert!((length(0) - 13.62).abs() < 0.05 && (length(1) - 10.52).abs() < 0.05);
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
    let cab: Vec<&PreviewPiece> = mesh.pieces.iter().collect();
    let off = |svg: &str| -> usize { cab.iter().map(|p| off_panel_in(p, svg, size)).sum() };
    assert_eq!(off(&svg), 0);
    // The FRONT panel slides down by 100 px.
    let moved = svg.replace(
        "<rect x=\"120.0\" y=\"1760.0\" width=\"1900.0\" height=\"1500.0\"",
        "<rect x=\"120.0\" y=\"1860.0\" width=\"1900.0\" height=\"1500.0\"",
    );
    assert_ne!(moved, svg);
    assert!(off(&moved) > 0);
    // The same goes for a panel of the Side skirts template, which only an accessory piece uses.
    let skirts = std::str::from_utf8(&package.template("side_skirts").unwrap().bytes)
        .unwrap()
        .to_owned();
    let option = &mesh.slot("sideskirt").unwrap().options[0];
    let option: Vec<&PreviewPiece> = option.pieces.iter().collect();
    let size = Size::new(1024.0, 1024.0);
    let off = |svg: &str| -> usize { option.iter().map(|p| off_panel_in(p, svg, size)).sum() };
    assert_eq!(off(&skirts), 0);
    assert!(off(&skirts.replace("40.0,75.0 984.0,75.0", "40.0,175.0 984.0,175.0")) > 0);
}
