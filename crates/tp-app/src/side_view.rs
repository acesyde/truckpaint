//! The Side View: the vehicle of the open texture, painted with the livery being edited and seen
//! from one of four fixed directions. The preview mesh comes from the installed package; the
//! texture is a large thumbnail of the drawn surface (see [`crate::surface_thumbnails`]).
//!
//! The vehicle is the preview mesh's cabin (or a trailer's chassis) and the option chosen in each of
//! its accessory slots, every piece placed at the locators of its slot. The drawing is a parallel
//! projection of all of them on a plane, their triangles sorted from far to near (painter's
//! algorithm) and shaded by their angle to the viewer, then drawn as one `egui::Mesh` per run of
//! triangles of one texture. Sorting and shading are done once per configuration and direction.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use egui::epaint::{Vertex, WHITE_UV};
use egui::{Color32, Mesh, Pos2, Rect, TextureId, Vec2};
use serde::{Deserialize, Serialize};
use tp_core::document::Rgba;
use tp_core::{Project, TexturePart};
use tp_vehicles::{CHASSIS_SLOT, PreviewLocator, PreviewMesh};

use crate::surface_thumbnails::SurfaceThumbnails;
use crate::vehicles::VehicleLibrary;

/// Longest side of the texture the Side View draws, in pixels.
const TEXTURE_SIDE: u32 = 1024;
/// Where the livery is not painted, the surface shows this plain gray.
const BASE: Rgba = Rgba::rgb(0xCF, 0xD4, 0xDC);

/// The four directions the vehicle is seen from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Right,
    Front,
    Back,
    /// From above, front towards the left, the vehicle's right side at the top.
    Top,
    /// Also what an unknown stored direction (a newer build's) falls back to: it must be the
    /// last variant.
    #[default]
    #[serde(other)]
    Left,
}

impl Direction {
    pub const ALL: [Self; 5] = [Self::Left, Self::Right, Self::Front, Self::Back, Self::Top];

    /// Message id of the direction's name.
    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "side-view-left",
            Self::Right => "side-view-right",
            Self::Front => "side-view-front",
            Self::Back => "side-view-back",
            Self::Top => "side-view-top",
        }
    }

    /// The position of `p` (x right, y up, z towards the back) on the plane seen by the viewer:
    /// across (towards the right of the drawing), up (towards its top) and depth (farther from
    /// the viewer is larger).
    fn project(self, [x, y, z]: [f32; 3]) -> (f32, f32, f32) {
        match self {
            Self::Left => (z, y, x),
            Self::Right => (-z, y, -x),
            Self::Front => (-x, y, z),
            Self::Back => (x, y, -z),
            Self::Top => (z, x, -y),
        }
    }

    /// Unit vector from the vehicle towards the viewer.
    fn towards_viewer(self) -> [f32; 3] {
        match self {
            Self::Left => [-1.0, 0.0, 0.0],
            Self::Right => [1.0, 0.0, 0.0],
            Self::Front => [0.0, 0.0, -1.0],
            Self::Back => [0.0, 0.0, 1.0],
            Self::Top => [0.0, 1.0, 0.0],
        }
    }
}

/// The option drawn in each slot of `mesh` (an index into the slot's options, `None` for None),
/// from the choices recorded in the project: a slot with no choice, or whose chosen option the mesh
/// no longer has, shows the game's default, or None when it has none. The chassis is never None.
pub fn configuration(
    mesh: &PreviewMesh,
    chosen: Option<&BTreeMap<String, Option<String>>>,
) -> Vec<Option<usize>> {
    mesh.slots
        .iter()
        .map(|slot| {
            let default = slot.options.iter().position(|o| o.default);
            let is_chassis = slot.id == CHASSIS_SLOT;
            let option = match chosen.and_then(|c| c.get(&slot.id)) {
                Some(Some(id)) => slot.options.iter().position(|o| &o.id == id).or(default),
                Some(None) if !is_chassis => None,
                _ => default,
            };
            // A vehicle always has a chassis.
            option.or_else(|| (is_chassis && !slot.options.is_empty()).then_some(0))
        })
        .collect()
}

/// Records that `slot` shows `option` (`None`: None) for `package`; a choice that is the default
/// is not recorded.
pub fn choose(
    project: &mut Project,
    package: &str,
    mesh: &PreviewMesh,
    slot: &str,
    option: Option<&str>,
) {
    let Some(index) = mesh.slots.iter().position(|s| s.id == slot) else {
        return;
    };
    let default =
        configuration(mesh, None)[index].map(|k| mesh.slots[index].options[k].id.as_str());
    let choices = project.side_view.entry(package.to_owned()).or_default();
    if option == default {
        choices.remove(slot);
    } else {
        choices.insert(slot.to_owned(), option.map(str::to_owned));
    }
    if choices.is_empty() {
        project.side_view.remove(package);
    }
}

/// A mesh seen from a direction: triangles from the farthest to the nearest, three vertices each,
/// on a plane where y is up.
#[derive(Debug, Default, PartialEq)]
pub struct Drawing {
    vertices: Vec<Vertex>,
    /// The texture of each triangle, an index into `textures`.
    texture_of: Vec<usize>,
    /// The ids of the textures the triangles are painted by, in order of first use.
    pub textures: Vec<String>,
    min: Pos2,
    max: Pos2,
}

impl Drawing {
    /// Projects the vehicle `mesh` in the configuration `config` (see [`configuration`]).
    pub fn new(mesh: &PreviewMesh, config: &[Option<usize>], direction: Direction) -> Self {
        let view = direction.towards_viewer();
        let mut textures: Vec<String> = Vec::new();
        let mut triangles: Vec<(f32, usize, [Vertex; 3])> = Vec::new();
        let mut add = |piece: &tp_vehicles::PreviewPiece, at: &PreviewLocator| {
            let texture = match textures.iter().position(|t| *t == piece.texture) {
                Some(i) => i,
                None => {
                    textures.push(piece.texture.clone());
                    textures.len() - 1
                }
            };
            let placed: Vec<[f32; 3]> = piece.positions.iter().map(|p| at.place(*p)).collect();
            for t in &piece.triangles {
                let at = t.map(|i| placed[i as usize]);
                let shade = shade(at, view);
                let color = Color32::from_gray((shade * 255.0).round() as u8);
                let across_depth = at.map(|p| direction.project(p));
                let depth = across_depth.iter().map(|p| p.2).sum::<f32>() / 3.0;
                let vertex = |k: usize| Vertex {
                    pos: Pos2::new(across_depth[k].0, across_depth[k].1),
                    uv: Pos2::new(piece.uv[t[k] as usize][0], piece.uv[t[k] as usize][1]),
                    color,
                };
                triangles.push((depth, texture, [vertex(0), vertex(1), vertex(2)]));
            }
        };
        for piece in &mesh.pieces {
            add(piece, &PreviewLocator::IDENTITY);
        }
        let chassis = mesh
            .slots
            .iter()
            .zip(config)
            .find(|(s, _)| s.id == CHASSIS_SLOT)
            .and_then(|(s, k)| s.options.get((*k)?));
        for (slot, option) in mesh.slots.iter().zip(config) {
            let Some(option) = option.and_then(|k| slot.options.get(k)) else {
                continue;
            };
            for at in mesh.mounts(slot, chassis) {
                for piece in &option.pieces {
                    add(piece, &at);
                }
            }
        }
        triangles.sort_by(|a, b| b.0.total_cmp(&a.0));
        let texture_of = triangles.iter().map(|t| t.1).collect();
        let vertices: Vec<Vertex> = triangles.into_iter().flat_map(|(_, _, t)| t).collect();
        let fold = |f: fn(f32, f32) -> f32, init: f32, axis: usize| {
            vertices
                .iter()
                .map(|v| if axis == 0 { v.pos.x } else { v.pos.y })
                .fold(init, f)
        };
        Self {
            min: Pos2::new(fold(f32::min, f32::MAX, 0), fold(f32::min, f32::MAX, 1)),
            max: Pos2::new(fold(f32::max, f32::MIN, 0), fold(f32::max, f32::MIN, 1)),
            vertices,
            texture_of,
            textures,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// The scale and the screen position of the drawing's origin that fit it in `rect`, centered,
    /// keeping its proportions.
    fn fit(&self, rect: Rect) -> (f32, Vec2) {
        let size = self.max - self.min;
        let scale =
            (rect.width() / size.x.max(f32::EPSILON)).min(rect.height() / size.y.max(f32::EPSILON));
        let center = Pos2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        );
        (
            scale,
            rect.center().to_vec2() - Vec2::new(center.x, -center.y) * scale,
        )
    }

    /// The meshes fitted in `rect`, back to front: one per run of triangles of one texture, drawn
    /// with the texture `texture` gives for its id, or in a plain neutral color when it gives none.
    pub fn meshes(&self, rect: Rect, texture: impl Fn(&str) -> Option<TextureId>) -> Vec<Mesh> {
        let (scale, offset) = self.fit(rect);
        let ids: Vec<Option<TextureId>> = self.textures.iter().map(|t| texture(t)).collect();
        let mut meshes: Vec<(Option<TextureId>, Mesh)> = Vec::new();
        for (triangle, vertices) in self.vertices.chunks(3).enumerate() {
            let which = self.texture_of[triangle];
            if meshes.last().is_none_or(|(id, _)| *id != ids[which]) {
                meshes.push((
                    ids[which],
                    Mesh::with_texture(ids[which].unwrap_or_default()),
                ));
            }
            let mesh = &mut meshes.last_mut().expect("just pushed").1;
            let first = mesh.vertices.len() as u32;
            for v in vertices {
                let (uv, color) = match ids[which] {
                    Some(_) => (v.uv, v.color),
                    None => (
                        WHITE_UV,
                        NEUTRAL.gamma_multiply(f32::from(v.color.r()) / 255.0),
                    ),
                };
                mesh.vertices.push(Vertex {
                    pos: Pos2::new(v.pos.x, -v.pos.y) * scale + offset,
                    uv,
                    color,
                });
            }
            mesh.indices.extend([first, first + 1, first + 2]);
        }
        meshes.into_iter().map(|(_, m)| m).collect()
    }
}

/// The color of a piece whose texture is not (yet) drawn.
const NEUTRAL: Color32 = Color32::from_rgb(0xCF, 0xD4, 0xDC);

/// How much light a triangle gets: from 0.55 edge-on to 1 facing the viewer.
fn shade([a, b, c]: [[f32; 3]; 3], view: [f32; 3]) -> f32 {
    let (u, v) = (sub(b, a), sub(c, a));
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = n.iter().map(|x| x * x).sum::<f32>().sqrt();
    if length <= f32::EPSILON {
        return 0.55;
    }
    let facing = (n[0] * view[0] + n[1] * view[1] + n[2] * view[2]).abs() / length;
    0.55 + 0.45 * facing
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// What the panel shows.
pub enum Target {
    /// The vehicle of surface `surface` (a main texture of `package`), in its configuration.
    Draw {
        surface: usize,
        package: String,
        mesh: Arc<PreviewMesh>,
        drawing: Arc<Drawing>,
    },
    /// Why there is nothing to draw: the message id.
    Message(&'static str),
}

/// A preview mesh read from an installed package: package id, version and main texture id.
type MeshKey = (String, String, String);

/// A drawing of one mesh, in one configuration, from one direction.
type DrawingKey = (MeshKey, Vec<Option<usize>>, Direction);

/// The Side View's state: the texture it draws and what it has computed.
pub struct SideView {
    thumbs: SurfaceThumbnails,
    /// The main texture last opened, by package id.
    last_main: HashMap<String, String>,
    /// The preview mesh of each main texture asked for (`None`: it has none).
    meshes: HashMap<MeshKey, Option<Arc<PreviewMesh>>>,
    drawing: Option<(DrawingKey, Arc<Drawing>)>,
    /// The surface the last [`SideView::target`] asked to draw.
    drawn: Option<usize>,
    /// The ids of the textures its drawing used.
    drawn_textures: Vec<String>,
}

impl Default for SideView {
    fn default() -> Self {
        Self {
            thumbs: SurfaceThumbnails::sized("side_view", TEXTURE_SIDE, Some(BASE)),
            last_main: HashMap::new(),
            meshes: HashMap::new(),
            drawing: None,
            drawn: None,
            drawn_textures: Vec::new(),
        }
    }
}

impl SideView {
    /// Remembers the main texture the painter works on, to keep drawing it when an accessory is
    /// opened.
    pub fn note(&mut self, project: &Project) {
        let Some(t) = project
            .surfaces
            .get(project.active_surface)
            .and_then(|s| s.template.as_ref())
            .filter(|t| t.part == TexturePart::Main)
        else {
            return;
        };
        if self.last_main.get(&t.package_id) != Some(&t.texture_id) {
            self.last_main
                .insert(t.package_id.clone(), t.texture_id.clone());
        }
    }

    /// The surface to draw: the active one when it is a main texture, else its vehicle's last
    /// opened main texture, or its first.
    fn main_surface(&self, project: &Project) -> Option<usize> {
        let main = |i: usize| {
            project.surfaces[i]
                .template
                .as_ref()
                .filter(|t| t.part == TexturePart::Main)
        };
        let active = main(project.active_surface).map(|_| project.active_surface);
        active.or_else(|| {
            let package = &project
                .surfaces
                .get(project.active_surface)?
                .template
                .as_ref()?
                .package_id;
            let mains: Vec<usize> = project
                .vehicle_range(package)
                .filter(|i| main(*i).is_some())
                .collect();
            let last = self.last_main.get(package);
            mains
                .iter()
                .copied()
                .find(|i| main(*i).is_some_and(|t| Some(&t.texture_id) == last))
                .or(mains.first().copied())
        })
    }

    /// What to show for `project`, seen from `direction`.
    pub fn target(
        &mut self,
        project: &Project,
        vehicles: &VehicleLibrary,
        direction: Direction,
    ) -> Target {
        let target = self.resolve(project, vehicles, direction);
        self.drawn = match &target {
            Target::Draw { surface, .. } => Some(*surface),
            Target::Message(_) => None,
        };
        self.drawn_textures = match &target {
            Target::Draw { drawing, .. } => drawing.textures.clone(),
            Target::Message(_) => Vec::new(),
        };
        target
    }

    /// The surface drawn at the last frame, if any (tests).
    pub fn drawn(&self) -> Option<usize> {
        self.drawn
    }

    /// The ids of the textures the last drawing used, in order of first use (tests).
    pub fn drawn_textures(&self) -> &[String] {
        &self.drawn_textures
    }

    /// Whether the texture is being rendered.
    pub fn is_rendering(&self) -> bool {
        self.thumbs.is_rendering()
    }

    fn resolve(
        &mut self,
        project: &Project,
        vehicles: &VehicleLibrary,
        direction: Direction,
    ) -> Target {
        let Some(surface) = self.main_surface(project) else {
            return Target::Message("side-view-none");
        };
        let t = project.surfaces[surface]
            .template
            .as_ref()
            .expect("a main surface has a template");
        if tp_pack::custom::is_custom(&t.package_id) {
            return Target::Message("side-view-custom");
        }
        let Some(installed) = project
            .vehicle(&t.package_id)
            .and_then(|v| vehicles.recorded(v))
        else {
            return Target::Message("side-view-package");
        };
        let key = (
            t.package_id.clone(),
            installed.manifest.version.to_string(),
            t.texture_id.clone(),
        );
        let mesh = self
            .meshes
            .entry(key.clone())
            .or_insert_with(|| read_mesh(&installed.path, &t.texture_id))
            .clone();
        let Some(mesh) = mesh else {
            let official = t.package_id.starts_with("official.");
            return Target::Message(if official {
                "side-view-regenerate"
            } else {
                "side-view-none"
            });
        };
        let config = configuration(&mesh, project.side_view.get(&t.package_id));
        let wanted = (key, config, direction);
        let cached = self
            .drawing
            .as_ref()
            .filter(|(k, _)| *k == wanted)
            .map(|(_, d)| d.clone());
        let drawing = cached.unwrap_or_else(|| {
            let drawing = Arc::new(Drawing::new(&mesh, &wanted.1, direction));
            self.drawing = Some((wanted, drawing.clone()));
            drawing
        });
        Target::Draw {
            surface,
            package: t.package_id.clone(),
            mesh,
            drawing,
        }
    }

    /// The surface of `project` that holds texture `texture` of `package`, if the project has it.
    pub fn surface_of(project: &Project, package: &str, texture: &str) -> Option<usize> {
        project.vehicle_range(package).find(|i| {
            project.surfaces[*i]
                .template
                .as_ref()
                .is_some_and(|t| t.texture_id == texture)
        })
    }

    /// Starts rendering the drawn surfaces whose artwork changed, picks up a finished render.
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        fonts: &tp_text::FontLibrary,
        surfaces: &[usize],
    ) {
        self.thumbs.update_only(ctx, project, fonts, surfaces);
    }

    /// Color of the rendered texture of `surface` at `uv` (tests).
    pub fn color_at(&self, surface: usize, uv: [f32; 2]) -> Option<Rgba> {
        self.thumbs.color_at(surface, uv)
    }

    /// The texture of `surface`, once rendered.
    pub fn texture(&self, surface: usize) -> Option<&egui::TextureHandle> {
        self.thumbs.texture(surface)
    }
}

fn read_mesh(path: &std::path::Path, texture: &str) -> Option<Arc<PreviewMesh>> {
    let read = std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| {
            tp_vehicles::Package::read_preview(&bytes, texture).map_err(|e| e.to_string())
        });
    match read {
        Ok(mesh) => mesh.map(Arc::new),
        Err(error) => {
            tracing::warn!(path = %path.display(), error, "cannot read the preview mesh");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use tp_vehicles::PreviewPiece;

    use super::*;

    /// A unit box centered on the origin; face `n` (−x, +x, −y, +y, −z, +z) has its UVs at `n`.
    fn unit_box() -> PreviewMesh {
        let mut piece = PreviewPiece {
            texture: "t".into(),
            positions: Vec::new(),
            uv: Vec::new(),
            triangles: Vec::new(),
        };
        for (n, axis, sign) in [
            (0, 0, -1.0),
            (1, 0, 1.0),
            (2, 1, -1.0),
            (3, 1, 1.0),
            (4, 2, -1.0),
            (5, 2, 1.0),
        ] {
            let first = piece.positions.len() as u32;
            for (a, b) in [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)] {
                let mut p = [a, b, b];
                p[axis] = sign * 0.5;
                let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                p[u] = a;
                p[v] = b;
                piece.positions.push(p);
                piece.uv.push([n as f32, 0.0]);
            }
            piece
                .triangles
                .extend([[first, first + 1, first + 2], [first, first + 2, first + 3]]);
        }
        PreviewMesh {
            format: 1,
            pieces: vec![piece],
            ..PreviewMesh::default()
        }
    }

    fn faces(drawing: &Drawing) -> Vec<u32> {
        drawing
            .vertices
            .chunks(3)
            .map(|t| t[0].uv.x as u32)
            .collect()
    }

    #[test]
    fn the_nearest_face_is_drawn_last_and_the_farthest_first() {
        let mesh = unit_box();
        // Seen from the left (viewer at −x): the −x face is nearest, the +x face farthest.
        let left = faces(&Drawing::new(&mesh, &[], Direction::Left));
        assert_eq!(&left[10..], [0, 0], "the −x face is drawn last");
        assert_eq!(
            &left[..2],
            [1, 1],
            "the +x face is drawn first, under the others"
        );
        let front = faces(&Drawing::new(&mesh, &[], Direction::Front));
        assert_eq!(&front[10..], [4, 4], "the −z face is the front");
        let back = faces(&Drawing::new(&mesh, &[], Direction::Back));
        assert_eq!(&back[10..], [5, 5]);
        // Faces turned to the viewer are fully lit.
        let drawing = Drawing::new(&mesh, &[], Direction::Right);
        assert_eq!(drawing.vertices.last().unwrap().color, Color32::WHITE);
    }

    #[test]
    fn from_above_the_top_face_is_drawn_on_top_and_the_right_side_is_up() {
        let mesh = unit_box();
        let top = Drawing::new(&mesh, &[], Direction::Top);
        assert_eq!(&faces(&top)[10..], [3, 3], "the +y face is drawn last");
        assert_eq!(&faces(&top)[..2], [2, 2], "the −y face is drawn first");
        // The +x face (the vehicle's right) is edge-on at the top of the drawing, the −x face
        // at the bottom; the front (−z) is on the left.
        let at = |face: u32| -> Vec<Pos2> {
            top.vertices
                .chunks(3)
                .filter(|t| t[0].uv.x as u32 == face)
                .flat_map(|t| t.iter().map(|v| v.pos))
                .collect()
        };
        assert!(at(1).iter().all(|p| p.y > 0.4), "+x at the top");
        assert!(at(0).iter().all(|p| p.y < -0.4), "−x at the bottom");
        assert!(at(4).iter().all(|p| p.x < -0.4), "−z (front) on the left");
    }

    #[test]
    fn every_piece_is_projected_with_its_own_texture() {
        let mesh = unit_box();
        assert_eq!(Drawing::new(&mesh, &[], Direction::Left).vertices.len(), 36);
        assert!(Drawing::new(&PreviewMesh::default(), &[], Direction::Left).is_empty());
    }

    fn triangle(texture: &str, z: f32) -> PreviewPiece {
        PreviewPiece {
            texture: texture.into(),
            positions: vec![[0.0, 0.0, z], [0.0, 1.0, z], [0.0, 0.0, z + 0.1]],
            uv: vec![[0.0, 0.0]; 3],
            triangles: vec![[0, 1, 2]],
        }
    }

    #[test]
    fn depth_order_is_kept_across_textures_one_mesh_per_run() {
        // Seen from the left (viewer at −x) the depth is x: three triangles at x = 0, 1, 2 painted
        // a, b, a. The farthest (x = 2) is drawn first, so the runs are a, b, a.
        let at = |x: f32, texture: &str| {
            let mut piece = triangle(texture, 0.0);
            piece.positions.iter_mut().for_each(|p| p[0] = x);
            piece
        };
        let mesh = PreviewMesh {
            format: 1,
            pieces: vec![at(0.0, "a"), at(1.0, "b"), at(2.0, "a")],
            ..PreviewMesh::default()
        };
        let drawing = Drawing::new(&mesh, &[], Direction::Left);
        let ids = |id: &str| Some(TextureId::User(if id == "a" { 1 } else { 2 }));
        let runs = drawing.meshes(Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0)), ids);
        let textures: Vec<TextureId> = runs.iter().map(|m| m.texture_id).collect();
        assert_eq!(
            textures,
            [TextureId::User(1), TextureId::User(2), TextureId::User(1)]
        );
        // A texture with none to show is a plain neutral color on the white pixel.
        let neutral = drawing.meshes(Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0)), |_| {
            None
        });
        assert!(
            neutral
                .iter()
                .all(|m| m.vertices.iter().all(|v| v.uv == WHITE_UV))
        );
        assert_eq!(neutral.len(), 1, "one texture-less run, not three");
    }

    /// A cab with a door slot on the cabin (locator at x = 1.075) and a bumper slot on the chassis.
    fn fitted() -> PreviewMesh {
        let option =
            |id: &str, default: bool, pieces: Vec<PreviewPiece>| tp_vehicles::PreviewOption {
                id: id.into(),
                name: id.into(),
                default,
                pieces,
                locators: BTreeMap::new(),
            };
        let at = |x: f32| PreviewLocator {
            position: [x, 0.0, 0.0],
            ..PreviewLocator::IDENTITY
        };
        let slot = |id: &str, parent, options| tp_vehicles::PreviewSlot {
            id: id.into(),
            parent,
            options,
        };
        let mut chassis = option("short", true, vec![triangle("t", 0.0)]);
        chassis
            .locators
            .insert("f_bumper".into(), vec![at(0.0), at(10.0)]);
        let mut door = triangle("door", 0.0);
        door.positions.iter_mut().for_each(|p| p[0] -= 1.0);
        PreviewMesh {
            format: 1,
            pieces: vec![triangle("t", 5.0)],
            locators: [("cab_door".to_owned(), vec![at(1.075)])].into(),
            slots: vec![
                slot("chassis", None, vec![chassis]),
                slot(
                    "cab_door",
                    Some(tp_vehicles::SlotParent::Cabin),
                    vec![
                        option("std", true, vec![door]),
                        option("win", false, vec![triangle("door", 0.0)]),
                    ],
                ),
                slot(
                    "f_bumper",
                    Some(tp_vehicles::SlotParent::Chassis),
                    vec![option("paint", false, vec![triangle("bumper", 0.0)])],
                ),
            ],
        }
    }

    #[test]
    fn the_configuration_starts_at_the_defaults_and_follows_the_choices() {
        let mesh = fitted();
        assert_eq!(configuration(&mesh, None), [Some(0), Some(0), None]);
        let choices = |pairs: &[(&str, Option<&str>)]| -> BTreeMap<String, Option<String>> {
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.map(str::to_owned)))
                .collect()
        };
        let chosen = choices(&[("cab_door", Some("win")), ("f_bumper", Some("paint"))]);
        assert_eq!(
            configuration(&mesh, Some(&chosen)),
            [Some(0), Some(1), Some(0)]
        );
        // None, an option the package no longer has, and a chassis that cannot be None.
        let chosen = choices(&[
            ("cab_door", None),
            ("f_bumper", Some("gone")),
            ("chassis", None),
        ]);
        assert_eq!(configuration(&mesh, Some(&chosen)), [Some(0), None, None]);
        let chosen = choices(&[("cab_door", Some("gone"))]);
        assert_eq!(
            configuration(&mesh, Some(&chosen))[1],
            Some(0),
            "back to the default"
        );
    }

    #[test]
    fn choices_are_recorded_only_when_they_differ_from_the_default() {
        use tp_core::TextureResolution;
        let mesh = fitted();
        let mut project = Project::new("x", TextureResolution::R2048);
        choose(&mut project, "p", &mesh, "cab_door", None);
        assert_eq!(project.side_view["p"]["cab_door"], None);
        choose(&mut project, "p", &mesh, "f_bumper", Some("paint"));
        choose(&mut project, "p", &mesh, "cab_door", Some("std"));
        assert_eq!(project.side_view["p"].len(), 1, "std is the default again");
        choose(&mut project, "p", &mesh, "f_bumper", None);
        assert!(project.side_view.is_empty());
        choose(&mut project, "p", &mesh, "unknown", None);
        assert!(project.side_view.is_empty());
    }

    #[test]
    fn options_are_drawn_at_every_locator_of_their_parent() {
        let mesh = fitted();
        let x_of = |config: &[Option<usize>], texture: &str| -> Vec<f32> {
            let drawing = Drawing::new(&mesh, config, Direction::Front);
            let at = drawing.textures.iter().position(|t| t == texture);
            let mut xs: Vec<f32> = drawing
                .vertices
                .chunks(3)
                .enumerate()
                .filter(|(i, _)| Some(drawing.texture_of[*i]) == at)
                .map(|(_, t)| -t[0].pos.x)
                .collect();
            xs.sort_by(f32::total_cmp);
            xs
        };
        // The door is modeled one meter left of its origin and placed at x = 1.075.
        let door = x_of(&[Some(0), Some(0), None], "door");
        assert!((door[0] - 0.075).abs() < 1e-5, "{door:?}");
        // The bumper is drawn once per chassis locator; with None it is not drawn at all.
        assert_eq!(x_of(&[Some(0), Some(0), Some(0)], "bumper"), [0.0, 10.0]);
        assert!(x_of(&[Some(0), Some(0), None], "bumper").is_empty());
        // Switching to the other door never draws both.
        let both = Drawing::new(&mesh, &[Some(0), Some(1), None], Direction::Front);
        assert_eq!(
            both.vertices.len(),
            3 * 3,
            "the cab, the chassis and one door"
        );
    }

    #[test]
    fn the_left_of_the_drawing_is_the_front_seen_from_the_left() {
        // A piece at the front (z = −1) and one at the back (z = 1), as triangles.
        let tri = |z: f32| PreviewPiece {
            texture: "t".into(),
            positions: vec![[0.0, 0.0, z], [0.0, 1.0, z], [0.0, 0.0, z + 0.1]],
            uv: vec![[0.0, 0.0]; 3],
            triangles: vec![[0, 1, 2]],
        };
        let mesh = PreviewMesh {
            format: 1,
            pieces: vec![tri(-1.0), tri(1.0)],
            ..PreviewMesh::default()
        };
        let x_of = |d: Direction, k: usize| {
            let drawing = Drawing::new(&mesh, &[], d);
            let mut xs: Vec<f32> = drawing.vertices.chunks(3).map(|t| t[0].pos.x).collect();
            xs.sort_by(f32::total_cmp);
            xs[k]
        };
        // Left: the front (−z) is on the left; Right: it is on the right.
        assert!(x_of(Direction::Left, 0) < 0.0);
        assert!(x_of(Direction::Right, 0) < 0.0 && x_of(Direction::Right, 1) > 0.0);
        // Seen from the front, the vehicle's right (+x) is on the left of the drawing.
        let side = |x: f32| PreviewPiece {
            texture: "t".into(),
            positions: vec![[x, 0.0, 0.0], [x, 1.0, 0.0], [x, 0.0, 0.1]],
            uv: vec![[0.0, 0.0]; 3],
            triangles: vec![[0, 1, 2]],
        };
        let mesh = PreviewMesh {
            format: 1,
            pieces: vec![side(1.0)],
            ..PreviewMesh::default()
        };
        assert!(Drawing::new(&mesh, &[], Direction::Front).vertices[0].pos.x < 0.0);
        assert!(Drawing::new(&mesh, &[], Direction::Back).vertices[0].pos.x > 0.0);
    }

    #[test]
    fn the_drawing_fits_the_rectangle_keeping_its_proportions() {
        let mesh = unit_box();
        let mut wide = mesh.clone();
        for p in &mut wide.pieces[0].positions {
            p[2] *= 4.0;
        }
        let drawing = Drawing::new(&wide, &[], Direction::Left);
        let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(400.0, 300.0));
        let out = &drawing.meshes(rect, |_| Some(TextureId::default()))[0];
        let bounds = out.vertices.iter().fold(Rect::NOTHING, |r, v| {
            r.union(Rect::from_min_max(v.pos, v.pos))
        });
        // Four times wider than tall: it fills the width, its height is a quarter, centered.
        assert!((bounds.width() - 400.0).abs() < 0.01, "{bounds:?}");
        assert!((bounds.height() - 100.0).abs() < 0.01, "{bounds:?}");
        assert!((bounds.center() - rect.center()).length() < 0.01);
        // y is up in the model and down on screen: the +y face edge is above.
        assert!(
            out.vertices
                .iter()
                .any(|v| (v.pos.y - bounds.top()).abs() < 0.01)
        );
    }
}
