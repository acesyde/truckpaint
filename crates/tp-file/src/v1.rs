//! Format version 1, the first released format: objects, paints, paths,
//! texts, images, guides, the palette, assets, the fleet's vehicles, and
//! each surface's template with the vehicle texture it belongs to. Frozen once released: a new
//! format version gets its own module and a migration from this one.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tp_core::document::{
    AssetId, Cap, CharStyle, ColorStop, DEFAULT_MITER_LIMIT, Dash, Frame, Gradient, GradientKind,
    Join, LineStyle, MIN_STOPS, Node, Object, ObjectId, Paint, PathData, Rgba, Shadow, ShapeKind,
    StrokeAlign, StrokeStyle, StyleId, Subpath, SwatchId, SymbolId, TextAlign, TextBlock,
};
use tp_core::kurbo::{Point, Size, Vec2};
use tp_core::{
    Asset, AssetKind, Axis, BrandKit, DEFAULT_SWATCH_PREFIX, GameData, GraphicStyle, Guide,
    LibraryKey, Look, ModSettings, Project, ProjectVehicle, RequiredMod, Surface, SurfaceTemplate,
    Swatch, Symbol, TemplateStatus, TextStyle, TexturePart, TextureResolution,
};

/// The document (`project.ron`). Asset bytes live in separate ZIP entries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileProject {
    pub name: String,
    /// Texture side in pixels (2048, 4096 or 8192).
    pub resolution: u32,
    pub active_surface: usize,
    /// Colors of the palette before swatches had names (read only: opened
    /// as "Color N" swatches).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub palette: Vec<[u8; 4]>,
    /// The palette's named swatches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub swatches: Vec<FileSwatch>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graphic_styles: Vec<FileGraphicStyle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text_styles: Vec<FileTextStyle>,
    /// Drawings placed as instances on the surfaces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbols: Vec<FileSymbol>,
    pub surfaces: Vec<FileSurface>,
    #[serde(default)]
    pub assets: Vec<FileAsset>,
    /// The vehicles of the fleet.
    #[serde(default)]
    pub vehicles: Vec<FileVehicle>,
    /// What the project exports as a mod (missing: the defaults).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mod_settings: Option<FileModSettings>,
    /// The game versions the mod is made for (`1.56.*`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub game_versions: Vec<String>,
}

/// The settings of the exported mod.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileModSettings {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    /// The internal name as typed (missing: follows the name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_name: Option<String>,
    pub price: u32,
    #[serde(default)]
    pub unlock_level: u32,
    /// Asset of the chosen shop icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<u64>,
    /// Asset of the chosen Mod Manager image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<u64>,
}

/// A symbol: its content on its own square artboard.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileSymbol {
    pub id: u64,
    pub name: String,
    /// Side of the artboard in pixels.
    pub size: f64,
    #[serde(default)]
    pub objects: Vec<FileObject>,
    #[serde(default)]
    pub guides: Vec<FileGuide>,
    /// The library entry it came from or was added to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// A named palette color.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileSwatch {
    pub id: u64,
    pub name: String,
    pub color: [u8; 4],
    /// The library entry it came from or was added to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// A named fill, stroke, opacity and shadow.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileGraphicStyle {
    pub id: u64,
    pub name: String,
    pub fill: FilePaint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_swatch: Option<u64>,
    #[serde(default)]
    pub stroke: Option<FileStroke>,
    pub opacity: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<FileShadow>,
    /// The library entry it came from or was added to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// A named character style.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileTextStyle {
    pub id: u64,
    pub name: String,
    pub family: String,
    pub weight: u16,
    pub italic: bool,
    pub size: f64,
    pub align: FileAlign,
    pub letter_spacing: f64,
    pub line_height: f64,
    /// The lettering's fill, stroke and opacity (missing: the default
    /// look of a new text).
    #[serde(default)]
    pub fill: Option<FilePaint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_swatch: Option<u64>,
    #[serde(default)]
    pub stroke: Option<FileStroke>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<FileShadow>,
    /// The library entry it came from or was added to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// A vehicle of the project: the package its templates come from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileVehicle {
    pub package_id: String,
    pub version: String,
    pub name: String,
    pub brand: String,
    pub kind: String,
    pub game: String,
    /// What the mod export needs (missing in files written before it was
    /// recorded).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_data: Option<FileGameData>,
}

/// What a vehicle is in the game, recorded from its package.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileGameData {
    pub path: String,
    pub versions: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub alt_uv: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub colour_picker: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<FileRequiredMod>,
    pub main_count: usize,
}

/// A mod a vehicle depends on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileRequiredMod {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileTemplateStatus {
    #[default]
    Current,
    LayoutChanged,
    Removed,
}

/// Whether a template's texture is a main texture or an accessory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilePart {
    Main,
    Accessory,
}

/// A surface's template: an asset shown over the artwork, and the vehicle
/// texture it belongs to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileTemplate {
    pub package_id: String,
    pub texture_id: String,
    pub part: FilePart,
    pub asset: u64,
    pub layout_version: u32,
    pub opacity: f32,
    pub visible: bool,
    #[serde(default)]
    pub status: FileTemplateStatus,
    /// The game ids of the texture.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub game_ids: Vec<String>,
    /// Accessory ids a main texture also paints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also_paints: Vec<String>,
    /// A main texture's position among the package's main textures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_index: Option<usize>,
    /// Objects an update asked to reposition (only with `LayoutChanged`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flagged_objects: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileSurface {
    pub name: String,
    /// Width and height in pixels.
    pub size: [f64; 2],
    #[serde(default)]
    pub objects: Vec<FileObject>,
    #[serde(default)]
    pub guides: Vec<FileGuide>,
    #[serde(default)]
    pub template: Option<FileTemplate>,
}

/// A guide: vertical at x = `position`, or horizontal at y = `position`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileGuide {
    pub vertical: bool,
    pub position: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FileKind {
    Rectangle {
        corner_radius: f64,
    },
    Ellipse,
    Group,
    Text,
    Image {
        asset: u64,
    },
    /// `star`: inner radius ratio of a star.
    Polygon {
        sides: u8,
        star: Option<f64>,
    },
    Path,
    /// An instance of a symbol; `placement` is the affine (a, b, c, d, e,
    /// f) mapping the symbol onto the surface. Its content is not stored:
    /// it is rebuilt from the symbol.
    Instance {
        symbol: u64,
        placement: [f64; 6],
    },
}

/// An anchor point (frame-local coordinates).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileNode {
    pub p: [f64; 2],
    #[serde(default)]
    pub hin: Option<[f64; 2]>,
    #[serde(default)]
    pub hout: Option<[f64; 2]>,
    #[serde(default)]
    pub smooth: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileSubpath {
    pub closed: bool,
    pub nodes: Vec<FileNode>,
}

/// A color at a location (0..=1) of a gradient.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileStop {
    pub offset: f32,
    pub color: [u8; 4],
    /// The swatch the color is linked to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swatch: Option<u64>,
}

/// A fill or stroke paint. Gradient points are in frame units: (0, 0) is
/// the frame's local top-left corner, (1, 1) its bottom-right.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FilePaint {
    Solid([u8; 4]),
    Linear {
        start: [f64; 2],
        end: [f64; 2],
        stops: Vec<FileStop>,
    },
    /// `minor`: end of the second radius.
    Radial {
        center: [f64; 2],
        end: [f64; 2],
        minor: [f64; 2],
        stops: Vec<FileStop>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileStroke {
    pub paint: FilePaint,
    pub width: f64,
    #[serde(default)]
    pub align: FileStrokeAlign,
    /// Dash pattern, caps and joins of the outline.
    #[serde(default)]
    pub line: FileLineStyle,
    /// The swatch a solid paint is linked to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swatch: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStrokeAlign {
    #[default]
    Center,
    Inside,
    Outside,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCap {
    Butt,
    #[default]
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

fn default_miter_limit() -> f64 {
    DEFAULT_MITER_LIMIT
}

/// Dash pattern (`[dash, gap]`), caps, joins and miter limit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileLineStyle {
    #[serde(default)]
    pub dash: Option<[f64; 2]>,
    #[serde(default)]
    pub cap: FileCap,
    #[serde(default)]
    pub join: FileJoin,
    #[serde(default = "default_miter_limit")]
    pub miter_limit: f64,
}

impl Default for FileLineStyle {
    fn default() -> Self {
        line_style_to_file(&LineStyle::default())
    }
}

fn line_style_to_file(l: &LineStyle) -> FileLineStyle {
    FileLineStyle {
        dash: l.dash.map(|d| [d.dash, d.gap]),
        cap: match l.cap {
            Cap::Butt => FileCap::Butt,
            Cap::Round => FileCap::Round,
            Cap::Square => FileCap::Square,
        },
        join: match l.join {
            Join::Miter => FileJoin::Miter,
            Join::Round => FileJoin::Round,
            Join::Bevel => FileJoin::Bevel,
        },
        miter_limit: l.miter_limit,
    }
}

/// The line style of `f`; out-of-range values fall back to the defaults.
fn line_style_from_file(f: &FileLineStyle) -> LineStyle {
    let valid = |v: f64| v.is_finite() && v >= 0.0;
    LineStyle {
        dash: f
            .dash
            .filter(|[dash, gap]| valid(*dash) && valid(*gap) && *gap > 0.0)
            .map(|[dash, gap]| Dash { dash, gap }),
        cap: match f.cap {
            FileCap::Butt => Cap::Butt,
            FileCap::Round => Cap::Round,
            FileCap::Square => Cap::Square,
        },
        join: match f.join {
            FileJoin::Miter => Join::Miter,
            FileJoin::Round => Join::Round,
            FileJoin::Bevel => Join::Bevel,
        },
        miter_limit: if f.miter_limit.is_finite() && f.miter_limit >= 1.0 {
            f.miter_limit
        } else {
            DEFAULT_MITER_LIMIT
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileText {
    pub content: String,
    pub family: String,
    pub weight: u16,
    pub italic: bool,
    pub size: f64,
    pub align: FileAlign,
    pub letter_spacing: f64,
    pub line_height: f64,
    pub layout_size: [f64; 2],
    pub scale: [f64; 2],
    /// The text style followed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileObject {
    pub id: u64,
    pub name: String,
    pub kind: FileKind,
    pub center: [f64; 2],
    pub size: [f64; 2],
    pub rotation: f64,
    pub fill: FilePaint,
    /// The swatch a solid fill is linked to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_swatch: Option<u64>,
    #[serde(default)]
    pub stroke: Option<FileStroke>,
    /// The graphic style followed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<u64>,
    pub opacity: f32,
    pub visible: bool,
    pub locked: bool,
    #[serde(default)]
    pub children: Vec<FileObject>,
    #[serde(default)]
    pub text: Option<FileText>,
    #[serde(default)]
    pub path: Option<Vec<FileSubpath>>,
    /// Width of the lines of a path's open subpaths.
    #[serde(default)]
    pub line_width: Option<f64>,
    /// Dash pattern, caps and joins of those lines.
    #[serde(default)]
    pub line_style: Option<FileLineStyle>,
    /// A text or an image drawn reversed across its vertical axis.
    #[serde(default, skip_serializing_if = "is_false")]
    pub mirrored: bool,
    /// The drop shadow (missing: none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<FileShadow>,
}

/// A drop shadow: color and its swatch link, opacity, offset and blur in
/// texture pixels.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileShadow {
    pub color: [u8; 4],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swatch: Option<u64>,
    pub opacity: f32,
    pub offset: [f64; 2],
    pub blur: f64,
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileAssetKind {
    Raster,
    Svg,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileAsset {
    pub id: u64,
    pub name: String,
    pub kind: FileAssetKind,
    /// ZIP entry holding the original bytes.
    pub entry: String,
    pub size: [f64; 2],
}

fn color(c: Rgba) -> [u8; 4] {
    [c.r, c.g, c.b, c.a]
}

fn rgba([r, g, b, a]: [u8; 4]) -> Rgba {
    Rgba { r, g, b, a }
}

fn paint_to_file(p: &Paint) -> FilePaint {
    let Paint::Gradient(g) = p else {
        return FilePaint::Solid(color(p.first_color()));
    };
    let xy = |p: Point| [p.x, p.y];
    let stops = g
        .stops()
        .iter()
        .map(|s| FileStop {
            offset: s.offset,
            color: color(s.color),
            swatch: s.swatch.map(|w| w.0),
        })
        .collect();
    match g.kind {
        GradientKind::Linear => FilePaint::Linear {
            start: xy(g.start),
            end: xy(g.end),
            stops,
        },
        GradientKind::Radial => FilePaint::Radial {
            center: xy(g.start),
            end: xy(g.end),
            minor: xy(g.minor),
            stops,
        },
    }
}

/// The paint of `f`; a gradient needs at least two stops and finite points.
fn paint_from_file(f: &FilePaint, id: u64) -> Result<Paint, String> {
    let (kind, start, end, minor, stops) = match f {
        FilePaint::Solid(c) => return Ok(Paint::Solid(rgba(*c))),
        FilePaint::Linear { start, end, stops } => (GradientKind::Linear, start, end, None, stops),
        FilePaint::Radial {
            center,
            end,
            minor,
            stops,
        } => (GradientKind::Radial, center, end, Some(minor), stops),
    };
    let finite = |p: &[f64; 2]| p.iter().all(|v| v.is_finite());
    if stops.len() < MIN_STOPS
        || !finite(start)
        || !finite(end)
        || !minor.is_none_or(finite)
        || stops.iter().any(|s| !s.offset.is_finite())
    {
        return Err(format!("object {id} has an invalid gradient"));
    }
    let stops: Vec<ColorStop> = stops
        .iter()
        .map(|s| ColorStop {
            swatch: s.swatch.map(SwatchId),
            ..ColorStop::new(s.offset, rgba(s.color))
        })
        .collect();
    let mut g = Gradient::new(kind, &stops);
    let pt = |p: &[f64; 2]| Point::new(p[0], p[1]);
    g.start = pt(start);
    g.end = pt(end);
    if let Some(m) = minor {
        g.minor = pt(m);
    }
    Ok(Paint::Gradient(g))
}

/// ZIP entry name of an asset: `assets/<id>.<png|jpg|svg>`.
pub fn asset_entry(asset: &Asset) -> String {
    let ext = match asset.kind {
        AssetKind::Svg => "svg",
        AssetKind::Raster if asset.bytes.starts_with(&[0xFF, 0xD8]) => "jpg",
        AssetKind::Raster => "png",
    };
    format!("assets/{}.{ext}", asset.id.0)
}

fn align_to_file(a: TextAlign) -> FileAlign {
    match a {
        TextAlign::Left => FileAlign::Left,
        TextAlign::Center => FileAlign::Center,
        TextAlign::Right => FileAlign::Right,
    }
}

fn align_from_file(a: FileAlign) -> TextAlign {
    match a {
        FileAlign::Left => TextAlign::Left,
        FileAlign::Center => TextAlign::Center,
        FileAlign::Right => TextAlign::Right,
    }
}

fn stroke_to_file(s: &StrokeStyle) -> FileStroke {
    FileStroke {
        paint: paint_to_file(&s.paint),
        width: s.width,
        align: match s.align {
            StrokeAlign::Center => FileStrokeAlign::Center,
            StrokeAlign::Inside => FileStrokeAlign::Inside,
            StrokeAlign::Outside => FileStrokeAlign::Outside,
        },
        line: line_style_to_file(&s.line),
        swatch: s.swatch.map(|w| w.0),
    }
}

/// The stroke of `s`; `id` names the object or style in errors.
fn stroke_from_file(s: &FileStroke, id: u64) -> Result<StrokeStyle, String> {
    Ok(StrokeStyle {
        paint: paint_from_file(&s.paint, id)?,
        width: s.width,
        align: match s.align {
            FileStrokeAlign::Center => StrokeAlign::Center,
            FileStrokeAlign::Inside => StrokeAlign::Inside,
            FileStrokeAlign::Outside => StrokeAlign::Outside,
        },
        line: line_style_from_file(&s.line),
        swatch: s.swatch.map(SwatchId),
    })
}

fn shadow_to_file(s: &Shadow) -> FileShadow {
    FileShadow {
        color: color(s.color),
        swatch: s.swatch.map(|w| w.0),
        opacity: s.opacity,
        offset: [s.offset.x, s.offset.y],
        blur: s.blur,
    }
}

/// The shadow of `s`, its values brought into range.
fn shadow_from_file(s: &FileShadow) -> Shadow {
    Shadow {
        color: rgba(s.color),
        swatch: s.swatch.map(SwatchId),
        opacity: s.opacity,
        offset: Vec2::new(s.offset[0], s.offset[1]),
        blur: s.blur,
    }
    .clamped()
}

fn origin_to_file(origin: Option<&LibraryKey>) -> Option<String> {
    origin.map(|k| k.0.clone())
}

fn origin_from_file(origin: Option<&String>) -> Option<LibraryKey> {
    origin.map(|k| LibraryKey(k.clone()))
}

fn brand_to_file(project: &Project, file: &mut FileProject) {
    file.swatches = project
        .palette
        .iter()
        .map(|s| FileSwatch {
            id: s.id.0,
            name: s.name.clone(),
            color: color(s.color),
            origin: origin_to_file(s.origin.as_ref()),
        })
        .collect();
    file.graphic_styles = project
        .graphic_styles
        .iter()
        .map(|s| FileGraphicStyle {
            id: s.id.0,
            name: s.name.clone(),
            fill: paint_to_file(&s.look.fill),
            fill_swatch: s.look.fill_swatch.map(|w| w.0),
            stroke: s.look.stroke.as_ref().map(stroke_to_file),
            opacity: s.look.opacity,
            shadow: s.look.shadow.as_ref().map(shadow_to_file),
            origin: origin_to_file(s.origin.as_ref()),
        })
        .collect();
    file.text_styles = project
        .text_styles
        .iter()
        .map(|s| FileTextStyle {
            id: s.id.0,
            name: s.name.clone(),
            family: s.style.family.clone(),
            weight: s.style.weight,
            italic: s.style.italic,
            size: s.style.size,
            align: align_to_file(s.style.align),
            letter_spacing: s.style.letter_spacing,
            line_height: s.style.line_height,
            fill: Some(paint_to_file(&s.look.fill)),
            fill_swatch: s.look.fill_swatch.map(|w| w.0),
            stroke: s.look.stroke.as_ref().map(stroke_to_file),
            opacity: Some(s.look.opacity),
            shadow: s.look.shadow.as_ref().map(shadow_to_file),
            origin: origin_to_file(s.origin.as_ref()),
        })
        .collect();
}

/// The palette and styles of `file` (an earlier palette of plain colors
/// is added afterwards, see [`into_project`]).
fn brand_from_file(file: &FileProject) -> Result<BrandKit, String> {
    let palette = file
        .swatches
        .iter()
        .map(|s| Swatch {
            id: SwatchId(s.id),
            name: s.name.clone(),
            color: rgba(s.color),
            origin: origin_from_file(s.origin.as_ref()),
        })
        .collect();
    let graphic_styles = file
        .graphic_styles
        .iter()
        .map(|s| {
            Ok(GraphicStyle {
                id: StyleId(s.id),
                name: s.name.clone(),
                look: Look {
                    fill: paint_from_file(&s.fill, s.id)?,
                    fill_swatch: s.fill_swatch.map(SwatchId),
                    stroke: s
                        .stroke
                        .as_ref()
                        .map(|st| stroke_from_file(st, s.id))
                        .transpose()?,
                    opacity: s.opacity.clamp(0.0, 1.0),
                    shadow: s.shadow.as_ref().map(shadow_from_file),
                },
                origin: origin_from_file(s.origin.as_ref()),
            })
        })
        .collect::<Result<_, String>>()?;
    let text_styles = file
        .text_styles
        .iter()
        .map(|s| {
            Ok(TextStyle {
                id: StyleId(s.id),
                name: s.name.clone(),
                style: CharStyle {
                    family: s.family.clone(),
                    weight: s.weight,
                    italic: s.italic,
                    size: s.size,
                    align: align_from_file(s.align),
                    letter_spacing: s.letter_spacing,
                    line_height: s.line_height,
                },
                look: Look {
                    fill: match &s.fill {
                        Some(f) => paint_from_file(f, s.id)?,
                        None => Paint::Solid(tp_core::document::DEFAULT_FILL),
                    },
                    fill_swatch: s.fill_swatch.map(SwatchId),
                    stroke: s
                        .stroke
                        .as_ref()
                        .map(|st| stroke_from_file(st, s.id))
                        .transpose()?,
                    opacity: s.opacity.unwrap_or(1.0).clamp(0.0, 1.0),
                    shadow: s.shadow.as_ref().map(shadow_from_file),
                },
                origin: origin_from_file(s.origin.as_ref()),
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(BrandKit {
        palette,
        graphic_styles,
        text_styles,
    })
}

fn guides_to_file(guides: &[Guide]) -> Vec<FileGuide> {
    guides
        .iter()
        .map(|g| FileGuide {
            vertical: g.axis == Axis::Vertical,
            position: g.position,
        })
        .collect()
}

fn guides_from_file(guides: &[FileGuide]) -> Vec<Guide> {
    guides
        .iter()
        .filter(|g| g.position.is_finite())
        .map(|g| {
            let axis = if g.vertical {
                Axis::Vertical
            } else {
                Axis::Horizontal
            };
            Guide::new(axis, g.position)
        })
        .collect()
}

fn object_to_file(o: &Object) -> FileObject {
    FileObject {
        id: o.id.0,
        name: o.name.clone(),
        kind: match o.kind {
            ShapeKind::Rectangle { corner_radius } => FileKind::Rectangle { corner_radius },
            ShapeKind::Ellipse => FileKind::Ellipse,
            ShapeKind::Group => FileKind::Group,
            ShapeKind::Text => FileKind::Text,
            ShapeKind::Image { asset } => FileKind::Image { asset: asset.0 },
            ShapeKind::Polygon { sides, star } => FileKind::Polygon { sides, star },
            ShapeKind::Path => FileKind::Path,
            ShapeKind::Instance { symbol, placement } => FileKind::Instance {
                symbol: symbol.0,
                placement: placement.as_coeffs(),
            },
        },
        center: [o.frame.center.x, o.frame.center.y],
        size: [o.frame.size.width, o.frame.size.height],
        rotation: o.frame.rotation_deg,
        fill: paint_to_file(&o.fill),
        fill_swatch: o.fill_swatch.map(|s| s.0),
        stroke: o.stroke.as_ref().map(stroke_to_file),
        style: o.style.map(|s| s.0),
        opacity: o.opacity,
        visible: o.visible,
        locked: o.locked,
        // An instance's children are rebuilt from its symbol.
        children: if o.is_instance() {
            Vec::new()
        } else {
            o.children.iter().map(|c| object_to_file(c)).collect()
        },
        text: o.text.as_ref().map(|t| FileText {
            content: t.content.clone(),
            family: t.style.family.clone(),
            weight: t.style.weight,
            italic: t.style.italic,
            size: t.style.size,
            align: align_to_file(t.style.align),
            letter_spacing: t.style.letter_spacing,
            line_height: t.style.line_height,
            layout_size: [t.layout_size.width, t.layout_size.height],
            scale: [t.scale.x, t.scale.y],
            style_id: t.style_id.map(|s| s.0),
        }),
        path: o.path_data().map(|p| {
            let xy = |p: tp_core::kurbo::Point| [p.x, p.y];
            p.subpaths
                .iter()
                .map(|s| FileSubpath {
                    closed: s.closed,
                    nodes: s
                        .nodes
                        .iter()
                        .map(|n| FileNode {
                            p: xy(n.point),
                            hin: n.handle_in.map(xy),
                            hout: n.handle_out.map(xy),
                            smooth: n.smooth,
                        })
                        .collect(),
                })
                .collect()
        }),
        line_width: o.path_data().map(|p| p.line_width),
        line_style: o.path_data().map(|p| line_style_to_file(&p.line_style)),
        mirrored: o.mirrored,
        shadow: o.shadow.as_ref().map(shadow_to_file),
    }
}

/// The document of `project` (asset bytes are written separately).
pub fn from_project(project: &Project) -> FileProject {
    let mut file = FileProject {
        name: project.name.clone(),
        resolution: project.resolution.side(),
        active_surface: project.active_surface,
        palette: Vec::new(),
        swatches: Vec::new(),
        graphic_styles: Vec::new(),
        text_styles: Vec::new(),
        symbols: project
            .symbols
            .iter()
            .map(|s| FileSymbol {
                id: s.id.0,
                name: s.name.clone(),
                size: s.surface.size.width,
                objects: s
                    .surface
                    .objects
                    .iter()
                    .map(|o| object_to_file(o))
                    .collect(),
                guides: guides_to_file(&s.surface.guides),
                origin: origin_to_file(s.origin.as_ref()),
            })
            .collect(),
        surfaces: project
            .surfaces
            .iter()
            .map(|s| FileSurface {
                name: s.name.clone(),
                size: [s.size.width, s.size.height],
                objects: s.objects.iter().map(|o| object_to_file(o)).collect(),
                guides: s
                    .guides
                    .iter()
                    .map(|g| FileGuide {
                        vertical: g.axis == Axis::Vertical,
                        position: g.position,
                    })
                    .collect(),
                template: s.template.as_ref().map(|t| FileTemplate {
                    package_id: t.package_id.clone(),
                    texture_id: t.texture_id.clone(),
                    part: match t.part {
                        TexturePart::Main => FilePart::Main,
                        TexturePart::Accessory => FilePart::Accessory,
                    },
                    asset: t.asset.0,
                    layout_version: t.layout_version,
                    opacity: t.opacity,
                    visible: t.visible,
                    status: match t.status {
                        TemplateStatus::Current => FileTemplateStatus::Current,
                        TemplateStatus::LayoutChanged => FileTemplateStatus::LayoutChanged,
                        TemplateStatus::Removed => FileTemplateStatus::Removed,
                    },
                    game_ids: t.game_ids.clone(),
                    also_paints: t.also_paints.clone(),
                    main_index: t.main_index,
                    flagged_objects: match t.status {
                        TemplateStatus::LayoutChanged => t.flagged.iter().map(|id| id.0).collect(),
                        _ => Vec::new(),
                    },
                }),
            })
            .collect(),
        assets: project
            .assets
            .values()
            .map(|a| FileAsset {
                id: a.id.0,
                name: a.name.clone(),
                kind: match a.kind {
                    AssetKind::Raster => FileAssetKind::Raster,
                    AssetKind::Svg => FileAssetKind::Svg,
                },
                entry: asset_entry(a),
                size: [a.size.width, a.size.height],
            })
            .collect(),
        vehicles: project
            .vehicles
            .iter()
            .map(|v| FileVehicle {
                package_id: v.package_id.clone(),
                version: v.version.clone(),
                name: v.name.clone(),
                brand: v.brand.clone(),
                kind: v.kind.clone(),
                game: v.game.clone(),
                game_data: v.game_data.as_ref().map(|g| FileGameData {
                    path: g.path.clone(),
                    versions: g.versions.clone(),
                    alt_uv: g.alt_uv,
                    colour_picker: g.colour_picker,
                    requires: g
                        .requires
                        .iter()
                        .map(|r| FileRequiredMod {
                            name: r.name.clone(),
                            version: r.version.clone(),
                        })
                        .collect(),
                    main_count: g.main_count,
                }),
            })
            .collect(),
        mod_settings: mod_settings_to_file(project),
        game_versions: project.game_versions.clone(),
    };
    brand_to_file(project, &mut file);
    file
}

/// The mod settings, left out while they are a new project's.
fn mod_settings_to_file(project: &Project) -> Option<FileModSettings> {
    let s = &project.mod_settings;
    (*s != ModSettings::for_project(&project.name)).then(|| FileModSettings {
        name: s.name.clone(),
        version: s.version.clone(),
        author: s.author.clone(),
        description: s.description.clone(),
        internal_name: s.internal_name.clone(),
        price: s.price,
        unlock_level: s.unlock_level,
        icon: s.icon.map(|a| a.0),
        image: s.image.map(|a| a.0),
    })
}

/// The mod settings of `file`; its chosen images must be assets.
fn mod_settings_from_file(file: &FileProject, project: &Project) -> Result<ModSettings, String> {
    let Some(s) = &file.mod_settings else {
        return Ok(ModSettings::for_project(&project.name));
    };
    let asset = |id: Option<u64>| match id.map(AssetId) {
        Some(a) if !project.assets.contains_key(&a) => {
            Err("an image of the mod settings is missing".to_owned())
        }
        a => Ok(a),
    };
    Ok(ModSettings {
        name: s.name.clone(),
        version: s.version.clone(),
        author: s.author.clone(),
        description: s.description.clone(),
        internal_name: s.internal_name.clone(),
        price: s.price,
        unlock_level: s.unlock_level,
        icon: asset(s.icon)?,
        image: asset(s.image)?,
    })
}

/// The project's vehicles.
fn vehicles_from_file(file: &FileProject) -> Vec<ProjectVehicle> {
    file.vehicles
        .iter()
        .map(|v| ProjectVehicle {
            package_id: v.package_id.clone(),
            version: v.version.clone(),
            name: v.name.clone(),
            brand: v.brand.clone(),
            kind: v.kind.clone(),
            game: v.game.clone(),
            game_data: v.game_data.as_ref().map(|g| GameData {
                path: g.path.clone(),
                versions: g.versions.clone(),
                alt_uv: g.alt_uv,
                colour_picker: g.colour_picker,
                requires: g
                    .requires
                    .iter()
                    .map(|r| RequiredMod {
                        name: r.name.clone(),
                        version: r.version.clone(),
                    })
                    .collect(),
                main_count: g.main_count,
            }),
        })
        .collect()
}

/// All vehicles share one game and none is repeated; every surface's
/// template names a vehicle of the project, no vehicle texture is painted
/// twice, and every vehicle paints at least one texture.
fn check_fleet(vehicles: &[ProjectVehicle], surfaces: &[Surface]) -> Result<(), String> {
    let Some(first) = vehicles.first() else {
        return Err("the project has no vehicle".to_owned());
    };
    for (i, v) in vehicles.iter().enumerate() {
        if v.game != first.game {
            return Err("the vehicles are for different games".to_owned());
        }
        if vehicles[..i].iter().any(|o| o.package_id == v.package_id) {
            return Err(format!("{} is listed twice", v.package_id));
        }
    }
    let mut keys = std::collections::HashSet::new();
    for s in surfaces {
        let Some(t) = &s.template else {
            return Err(format!("{} has no vehicle texture", s.name));
        };
        if !vehicles.iter().any(|v| v.package_id == t.package_id) {
            return Err(format!("{} belongs to an unknown vehicle", s.name));
        }
        if !keys.insert(t.key()) {
            return Err(format!("{} is painted twice", s.name));
        }
    }
    for v in vehicles {
        if !keys.iter().any(|k| k.package_id == v.package_id) {
            return Err(format!("{} paints no texture", v.package_id));
        }
    }
    Ok(())
}

fn object_from_file(
    f: &FileObject,
    assets: &BTreeMap<AssetId, Arc<Asset>>,
) -> Result<Object, String> {
    let kind = match f.kind {
        FileKind::Rectangle { corner_radius } => ShapeKind::Rectangle { corner_radius },
        FileKind::Ellipse => ShapeKind::Ellipse,
        FileKind::Group => ShapeKind::Group,
        FileKind::Text => ShapeKind::Text,
        FileKind::Polygon { sides, star } => ShapeKind::Polygon {
            sides: sides.clamp(3, 12),
            star: star.map(|r| r.clamp(0.1, 0.9)),
        },
        FileKind::Path => ShapeKind::Path,
        FileKind::Instance { symbol, placement } => {
            if placement.iter().any(|v| !v.is_finite()) {
                return Err(format!("instance {} has an invalid placement", f.id));
            }
            ShapeKind::Instance {
                symbol: SymbolId(symbol),
                placement: tp_core::kurbo::Affine::new(placement),
            }
        }
        FileKind::Image { asset } => {
            if !assets.contains_key(&AssetId(asset)) {
                return Err(format!("object {} uses a missing asset", f.id));
            }
            ShapeKind::Image {
                asset: AssetId(asset),
            }
        }
    };
    let frame = Frame::new(
        Point::new(f.center[0], f.center[1]),
        Size::new(f.size[0], f.size[1]),
        f.rotation,
    );
    let mut o = Object::new(ObjectId(f.id), kind, frame);
    o.name.clone_from(&f.name);
    o.fill = paint_from_file(&f.fill, f.id)?;
    o.fill_swatch = f.fill_swatch.map(SwatchId);
    o.stroke = f
        .stroke
        .as_ref()
        .map(|s| stroke_from_file(s, f.id))
        .transpose()?;
    o.style = f.style.map(StyleId);
    o.opacity = f.opacity.clamp(0.0, 1.0);
    o.visible = f.visible;
    o.locked = f.locked;
    // Only texts and images carry a mirror; other kinds carry it in their
    // geometry.
    o.mirrored = f.mirrored && matches!(kind, ShapeKind::Text | ShapeKind::Image { .. });
    if o.takes_shadow() {
        o.shadow = f.shadow.as_ref().map(shadow_from_file);
    }
    o.children = f
        .children
        .iter()
        .map(|c| object_from_file(c, assets).map(Arc::new))
        .collect::<Result<_, _>>()?;
    if kind == ShapeKind::Text && f.text.is_none() {
        return Err(format!("text {} has no content", f.id));
    }
    o.text = f.text.as_ref().map(|t| TextBlock {
        content: t.content.clone(),
        style: CharStyle {
            family: t.family.clone(),
            weight: t.weight,
            italic: t.italic,
            size: t.size,
            align: align_from_file(t.align),
            letter_spacing: t.letter_spacing,
            line_height: t.line_height,
        },
        layout_size: Size::new(t.layout_size[0], t.layout_size[1]),
        scale: Vec2::new(t.scale[0], t.scale[1]),
        style_id: t.style_id.map(StyleId),
    });
    if kind == ShapeKind::Path {
        let pt = |p: [f64; 2]| Point::new(p[0], p[1]);
        let subpaths = f
            .path
            .as_ref()
            .ok_or_else(|| format!("path {} has no points", f.id))?
            .iter()
            .map(|s| Subpath {
                closed: s.closed,
                nodes: s
                    .nodes
                    .iter()
                    .map(|n| Node {
                        point: pt(n.p),
                        handle_in: n.hin.map(pt),
                        handle_out: n.hout.map(pt),
                        smooth: n.smooth,
                    })
                    .collect(),
            })
            .collect();
        // Stored as written: the frame was fitted to the points on save.
        let mut data = PathData::new(subpaths);
        if let Some(width) = f.line_width.filter(|w| w.is_finite() && *w > 0.0) {
            data.line_width = width;
        }
        if let Some(style) = &f.line_style {
            data.line_style = line_style_from_file(style);
        }
        o.path = Some(Arc::new(data));
    }
    if o.has_content() {
        o.refresh_group_frame();
    }
    Ok(o)
}

/// Rebuilds the project and checks its fleet; `bytes_of` returns the
/// content of a ZIP entry.
pub fn into_project(
    file: &FileProject,
    bytes_of: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Result<Project, String> {
    let project = into_document(file, bytes_of)?;
    check_fleet(&project.vehicles, &project.surfaces)?;
    Ok(project)
}

/// Rebuilds the document without checking its fleet (the library has no
/// vehicle); `bytes_of` returns the content of a ZIP entry.
pub fn into_document(
    file: &FileProject,
    mut bytes_of: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Result<Project, String> {
    let resolution = TextureResolution::ALL
        .into_iter()
        .find(|r| r.side() == file.resolution)
        .ok_or_else(|| format!("unknown resolution {}", file.resolution))?;
    let mut assets = BTreeMap::new();
    for a in &file.assets {
        let bytes = bytes_of(&a.entry).ok_or_else(|| format!("missing {}", a.entry))?;
        let kind = match a.kind {
            FileAssetKind::Raster => AssetKind::Raster,
            FileAssetKind::Svg => AssetKind::Svg,
        };
        let asset = Asset::new(
            AssetId(a.id),
            &a.name,
            kind,
            bytes.into(),
            Size::new(a.size[0], a.size[1]),
        );
        assets.insert(asset.id, Arc::new(asset));
    }
    if file.surfaces.is_empty() {
        return Err("the project has no surface".to_owned());
    }
    let surfaces = file
        .surfaces
        .iter()
        .map(|s| {
            let [width, height] = s.size;
            if !(width.is_finite() && width > 0.0 && height.is_finite() && height > 0.0) {
                return Err(format!("the surface {} has an invalid size", s.name));
            }
            let mut surface = Surface::new(s.name.clone(), Size::new(width, height));
            surface.guides = s
                .guides
                .iter()
                .filter(|g| g.position.is_finite())
                .map(|g| {
                    let axis = if g.vertical {
                        Axis::Vertical
                    } else {
                        Axis::Horizontal
                    };
                    Guide::new(axis, g.position)
                })
                .collect();
            surface.objects = s
                .objects
                .iter()
                .map(|o| object_from_file(o, &assets).map(Arc::new))
                .collect::<Result<_, _>>()?;
            surface.template = match &s.template {
                Some(t) => {
                    if !assets.contains_key(&AssetId(t.asset)) {
                        return Err(format!("the template of {} is missing", s.name));
                    }
                    // Only objects still on the surface, and only while the
                    // texture is flagged: a stale record shows no mark.
                    let on: BTreeSet<ObjectId> =
                        tp_core::objects_on(&surface).into_iter().collect();
                    let flagged = match t.status {
                        FileTemplateStatus::LayoutChanged => t
                            .flagged_objects
                            .iter()
                            .map(|&id| ObjectId(id))
                            .filter(|id| on.contains(id))
                            .collect(),
                        _ => BTreeSet::new(),
                    };
                    Some(SurfaceTemplate {
                        package_id: t.package_id.clone(),
                        texture_id: t.texture_id.clone(),
                        part: match t.part {
                            FilePart::Main => TexturePart::Main,
                            FilePart::Accessory => TexturePart::Accessory,
                        },
                        asset: AssetId(t.asset),
                        layout_version: t.layout_version,
                        opacity: if t.opacity.is_finite() {
                            t.opacity.clamp(0.0, 1.0)
                        } else {
                            SurfaceTemplate::DEFAULT_OPACITY
                        },
                        visible: t.visible,
                        status: match t.status {
                            FileTemplateStatus::Current => TemplateStatus::Current,
                            FileTemplateStatus::LayoutChanged => TemplateStatus::LayoutChanged,
                            FileTemplateStatus::Removed => TemplateStatus::Removed,
                        },
                        game_ids: t.game_ids.clone(),
                        also_paints: t.also_paints.clone(),
                        main_index: t.main_index,
                        flagged,
                    })
                }
                None => None,
            };
            Ok(surface)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut project = Project::from_parts(
        &file.name,
        resolution,
        surfaces,
        file.active_surface,
        brand_from_file(file)?,
        assets,
    );
    if file.swatches.is_empty() {
        for c in &file.palette {
            project.add_swatch(rgba(*c), DEFAULT_SWATCH_PREFIX);
        }
    }
    let symbols = file
        .symbols
        .iter()
        .map(|s| {
            if !(s.size.is_finite() && s.size > 0.0) {
                return Err(format!("symbol {} has an invalid size", s.id));
            }
            let mut surface = Surface::new(s.name.clone(), Size::new(s.size, s.size));
            surface.objects = s
                .objects
                .iter()
                .map(|o| object_from_file(o, &project.assets).map(Arc::new))
                .collect::<Result<_, _>>()?;
            surface.guides = guides_from_file(&s.guides);
            Ok(Symbol {
                id: SymbolId(s.id),
                name: s.name.clone(),
                surface,
                origin: origin_from_file(s.origin.as_ref()),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    project.set_symbols(symbols);
    // A link that disagrees with its source (a hand-edited file) is dropped;
    // instances are expanded from their symbols.
    project.relink();
    project.vehicles = vehicles_from_file(file);
    project.mod_settings = mod_settings_from_file(file, &project)?;
    project.game_versions = file.game_versions.clone();
    Ok(project)
}
