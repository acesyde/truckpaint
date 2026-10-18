//! Vehicle packages (`.tpv`): a ZIP holding a `vehicle.json` manifest and
//! the template images of one truck or trailer. Packages are data only;
//! this crate reads and validates them from bytes (no filesystem access).

use std::collections::HashSet;
use std::io::{Cursor, Read};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod catalog;
pub mod sample;

/// Highest package format this version understands.
pub const FORMAT: u32 = 1;
/// File extension of packages, without the dot.
pub const EXTENSION: &str = "tpv";
/// Name of the manifest inside a package.
pub const MANIFEST: &str = "vehicle.json";
/// Smallest side of a texture (pixels).
pub const MIN_SIDE: u32 = 64;
/// Largest side of a texture (pixels).
pub const MAX_SIDE: u32 = 8192;
/// Largest total uncompressed size of a package.
pub const MAX_UNCOMPRESSED: u64 = 512 * 1024 * 1024;
/// Largest side of a template image.
pub const MAX_IMAGE_SIDE: u32 = 16_384;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Truck,
    Trailer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Ets2,
    Ats,
}

impl Game {
    /// Every game.
    pub const ALL: [Game; 2] = [Game::Ets2, Game::Ats];

    pub fn code(self) -> &'static str {
        match self {
            Game::Ets2 => "ets2",
            Game::Ats => "ats",
        }
    }

    /// The game of code `code` (`ets2`, `ats`).
    pub fn from_code(code: &str) -> Option<Game> {
        Game::ALL.into_iter().find(|g| g.code() == code)
    }
}

impl Kind {
    pub fn code(self) -> &'static str {
        match self {
            Kind::Truck => "truck",
            Kind::Trailer => "trailer",
        }
    }
}

/// A mod the vehicle depends on (community vehicles).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Requirement {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

/// A texture's width and height in pixels, written `[width, height]` in a
/// manifest. Each side is a power of two from [`MIN_SIDE`] to [`MAX_SIDE`]
/// once validated; a square texture has equal sides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(from = "[u32; 2]", into = "[u32; 2]")]
pub struct TextureSize {
    pub width: u32,
    pub height: u32,
}

impl TextureSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Whether `side` is a power of two from [`MIN_SIDE`] to [`MAX_SIDE`].
    pub fn is_allowed_side(side: u32) -> bool {
        side.is_power_of_two() && (MIN_SIDE..=MAX_SIDE).contains(&side)
    }

    /// The allowed sides, smallest first.
    pub fn allowed_sides() -> impl Iterator<Item = u32> {
        (MIN_SIDE.trailing_zeros()..=MAX_SIDE.trailing_zeros()).map(|p| 1 << p)
    }

    /// Whether both sides are allowed.
    pub fn is_allowed(self) -> bool {
        Self::is_allowed_side(self.width) && Self::is_allowed_side(self.height)
    }

    /// "4096²" when the sides are equal, "8192 × 1024" otherwise.
    pub fn label(self) -> String {
        if self.width == self.height {
            format!("{}²", self.width)
        } else {
            format!("{} × {}", self.width, self.height)
        }
    }
}

impl From<[u32; 2]> for TextureSize {
    fn from([width, height]: [u32; 2]) -> Self {
        Self { width, height }
    }
}

impl From<TextureSize> for [u32; 2] {
    fn from(size: TextureSize) -> Self {
        [size.width, size.height]
    }
}

/// What a part's template looks like on the texture.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Texture {
    pub size: TextureSize,
    /// Path of the template image inside the package.
    pub template: String,
    pub layout_version: u32,
}

/// A paintable part of the vehicle: one texture, and what it covers in the
/// game (cabin internal names for a main texture, accessory ids for an
/// accessory).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Part {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub game_ids: Vec<String>,
    /// A main texture's accessory ids the game paints with this same
    /// texture (a trailer body drawn on the main template). Empty for an
    /// accessory.
    #[serde(default)]
    pub also_paints: Vec<String>,
    pub texture: Texture,
}

/// Whether a part is a main texture or an accessory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Main,
    Accessory,
}

/// The vehicle's paint job: main textures (one per cabin layout, or one for
/// the whole vehicle) and accessory textures shared by the whole vehicle.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PaintJob {
    #[serde(default)]
    pub main: Vec<Part>,
    #[serde(default)]
    pub accessories: Vec<Part>,
}

impl PaintJob {
    /// Every part in package order: main textures, then accessories.
    pub fn parts(&self) -> impl Iterator<Item = (Role, &Part)> {
        self.main
            .iter()
            .map(|p| (Role::Main, p))
            .chain(self.accessories.iter().map(|p| (Role::Accessory, p)))
    }

    /// The part `id` and its role.
    pub fn part(&self, id: &str) -> Option<(Role, &Part)> {
        self.parts().find(|(_, p)| p.id == id)
    }

    /// Position of part `id` in package order.
    pub fn position(&self, id: &str) -> Option<usize> {
        self.parts().position(|(_, p)| p.id == id)
    }
}

/// What the package targets in the game.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct GameTarget {
    pub id: Game,
    /// The game versions the package is made for.
    pub versions: semver::VersionReq,
    /// The vehicle's path in the game's definitions (`scania.r_2016`).
    pub path: String,
    /// The paint job uses the vehicle's alternate UV set.
    #[serde(default)]
    pub alt_uv: bool,
    /// The paint job lets the player pick a base color.
    #[serde(default)]
    pub colour_picker: bool,
    #[serde(default)]
    pub requires: Vec<Requirement>,
}

/// The `vehicle.json` manifest (unknown fields are ignored).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Manifest {
    pub format: u32,
    pub id: String,
    pub version: semver::Version,
    pub name: String,
    pub brand: String,
    pub kind: Kind,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,
    /// What changed in this version, plain text of at most
    /// [`MAX_CHANGELOG`] characters.
    #[serde(default)]
    pub changelog: Option<String>,
    pub game: GameTarget,
    pub paint_job: PaintJob,
}

/// What a version changes compared with an older one, by texture name and
/// in package order; parts are matched by id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ManifestDiff {
    /// Kept textures whose layout version differs.
    pub layout_changed: Vec<String>,
    /// Kept textures whose size differs.
    pub resized: Vec<String>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

impl ManifestDiff {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// What `new` changes compared with `old`.
pub fn diff(old: &Manifest, new: &Manifest) -> ManifestDiff {
    let mut out = ManifestDiff::default();
    for (_, part) in new.paint_job.parts() {
        match old.paint_job.part(&part.id) {
            None => out.added.push(part.name.clone()),
            Some((_, before)) => {
                if before.texture.layout_version != part.texture.layout_version {
                    out.layout_changed.push(part.name.clone());
                }
                if before.texture.size != part.texture.size {
                    out.resized.push(part.name.clone());
                }
            }
        }
    }
    out.removed = old
        .paint_job
        .parts()
        .filter(|(_, p)| new.paint_job.part(&p.id).is_none())
        .map(|(_, p)| p.name.clone())
        .collect();
    out
}

/// Why a package is refused. Messages are built by the application.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageError {
    NotAZip,
    NoManifest,
    /// Invalid JSON or a missing / mistyped field (serde message).
    BadManifest(String),
    NewerFormat(u32),
    BadId(String),
    BadGamePath(String),
    NoMainTexture,
    /// Two parts share an id.
    DuplicatePart(String),
    /// An accessory, or one of several main textures, has no game id
    /// (the part's name).
    MissingGameIds(String),
    BadGameId(String),
    /// A game id appears twice among the main textures or among the
    /// accessories, or an `also_paints` id appears twice or is also an
    /// accessory's game id.
    DuplicateGameId(String),
    /// An accessory has `also_paints` (the accessory's name).
    BadAlsoPaints(String),
    /// A texture's size is not a `[width, height]` pair of allowed sides
    /// (the size as written in the manifest).
    BadSize {
        texture: String,
        size: String,
    },
    UnsafePath(String),
    TooLarge,
    MissingTemplate {
        texture: String,
        path: String,
    },
    BadTemplate {
        texture: String,
        path: String,
    },
    TemplateTooLarge {
        texture: String,
    },
    /// The changelog is longer than [`MAX_CHANGELOG`] characters.
    BadChangelog,
}

/// The longest changelog a package may carry, in characters.
pub const MAX_CHANGELOG: usize = 1000;

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PackageError {}

/// Kind of template image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageKind {
    Png,
    Svg,
}

/// A template image read from a package.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateImage {
    pub bytes: Vec<u8>,
    pub kind: ImageKind,
    /// Natural size in pixels.
    pub width: f64,
    pub height: f64,
}

/// A validated package: its manifest and every template, in package order.
#[derive(Clone, Debug)]
pub struct Package {
    pub manifest: Manifest,
    /// `(part id, image)`.
    pub templates: Vec<(String, TemplateImage)>,
}

impl Package {
    /// The template of part `id`.
    pub fn template(&self, id: &str) -> Option<&TemplateImage> {
        self.templates.iter().find(|(p, _)| p == id).map(|(_, i)| i)
    }

    /// Reads and fully validates a package.
    pub fn read(bytes: &[u8]) -> Result<Package, PackageError> {
        let mut zip = open(bytes)?;
        check_entries(&mut zip)?;
        let manifest = parse_manifest(&mut zip)?;
        validate(&manifest)?;
        let mut templates = Vec::new();
        for (_, part) in manifest.paint_job.parts() {
            let image = read_template(&mut zip, part)?;
            templates.push((part.id.clone(), image));
        }
        Ok(Package {
            manifest,
            templates,
        })
    }

    /// Reads only the manifest (listing an installed library).
    pub fn read_manifest(bytes: &[u8]) -> Result<Manifest, PackageError> {
        let mut zip = open(bytes)?;
        let manifest = parse_manifest(&mut zip)?;
        validate(&manifest)?;
        Ok(manifest)
    }
}

type Zip<'a> = zip::ZipArchive<Cursor<&'a [u8]>>;

fn open(bytes: &[u8]) -> Result<Zip<'_>, PackageError> {
    zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| PackageError::NotAZip)
}

/// Every entry path is relative without `..`; the total size is bounded.
fn check_entries(zip: &mut Zip<'_>) -> Result<(), PackageError> {
    let mut total: u64 = 0;
    for i in 0..zip.len() {
        let entry = zip.by_index_raw(i).map_err(|_| PackageError::NotAZip)?;
        let name = entry.name().to_owned();
        if !is_safe_path(&name) {
            return Err(PackageError::UnsafePath(name));
        }
        total = total.saturating_add(entry.size());
    }
    if total > MAX_UNCOMPRESSED {
        return Err(PackageError::TooLarge);
    }
    Ok(())
}

/// Relative, `/`-separated, no `..`, no drive or root.
pub fn is_safe_path(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.starts_with('\\')
        && !name.contains(':')
        && name.split(['/', '\\']).all(|part| part != "..")
}

fn read_entry(zip: &mut Zip<'_>, name: &str) -> Option<Vec<u8>> {
    let mut entry = zip.by_name(name).ok()?;
    if entry.size() > MAX_UNCOMPRESSED {
        return None;
    }
    let mut out = Vec::with_capacity(entry.size() as usize);
    entry
        .by_ref()
        .take(MAX_UNCOMPRESSED)
        .read_to_end(&mut out)
        .ok()?;
    Some(out)
}

fn parse_manifest(zip: &mut Zip<'_>) -> Result<Manifest, PackageError> {
    let bytes = read_entry(zip, MANIFEST).ok_or(PackageError::NoManifest)?;
    // The format number first, so a newer package is reported as such
    // rather than as an unreadable manifest.
    #[derive(Deserialize)]
    struct Head {
        format: u32,
    }
    if let Ok(head) = serde_json::from_slice::<Head>(&bytes)
        && head.format > FORMAT
    {
        return Err(PackageError::NewerFormat(head.format));
    }
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|e| PackageError::BadManifest(e.to_string()))?;
    manifest_from_value(&value)
}

/// Reads a manifest from its JSON. A texture size that isn't a pair of
/// numbers (an older single size) is reported as a bad size of that
/// texture rather than as an unreadable manifest.
pub fn manifest_from_value(value: &Value) -> Result<Manifest, PackageError> {
    let parts = ["main", "accessories"]
        .iter()
        .filter_map(|key| value.get("paint_job")?.get(key)?.as_array())
        .flatten();
    for part in parts {
        let Some(size) = part.get("texture").and_then(|t| t.get("size")) else {
            continue;
        };
        let pair = size.as_array().is_some_and(|a| {
            a.len() == 2
                && a.iter()
                    .all(|v| v.as_u64().is_some_and(|n| u32::try_from(n).is_ok()))
        });
        if !pair {
            let name = part.get("name").and_then(Value::as_str).unwrap_or("");
            return Err(PackageError::BadSize {
                texture: name.to_owned(),
                size: size.to_string(),
            });
        }
    }
    Manifest::deserialize(value).map_err(|e| PackageError::BadManifest(e.to_string()))
}

/// Lowercase words separated by dots, at least two.
pub fn is_valid_id(id: &str) -> bool {
    let words: Vec<&str> = id.split('.').collect();
    words.len() >= 2
        && words.iter().all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        })
}

/// Words of a–z, 0–9 and `_` separated by dots (the game's unit names;
/// no length limit, real accessory ids exceed the 12 characters of a token).
pub fn is_valid_game_name(name: &str) -> bool {
    name.split('.').all(|w| {
        !w.is_empty()
            && w.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    })
}

fn validate(m: &Manifest) -> Result<(), PackageError> {
    if m.format > FORMAT {
        return Err(PackageError::NewerFormat(m.format));
    }
    if !is_valid_id(&m.id) {
        return Err(PackageError::BadId(m.id.clone()));
    }
    if !is_valid_game_name(&m.game.path) {
        return Err(PackageError::BadGamePath(m.game.path.clone()));
    }
    if m.changelog
        .as_ref()
        .is_some_and(|c| c.chars().count() > MAX_CHANGELOG)
    {
        return Err(PackageError::BadChangelog);
    }
    let job = &m.paint_job;
    if job.main.is_empty() {
        return Err(PackageError::NoMainTexture);
    }
    let mut ids = HashSet::new();
    for (_, part) in job.parts() {
        if !ids.insert(&part.id) {
            return Err(PackageError::DuplicatePart(part.id.clone()));
        }
        validate_part(part)?;
    }
    // Several main textures each say which cabins they are for; an
    // accessory always says which accessories it covers.
    validate_game_ids(&job.main, job.main.len() > 1)?;
    validate_game_ids(&job.accessories, true)?;
    validate_also_paints(job)
}

/// Accessory ids painted with a main texture: only on main textures, well
/// formed, listed once in the package and not also an accessory's game id
/// (the game would paint them twice).
fn validate_also_paints(job: &PaintJob) -> Result<(), PackageError> {
    if let Some(part) = job.accessories.iter().find(|p| !p.also_paints.is_empty()) {
        return Err(PackageError::BadAlsoPaints(part.name.clone()));
    }
    let accessory_ids: HashSet<&String> =
        job.accessories.iter().flat_map(|p| &p.game_ids).collect();
    let mut seen = HashSet::new();
    for id in job.main.iter().flat_map(|p| &p.also_paints) {
        if !is_valid_game_name(id) {
            return Err(PackageError::BadGameId(id.clone()));
        }
        if !seen.insert(id) || accessory_ids.contains(id) {
            return Err(PackageError::DuplicateGameId(id.clone()));
        }
    }
    Ok(())
}

fn validate_part(part: &Part) -> Result<(), PackageError> {
    let size = part.texture.size;
    if !size.is_allowed() {
        return Err(PackageError::BadSize {
            texture: part.name.clone(),
            size: format!("[{}, {}]", size.width, size.height),
        });
    }
    if !is_safe_path(&part.texture.template) {
        return Err(PackageError::UnsafePath(part.texture.template.clone()));
    }
    Ok(())
}

/// Game ids of a list of parts: well formed, unique in the list, and
/// present on each part when `required`.
fn validate_game_ids(parts: &[Part], required: bool) -> Result<(), PackageError> {
    let mut seen = HashSet::new();
    for part in parts {
        if required && part.game_ids.is_empty() {
            return Err(PackageError::MissingGameIds(part.name.clone()));
        }
        for id in &part.game_ids {
            if !is_valid_game_name(id) {
                return Err(PackageError::BadGameId(id.clone()));
            }
            if !seen.insert(id) {
                return Err(PackageError::DuplicateGameId(id.clone()));
            }
        }
    }
    Ok(())
}

/// Options to check an SVG and read its size: texts are left out without
/// looking for fonts (the default resolver would warn for every `<text>`;
/// the application renders templates with its own fonts).
fn size_only_options() -> resvg::usvg::Options<'static> {
    resvg::usvg::Options {
        font_resolver: resvg::usvg::FontResolver {
            select_font: Box::new(|_, _| None),
            select_fallback: Box::new(|_, _, _| None),
        },
        ..resvg::usvg::Options::default()
    }
}

/// The natural size of a PNG or SVG image in pixels, or `None` when it
/// can't be read. Only the PNG header is read.
pub fn image_size(kind: ImageKind, bytes: &[u8]) -> Option<(f64, f64)> {
    match kind {
        ImageKind::Png => {
            let reader =
                image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
            let (w, h) = reader.into_dimensions().ok()?;
            Some((f64::from(w), f64::from(h)))
        }
        ImageKind::Svg => {
            let tree = resvg::usvg::Tree::from_data(bytes, &size_only_options()).ok()?;
            let s = tree.size();
            Some((f64::from(s.width()), f64::from(s.height())))
        }
    }
}

fn read_template(zip: &mut Zip<'_>, part: &Part) -> Result<TemplateImage, PackageError> {
    let t = &part.texture;
    let bytes = read_entry(zip, &t.template).ok_or_else(|| PackageError::MissingTemplate {
        texture: part.name.clone(),
        path: t.template.clone(),
    })?;
    let bad = || PackageError::BadTemplate {
        texture: part.name.clone(),
        path: t.template.clone(),
    };
    let lower = t.template.to_ascii_lowercase();
    let kind = if lower.ends_with(".png") {
        ImageKind::Png
    } else if lower.ends_with(".svg") {
        ImageKind::Svg
    } else {
        return Err(bad());
    };
    let (width, height) = image_size(kind, &bytes).ok_or_else(bad)?;
    if kind == ImageKind::Png
        && (width > f64::from(MAX_IMAGE_SIDE) || height > f64::from(MAX_IMAGE_SIDE))
    {
        return Err(PackageError::TemplateTooLarge {
            texture: part.name.clone(),
        });
    }
    Ok(TemplateImage {
        bytes,
        kind,
        width,
        height,
    })
}

#[cfg(test)]
mod tests;
