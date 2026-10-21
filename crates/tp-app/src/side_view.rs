//! The Side View: the vehicle of the open texture, painted with the livery being edited and seen
//! from one of four fixed directions. The preview mesh comes from the installed package; the
//! texture is a large thumbnail of the drawn surface (see [`crate::surface_thumbnails`]).
//!
//! The drawing is a parallel projection of the mesh on a plane, its triangles sorted from far to
//! near (painter's algorithm) and shaded by their angle to the viewer, then drawn as one
//! `egui::Mesh`. Sorting and shading are done once per mesh and direction.

use std::collections::HashMap;
use std::sync::Arc;

use egui::epaint::Vertex;
use egui::{Color32, Mesh, Pos2, Rect, TextureId, Vec2};
use serde::{Deserialize, Serialize};
use tp_core::document::Rgba;
use tp_core::{Project, TexturePart};
use tp_vehicles::PreviewMesh;

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

/// A mesh seen from a direction: triangles from the farthest to the nearest, three vertices each,
/// on a plane where y is up.
#[derive(Debug, Default, PartialEq)]
pub struct Drawing {
    vertices: Vec<Vertex>,
    min: Pos2,
    max: Pos2,
}

impl Drawing {
    /// Projects the pieces of `mesh` painted by texture `texture`.
    pub fn new(mesh: &PreviewMesh, texture: &str, direction: Direction) -> Self {
        let view = direction.towards_viewer();
        let mut triangles: Vec<(f32, [Vertex; 3])> = Vec::new();
        for piece in mesh.pieces.iter().filter(|p| p.texture == texture) {
            for t in &piece.triangles {
                let at = t.map(|i| piece.positions[i as usize]);
                let shade = shade(at, view);
                let color = Color32::from_gray((shade * 255.0).round() as u8);
                let across_depth = at.map(|p| direction.project(p));
                let depth = across_depth.iter().map(|p| p.2).sum::<f32>() / 3.0;
                let vertex = |k: usize| Vertex {
                    pos: Pos2::new(across_depth[k].0, across_depth[k].1),
                    uv: Pos2::new(piece.uv[t[k] as usize][0], piece.uv[t[k] as usize][1]),
                    color,
                };
                triangles.push((depth, [vertex(0), vertex(1), vertex(2)]));
            }
        }
        triangles.sort_by(|a, b| b.0.total_cmp(&a.0));
        let vertices: Vec<Vertex> = triangles.into_iter().flat_map(|(_, t)| t).collect();
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

    /// The mesh fitted in `rect`, textured with `texture`.
    pub fn mesh(&self, rect: Rect, texture: TextureId) -> Mesh {
        let (scale, offset) = self.fit(rect);
        let mut mesh = Mesh::with_texture(texture);
        mesh.vertices = self
            .vertices
            .iter()
            .map(|v| Vertex {
                pos: Pos2::new(v.pos.x, -v.pos.y) * scale + offset,
                ..*v
            })
            .collect();
        mesh.indices = (0..self.vertices.len() as u32).collect();
        mesh
    }
}

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
    /// The drawing of surface `surface`, painted with its artwork.
    Draw {
        surface: usize,
        drawing: Arc<Drawing>,
    },
    /// Why there is nothing to draw: the message id.
    Message(&'static str),
}

/// A preview mesh read from an installed package: package id, version and main texture id.
type MeshKey = (String, String, String);

/// The Side View's state: the texture it draws and what it has computed.
pub struct SideView {
    thumbs: SurfaceThumbnails,
    /// The main texture last opened, by package id.
    last_main: HashMap<String, String>,
    /// The preview mesh of each main texture asked for (`None`: it has none).
    meshes: HashMap<MeshKey, Option<Arc<PreviewMesh>>>,
    drawing: Option<((MeshKey, Direction), Arc<Drawing>)>,
    /// The surface the last [`SideView::target`] asked to draw.
    drawn: Option<usize>,
}

impl Default for SideView {
    fn default() -> Self {
        Self {
            thumbs: SurfaceThumbnails::sized("side_view", TEXTURE_SIDE, Some(BASE)),
            last_main: HashMap::new(),
            meshes: HashMap::new(),
            drawing: None,
            drawn: None,
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
        target
    }

    /// The surface drawn at the last frame, if any (tests).
    pub fn drawn(&self) -> Option<usize> {
        self.drawn
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
            // A trailer whose parts all have their own texture keeps a plain 64 × 64 main texture.
            let blank = installed
                .manifest
                .paint_job
                .part(&t.texture_id)
                .is_some_and(|(_, p)| p.texture.size == tp_vehicles::TextureSize::new(64, 64));
            return Target::Message(match (official, blank) {
                (true, true) => "side-view-blank",
                (true, false) => "side-view-regenerate",
                _ => "side-view-none",
            });
        };
        let cached = self
            .drawing
            .as_ref()
            .filter(|(k, _)| *k == (key.clone(), direction))
            .map(|(_, d)| d.clone());
        let drawing = cached.unwrap_or_else(|| {
            let drawing = Arc::new(Drawing::new(&mesh, &t.texture_id, direction));
            self.drawing = Some(((key, direction), drawing.clone()));
            drawing
        });
        Target::Draw { surface, drawing }
    }

    /// Starts rendering the drawn surface when its artwork changed, picks up a finished render.
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        fonts: &tp_text::FontLibrary,
        surface: usize,
    ) {
        self.thumbs.update_only(ctx, project, fonts, &[surface]);
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
        let left = faces(&Drawing::new(&mesh, "t", Direction::Left));
        assert_eq!(&left[10..], [0, 0], "the −x face is drawn last");
        assert_eq!(
            &left[..2],
            [1, 1],
            "the +x face is drawn first, under the others"
        );
        let front = faces(&Drawing::new(&mesh, "t", Direction::Front));
        assert_eq!(&front[10..], [4, 4], "the −z face is the front");
        let back = faces(&Drawing::new(&mesh, "t", Direction::Back));
        assert_eq!(&back[10..], [5, 5]);
        // Faces turned to the viewer are fully lit.
        let drawing = Drawing::new(&mesh, "t", Direction::Right);
        assert_eq!(drawing.vertices.last().unwrap().color, Color32::WHITE);
    }

    #[test]
    fn from_above_the_top_face_is_drawn_on_top_and_the_right_side_is_up() {
        let mesh = unit_box();
        let top = Drawing::new(&mesh, "t", Direction::Top);
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
    fn only_the_pieces_of_the_drawn_texture_are_projected() {
        let mesh = unit_box();
        assert!(Drawing::new(&mesh, "other", Direction::Left).is_empty());
        assert_eq!(Drawing::new(&mesh, "t", Direction::Left).vertices.len(), 36);
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
        };
        let x_of = |d: Direction, k: usize| {
            let drawing = Drawing::new(&mesh, "t", d);
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
        };
        assert!(Drawing::new(&mesh, "t", Direction::Front).vertices[0].pos.x < 0.0);
        assert!(Drawing::new(&mesh, "t", Direction::Back).vertices[0].pos.x > 0.0);
    }

    #[test]
    fn the_drawing_fits_the_rectangle_keeping_its_proportions() {
        let mesh = unit_box();
        let mut wide = mesh.clone();
        for p in &mut wide.pieces[0].positions {
            p[2] *= 4.0;
        }
        let drawing = Drawing::new(&wide, "t", Direction::Left);
        let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(400.0, 300.0));
        let out = drawing.mesh(rect, TextureId::default());
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
