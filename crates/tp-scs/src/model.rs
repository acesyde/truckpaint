//! Vehicle models: geometry (`.pmg`), descriptor (`.pmd`) and the effect of a material (`.mat`).
//! Only what painted surfaces and their preview need is read; skeletons, normals and weights are skipped.

const PMG_HEADER: usize = 112;
const PMG_PART: usize = 24;
const PMG_LOCATOR: u64 = 44;
const PMG_PIECE: usize = 100;
const PMD_HEADER: usize = 64;

/// A texture-coordinate set of a piece.
#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    /// The `n` of each `_TEXCOORDn` alias this set carries.
    pub aliases: Vec<u32>,
    pub uv: Vec<[f32; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub material: usize,
    /// One `[x, y, z]` per vertex, in meters.
    pub positions: Vec<[f32; 3]>,
    pub streams: Vec<Stream>,
    pub triangles: Vec<[u32; 3]>,
}

/// An attachment point: where an accessory of the same name is mounted.
#[derive(Debug, Clone, PartialEq)]
pub struct Locator {
    pub name: String,
    pub position: [f32; 3],
    pub scale: f32,
    /// A unit quaternion `[w, x, y, z]`, as stored (the game applies its inverse; see `tp-vehicles`).
    pub rotation: [f32; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub name: String,
    pub pieces: Vec<usize>,
    pub locators: Vec<Locator>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub pieces: Vec<Piece>,
    pub parts: Vec<Part>,
}

/// A look: its name and the material file of each material slot.
#[derive(Debug, Clone, PartialEq)]
pub struct Look {
    pub name: String,
    pub materials: Vec<String>,
}

/// A variant: for each model part, its `(attribute name, integer value)` pairs.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub parts: Vec<Vec<(String, i32)>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Descriptor {
    pub looks: Vec<Look>,
    pub variants: Vec<Variant>,
}

/// Bounds-checked little-endian reads; every failure names the file.
struct Bytes<'a> {
    path: &'a str,
    data: &'a [u8],
}

impl Bytes<'_> {
    fn fail(&self, cause: &str) -> String {
        format!("{}: {cause}", self.path)
    }

    fn slice(&self, at: u64, size: u64) -> Result<&[u8], String> {
        let end = at
            .checked_add(size)
            .filter(|end| *end <= self.data.len() as u64);
        end.map(|end| &self.data[at as usize..end as usize])
            .ok_or_else(|| self.fail("data lies outside the file"))
    }

    fn u32(&self, at: u64) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.slice(at, 4)?.try_into().unwrap()))
    }

    fn i32(&self, at: u64) -> Result<i32, String> {
        Ok(self.u32(at)? as i32)
    }

    fn u64(&self, at: u64) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.slice(at, 8)?.try_into().unwrap()))
    }

    /// A non-negative count or offset.
    fn index(&self, at: u64) -> Result<u64, String> {
        u64::try_from(self.i32(at)?)
            .map_err(|_| self.fail(&format!("negative count or offset at byte {at}")))
    }

    /// The text at `at` up to its terminator.
    fn text(&self, at: u64) -> Result<String, String> {
        let rest = usize::try_from(at)
            .ok()
            .and_then(|at| self.data.get(at..))
            .ok_or_else(|| self.fail("data lies outside the file"))?;
        let end = rest.iter().position(|b| *b == 0).unwrap_or(rest.len());
        Ok(String::from_utf8_lossy(&rest[..end]).into_owned())
    }
}

/// The name a 64-bit token packs: base-38 digits, lowest first, ending at the first zero digit.
fn token(mut value: u64) -> String {
    const LETTERS: &[u8; 38] = b"\x000123456789abcdefghijklmnopqrstuvwxyz_";
    let mut name = String::new();
    while value != 0 {
        let digit = LETTERS[(value % 38) as usize];
        if digit == 0 {
            break;
        }
        name.push(digit as char);
        value /= 38;
    }
    name
}

impl Model {
    /// Reads a `.pmg`; only version 0x15 exists in the installed game.
    pub fn parse(path: &str, data: &[u8]) -> Result<Self, String> {
        let b = Bytes { path, data };
        let signature = data
            .get(..4)
            .ok_or_else(|| b.fail("geometry file is malformed"))?;
        if signature[1..] != *b"gmP" || signature[0] != 0x15 {
            return Err(b.fail(&format!(
                "geometry version {:#04x} is not supported yet (only 0x15)",
                signature[0]
            )));
        }
        if data.len() < PMG_HEADER {
            return Err(b.fail("geometry file is malformed"));
        }
        let (piece_count, part_count) = (b.index(4)?, b.index(8)?);
        let (parts_at, pieces_at) = (b.index(76)?, b.index(84)?);
        let (locator_count, locators_at) = (b.index(20)?, b.index(80)?);
        let mut parts = Vec::new();
        for i in 0..part_count {
            let at = parts_at + i * PMG_PART as u64;
            let count = b.index(at + 8)?;
            if count > piece_count {
                return Err(b.fail("part lists more pieces than the model has"));
            }
            // An empty part may carry any first index.
            let first = if count == 0 { 0 } else { b.index(at + 12)? };
            let (locators, first_locator) = (b.index(at + 16)?, b.index(at + 20)?);
            if locators > 0 && first_locator + locators > locator_count {
                return Err(b.fail("part lists more locators than the model has"));
            }
            parts.push(Part {
                name: token(b.u64(at)?),
                pieces: (first..first + count).map(|p| p as usize).collect(),
                locators: (first_locator..first_locator + locators)
                    .map(|l| Self::locator(&b, locators_at + l * PMG_LOCATOR))
                    .collect::<Result<_, _>>()?,
            });
        }
        let mut pieces = Vec::new();
        for i in 0..piece_count {
            pieces.push(Self::piece(&b, pieces_at + i * PMG_PIECE as u64)?);
        }
        Ok(Self { pieces, parts })
    }

    fn locator(b: &Bytes, at: u64) -> Result<Locator, String> {
        let float = |at| b.u32(at).map(f32::from_bits);
        Ok(Locator {
            name: token(b.u64(at)?),
            position: [float(at + 8)?, float(at + 12)?, float(at + 16)?],
            scale: float(at + 20)?,
            rotation: [
                float(at + 24)?,
                float(at + 28)?,
                float(at + 32)?,
                float(at + 36)?,
            ],
        })
    }

    fn piece(b: &Bytes, at: u64) -> Result<Piece, String> {
        let (edges, vertices) = (b.index(at)?, b.index(at + 4)?);
        let (mask, width) = (b.u32(at + 8)?, b.index(at + 12)?);
        let material = b.index(at + 16)? as usize;
        if width > 8 {
            return Err(b.fail("piece has more than 8 texture-coordinate sets"));
        }
        // Offsets are absolute; -1 means the stream is absent.
        let offset = |field: u64| -> Result<Option<u64>, String> {
            Ok(match b.i32(at + field)? {
                -1 => None,
                _ => Some(b.index(at + field)?),
            })
        };
        let position = offset(64)?;
        let texcoord = offset(72)?;
        // Every present stream adds to the vertex stride, whatever is read from it.
        let mut stride = 0;
        for (field, size) in [
            (64, 12),
            (68, 12),
            (84, 16),
            (72, 8 * width),
            (76, 4),
            (80, 4),
            (88, 8),
        ] {
            if offset(field)?.is_some() {
                stride += size;
            }
        }
        let first = position.ok_or_else(|| b.fail("piece has no position stream"))?;
        let positions = (0..vertices)
            .map(|v| {
                let at = first + stride * v;
                Ok([
                    f32::from_bits(b.u32(at)?),
                    f32::from_bits(b.u32(at + 4)?),
                    f32::from_bits(b.u32(at + 8)?),
                ])
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut streams = Vec::new();
        if let Some(first) = texcoord {
            for set in 0..width {
                let uv = (0..vertices)
                    .map(|v| {
                        let at = first + stride * v + 8 * set;
                        Ok([f32::from_bits(b.u32(at)?), f32::from_bits(b.u32(at + 4)?)])
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let aliases = (0..8)
                    .filter(|n| u64::from((mask >> (n * 4)) & 0xF) == set)
                    .collect();
                streams.push(Stream { aliases, uv });
            }
        }
        let indices = b.index(at + 96)?;
        let triangles = (0..edges / 3)
            .map(|t| {
                let at = indices + 6 * t;
                Ok([
                    u32::from(u16::from_le_bytes(b.slice(at, 2)?.try_into().unwrap())),
                    u32::from(u16::from_le_bytes(b.slice(at + 2, 2)?.try_into().unwrap())),
                    u32::from(u16::from_le_bytes(b.slice(at + 4, 2)?.try_into().unwrap())),
                ])
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Piece {
            material,
            positions,
            streams,
            triangles,
        })
    }
}

impl Descriptor {
    /// Reads a `.pmd`; only version 4 exists in the installed game.
    pub fn parse(path: &str, data: &[u8]) -> Result<Self, String> {
        let b = Bytes { path, data };
        let version = b.u32(0)?;
        if version != 4 {
            return Err(b.fail(&format!(
                "descriptor version {version} is not supported yet (only 4)"
            )));
        }
        if data.len() < PMD_HEADER {
            return Err(b.fail("descriptor file is malformed"));
        }
        let field = |at| b.index(at);
        let (materials, looks, variants) = (field(4)?, field(8)?, field(16)?);
        let (parts, attributes) = (field(20)?, field(24)?);
        let values_size = field(28)?;
        let (look_at, variant_at, links_at) = (field(36)?, field(40)?, field(44)?);
        let (values_at, defs_at, material_at) = (field(48)?, field(52)?, field(56)?);
        let mut looks_list = Vec::new();
        for look in 0..looks {
            let mut files = Vec::new();
            for material in 0..materials {
                let entry = material_at + (look * materials + material) * 4;
                files.push(b.text(b.u32(entry)?.into())?);
            }
            looks_list.push(Look {
                name: token(b.u64(look_at + look * 8)?),
                materials: files,
            });
        }
        let mut variants_list = Vec::new();
        for variant in 0..variants {
            let mut parts_list = Vec::new();
            for part in 0..parts {
                let link = links_at + part * 8;
                let (from, to) = (b.i32(link)?, b.i32(link + 4)?);
                if from < 0 || from > to || i64::from(to) > attributes as i64 {
                    return Err(b.fail("part attributes lie outside the attribute table"));
                }
                let mut list = Vec::new();
                for def in from..to {
                    let def = defs_at + def as u64 * 16;
                    let kind = b.i32(def + 8)?;
                    if kind != 0 {
                        return Err(b.fail(&format!("attribute type {kind} is not supported yet")));
                    }
                    let value = values_at + b.index(def + 12)? + variant * values_size;
                    list.push((token(b.u64(def)?), b.i32(value)?));
                }
                parts_list.push(list);
            }
            variants_list.push(Variant {
                name: token(b.u64(variant_at + variant * 8)?),
                parts: parts_list,
            });
        }
        Ok(Self {
            looks: looks_list,
            variants: variants_list,
        })
    }
}

/// The effect named by a `.mat` file (`effect : "name" { ... }`, or the older `material`).
pub fn material_effect(path: &str, data: &[u8]) -> Result<String, String> {
    let fail = |cause: &str| format!("{path}: {cause}");
    let end = data.iter().position(|b| *b == 0).unwrap_or(data.len());
    let text = String::from_utf8_lossy(&data[..end]);
    let (kind, rest) = text
        .split_once(':')
        .ok_or_else(|| fail("material has no ':'"))?;
    let kind = kind.split_whitespace().collect::<String>();
    if kind != "material" && kind != "effect" {
        return Err(fail("invalid material format"));
    }
    let (head, body) = rest
        .split_once('{')
        .ok_or_else(|| fail("material has no left brace"))?;
    if !body.contains('}') {
        return Err(fail("material has no right brace"));
    }
    let head = head.split_whitespace().collect::<String>();
    let effect = head
        .split_once('"')
        .and_then(|(_, tail)| tail.split_once('"'))
        .ok_or_else(|| fail("material effect is not quoted"))?
        .0;
    Ok(effect.replace(".rfx", "").replace(".fx", ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::{LocatorSpec, PieceSpec, pmd, pmg, pmg_located, token_of};

    #[test]
    fn tokens_decode_to_their_names() {
        assert_eq!(token(token_of("default")), "default");
        assert_eq!(token(token_of("a_1")), "a_1");
        assert_eq!(token(0), "");
    }

    fn sample() -> Vec<u8> {
        let quad = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let other = vec![
            [0.25, 0.25],
            [f32::from_bits(0x3eaa_aaab), 0.25],
            [0.25, 0.75],
        ];
        pmg(
            &[("body", 0, 2), ("door", 2, 1)],
            &[
                // The second set carries `_TEXCOORD1`, the first `_TEXCOORD0` and `_TEXCOORD2`.
                PieceSpec {
                    material: 0,
                    mask: 0x10,
                    positions: vec![[1.5, -2.0, 0.25], [0.0, 1.0, 2.0], [3.0, 4.0, 5.0]],
                    uv_sets: vec![other.clone(), quad.clone()],
                    triangles: vec![[0, 1, 2]],
                },
                PieceSpec {
                    material: 1,
                    mask: 0x10,
                    positions: vec![],
                    uv_sets: vec![quad.clone(), other],
                    triangles: vec![],
                },
                PieceSpec {
                    material: 0,
                    mask: 0,
                    positions: vec![],
                    uv_sets: vec![],
                    triangles: vec![[0, 1, 2]],
                },
            ],
        )
    }

    #[test]
    fn geometry_keeps_exact_floats_aliases_parts_and_empty_pieces() {
        let model = Model::parse("/m.pmg", &sample()).unwrap();
        assert_eq!(
            model.parts[0],
            Part {
                name: "body".into(),
                pieces: vec![0, 1],
                locators: vec![]
            }
        );
        assert_eq!(model.parts[1].pieces, [2]);
        assert_eq!(model.pieces.len(), 3);
        let first = &model.pieces[0];
        assert_eq!(first.triangles, [[0, 1, 2]]);
        assert_eq!(first.positions[0], [1.5, -2.0, 0.25]);
        assert_eq!(first.positions[2], [3.0, 4.0, 5.0]);
        // Mask 0x10: nibble 1 is set 1, so `_TEXCOORD1` aliases the second set whatever its position.
        assert_eq!(first.streams[0].aliases, [0, 2, 3, 4, 5, 6, 7]);
        assert_eq!(first.streams[1].aliases, [1]);
        assert_eq!(first.streams[0].uv[1][0].to_bits(), 0x3eaa_aaab);
        assert!(model.pieces[1].triangles.is_empty());
        assert!(model.pieces[2].streams.is_empty());
    }

    #[test]
    fn locators_belong_to_their_part_and_a_cut_table_names_the_file() {
        let locator = |name, x| LocatorSpec {
            name,
            position: [x, 2.0, -3.0],
            scale: 1.5,
            rotation: [0.5, 0.5, -0.5, 0.5],
        };
        let data = pmg_located(
            &[("a", 0, 0), ("b", 0, 0), ("c", 0, 0)],
            &[],
            &[
                vec![locator("cab_door", 1.0), locator("r_deflector", 2.0)],
                vec![],
                vec![locator("f_bumper", 3.0)],
            ],
        );
        let model = Model::parse("/m.pmg", &data).unwrap();
        assert!(model.parts[1].locators.is_empty());
        assert_eq!(model.parts[0].locators[1].name, "r_deflector");
        assert_eq!(model.parts[0].locators[1].position, [2.0, 2.0, -3.0]);
        assert_eq!(model.parts[0].locators[0].scale, 1.5);
        assert_eq!(model.parts[0].locators[0].rotation, [0.5, 0.5, -0.5, 0.5]);
        assert_eq!(model.parts[2].locators[0].name, "f_bumper");
        // The table ends the file: cutting it, or counting more than it holds, fails the file.
        let error = Model::parse("/m.pmg", &data[..data.len() - 10]).unwrap_err();
        assert!(error.starts_with("/m.pmg: "), "{error}");
        let mut more = data.clone();
        more[16 + 112 + 24..][..4].copy_from_slice(&9i32.to_le_bytes());
        assert!(Model::parse("/m.pmg", &more).is_err());
    }

    #[test]
    fn truncated_position_stream_names_the_file() {
        let mut data = sample();
        // Point the first piece's position stream past the end of the file.
        let piece = 112 + 24 * 2;
        let past = (data.len() as i32 - 4).to_le_bytes();
        data[piece + 64..piece + 68].copy_from_slice(&past);
        let error = Model::parse("/m.pmg", &data).unwrap_err();
        assert!(error.starts_with("/m.pmg: "), "{error}");
        let mut absent = sample();
        absent[piece + 64..piece + 68].copy_from_slice(&(-1i32).to_le_bytes());
        assert!(Model::parse("/m.pmg", &absent).is_err());
    }

    #[test]
    fn geometry_refuses_other_versions_and_damage_without_panic() {
        let data = sample();
        for (version, bytes) in [(0x13u8, 0x13u8), (0x14, 0x14), (0x16, 0x16)] {
            let mut other = data.clone();
            other[0] = bytes;
            let error = Model::parse("/m.pmg", &other).unwrap_err();
            assert!(error.starts_with("/m.pmg: "), "{error}");
            assert!(error.contains(&format!("{version:#04x}")), "{error}");
        }
        for length in 0..data.len() {
            assert!(Model::parse("/m.pmg", &data[..length]).is_err(), "{length}");
        }
        for at in 0..data.len() {
            let mut damaged = data.clone();
            damaged[at] ^= 0xFF;
            let _ = Model::parse("/m.pmg", &damaged);
        }
    }

    #[test]
    fn descriptor_gives_looks_variants_and_visibility() {
        let data = pmd(
            &[
                ("default", &["/m/a.mat", "b.mat"]),
                ("white", &["/m/c.mat", "/m/d.mat"]),
            ],
            &[("default", &[1, 0]), ("hidden", &[0, 0])],
            2,
        );
        let descriptor = Descriptor::parse("/m.pmd", &data).unwrap();
        assert_eq!(descriptor.looks[1].name, "white");
        assert_eq!(descriptor.looks[0].materials, ["/m/a.mat", "b.mat"]);
        assert_eq!(descriptor.variants[0].parts[0], [("visible".to_owned(), 1)]);
        assert_eq!(descriptor.variants[1].name, "hidden");
        assert_eq!(descriptor.variants[0].parts[1], [("visible".to_owned(), 0)]);
        let mut newer = data.clone();
        newer[0] = 5;
        let error = Descriptor::parse("/m.pmd", &newer).unwrap_err();
        assert!(
            error.contains("/m.pmd") && error.contains("version 5"),
            "{error}"
        );
        for length in 0..data.len() {
            let parsed = Descriptor::parse("/m.pmd", &data[..length]);
            // Only a cut inside the last file name still reads (as a shorter name).
            assert!(parsed.is_err() || length + 9 >= data.len(), "{length}");
        }
        for at in 0..data.len() {
            let mut damaged = data.clone();
            damaged[at] ^= 0xFF;
            let _ = Descriptor::parse("/m.pmd", &damaged);
        }
    }

    #[test]
    fn material_effects_follow_the_converter() {
        let text = |s: &str| material_effect("/a.mat", s.as_bytes());
        assert_eq!(
            text("effect : \"eut2.truckpaint.tsnmapuv\" {\n a: 1\n}\n").unwrap(),
            "eut2.truckpaint.tsnmapuv"
        );
        assert_eq!(text("material:\"eut2.dif.fx\"{ }").unwrap(), "eut2.dif");
        assert_eq!(text("effect : \"a.rfx\" {}").unwrap(), "a");
        for bad in [
            "",
            "other : \"x\" {}",
            "effect : x {}",
            "effect : \"x\" {",
            "effect : \"x\"",
        ] {
            assert!(text(bad).is_err(), "{bad}");
        }
    }
}
