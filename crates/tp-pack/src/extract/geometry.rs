use std::collections::{BTreeMap, BTreeSet};

use tp_scs::{Descriptor, Locator, Look, Model};

/// Attachment points by name.
pub type Locators = BTreeMap<String, Vec<Locator>>;

#[derive(Debug, Clone)]
pub struct Mesh {
    /// One per vertex, in meters as stored by the game.
    pub positions: Vec<[f32; 3]>,
    pub uv: Vec<[f64; 2]>,
    pub triangles: Vec<[usize; 3]>,
    pub shader: String,
    pub alt_uv: bool,
}

fn paint_shader(effect: &str) -> Result<Option<bool>, String> {
    // Alpha-test suffixes change opacity, not paint UV bindings.
    let family = effect
        .strip_suffix(".asafew.a")
        .or_else(|| effect.strip_suffix(".a"))
        .unwrap_or(effect);
    let Some(flavors) = family.strip_prefix("eut2.truckpaint") else {
        return Ok(None);
    };
    let mut groups = BTreeSet::new();
    if !flavors.is_empty() {
        let Some(flavors) = flavors.strip_prefix('.') else {
            return Err(format!("unsupported paint shader {effect}"));
        };
        for flavor in flavors.split('.') {
            let group = match flavor {
                "tsnmap" | "tsnmapuv" => "normal",
                "airbrush" | "flipflake" | "altuv" => flavor,
                _ => return Err(format!("unsupported paint shader {effect}")),
            };
            if !groups.insert(group) {
                return Err(format!("unsupported paint shader {effect}"));
            }
        }
    }
    Ok(Some(groups.contains("altuv")))
}

fn only<T>(items: impl IntoIterator<Item = T>, problem: &str) -> Result<T, String> {
    let mut items = items.into_iter();
    match (items.next(), items.next()) {
        (Some(item), None) => Ok(item),
        _ => Err(problem.to_owned()),
    }
}

/// The one item named `name` (any case); some game models repeat a look or variant with
/// the same content (Schwarzmüller Food Tank cage), which counts as one.
fn selected<T: PartialEq>(
    kind: &str,
    name: &str,
    items: impl IntoIterator<Item = Result<T, String>>,
) -> Result<T, String> {
    let mut matches = Vec::new();
    for item in items {
        let item = item?;
        if matches.last() != Some(&item) {
            matches.push(item);
        }
    }
    only(matches, &format!("missing or ambiguous {kind}: {name}"))
}

/// The parts of `model` the variant `variant` shows, by index.
fn visible_parts(
    model: &Model,
    descriptor: &Descriptor,
    variant: &str,
) -> Result<Vec<usize>, String> {
    let variant = selected(
        "Variant",
        variant,
        descriptor
            .variants
            .iter()
            .filter(|v| v.name.eq_ignore_ascii_case(variant))
            .map(|v| Ok(&v.parts)),
    )?;
    if variant.len() != model.parts.len() {
        return Err("variant and model have different part counts".into());
    }
    let mut shown = Vec::new();
    for (index, attributes) in variant.iter().enumerate() {
        let (_, visible) = only(
            attributes.iter().filter(|(tag, _)| tag == "visible"),
            "missing or ambiguous part visibility",
        )?;
        match visible {
            1 => shown.push(index),
            0 => {}
            _ => return Err("invalid part visibility".into()),
        }
    }
    Ok(shown)
}

/// The attachment points of the parts the variant shows, by name.
pub fn locators(model: &Model, descriptor: &Descriptor, variant: &str) -> Result<Locators, String> {
    let mut result = Locators::new();
    for part in visible_parts(model, descriptor, variant)? {
        for l in &model.parts[part].locators {
            result.entry(l.name.clone()).or_default().push(l.clone());
        }
    }
    Ok(result)
}

/// The painted meshes of one look and variant of a model. `effects` gives the effect of
/// each material slot of a look.
pub fn painted(
    model: &Model,
    descriptor: &Descriptor,
    mut effects: impl FnMut(&Look) -> Result<Vec<String>, String>,
    look: &str,
    variant: &str,
    alt_uv: Option<bool>,
) -> Result<Vec<Mesh>, String> {
    let effects = selected(
        "Look",
        look,
        descriptor
            .looks
            .iter()
            .filter(|l| l.name.eq_ignore_ascii_case(look))
            .map(&mut effects),
    )?;
    let mut allowed = BTreeSet::new();
    for part in visible_parts(model, descriptor, variant)? {
        allowed.extend(model.parts[part].pieces.iter().copied());
    }
    let mut result = Vec::new();
    for (index, piece) in model.pieces.iter().enumerate() {
        if !allowed.remove(&index) {
            continue;
        }
        let effect = effects
            .get(piece.material)
            .ok_or("material index out of bounds")?;
        let Some(source_alt_uv) = paint_shader(effect)? else {
            continue;
        };
        let alt_uv = alt_uv.unwrap_or(source_alt_uv);
        let alias = if alt_uv { 2 } else { 1 };
        let stream = only(
            piece.streams.iter().filter(|s| s.aliases.contains(&alias)),
            &format!("missing or ambiguous paint UV alias _TEXCOORD{alias}"),
        )?;
        let uv = stream
            .uv
            .iter()
            .map(|uv| uv.map(f64::from))
            .collect::<Vec<_>>();
        let triangles = piece
            .triangles
            .iter()
            .map(|t| t.map(|i| i as usize))
            .collect::<Vec<_>>();
        // Finite overflow is valid; coordinates must still fit SVG edge quantization.
        if uv
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() >= i64::MAX as f64 / 10_000_000.0)
        {
            return Err("invalid paint UV coordinate".into());
        }
        if triangles.iter().flatten().any(|i| *i >= uv.len()) {
            return Err("triangle index out of bounds".into());
        }
        if piece.positions.len() != uv.len() {
            return Err("positions and paint UVs differ in count".into());
        }
        result.push(Mesh {
            positions: piece.positions.clone(),
            uv,
            triangles,
            shader: effect.clone(),
            alt_uv,
        });
    }
    if !allowed.is_empty() {
        return Err("visible part references a missing piece".into());
    }
    Ok(result)
}

/// The preview pieces of the meshes painted by texture `texture`: positions rounded to 1 mm.
pub fn preview_pieces(meshes: &[Mesh], texture: &str) -> Vec<tp_vehicles::PreviewPiece> {
    meshes
        .iter()
        .map(|m| tp_vehicles::PreviewPiece {
            texture: texture.to_owned(),
            positions: m.positions.iter().map(|p| p.map(millimeter)).collect(),
            uv: m.uv.iter().map(|uv| uv.map(|v| v as f32)).collect(),
            triangles: m.triangles.iter().map(|t| t.map(|i| i as u32)).collect(),
        })
        .collect()
}

/// A locator as the preview file stores it: position rounded to 1 mm.
pub fn preview_locator(l: &Locator) -> tp_vehicles::PreviewLocator {
    tp_vehicles::PreviewLocator {
        position: l.position.map(millimeter),
        rotation: l.rotation,
        scale: l.scale,
    }
}

fn millimeter(meters: f32) -> f32 {
    (meters * 1000.0).round() / 1000.0
}

/// A template with no layout, for a texture that only carries a base colour.
pub fn blank_svg([width, height]: [u32; 2]) -> Vec<u8> {
    format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\"/>\n").into_bytes()
}

pub fn svg(meshes: &[Mesh], size: [u32; 2]) -> Result<(Vec<u8>, usize), String> {
    let mut edges = BTreeSet::new();
    for mesh in meshes {
        for triangle in &mesh.triangles {
            let points = triangle.map(|i| mesh.uv[i].map(|v| (v * 10_000_000.0).round() as i64));
            for (a, b) in [
                (points[0], points[1]),
                (points[1], points[2]),
                (points[2], points[0]),
            ] {
                if a != b {
                    edges.insert(if a < b { (a, b) } else { (b, a) });
                }
            }
        }
    }
    if edges.is_empty() {
        return Err("empty paint template".into());
    }
    let [width, height] = size;
    let lines = edges
        .iter()
        .map(|(a, b)| {
            format!(
                "M{:.3},{:.3}L{:.3},{:.3}",
                a[0] as f64 / 10_000_000.0 * width as f64,
                a[1] as f64 / 10_000_000.0 * height as f64,
                b[0] as f64 / 10_000_000.0 * width as f64,
                b[1] as f64 / 10_000_000.0 * height as f64
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok((format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n<path fill=\"none\" stroke=\"#444\" stroke-width=\"0.6\" d=\"{lines}\"/>\n</svg>\n").into_bytes(), edges.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tp_scs::{Part, Stream, Variant};

    const EFFECT: &str = "eut2.truckpaint.tsnmapuv";
    const CORNERS: [[f32; 2]; 3] = [[0., 0.], [1., 0.], [0., 1.]];
    const INNER: [[f32; 2]; 3] = [[0.25, 0.25], [0.75, 0.25], [0.25, 0.75]];

    fn stream(aliases: &[u32], uv: [[f32; 2]; 3]) -> Stream {
        Stream {
            aliases: aliases.to_vec(),
            uv: uv.to_vec(),
        }
    }

    fn model(streams: Vec<Stream>) -> Model {
        Model {
            pieces: vec![tp_scs::Piece {
                material: 0,
                positions: vec![[0.; 3]; 3],
                streams,
                triangles: vec![[0, 1, 2]],
            }],
            parts: vec![Part {
                name: "body".into(),
                pieces: vec![0],
                locators: vec![],
            }],
        }
    }

    fn descriptor(visible: i32) -> Descriptor {
        Descriptor {
            looks: vec![Look {
                name: "default".into(),
                materials: vec![],
            }],
            variants: vec![Variant {
                name: "default".into(),
                parts: vec![vec![("visible".into(), visible)]],
            }],
        }
    }

    fn paint(
        model: &Model,
        descriptor: &Descriptor,
        effect: &str,
        forced: Option<bool>,
    ) -> Result<Vec<Mesh>, String> {
        painted(
            model,
            descriptor,
            |_| Ok(vec![effect.to_owned()]),
            "Default",
            "Default",
            forced,
        )
    }

    /// The set `_TEXCOORD1` aliases is not the first one: the alias decides, not the position.
    fn aliased() -> Model {
        model(vec![stream(&[3], INNER), stream(&[1, 2], CORNERS)])
    }

    #[test]
    fn semantic_alias_visibility_and_image_coordinates() {
        let meshes = paint(&aliased(), &descriptor(1), EFFECT, None).unwrap();
        assert_eq!(meshes[0].uv, [[0., 0.], [1., 0.], [0., 1.]]);
        let (svg, edges) = svg(&meshes, [2048, 1024]).unwrap();
        assert_eq!(edges, 3);
        assert!(String::from_utf8(svg).unwrap().contains("0.000,1024.000"));
        assert!(
            paint(&aliased(), &descriptor(0), EFFECT, None)
                .unwrap()
                .is_empty()
        );
        // A piece list that is empty paints nothing.
        let mut empty = aliased();
        empty.parts[0].pieces.clear();
        assert!(
            paint(&empty, &descriptor(1), EFFECT, None)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn identical_repeated_looks_are_one_look() {
        let look = |effect: &str| Look {
            name: "default".into(),
            materials: vec![effect.into()],
        };
        let run = |looks: Vec<Look>| {
            let mut descriptor = descriptor(1);
            descriptor.looks = looks;
            painted(
                &aliased(),
                &descriptor,
                |look| Ok(look.materials.clone()),
                "default",
                "default",
                None,
            )
        };
        assert_eq!(run(vec![look(EFFECT), look(EFFECT)]).unwrap().len(), 1);
        let error = run(vec![look(EFFECT), look("eut2.dif")]).unwrap_err();
        assert_eq!(error, "missing or ambiguous Look: default");
    }

    #[test]
    fn shader_families_and_export_mode_select_the_paint_uv() {
        let model = model(vec![stream(&[1], CORNERS), stream(&[2], INNER)]);
        for normal in ["", ".tsnmap", ".tsnmapuv"] {
            for mode in ["", ".airbrush"] {
                for (alternate, alpha, metallic) in [false, true]
                    .into_iter()
                    .flat_map(|alt| ["", ".a", ".asafew.a"].map(|alpha| (alt, alpha)))
                    .flat_map(|(alt, alpha)| {
                        ["", ".flipflake"].map(|metallic| (alt, alpha, metallic))
                    })
                {
                    let effect = format!(
                        "eut2.truckpaint{normal}{mode}{metallic}{}{alpha}",
                        if alternate { ".altuv" } else { "" }
                    );
                    for forced in [None, Some(false), Some(true)] {
                        let meshes = paint(&model, &descriptor(1), &effect, forced).unwrap();
                        let alt_uv = forced.unwrap_or(alternate);
                        assert_eq!(meshes[0].alt_uv, alt_uv);
                        assert_eq!(meshes[0].shader, effect);
                        assert_eq!(
                            meshes[0].uv[0],
                            if alt_uv { [0.25, 0.25] } else { [0., 0.] }
                        );
                    }
                }
            }
        }
        let no_alternate = self::model(vec![stream(&[1], CORNERS), stream(&[3], INNER)]);
        assert!(paint(&no_alternate, &descriptor(1), EFFECT, Some(true)).is_err());
        let duplicate = self::model(vec![stream(&[2], CORNERS), stream(&[2], INNER)]);
        assert!(paint(&duplicate, &descriptor(1), EFFECT, Some(true)).is_err());
        for effect in [
            "eut2.truckpaint.unknown",
            "eut2.truckpaint.tsnmap.tsnmapuv",
            "eut2.truckpaint.altuv.altuv",
            "eut2.truckpaint.flipflake.flipflake",
            "eut2.truckpaint_extra",
        ] {
            assert!(
                paint(&model, &descriptor(1), effect, None).is_err(),
                "{effect}"
            );
        }
        // Other effects are not painted at all.
        assert!(
            paint(&model, &descriptor(1), "eut2.dif", None)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn finite_uv_overflow_preserves_coordinates_and_selected_channel() {
        let mut uv = CORNERS;
        uv[0] = [-2.125, 1.75];
        let mesh = paint(&model(vec![stream(&[1], uv)]), &descriptor(1), EFFECT, None).unwrap();
        assert_eq!(mesh[0].uv[0], [-2.125, 1.75]);
        assert!(!mesh[0].alt_uv);
        let (bytes, _) = svg(&mesh, [4096, 4096]).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("-8704.000,7168.000"));
        assert!(text.contains("viewBox=\"0 0 4096 4096\""));
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1e30] {
            let mut uv = CORNERS;
            uv[1][0] = value;
            assert!(
                paint(&model(vec![stream(&[1], uv)]), &descriptor(1), EFFECT, None).is_err(),
                "{value}"
            );
        }
    }

    #[test]
    #[ignore = "needs the game and its reports: TRUCKPAINT_BASELINE_REPORTS"]
    fn replay_validated_game_templates() {
        use serde_json::Value;
        use std::{fs, io::Read};

        let reports = std::env::var_os("TRUCKPAINT_BASELINE_REPORTS")
            .expect("set TRUCKPAINT_BASELINE_REPORTS");
        for report_path in std::env::split_paths(&reports) {
            let report: Value = serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
            // The models are read from the installation the report names.
            let sources =
                super::super::Sources::new(report["installation"].as_str().unwrap().as_ref())
                    .unwrap();
            for result in report["results"].as_array().unwrap() {
                if result["status"] != "success" {
                    continue;
                }
                let package_path = report_path
                    .parent()
                    .unwrap()
                    .join(result["package"].as_str().unwrap());
                let mut archive =
                    zip::ZipArchive::new(fs::File::open(package_path).unwrap()).unwrap();
                let manifest: Value =
                    serde_json::from_reader(archive.by_name("vehicle.json").unwrap()).unwrap();
                for texture in result["textures"].as_array().unwrap() {
                    // A plain base-colour texture of a trailer has no layout to replay.
                    if texture["edges"] == 0 {
                        continue;
                    }
                    let mut meshes = Vec::new();
                    for contributor in texture["contributors"].as_array().unwrap() {
                        for selection in contributor["selections"].as_array().unwrap() {
                            meshes.extend(
                                super::super::model_geometry(
                                    &sources,
                                    selection["model"].as_str().unwrap(),
                                    selection["look"].as_str().unwrap(),
                                    selection["variant"].as_str().unwrap(),
                                    Some(manifest["game"]["alt_uv"].as_bool().unwrap()),
                                )
                                .unwrap()
                                .0,
                            );
                        }
                    }
                    let size = serde_json::from_value(texture["size"].clone()).unwrap();
                    let (actual, edges) = svg(&meshes, size).unwrap();
                    let id = texture["part"].as_str().unwrap();
                    let mut expected = Vec::new();
                    archive
                        .by_name(&format!("templates/{id}.svg"))
                        .unwrap()
                        .read_to_end(&mut expected)
                        .unwrap();
                    assert!(
                        actual == expected,
                        "SVG changed: {} / {id}",
                        result["vehicle"]
                    );
                    assert_eq!(edges as u64, texture["edges"].as_u64().unwrap());
                }
            }
        }
    }

    #[test]
    fn ambiguity_unsupported_shaders_and_bad_geometry_fail() {
        let good = aliased();
        let with = |change: &dyn Fn(&mut Model)| {
            let mut model = good.clone();
            change(&mut model);
            paint(&model, &descriptor(1), EFFECT, None)
        };
        assert!(paint(&good, &descriptor(1), "eut2.truckpaint.unknown", None).is_err());
        // A triangle past the last vertex, no paint UV set, two of them, a missing material.
        assert!(with(&|m| m.pieces[0].triangles[0] = [0, 1, 3]).is_err());
        assert!(with(&|m| m.pieces[0].streams[1].aliases = vec![3]).is_err());
        assert!(with(&|m| m.pieces[0].streams[0].aliases = vec![1]).is_err());
        assert!(with(&|m| m.pieces[0].material = 4).is_err());
        // A visible part naming a piece the model lacks, and an unreadable visibility.
        assert!(with(&|m| m.parts[0].pieces.push(5)).is_err());
        assert!(paint(&good, &descriptor(2), EFFECT, None).is_err());
        let mut hidden_twice = descriptor(1);
        hidden_twice.variants[0].parts[0].push(("visible".into(), 0));
        assert!(paint(&good, &hidden_twice, EFFECT, None).is_err());
        assert!(svg(&[], [4096, 4096]).is_err());
    }
}
