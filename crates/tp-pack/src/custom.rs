//! Custom vehicles: a vehicle described in the application from template
//! files (the Custom Vehicle dialog), packed as `tpv` packs a folder. The
//! form's rules live here; the application shows them in its language.

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use serde_json::{Map, Value, json};
use tp_vehicles::{Game, ImageKind, Kind, Manifest, Package, Role, TextureSize};

use crate::dds::{self, DdsError};
use crate::{PackError, Packed};

/// Start of the ids of vehicles made in TruckPaint.
pub const ID_PREFIX: &str = "custom.";

/// Whether `id` is a vehicle made in TruckPaint (it offers New Version…).
pub fn is_custom(id: &str) -> bool {
    id.starts_with(ID_PREFIX)
}

/// File format of a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateFormat {
    Png,
    Svg,
    Dds,
}

impl TemplateFormat {
    pub fn extension(self) -> &'static str {
        match self {
            TemplateFormat::Png => "png",
            TemplateFormat::Svg => "svg",
            TemplateFormat::Dds => "dds",
        }
    }

    /// The format of `file_name`, from its extension.
    pub fn of(file_name: &str) -> Option<Self> {
        let lower = file_name.to_ascii_lowercase();
        [Self::Png, Self::Svg, Self::Dds]
            .into_iter()
            .find(|f| lower.ends_with(&format!(".{}", f.extension())))
    }
}

/// A template file chosen by the player.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateFile {
    /// The file's name, without its folder.
    pub file_name: String,
    pub bytes: Arc<[u8]>,
    pub format: TemplateFormat,
    pub width: u32,
    pub height: u32,
}

impl TemplateFile {
    /// The texture size for this image: its width and height, each rounded
    /// to the nearest allowed side when it isn't one (the larger one on a
    /// tie).
    pub fn default_size(&self) -> TextureSize {
        TextureSize::new(nearest_side(self.width), nearest_side(self.height))
    }

    /// Whether the image must be scaled to fill a texture of `size`.
    pub fn is_scaled_to(&self, size: TextureSize) -> bool {
        (self.width, self.height) != (size.width, size.height)
    }
}

/// The allowed texture side nearest to `side` (the larger one on a tie).
fn nearest_side(side: u32) -> u32 {
    let side = i64::from(side);
    TextureSize::allowed_sides()
        .min_by_key(|s| ((i64::from(*s) - side).abs(), -i64::from(*s)))
        .expect("sizes")
}

/// Why a file can't be a template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateError {
    /// Not a PNG, SVG or DDS file, or unreadable as one.
    NotAnImage,
    /// A DDS file in a format that isn't supported, named ("BC7").
    UnsupportedDds(String),
    /// A DDS file whose data is cut short.
    Damaged,
    /// Larger than [`tp_vehicles::MAX_IMAGE_SIDE`] on a side.
    TooLarge,
}

/// Reads the size of a template file without decoding it, and checks
/// that a DDS file is in a format the packer converts.
pub fn probe(file_name: &str, bytes: Arc<[u8]>) -> Result<TemplateFile, TemplateError> {
    let format = TemplateFormat::of(file_name).ok_or(TemplateError::NotAnImage)?;
    let (width, height) = match format {
        TemplateFormat::Dds => dds::probe(&bytes).map_err(|e| match e {
            DdsError::TooLarge => TemplateError::TooLarge,
            DdsError::Truncated | DdsError::TruncatedDx10Header => TemplateError::Damaged,
            e => e
                .unsupported_format()
                .map_or(TemplateError::NotAnImage, TemplateError::UnsupportedDds),
        })?,
        TemplateFormat::Png | TemplateFormat::Svg => {
            let kind = if format == TemplateFormat::Png {
                ImageKind::Png
            } else {
                ImageKind::Svg
            };
            let (w, h) = tp_vehicles::image_size(kind, &bytes).ok_or(TemplateError::NotAnImage)?;
            (w.ceil() as u32, h.ceil() as u32)
        }
    };
    if format != TemplateFormat::Svg
        && (width > tp_vehicles::MAX_IMAGE_SIDE || height > tp_vehicles::MAX_IMAGE_SIDE)
    {
        return Err(TemplateError::TooLarge);
    }
    let file_name = file_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(file_name)
        .to_owned();
    Ok(TemplateFile {
        file_name,
        bytes,
        format,
        width,
        height,
    })
}

/// A texture name from a file name: its stem, with `_` and `-` as spaces.
pub fn name_from_file(file_name: &str) -> String {
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    stem.replace(['_', '-'], " ").trim().to_owned()
}

/// An id word: lowercase, characters outside `a`–`z`, `0`–`9`, `_` and
/// `-` replaced by `_`, repeated `_` collapsed and trimmed; `fallback`
/// when nothing is left.
pub fn slug(text: &str, fallback: &str) -> String {
    let mut out = String::new();
    for c in text.to_lowercase().chars() {
        let c = if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
            c
        } else {
            '_'
        };
        if !(c == '_' && out.ends_with('_')) {
            out.push(c);
        }
    }
    let out = out.trim_matches('_');
    if out.is_empty() {
        fallback.to_owned()
    } else {
        out.to_owned()
    }
}

/// The id of a custom vehicle: `custom.<brand>.<name>`.
pub fn vehicle_id(brand: &str, name: &str) -> String {
    format!(
        "{ID_PREFIX}{}.{}",
        slug(brand, "vehicle"),
        slug(name, "vehicle")
    )
}

/// One texture row of the form.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomPart {
    /// Texture id and layout version kept from the version a new version
    /// is made from.
    pub base: Option<(String, u32)>,
    /// A new template was picked for a kept texture.
    pub replaced: bool,
    pub name: String,
    pub role: Role,
    /// Game ids as typed, separated by commas or spaces.
    pub game_ids: String,
    pub size: TextureSize,
    pub template: TemplateFile,
}

impl CustomPart {
    /// A new row for `template`, named after its file.
    pub fn new(template: TemplateFile, role: Role) -> Self {
        Self {
            base: None,
            replaced: false,
            name: name_from_file(&template.file_name),
            role,
            game_ids: String::new(),
            size: template.default_size(),
            template,
        }
    }

    /// The typed game ids.
    pub fn game_ids(&self) -> Vec<String> {
        self.game_ids
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// Whether the template will be scaled to the texture's size.
    pub fn is_scaled(&self) -> bool {
        self.template.is_scaled_to(self.size)
    }

    /// Picks another template; the name, role and game ids are kept.
    pub fn replace(&mut self, template: TemplateFile) {
        self.size = template.default_size();
        self.template = template;
        self.replaced = self.base.is_some();
    }
}

/// What the form can't accept yet; a row index for a row's problem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    NameMissing,
    BrandMissing,
    BadGamePath,
    BadGameVersions,
    BadVersion,
    /// The version must be higher than this installed one.
    VersionNotHigher(semver::Version),
    /// A vehicle with this id is installed: make a new version of it.
    AlreadyInstalled(String),
    NoMainTexture,
    /// A trailer has one main texture.
    TrailerMainTextures,
    TextureNameMissing(usize),
    /// One of several main textures without cabin names.
    MissingCabins(usize),
    /// An accessory without accessory ids.
    MissingAccessoryIds(usize),
    BadGameId(usize, String),
    DuplicateGameId(usize, String),
    /// "What changed" is longer than [`tp_vehicles::MAX_CHANGELOG`] characters.
    ChangelogTooLong,
}

impl Problem {
    /// The row the problem is about.
    pub fn row(&self) -> Option<usize> {
        match self {
            Problem::TextureNameMissing(i)
            | Problem::MissingCabins(i)
            | Problem::MissingAccessoryIds(i)
            | Problem::BadGameId(i, _)
            | Problem::DuplicateGameId(i, _) => Some(*i),
            _ => None,
        }
    }
}

/// A custom vehicle as the form describes it.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomVehicle {
    /// The version a new version is made from; `None` for a new vehicle.
    pub base: Option<Manifest>,
    /// The version to build, as typed (1.0.0 for a new vehicle).
    pub version: String,
    pub name: String,
    pub brand: String,
    pub kind: Kind,
    pub game: Game,
    pub path: String,
    /// Supported game versions as typed; empty means any.
    pub versions: String,
    pub alt_uv: bool,
    pub colour_picker: bool,
    pub rows: Vec<CustomPart>,
    /// What changed in a new version; empty writes no changelog.
    pub changelog: String,
}

impl CustomVehicle {
    /// An empty truck form for `game`.
    pub fn new(game: Game) -> Self {
        Self {
            base: None,
            version: "1.0.0".into(),
            name: String::new(),
            brand: String::new(),
            kind: Kind::Truck,
            game,
            path: String::new(),
            versions: String::new(),
            alt_uv: false,
            colour_picker: false,
            rows: Vec::new(),
            changelog: String::new(),
        }
    }

    /// A new version of an installed custom vehicle, filled in from it:
    /// the next minor version, and every texture kept.
    pub fn from_package(package: &Package) -> Self {
        let m = &package.manifest;
        let g = &m.game;
        let rows = m
            .paint_job
            .parts()
            .filter_map(|(role, part)| {
                let image = package.template(&part.id)?;
                let format = match image.kind {
                    ImageKind::Png => TemplateFormat::Png,
                    ImageKind::Svg => TemplateFormat::Svg,
                };
                let file_name = part
                    .texture
                    .template
                    .rsplit('/')
                    .next()
                    .unwrap_or(&part.texture.template)
                    .to_owned();
                Some(CustomPart {
                    base: Some((part.id.clone(), part.texture.layout_version)),
                    replaced: false,
                    name: part.name.clone(),
                    role,
                    game_ids: part.game_ids.join(", "),
                    size: part.texture.size,
                    template: TemplateFile {
                        file_name,
                        bytes: image.bytes.clone().into(),
                        format,
                        width: image.width.ceil() as u32,
                        height: image.height.ceil() as u32,
                    },
                })
            })
            .collect();
        Self {
            version: semver::Version::new(m.version.major, m.version.minor + 1, 0).to_string(),
            name: m.name.clone(),
            brand: m.brand.clone(),
            kind: m.kind,
            game: g.id,
            path: g.path.clone(),
            versions: if g.versions == semver::VersionReq::STAR {
                String::new()
            } else {
                g.versions.to_string()
            },
            alt_uv: g.alt_uv,
            colour_picker: g.colour_picker,
            rows,
            // Starts empty: the previous version's text describes that version.
            changelog: String::new(),
            base: Some(m.clone()),
        }
    }

    /// The vehicle's id: kept for a new version, else made from the brand
    /// and name.
    pub fn id(&self) -> String {
        self.base
            .as_ref()
            .map_or_else(|| vehicle_id(&self.brand, &self.name), |b| b.id.clone())
    }

    /// Adds a row for `template`: a main texture while there is none, an
    /// accessory otherwise.
    pub fn add(&mut self, template: TemplateFile) {
        let role = if self.rows.iter().any(|r| r.role == Role::Main) {
            Role::Accessory
        } else {
            Role::Main
        };
        self.rows.push(CustomPart::new(template, role));
    }

    /// Everything that prevents building the package. `installed` lists
    /// the installed vehicle versions (id, version).
    pub fn problems(&self, installed: &[(String, semver::Version)]) -> Vec<Problem> {
        let mut out = Vec::new();
        if self.changelog.trim().chars().count() > tp_vehicles::MAX_CHANGELOG {
            out.push(Problem::ChangelogTooLong);
        }
        if self.name.trim().is_empty() {
            out.push(Problem::NameMissing);
        }
        if self.brand.trim().is_empty() {
            out.push(Problem::BrandMissing);
        }
        if !tp_vehicles::is_valid_game_name(self.path.trim()) {
            out.push(Problem::BadGamePath);
        }
        let versions = self.versions.trim();
        if !versions.is_empty() && semver::VersionReq::parse(versions).is_err() {
            out.push(Problem::BadGameVersions);
        }
        let id = self.id();
        match &self.base {
            Some(base) => match semver::Version::parse(self.version.trim()) {
                Err(_) => out.push(Problem::BadVersion),
                Ok(version) => {
                    let newest = installed
                        .iter()
                        .filter(|(i, _)| *i == id)
                        .map(|(_, v)| v)
                        .chain([&base.version])
                        .max()
                        .expect("the base version");
                    if version <= *newest {
                        out.push(Problem::VersionNotHigher(newest.clone()));
                    }
                }
            },
            None => {
                if !self.name.trim().is_empty()
                    && !self.brand.trim().is_empty()
                    && installed.iter().any(|(i, _)| *i == id)
                {
                    out.push(Problem::AlreadyInstalled(id));
                }
            }
        }
        let main = self.rows.iter().filter(|r| r.role == Role::Main).count();
        if main == 0 {
            out.push(Problem::NoMainTexture);
        } else if self.kind == Kind::Trailer && main > 1 {
            out.push(Problem::TrailerMainTextures);
        }
        let mut seen: [HashSet<String>; 2] = Default::default();
        for (i, row) in self.rows.iter().enumerate() {
            if row.name.trim().is_empty() {
                out.push(Problem::TextureNameMissing(i));
            }
            let ids = row.game_ids();
            if ids.is_empty() {
                match row.role {
                    Role::Main if main > 1 => out.push(Problem::MissingCabins(i)),
                    Role::Accessory => out.push(Problem::MissingAccessoryIds(i)),
                    Role::Main => {}
                }
            }
            let seen = &mut seen[usize::from(row.role == Role::Accessory)];
            for game_id in ids {
                if !tp_vehicles::is_valid_game_name(&game_id) {
                    out.push(Problem::BadGameId(i, game_id));
                } else if !seen.insert(game_id.clone()) {
                    out.push(Problem::DuplicateGameId(i, game_id));
                }
            }
        }
        out
    }

    /// The texture id of each row: kept from the base version, or made
    /// from its name, unique within the vehicle and never the id of a
    /// texture of the base version.
    pub fn part_ids(&self) -> Vec<String> {
        let mut taken: HashSet<String> = self
            .rows
            .iter()
            .filter_map(|r| r.base.as_ref().map(|(id, _)| id.clone()))
            .collect();
        if let Some(base) = &self.base {
            taken.extend(base.paint_job.parts().map(|(_, p)| p.id.clone()));
        }
        self.rows
            .iter()
            .map(|row| match &row.base {
                Some((id, _)) => id.clone(),
                None => {
                    let stem = slug(&row.name, "texture");
                    let mut id = stem.clone();
                    let mut n = 2;
                    while taken.contains(&id) {
                        id = format!("{stem}_{n}");
                        n += 1;
                    }
                    taken.insert(id.clone());
                    id
                }
            })
            .collect()
    }

    /// The layout version of a row: one more than before when its
    /// template was replaced, unchanged otherwise, 1 for a new texture.
    pub fn layout_version(row: &CustomPart) -> u32 {
        match row.base {
            Some((_, layout)) if row.replaced => layout + 1,
            Some((_, layout)) => layout,
            None => 1,
        }
    }

    fn template_path(id: &str, row: &CustomPart) -> String {
        format!("templates/{id}.{}", row.template.format.extension())
    }

    /// The package's `vehicle.json`, templates given by their source
    /// format (DDS is converted when packing).
    pub fn manifest_json(&self) -> Value {
        let ids = self.part_ids();
        let part = |(id, row): (&String, &CustomPart)| {
            json!({
                "id": id,
                "name": row.name.trim(),
                "game_ids": row.game_ids(),
                "texture": {
                    "size": row.size,
                    "template": Self::template_path(id, row),
                    "layout_version": Self::layout_version(row),
                },
            })
        };
        let of_role = |role: Role| -> Vec<Value> {
            ids.iter()
                .zip(&self.rows)
                .filter(|(_, row)| row.role == role)
                .map(part)
                .collect()
        };
        let versions = self.versions.trim();
        let mut game = json!({
            "id": self.game.code(),
            "versions": if versions.is_empty() { "*" } else { versions },
            "path": self.path.trim(),
            "alt_uv": self.alt_uv,
            "colour_picker": self.colour_picker,
        });
        let mut m = Map::new();
        m.insert("format".into(), json!(tp_vehicles::FORMAT));
        m.insert("id".into(), json!(self.id()));
        m.insert("version".into(), json!(self.version.trim()));
        m.insert("name".into(), json!(self.name.trim()));
        m.insert("brand".into(), json!(self.brand.trim()));
        m.insert("kind".into(), json!(self.kind.code()));
        if !self.changelog.trim().is_empty() {
            m.insert("changelog".into(), json!(self.changelog.trim()));
        }
        if let Some(base) = &self.base {
            if !base.authors.is_empty() {
                m.insert("authors".into(), json!(base.authors));
            }
            for (key, value) in [
                ("license", &base.license),
                ("homepage", &base.homepage),
                ("description", &base.description),
            ] {
                if let Some(value) = value {
                    m.insert(key.into(), json!(value));
                }
            }
            if !base.game.requires.is_empty() {
                game["requires"] = base
                    .game
                    .requires
                    .iter()
                    .map(|r| match &r.version {
                        Some(v) => json!({ "name": r.name, "version": v }),
                        None => json!({ "name": r.name }),
                    })
                    .collect();
            }
        }
        m.insert("game".into(), game);
        m.insert(
            "paint_job".into(),
            json!({ "main": of_role(Role::Main), "accessories": of_role(Role::Accessory) }),
        );
        Value::Object(m)
    }

    /// The template files, by their path in the package.
    pub fn files(&self) -> BTreeMap<String, Vec<u8>> {
        self.part_ids()
            .iter()
            .zip(&self.rows)
            .map(|(id, row)| (Self::template_path(id, row), row.template.bytes.to_vec()))
            .collect()
    }

    /// Builds and validates the package, calling `converted` after each
    /// DDS template is converted.
    pub fn pack(&self, converted: &mut dyn FnMut()) -> Result<Packed, PackError> {
        crate::pack_entries_with(self.manifest_json(), self.files(), converted)
    }

    /// How many templates packing converts (for progress).
    pub fn conversions(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| r.template.format == TemplateFormat::Dds)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(side: u32) -> TemplateFile {
        let image = image::RgbaImage::from_pixel(side, side, image::Rgba([9, 9, 9, 255]));
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        probe("templates/cabin.png", bytes.into()).unwrap()
    }

    fn dds_file(name: &str) -> TemplateFile {
        let bytes = dds::build::bc(&dds::build::quadrants(), texpresso::Format::Bc3, b"DXT5");
        probe(name, bytes.into()).unwrap()
    }

    fn sized(width: u32, height: u32) -> TemplateFile {
        TemplateFile {
            width,
            height,
            ..png(4)
        }
    }

    /// A valid ETS2 truck: one main texture without game ids and one
    /// accessory.
    fn truck() -> CustomVehicle {
        let mut v = CustomVehicle::new(Game::Ets2);
        v.name = "R 2024".into();
        v.brand = "Scania".into();
        v.path = "scania.r_2024".into();
        v.add(dds_file("cabin.dds"));
        v.add(png(8));
        v.rows[1].name = "Mirrors".into();
        v.rows[1].game_ids = "mirror.painted".into();
        v
    }

    #[test]
    fn probing_templates() {
        let file = png(16);
        assert_eq!(file.file_name, "cabin.png");
        assert_eq!(
            (file.width, file.height, file.format),
            (16, 16, TemplateFormat::Png)
        );
        let dds = dds_file("glass.DDS");
        assert_eq!((dds.width, dds.format), (16, TemplateFormat::Dds));
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="512"/>"#;
        let svg = probe("a.svg", svg.to_vec().into()).unwrap();
        assert_eq!((svg.width, svg.height), (1024, 512));
        assert_eq!(svg.default_size(), TextureSize::new(1024, 512));

        let bc7 = dds::build::dx10(&dds::build::quadrants(), texpresso::Format::Bc3, 98);
        assert_eq!(
            probe("glass.dds", bc7.into()),
            Err(TemplateError::UnsupportedDds("BC7".into()))
        );
        assert_eq!(
            probe("notes.txt", b"hello".to_vec().into()),
            Err(TemplateError::NotAnImage)
        );
        assert_eq!(
            probe("fake.png", b"hello".to_vec().into()),
            Err(TemplateError::NotAnImage)
        );
        let mut cut = dds::build::bc(&dds::build::quadrants(), texpresso::Format::Bc1, b"DXT1");
        cut.truncate(cut.len() - 1);
        assert_eq!(probe("cut.dds", cut.into()), Err(TemplateError::Damaged));
    }

    #[test]
    fn default_sizes() {
        let size = |w, h| TextureSize::new(w, h);
        assert_eq!(sized(4096, 4096).default_size(), size(4096, 4096));
        assert_eq!(sized(3000, 3000).default_size(), size(2048, 2048));
        assert_eq!(sized(3072, 3072).default_size(), size(4096, 4096));
        assert_eq!(sized(100, 100).default_size(), size(128, 128));
        assert_eq!(sized(40, 20000).default_size(), size(64, 8192));
        assert_eq!(sized(8192, 1024).default_size(), size(8192, 1024));
    }

    #[test]
    fn not_square() {
        // A 4096×2048 image keeps its shape, without a warning.
        let mut v = CustomVehicle::new(Game::Ets2);
        v.add(sized(4096, 2048));
        assert_eq!(v.rows[0].size, TextureSize::new(4096, 2048));
        assert_eq!(v.rows[0].size.label(), "4096 × 2048");
        assert!(!v.rows[0].is_scaled());
    }

    #[test]
    fn size_that_isnt_a_power_of_two() {
        // 4000×1000 rounds to 4096 × 1024, and the template is scaled.
        let mut v = CustomVehicle::new(Game::Ets2);
        v.add(sized(4000, 1000));
        assert_eq!(v.rows[0].size, TextureSize::new(4096, 1024));
        assert!(v.rows[0].is_scaled());
        // A size chosen by hand that differs from the image also warns.
        v.rows[0].size = TextureSize::new(4096, 4096);
        assert!(v.rows[0].is_scaled());
    }

    #[test]
    fn rows_from_files() {
        let mut v = CustomVehicle::new(Game::Ets2);
        v.add(dds_file("highline_8x4.dds"));
        v.add(dds_file("side-skirts.dds"));
        assert_eq!(v.rows[0].name, "highline 8x4");
        assert_eq!(v.rows[0].role, Role::Main);
        assert_eq!(v.rows[1].name, "side skirts");
        assert_eq!(v.rows[1].role, Role::Accessory);
        v.rows[1].game_ids = "sideskirt.a, sideskirt.b  sideskirt.c,".into();
        assert_eq!(
            v.rows[1].game_ids(),
            ["sideskirt.a", "sideskirt.b", "sideskirt.c"]
        );
        // Replacing keeps the name, role and game ids; a new row isn't
        // "replaced".
        v.rows[1].replace(png(512));
        assert_eq!(v.rows[1].name, "side skirts");
        assert_eq!(v.rows[1].role, Role::Accessory);
        assert_eq!(v.rows[1].game_ids().len(), 3);
        assert_eq!(v.rows[1].size, TextureSize::new(512, 512));
        assert!(!v.rows[1].replaced);
    }

    #[test]
    fn ids() {
        assert_eq!(vehicle_id("Scania", "R 2024"), "custom.scania.r_2024");
        assert_eq!(vehicle_id("Ä", "  Big--Rig!! "), "custom.vehicle.big--rig");
        assert_eq!(slug("__a__b__", "x"), "a_b");
        assert!(tp_vehicles::is_valid_id(&vehicle_id("Ä", "Ö")));

        let mut v = truck();
        v.add(png(8));
        v.rows[2].name = "Mirrors".into();
        v.rows[0].name = String::new();
        assert_eq!(v.part_ids(), ["texture", "mirrors", "mirrors_2"]);
    }

    #[test]
    fn valid_forms() {
        let v = truck();
        assert_eq!(v.problems(&[]), []);
        let mut trailer = truck();
        trailer.kind = Kind::Trailer;
        trailer.add(png(8));
        trailer.rows[2].game_ids = "body.curtain".into();
        assert_eq!(trailer.problems(&[]), []);
        let mut versions = truck();
        versions.versions = ">=1.53, <1.55".into();
        assert_eq!(versions.problems(&[]), []);
    }

    #[test]
    fn each_problem() {
        let check = |edit: &dyn Fn(&mut CustomVehicle), expected: Problem| {
            let mut v = truck();
            edit(&mut v);
            assert!(
                v.problems(&[]).contains(&expected),
                "{expected:?} in {:?}",
                v.problems(&[])
            );
        };
        check(&|v| v.name = " ".into(), Problem::NameMissing);
        check(&|v| v.brand.clear(), Problem::BrandMissing);
        check(&|v| v.path = "Scania R".into(), Problem::BadGamePath);
        check(&|v| v.path.clear(), Problem::BadGamePath);
        check(&|v| v.versions = "soon".into(), Problem::BadGameVersions);
        check(
            &|v| v.rows[0].role = Role::Accessory,
            Problem::NoMainTexture,
        );
        check(
            &|v| {
                v.kind = Kind::Trailer;
                v.rows[1].role = Role::Main;
            },
            Problem::TrailerMainTextures,
        );
        check(&|v| v.rows[1].name.clear(), Problem::TextureNameMissing(1));
        check(
            &|v| {
                v.rows[1].role = Role::Main;
                v.rows[1].game_ids = "highline".into();
            },
            Problem::MissingCabins(0),
        );
        check(
            &|v| v.rows[1].game_ids.clear(),
            Problem::MissingAccessoryIds(1),
        );
        check(
            &|v| v.rows[1].game_ids = "Mirror".into(),
            Problem::BadGameId(1, "Mirror".into()),
        );
        check(
            &|v| v.rows[1].game_ids = "mirror.painted, mirror.painted".into(),
            Problem::DuplicateGameId(1, "mirror.painted".into()),
        );
        // The same id may be a cabin and an accessory id.
        let mut v = truck();
        v.rows[0].game_ids = "mirror.painted".into();
        assert_eq!(v.problems(&[]), []);

        let installed = [(
            "custom.scania.r_2024".to_owned(),
            semver::Version::new(1, 0, 0),
        )];
        assert_eq!(
            truck().problems(&installed),
            [Problem::AlreadyInstalled("custom.scania.r_2024".into())]
        );
        assert_eq!(Problem::MissingCabins(3).row(), Some(3));
        assert_eq!(Problem::NameMissing.row(), None);
    }

    fn package(v: &CustomVehicle) -> Package {
        Package::read(&v.pack(&mut || {}).unwrap().bytes).unwrap()
    }

    #[test]
    fn packing_a_custom_truck() {
        let v = truck();
        let mut converted = 0;
        let packed = v.pack(&mut || converted += 1).unwrap();
        assert_eq!(converted, v.conversions());
        assert_eq!(packed.file_name(), "custom.scania.r_2024-1.0.0.tpv");
        let p = Package::read(&packed.bytes).unwrap();
        let m = &p.manifest;
        assert_eq!(m.game.versions, semver::VersionReq::STAR);
        assert_eq!(m.paint_job.main[0].id, "cabin");
        assert_eq!(m.paint_job.main[0].texture.template, "templates/cabin.png");
        assert_eq!(m.paint_job.main[0].texture.layout_version, 1);
        assert!(m.paint_job.main[0].game_ids.is_empty());
        assert_eq!(m.paint_job.accessories[0].id, "mirrors");
        assert_eq!(m.paint_job.accessories[0].game_ids, ["mirror.painted"]);
        let cabin = p.template("cabin").unwrap();
        assert_eq!((cabin.width, cabin.height), (16.0, 16.0));
        let text = crate::summary(m);
        assert!(text.contains("game path: scania.r_2024"), "{text}");
        assert!(
            text.contains("  main textures:\n    cabin (cabin)"),
            "{text}"
        );
        assert!(
            text.contains("  accessories:\n    mirrors (Mirrors)"),
            "{text}"
        );
    }

    #[test]
    fn new_version_round_trip() {
        let mut v = truck();
        v.add(png(8));
        v.rows[2].name = "Side skirts".into();
        v.rows[2].game_ids = "sideskirt.a".into();
        let mut base_pkg = package(&v);
        // Carried fields.
        base_pkg.manifest.authors = vec!["Jane".into()];
        base_pkg.manifest.license = Some("CC-BY-4.0".into());

        let mut next = CustomVehicle::from_package(&base_pkg);
        assert_eq!(next.version, "1.1.0");
        assert_eq!(next.id(), "custom.scania.r_2024");
        assert_eq!(next.rows.len(), 3);
        assert_eq!(next.rows[0].template.format, TemplateFormat::Png);
        assert_eq!(next.rows[1].game_ids, "mirror.painted");
        // Renaming keeps the id; replacing bumps the layout; Side skirts
        // is removed and a new texture of the same name added.
        next.rows[0].name = "Cab".into();
        next.rows[0].replace(png(4));
        next.rows.remove(2);
        next.add(png(8));
        next.rows[2].name = "Side skirts".into();
        next.rows[2].game_ids = "sideskirt.a".into();
        let installed = [(
            "custom.scania.r_2024".to_owned(),
            semver::Version::new(1, 0, 0),
        )];
        assert_eq!(next.problems(&installed), []);
        let p = package(&next);
        let m = &p.manifest;
        assert_eq!(m.version, semver::Version::new(1, 1, 0));
        assert_eq!(m.authors, ["Jane"]);
        assert_eq!(m.license.as_deref(), Some("CC-BY-4.0"));
        assert_eq!(m.paint_job.main[0].id, "cabin");
        assert_eq!(m.paint_job.main[0].name, "Cab");
        assert_eq!(m.paint_job.main[0].texture.layout_version, 2);
        assert_eq!(m.paint_job.accessories[0].id, "mirrors");
        assert_eq!(m.paint_job.accessories[0].texture.layout_version, 1);
        assert_eq!(m.paint_job.accessories[1].id, "side_skirts_2");
        assert_eq!(m.paint_job.accessories[1].texture.layout_version, 1);
        assert!(m.paint_job.part("side_skirts").is_none());

        // The version must be higher than every installed one.
        next.version = "1.0.0".into();
        assert_eq!(
            next.problems(&installed),
            [Problem::VersionNotHigher(semver::Version::new(1, 0, 0))]
        );
        next.version = "1.2".into();
        assert_eq!(next.problems(&installed), [Problem::BadVersion]);
        next.version = "1.1.0".into();
        let newer = [
            installed[0].clone(),
            (
                "custom.scania.r_2024".to_owned(),
                semver::Version::new(1, 3, 0),
            ),
        ];
        assert_eq!(
            next.problems(&newer),
            [Problem::VersionNotHigher(semver::Version::new(1, 3, 0))]
        );
    }
}
