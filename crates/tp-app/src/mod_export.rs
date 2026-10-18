//! Mod export: what blocks it, the mod's files (definitions, textures,
//! icon, manifest) planned from the project, and the background job that
//! renders and writes them as one `.scs` archive.

use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use tp_core::document::Rgba;
use tp_core::{
    CheckReason, ModSettings, Project, ProjectVehicle, TemplateStatus, TexturePart, TextureState,
};
use tp_i18n::tr;
use tp_render::{DdsEncoding, Pixmap, RenderOptions};
use tp_text::FontLibrary;
use tp_vehicles::Manifest;

use crate::export::ExportOutcome;
use crate::workspace::Workspace;

/// Size of the shop icon, in pixels.
pub const ICON_SIZE: (u32, u32) = (256, 64);
/// Size of the Mod Manager image, in pixels.
pub const IMAGE_SIZE: (u32, u32) = (276, 162);
/// Extension of a mod archive.
pub const EXTENSION: &str = "scs";

/// The folder name of game `game` (`ets2` or `ats`).
fn game_folder(game: &str) -> Option<&'static str> {
    match game {
        "ets2" => Some("Euro Truck Simulator 2"),
        "ats" => Some("American Truck Simulator"),
        _ => None,
    }
}

/// Where game `game` reads mods on this platform: in the user's documents
/// folder on Windows, and in the user's data folder elsewhere
/// (`~/Library/Application Support` on macOS, `~/.local/share` on Linux).
pub fn mod_folder_in(game: &str, documents: Option<&Path>, data: Option<&Path>) -> Option<PathBuf> {
    let base = if cfg!(windows) { documents } else { data }?;
    Some(base.join(game_folder(game)?).join("mod"))
}

/// The mod folder of game `game` under `documents` or `data` (see
/// [`mod_folder_in`]), when it exists.
pub fn existing_mod_folder(
    game: &str,
    documents: Option<&Path>,
    data: Option<&Path>,
) -> Option<PathBuf> {
    mod_folder_in(game, documents, data).filter(|p| p.is_dir())
}

/// The file name proposed for a mod named `name`.
pub fn suggested_name(name: &str) -> String {
    let stem = crate::export::file_stem_safe(name);
    let stem = if stem.is_empty() { "mod" } else { &stem };
    format!("{stem}.{EXTENSION}")
}

/// `path` with the `.scs` extension, added when the user left it out.
pub fn with_extension(path: PathBuf) -> PathBuf {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSION))
    {
        return path;
    }
    let mut name = path.into_os_string();
    name.push(".");
    name.push(EXTENSION);
    PathBuf::from(name)
}

/// Where to export a mod by default: the game's mod folder when it exists,
/// else the folder of the session's last mod export, else the user's
/// Documents folder, else their home folder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Folders {
    pub documents: Option<PathBuf>,
    /// The user's data folder (`~/Library/Application Support`,
    /// `~/.local/share`).
    pub data: Option<PathBuf>,
    pub home: Option<PathBuf>,
}

impl Folders {
    /// This computer's folders.
    pub fn of_user() -> Self {
        let user = directories::UserDirs::new();
        let base = directories::BaseDirs::new();
        Self {
            documents: user
                .as_ref()
                .and_then(|u| u.document_dir().map(Path::to_path_buf)),
            data: base.as_ref().map(|b| b.data_dir().to_path_buf()),
            home: base.as_ref().map(|b| b.home_dir().to_path_buf()),
        }
    }
}

/// The file proposed for the mod of `project` when the Export Mod dialog
/// opens: "<Name>.scs" in the mod folder `chosen` for the game when it exists,
/// else in the game's mod folder when it exists under `folders`, else in `last_folder` (the last mod export of the session),
/// else in the Documents folder, else in the home folder.
pub fn destination(
    project: &Project,
    chosen: &BTreeMap<String, PathBuf>,
    last_folder: Option<&Path>,
    folders: &Folders,
) -> PathBuf {
    let folder = project
        .game()
        .and_then(|game| {
            chosen
                .get(game)
                .filter(|p| p.is_dir())
                .cloned()
                .or_else(|| {
                    existing_mod_folder(game, folders.documents.as_deref(), folders.data.as_deref())
                })
        })
        .or_else(|| last_folder.map(Path::to_path_buf))
        .or_else(|| folders.documents.clone())
        .or_else(|| folders.home.clone())
        .unwrap_or_default();
    folder.join(suggested_name(&project.mod_settings.name))
}

/// A mod setting, where a problem is fixed (see [`Problem::place`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModField {
    Name,
    Author,
    Version,
    Description,
    Price,
    UnlockLevel,
    InternalName,
    GameVersions,
}

impl ModField {
    /// The field's label in the current language.
    pub fn label(self) -> String {
        tr(match self {
            ModField::Name => "mod-name",
            ModField::Author => "mod-author",
            ModField::Version => "mod-version",
            ModField::Description => "mod-description",
            ModField::Price => "mod-price",
            ModField::UnlockLevel => "mod-unlock",
            ModField::InternalName => "mod-internal-name",
            ModField::GameVersions => "project-game-versions",
        })
    }
}

/// Where a problem is fixed in the Project space.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProblemPlace {
    /// A field of the Mod information column.
    Field(ModField),
    /// The card of the vehicle with this package id.
    Vehicle(String),
}

/// Something that blocks the export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    NameEmpty,
    /// The Name contains `"`, `\` or a line break.
    NameInvalid,
    VersionInvalid,
    AuthorInvalid,
    InternalNameEmpty,
    /// A character other than `a`–`z`, `0`–`9` and `_`.
    InternalNameInvalid,
    InternalNameTooLong {
        max: usize,
    },
    PriceZero,
    /// Two vehicles share a game path; `package_id` is the second one's.
    SamePath {
        path: String,
        first: String,
        second: String,
        package_id: String,
    },
    /// A vehicle's game data is missing: that package version must be
    /// installed.
    MissingGameData {
        vehicle: String,
        version: String,
        package_id: String,
    },
    /// A listed game version isn't written like the game's.
    BadGameVersion {
        version: String,
    },
    /// A listed game version a vehicle's package doesn't support.
    UnsupportedGameVersion {
        version: String,
        vehicle: crate::game_versions::VehicleRange,
    },
    /// The vehicles have no game version in common.
    NoCommonGameVersion {
        first: crate::game_versions::VehicleRange,
        second: crate::game_versions::VehicleRange,
    },
}

/// Whether `text` can be written inside a quoted SII string.
fn sii_safe(text: &str) -> bool {
    !text.contains(['"', '\\', '\n', '\r'])
}

impl Problem {
    /// Where the problem is fixed: a field of the Mod information column,
    /// or a vehicle's card.
    pub fn place(&self) -> ProblemPlace {
        let field = match self {
            Problem::NameEmpty | Problem::NameInvalid => ModField::Name,
            Problem::VersionInvalid => ModField::Version,
            Problem::AuthorInvalid => ModField::Author,
            Problem::InternalNameEmpty
            | Problem::InternalNameInvalid
            | Problem::InternalNameTooLong { .. } => ModField::InternalName,
            Problem::PriceZero => ModField::Price,
            Problem::BadGameVersion { .. } | Problem::UnsupportedGameVersion { .. } => {
                ModField::GameVersions
            }
            Problem::SamePath { package_id, .. } | Problem::MissingGameData { package_id, .. } => {
                return ProblemPlace::Vehicle(package_id.clone());
            }
            Problem::NoCommonGameVersion { first, .. } => {
                return ProblemPlace::Vehicle(first.package_id.clone());
            }
        };
        ProblemPlace::Field(field)
    }

    /// The problem in the current language.
    pub fn message(&self) -> String {
        match self {
            Problem::NameEmpty => tr("mod-problem-name-empty"),
            Problem::NameInvalid => tr("mod-problem-name-invalid"),
            Problem::VersionInvalid => tr("mod-problem-version-invalid"),
            Problem::AuthorInvalid => tr("mod-problem-author-invalid"),
            Problem::InternalNameEmpty => tr("mod-problem-internal-empty"),
            Problem::InternalNameInvalid => tr("mod-problem-internal-invalid"),
            Problem::InternalNameTooLong { max } => {
                tr!("mod-problem-internal-long", max = max.to_string())
            }
            Problem::PriceZero => tr("mod-problem-price"),
            Problem::SamePath {
                path,
                first,
                second,
                ..
            } => tr!(
                "mod-problem-same-path",
                path = path.as_str(),
                first = first.as_str(),
                second = second.as_str()
            ),
            Problem::MissingGameData {
                vehicle, version, ..
            } => tr!(
                "mod-problem-game-data",
                vehicle = vehicle.as_str(),
                version = version.as_str()
            ),
            Problem::BadGameVersion { version } => {
                tr!("mod-problem-bad-game-version", version = version.as_str())
            }
            Problem::UnsupportedGameVersion { version, vehicle } => tr!(
                "mod-problem-unsupported-game-version",
                version = version.as_str(),
                vehicle = vehicle.vehicle.as_str(),
                range = vehicle.range.as_str()
            ),
            Problem::NoCommonGameVersion { first, second } => tr!(
                "mod-problem-no-common-game-version",
                first = first.vehicle.as_str(),
                first_range = first.range.as_str(),
                second = second.vehicle.as_str(),
                second_range = second.range.as_str()
            ),
        }
    }
}

/// Everything that blocks exporting `project`, in the dialog's order.
pub fn problems(project: &Project) -> Vec<Problem> {
    let s = &project.mod_settings;
    let mut out = Vec::new();
    if s.name.trim().is_empty() {
        out.push(Problem::NameEmpty);
    } else if !sii_safe(&s.name) {
        out.push(Problem::NameInvalid);
    }
    if !sii_safe(&s.version) {
        out.push(Problem::VersionInvalid);
    }
    if !sii_safe(&s.author) {
        out.push(Problem::AuthorInvalid);
    }
    let max = project.internal_name_limit();
    let internal = s.internal_name(max);
    if internal.is_empty() {
        out.push(Problem::InternalNameEmpty);
    } else if !internal
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        out.push(Problem::InternalNameInvalid);
    } else if internal.chars().count() > max {
        out.push(Problem::InternalNameTooLong { max });
    }
    if s.price == 0 {
        out.push(Problem::PriceZero);
    }
    let mut paths: BTreeMap<&str, &str> = BTreeMap::new();
    for v in &project.vehicles {
        let Some(data) = &v.game_data else {
            out.push(Problem::MissingGameData {
                vehicle: v.name.clone(),
                version: v.version.clone(),
                package_id: v.package_id.clone(),
            });
            continue;
        };
        if let Some(first) = paths.insert(&data.path, &v.name) {
            out.push(Problem::SamePath {
                path: data.path.clone(),
                first: first.to_owned(),
                second: v.name.clone(),
                package_id: v.package_id.clone(),
            });
        }
    }
    game_version_problems(project, &mut out);
    out
}

/// The problems of the project's game versions: badly written ones, ones a
/// vehicle doesn't support, and vehicles with no version in common.
fn game_version_problems(project: &Project, out: &mut Vec<Problem>) {
    use crate::game_versions::{FleetVersions, fleet, of_listed, vehicle_ranges};
    let ranges = vehicle_ranges(project);
    for version in &project.game_versions {
        match of_listed(version) {
            None => out.push(Problem::BadGameVersion {
                version: version.clone(),
            }),
            Some(listed) => {
                if let Some((vehicle, _)) = ranges.iter().find(|(_, r)| !r.contains(&listed)) {
                    out.push(Problem::UnsupportedGameVersion {
                        version: version.clone(),
                        vehicle: vehicle.clone(),
                    });
                }
            }
        }
    }
    if let FleetVersions::Conflict { first, second } = fleet(project) {
        out.push(Problem::NoCommonGameVersion { first, second });
    }
}

/// Something to look at before exporting, which doesn't block the export
/// (the Project space's Before exporting, and the Export Mod dialog).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Warning {
    /// Surface `surface` is To check.
    ToCheck { surface: usize, reason: CheckReason },
    /// These surfaces are Empty: exported transparent.
    Empty { surfaces: Vec<usize> },
}

/// The warnings of `project`: each texture To check in project order, then
/// one line for the Empty textures.
pub fn warnings(project: &Project) -> Vec<Warning> {
    let mut out = Vec::new();
    let mut empty = Vec::new();
    for (i, surface) in project.surfaces.iter().enumerate() {
        match surface.state() {
            TextureState::ToCheck(reason) => out.push(Warning::ToCheck { surface: i, reason }),
            TextureState::Empty => empty.push(i),
            TextureState::Modified => {}
        }
    }
    if !empty.is_empty() {
        out.push(Warning::Empty { surfaces: empty });
    }
    out
}

/// The name of surface `i` in a warning: its name, preceded by its vehicle
/// ("<vehicle> › <texture>") when another surface has the same name.
pub fn texture_label(project: &Project, i: usize) -> String {
    let name = &project.surfaces[i].name;
    let shared = project
        .surfaces
        .iter()
        .enumerate()
        .any(|(j, s)| j != i && s.name == *name);
    match project.surface_names(i) {
        Some((vehicle, texture)) if shared => format!("{vehicle} › {texture}"),
        _ => name.clone(),
    }
}

impl Warning {
    /// The warning in the current language.
    pub fn message(&self, project: &Project) -> String {
        match self {
            Warning::ToCheck { surface, reason } => {
                let texture = texture_label(project, *surface);
                match reason {
                    CheckReason::LayoutChanged => {
                        tr!("mod-warning-layout-changed", texture = texture)
                    }
                    CheckReason::NotInVersion => {
                        tr!("mod-warning-not-in-version", texture = texture)
                    }
                }
            }
            Warning::Empty { surfaces } => match surfaces.as_slice() {
                [one] => tr!(
                    "mod-warning-empty-one",
                    texture = texture_label(project, *one)
                ),
                _ => tr!("mod-warning-empty-many", count = surfaces.len()),
            },
        }
    }
}

/// What the mod holds for one vehicle (the dialog's summary).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleSummary {
    pub name: String,
    /// A trailer: its main texture has no cabins.
    pub trailer: bool,
    /// Main textures in package order, painted or not.
    pub mains: Vec<MainSummary>,
    /// Names of the painted accessories.
    pub accessories: Vec<String>,
}

/// A main texture of a vehicle in the summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MainSummary {
    pub name: String,
    /// The cabins it is painted on (none: every cabin).
    pub cabins: Vec<String>,
    pub painted: bool,
}

/// What the mod holds for each vehicle of `project`. `manifest` gives the
/// installed package version of a vehicle, to name the main textures it
/// doesn't paint (left out when it isn't installed).
pub fn summary(
    project: &Project,
    manifest: &dyn Fn(&ProjectVehicle) -> Option<Manifest>,
) -> Vec<VehicleSummary> {
    project
        .vehicles
        .iter()
        .map(|v| {
            let mut mains: Vec<(usize, MainSummary)> = Vec::new();
            let mut accessories = Vec::new();
            for i in project.vehicle_range(&v.package_id) {
                let surface = &project.surfaces[i];
                let Some(t) = surface
                    .template
                    .as_ref()
                    .filter(|t| t.status != TemplateStatus::Removed)
                else {
                    continue;
                };
                match t.part {
                    TexturePart::Main => mains.push((
                        t.main_index.unwrap_or(mains.len()),
                        MainSummary {
                            name: surface.name.clone(),
                            cabins: t.game_ids.clone(),
                            painted: true,
                        },
                    )),
                    TexturePart::Accessory => accessories.push(surface.name.clone()),
                }
            }
            if let Some(m) = manifest(v) {
                for (index, part) in m.paint_job.main.iter().enumerate() {
                    if !mains.iter().any(|(i, _)| *i == index) {
                        mains.push((
                            index,
                            MainSummary {
                                name: part.name.clone(),
                                cabins: part.game_ids.clone(),
                                painted: false,
                            },
                        ));
                    }
                }
            }
            mains.sort_by_key(|(i, _)| *i);
            VehicleSummary {
                name: v.name.clone(),
                trailer: v.kind == "trailer",
                mains: mains.into_iter().map(|(_, m)| m).collect(),
                accessories,
            }
        })
        .collect()
}

/// One file of the mod.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// A definition, material or text file.
    Text(String),
    /// A `.tobj`.
    Bytes(Vec<u8>),
    /// Surface `surface` rendered at `width` × `height` pixels, as DDS.
    Texture {
        surface: usize,
        width: u32,
        height: u32,
    },
    /// The shop icon, as DDS.
    Icon,
    /// The Mod Manager image, as JPEG.
    Image,
}

/// The mod's files by path in the archive, sorted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModPlan {
    pub files: BTreeMap<String, Entry>,
}

impl ModPlan {
    /// The text of file `path`, if it is a text file.
    pub fn text(&self, path: &str) -> Option<&str> {
        match self.files.get(path)? {
            Entry::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Total pixels to render: textures, icon and image.
    pub fn pixels(&self) -> u64 {
        self.files
            .values()
            .map(|e| match e {
                Entry::Texture { width, height, .. } => u64::from(*width) * u64::from(*height),
                Entry::Icon => u64::from(ICON_SIZE.0 * ICON_SIZE.1),
                Entry::Image => u64::from(IMAGE_SIZE.0 * IMAGE_SIZE.1),
                Entry::Text(_) | Entry::Bytes(_) => 0,
            })
            .sum()
    }
}

/// A binary `.tobj` pointing to the absolute `dds` path (`/vehicle/….dds`):
/// the header the games expect, then the path's length and the path.
pub fn tobj(dds: &str) -> Vec<u8> {
    const HEADER: [u8; 40] = [
        0x01, 0x0A, 0xB1, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x02, 0x00, 0x03, 0x03, 0x03, 0x00,
        0x02, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
    ];
    let mut out = HEADER.to_vec();
    out.extend_from_slice(&(dds.len() as u64).to_le_bytes());
    out.extend_from_slice(dds.as_bytes());
    out
}

/// The game folder of a vehicle kind.
fn game_type(vehicle: &ProjectVehicle) -> &'static str {
    if vehicle.kind == "trailer" {
        "trailer_owned"
    } else {
        "truck"
    }
}

/// `id` as a file name: `a`–`z`, `0`–`9`, `_` and `-`.
fn file_stem(id: &str) -> String {
    id.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// The name of the paint job of main texture `index` among `count`.
fn unit_name(internal: &str, count: usize, index: usize) -> String {
    if count <= 1 {
        return internal.to_owned();
    }
    match u8::try_from(index).ok().filter(|i| *i < 26) {
        Some(i) => format!("{internal}_{}", char::from(b'a' + i)),
        None => format!("{internal}_{index}"),
    }
}

/// A painted texture of a vehicle.
struct Painted<'a> {
    part: TexturePart,
    game_ids: &'a [String],
    /// Accessory ids a main texture also paints.
    also_paints: &'a [String],
    main_index: Option<usize>,
    /// The texture's `.tobj`, absolute.
    tobj: String,
}

/// The mod's files for `project`, or what blocks the export.
pub fn plan(project: &Project) -> Result<ModPlan, Vec<Problem>> {
    let problems = problems(project);
    if !problems.is_empty() {
        return Err(problems);
    }
    let s = &project.mod_settings;
    let id = project.internal_name();
    let mut files = BTreeMap::new();
    files.insert(
        "manifest.sii".to_owned(),
        Entry::Text(format!(
            "SiiNunit\n{{\nmod_package : .package_name\n{{\n\
             \tpackage_version: \"{}\"\n\
             \tdisplay_name: \"{}\"\n\
             \tauthor: \"{}\"\n\
             \tcategory[]: \"paint_job\"\n\
             \tmp_mod_optional: true\n\
             \ticon: \"icon.jpg\"\n\
             \tdescription_file: \"description.txt\"\n{}\
             }}\n}}\n",
            s.version,
            s.name,
            s.author,
            project
                .game_versions
                .iter()
                .map(|v| format!("\tcompatible_versions[]: \"{v}\"\n"))
                .collect::<String>()
        )),
    );
    files.insert(
        "description.txt".to_owned(),
        Entry::Text(description(project)),
    );
    files.insert(
        format!("material/ui/accessory/{id}_icon.mat"),
        Entry::Text(format!(
            "material: \"ui\"\n{{\n\ttexture: \"{id}_icon.tobj\"\n\ttexture_name: \"texture\"\n}}\n"
        )),
    );
    files.insert(
        format!("material/ui/accessory/{id}_icon.tobj"),
        Entry::Bytes(tobj(&format!("/material/ui/accessory/{id}_icon.dds"))),
    );
    files.insert(format!("material/ui/accessory/{id}_icon.dds"), Entry::Icon);
    files.insert("icon.jpg".into(), Entry::Image);

    for vehicle in &project.vehicles {
        let data = vehicle.game_data.as_ref().expect("checked by problems()");
        let kind = game_type(vehicle);
        let def = format!("def/vehicle/{kind}/{}/paint_job", data.path);
        let textures = format!("vehicle/{kind}/upgrade/paintjob/{id}/{}", data.path);
        let mut painted = Vec::new();
        let mut stems: Vec<String> = Vec::new();
        for i in project.vehicle_range(&vehicle.package_id) {
            let surface = &project.surfaces[i];
            let Some(t) = &surface.template else {
                continue;
            };
            // A texture the package version no longer has is left out.
            if t.status == TemplateStatus::Removed {
                continue;
            }
            let mut stem = file_stem(&t.texture_id);
            if stems.contains(&stem) {
                stem = format!("{stem}_{i}");
            }
            stems.push(stem.clone());
            files.insert(
                format!("{textures}/{stem}.dds"),
                Entry::Texture {
                    surface: i,
                    width: surface.size.width.round() as u32,
                    height: surface.size.height.round() as u32,
                },
            );
            let tobj_path = format!("{textures}/{stem}.tobj");
            files.insert(
                tobj_path.clone(),
                Entry::Bytes(tobj(&format!("/{textures}/{stem}.dds"))),
            );
            painted.push(Painted {
                part: t.part,
                game_ids: &t.game_ids,
                also_paints: &t.also_paints,
                main_index: t.main_index,
                tobj: format!("/{tobj_path}"),
            });
        }
        let mut settings = format!(
            "\tname: \"{}\"\n\tprice: {}\n\tunlock: {}\n\ticon: \"{id}_icon\"\n\tairbrush: true\n",
            s.name, s.price, s.unlock_level
        );
        if data.alt_uv {
            settings.push_str("\talternate_uvset: true\n");
        }
        if data.colour_picker {
            settings.push_str("\tbase_color_locked: false\n");
        }
        files.insert(format!("{def}/{id}_settings.sui"), Entry::Text(settings));
        let accessories: Vec<&Painted> = painted
            .iter()
            .filter(|p| p.part == TexturePart::Accessory)
            .collect();
        // One `simple_paint_job_data` per painted accessory, then one for
        // the accessories the unit's main texture also paints.
        let accessory_file = |main: &Painted| {
            let entries: Vec<(&str, &[String])> = accessories
                .iter()
                .map(|a| (a.tobj.as_str(), a.game_ids))
                .chain(
                    (!main.also_paints.is_empty())
                        .then_some((main.tobj.as_str(), main.also_paints)),
                )
                .collect();
            if entries.is_empty() {
                return None;
            }
            let mut out = String::from("SiiNunit\n{\n");
            for (n, (tobj, game_ids)) in entries.iter().enumerate() {
                out.push_str(&format!(
                    "\nsimple_paint_job_data : .ovr{n}\n{{\n\tpaint_job_mask: \"{tobj}\"\n"
                ));
                for game_id in *game_ids {
                    out.push_str(&format!("\tacc_list[]: \"{game_id}\"\n"));
                }
                out.push_str("}\n");
            }
            out.push_str("}\n");
            Some(out)
        };
        for main in painted.iter().filter(|p| p.part == TexturePart::Main) {
            let unit = unit_name(&id, data.main_count, main.main_index.unwrap_or(0));
            let mut out = format!(
                "SiiNunit\n{{\naccessory_paint_job_data : {unit}.{path}.paint_job\n{{\n\
                 @include \"{id}_settings.sui\"\n",
                path = data.path
            );
            for cabin in main.game_ids {
                out.push_str(&format!(
                    "\tsuitable_for[]: \"{cabin}.{}.cabin\"\n",
                    data.path
                ));
            }
            out.push_str(&format!("\tpaint_job_mask: \"{}\"\n}}\n}}\n", main.tobj));
            files.insert(format!("{def}/{unit}.sii"), Entry::Text(out));
            if let Some(acc) = accessory_file(main) {
                files.insert(format!("{def}/accessory/{unit}.sii"), Entry::Text(acc));
            }
        }
    }
    Ok(ModPlan { files })
}

/// `description.txt`: the player's description, the vehicles and the mods
/// they require.
fn description(project: &Project) -> String {
    let mut out = String::new();
    let own = project.mod_settings.description.trim_end();
    if !own.is_empty() {
        out.push_str(own);
        out.push_str("\n\n");
    }
    out.push_str("Vehicles supported:\n");
    for v in &project.vehicles {
        out.push_str(&format!("- {}\n", v.name));
    }
    let mut required: Vec<String> = Vec::new();
    for r in project
        .vehicles
        .iter()
        .filter_map(|v| v.game_data.as_ref())
        .flat_map(|g| &g.requires)
    {
        let line = match &r.version {
            Some(version) => format!("- {} {version}", r.name),
            None => format!("- {}", r.name),
        };
        if !required.contains(&line) {
            required.push(line);
        }
    }
    if !required.is_empty() {
        out.push_str("\nRequires:\n");
        for line in required {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// A picture chosen for the mod, not yet in the project.
#[derive(Clone, Debug, PartialEq)]
pub struct PictureFile {
    pub name: String,
    pub bytes: Arc<[u8]>,
    /// Pixel size.
    pub size: (u32, u32),
}

impl PictureFile {
    /// The picture of PNG or JPEG `bytes`, or why it can't be read.
    pub fn read(name: &str, bytes: Vec<u8>) -> Result<Self, String> {
        let image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
        Ok(Self {
            name: name.to_owned(),
            bytes: Arc::from(bytes),
            size: (image.width(), image.height()),
        })
    }
}

/// Where a mod picture comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum Picture {
    /// Rendered from the first texture.
    Generated,
    /// A chosen picture already in the project.
    Asset(tp_core::document::AssetId),
    /// A newly chosen picture.
    File(PictureFile),
}

impl Picture {
    /// The picture of the chosen asset `asset`, if any.
    pub fn of(asset: Option<tp_core::document::AssetId>) -> Self {
        asset.map_or(Self::Generated, Self::Asset)
    }

    /// The image bytes, unless generated.
    pub fn bytes<'a>(&'a self, project: &'a Project) -> Option<&'a [u8]> {
        match self {
            Self::Generated => None,
            Self::Asset(a) => chosen_bytes(project, Some(*a)),
            Self::File(f) => Some(&f.bytes),
        }
    }

    /// Adds a newly chosen picture to the project; returns its asset.
    fn add_to(&self, project: &mut Project) -> Option<tp_core::document::AssetId> {
        match self {
            Self::Generated => None,
            Self::Asset(a) => Some(*a),
            Self::File(f) => Some(
                project
                    .add_asset(
                        &f.name,
                        tp_core::AssetKind::Raster,
                        f.bytes.clone(),
                        tp_core::kurbo::Size::new(f64::from(f.size.0), f64::from(f.size.1)),
                    )
                    .0,
            ),
        }
    }
}

/// One of the mod's two pictures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModPicture {
    /// The shop icon.
    Icon,
    /// The Mod Manager image.
    Image,
}

impl ModPicture {
    /// Its size in pixels.
    pub fn size(self) -> (u32, u32) {
        match self {
            ModPicture::Icon => ICON_SIZE,
            ModPicture::Image => IMAGE_SIZE,
        }
    }
}

impl Workspace {
    /// Changes the mod settings with `f` as one "Edit Mod Settings" undo
    /// step; records nothing when nothing changed.
    pub fn set_mod_setting(&mut self, now: f64, f: impl FnOnce(&mut ModSettings)) {
        self.edit("undo-edit-mod-settings", now, false, |project, _| {
            f(&mut project.mod_settings);
        });
    }

    /// Commits the mod setting being typed in (see [`Self::mod_draft`]) as
    /// its field would when left: one step, nothing when unchanged.
    pub fn commit_mod_draft(&mut self, now: f64) {
        let Some((field, text)) = self.mod_draft.take() else {
            return;
        };
        self.commit_mod_field(field, text, now);
    }

    /// Commits `text` typed in the field of `field`. The internal name is
    /// only set when it differs from the one in use, so that leaving its
    /// field untouched keeps it following the Name; the game versions are
    /// added to the list. The Price and the Unlock level take numbers (see
    /// their fields) and are left alone.
    pub fn commit_mod_field(&mut self, field: ModField, text: String, now: f64) {
        match field {
            ModField::Name => self.set_mod_setting(now, |s| s.name = text),
            ModField::Author => self.set_mod_setting(now, |s| s.author = text),
            ModField::Version => self.set_mod_setting(now, |s| s.version = text),
            ModField::Description => self.set_mod_setting(now, |s| s.description = text),
            ModField::InternalName => {
                let limit = self.project.internal_name_limit();
                self.set_mod_setting(now, |s| {
                    if text != s.internal_name(limit) {
                        s.internal_name = Some(text);
                    }
                });
            }
            ModField::GameVersions => {
                self.add_game_versions(&crate::game_versions::parse_list(&text), now);
            }
            ModField::Price | ModField::UnlockLevel => {}
        }
    }

    /// Makes file `name` (its `bytes`, or why they couldn't be read)
    /// picture `which`, as one undo step. A file that isn't a PNG or JPEG
    /// changes nothing and is reported under the picture.
    pub fn use_picture_file(
        &mut self,
        which: ModPicture,
        name: &str,
        bytes: Result<Vec<u8>, String>,
        now: f64,
    ) {
        let stem = Path::new(name)
            .file_stem()
            .map_or_else(|| name.to_owned(), |s| s.to_string_lossy().into_owned());
        match bytes.and_then(|bytes| PictureFile::read(&stem, bytes)) {
            Ok(file) => {
                self.picture_error = None;
                self.set_mod_picture(which, &Picture::File(file), now);
            }
            Err(reason) => {
                self.picture_error = Some((
                    which,
                    tr!("mod-picture-failed", file = name, reason = reason),
                ));
            }
        }
    }

    /// Sets picture `which` as one "Edit Mod Settings" undo step. A newly
    /// chosen picture becomes an asset, and the previous one is removed
    /// when no longer used. Records nothing when nothing changed.
    pub fn set_mod_picture(&mut self, which: ModPicture, picture: &Picture, now: f64) {
        self.edit("undo-edit-mod-settings", now, false, |project, _| {
            let asset = picture.add_to(project);
            let slot = match which {
                ModPicture::Icon => &mut project.mod_settings.icon,
                ModPicture::Image => &mut project.mod_settings.image,
            };
            let previous = std::mem::replace(slot, asset);
            if let Some(previous) = previous.filter(|p| Some(*p) != asset) {
                project.remove_asset(previous);
            }
        });
    }
}

/// Background of the generated pictures.
const PICTURE_BACKGROUND: Rgba = Rgba::rgb(255, 255, 255);
/// JPEG quality of the Mod Manager image.
const JPEG_QUALITY: u8 = 90;

/// The mod's picture of `size`: the image file `chosen` (PNG or JPEG)
/// scaled to cover it, or else the project's first texture.
pub fn picture(
    project: &Project,
    chosen: Option<&[u8]>,
    size: (u32, u32),
    fonts: &mut FontLibrary,
) -> Result<Pixmap, String> {
    match chosen {
        Some(bytes) => {
            let image = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
            Ok(tp_render::cover_image(&image.to_rgba8(), size.0, size.1))
        }
        None => Ok(tp_render::render_cover(
            project,
            0,
            size.0,
            size.1,
            Some(PICTURE_BACKGROUND),
            fonts,
        )),
    }
}

/// The bytes of the chosen mod image `asset`, if any.
fn chosen_bytes(project: &Project, asset: Option<tp_core::document::AssetId>) -> Option<&[u8]> {
    project.assets.get(&asset?).map(|a| &*a.bytes)
}

/// Renders, encodes and zips the mod, then writes it (worker side).
fn run_mod_export(
    project: &Project,
    plan: &ModPlan,
    mut fonts: FontLibrary,
    path: &Path,
    progress: &AtomicU32,
    cancel: &AtomicBool,
    notify: &(dyn Fn() + Send + Sync),
) -> ExportOutcome {
    let failed = |reason: String| ExportOutcome::Failed {
        path: path.to_path_buf(),
        reason,
    };
    let cancelled = || cancel.load(Ordering::Relaxed);
    // Progress up to 950 is shared by the pictures in proportion to their
    // pixels; writing the file takes the rest.
    let total = plan.pixels().max(1) as f64;
    let mut done = 0u64;
    let set = |done: u64, part: f64, weight: u64| {
        let value = (done as f64 + part * weight as f64) / total * 950.0;
        progress.store(value as u32, Ordering::Relaxed);
        notify();
    };
    let stored = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644);
    let deflated = stored.compression_method(zip::CompressionMethod::Deflated);
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, entry) in &plan.files {
        let (bytes, options) = match entry {
            Entry::Text(text) => (text.clone().into_bytes(), deflated),
            Entry::Bytes(bytes) => (bytes.clone(), deflated),
            Entry::Texture {
                surface,
                width,
                height,
            } => {
                let weight = u64::from(*width) * u64::from(*height);
                let options = RenderOptions {
                    width: *width,
                    height: *height,
                    background: None,
                };
                let rendered =
                    tp_render::render(project, *surface, options, &mut fonts, &mut |i, n| {
                        set(done, 0.7 * i as f64 / n.max(1) as f64, weight);
                        !cancelled()
                    });
                let Ok(pixmap) = rendered else {
                    return ExportOutcome::Cancelled;
                };
                let encoded =
                    tp_render::encode_dds(&pixmap, DdsEncoding::Bc3, &mut |level, levels| {
                        set(
                            done,
                            0.7 + 0.3 * level as f64 / levels.max(1) as f64,
                            weight,
                        );
                        !cancelled()
                    });
                let Ok(bytes) = encoded else {
                    return ExportOutcome::Cancelled;
                };
                done += weight;
                (bytes, stored)
            }
            Entry::Icon => {
                let chosen = chosen_bytes(project, project.mod_settings.icon);
                let pixmap = match picture(project, chosen, ICON_SIZE, &mut fonts) {
                    Ok(p) => p,
                    Err(reason) => return failed(reason),
                };
                let Ok(bytes) =
                    tp_render::encode_dds(&pixmap, DdsEncoding::Bc3, &mut |_, _| !cancelled())
                else {
                    return ExportOutcome::Cancelled;
                };
                done += u64::from(ICON_SIZE.0 * ICON_SIZE.1);
                (bytes, stored)
            }
            Entry::Image => {
                let chosen = chosen_bytes(project, project.mod_settings.image);
                let encoded = picture(project, chosen, IMAGE_SIZE, &mut fonts)
                    .and_then(|p| tp_render::encode_jpeg(&p, JPEG_QUALITY));
                match encoded {
                    Ok(bytes) => {
                        done += u64::from(IMAGE_SIZE.0 * IMAGE_SIZE.1);
                        (bytes, stored)
                    }
                    Err(reason) => return failed(reason),
                }
            }
        };
        if cancelled() {
            return ExportOutcome::Cancelled;
        }
        set(done, 0.0, 0);
        // Writing into memory cannot fail.
        zip.start_file(name.as_str(), options).expect("zip");
        zip.write_all(&bytes).expect("zip");
    }
    let bytes = zip.finish().expect("zip").into_inner();
    if cancelled() {
        return ExportOutcome::Cancelled;
    }
    match tp_file::write_atomic(path, &bytes) {
        Ok(()) => {
            progress.store(1000, Ordering::Relaxed);
            notify();
            ExportOutcome::Written(path.to_path_buf())
        }
        Err(err) => failed(match err {
            tp_file::Error::Io(e) => e.to_string(),
            other => other.to_string(),
        }),
    }
}

/// A mod export running on a worker thread.
pub struct ModJob {
    pub path: PathBuf,
    progress: Arc<AtomicU32>,
    cancel: Arc<AtomicBool>,
    done: Receiver<ExportOutcome>,
}

impl ModJob {
    /// Starts writing the mod `plan` of `project` (a snapshot: editing may
    /// continue) to `path`.
    pub fn start(
        project: Project,
        plan: ModPlan,
        fonts: FontLibrary,
        path: PathBuf,
        notify: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let progress = Arc::new(AtomicU32::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, done) = mpsc::channel();
        let (p, c, target) = (progress.clone(), cancel.clone(), path.clone());
        let spawned = thread::Builder::new()
            .name("mod-export".into())
            .spawn(move || {
                let outcome = run_mod_export(&project, &plan, fonts, &target, &p, &c, &notify);
                let _ = tx.send(outcome);
                notify();
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot start the mod export");
        }
        Self {
            path,
            progress,
            cancel,
            done,
        }
    }

    /// Progress from 0.0 to 1.0.
    pub fn progress(&self) -> f32 {
        self.progress.load(Ordering::Relaxed) as f32 / 1000.0
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// The outcome once finished.
    pub fn poll(&self) -> Option<ExportOutcome> {
        match self.done.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(ExportOutcome::Failed {
                path: self.path.clone(),
                reason: tr("export-stopped"),
            }),
        }
    }

    /// Blocks until the export ends (tests).
    pub fn wait(&self) -> ExportOutcome {
        self.done.recv().unwrap_or(ExportOutcome::Failed {
            path: self.path.clone(),
            reason: tr("export-stopped"),
        })
    }
}

#[cfg(test)]
mod tests {
    use tp_core::RequiredMod;
    use tp_vehicles::Package;

    use super::*;
    use crate::vehicle_project::fleet_project;
    use crate::workspace::Workspace;

    /// A version of a committed sample package.
    fn example(name: &str, version: &str) -> Package {
        let path = format!(
            "{}/../../examples/vehicles/community.truckpaint.{name}-{version}.tpv",
            env!("CARGO_MANIFEST_DIR")
        );
        Package::read(&std::fs::read(path).unwrap()).unwrap()
    }

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    /// A project painting `textures` of the sample truck 1.1.0, with
    /// internal name "ace".
    fn truck(textures: &[&str]) -> Project {
        let mut p = fleet_project(
            "ACE",
            &[(&example("sample_truck", "1.1.0"), &ids(textures))],
        )
        .unwrap();
        p.mod_settings.internal_name = Some("ace".into());
        p
    }

    fn trailer(textures: &[&str]) -> Project {
        let mut p = fleet_project(
            "ACE",
            &[(&example("sample_trailer", "1.0.0"), &ids(textures))],
        )
        .unwrap();
        p.mod_settings.internal_name = Some("ace".into());
        p
    }

    /// A small truck (256 px textures: main "cabin", accessory "chassis")
    /// with internal name "ace" and a rectangle on its cabin.
    fn small_truck() -> Project {
        use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
        use tp_core::kurbo::{Point, Size};
        let textures = [
            tp_vehicles::sample::SampleTexture {
                id: "cabin",
                name: "Cabin",
                size: tp_vehicles::TextureSize::new(256, 256),
                layout: 1,
            },
            tp_vehicles::sample::SampleTexture {
                id: "chassis",
                name: "Chassis",
                size: tp_vehicles::TextureSize::new(256, 256),
                layout: 1,
            },
        ];
        let package = Package::read(&tp_vehicles::sample::package(
            "scs.sample.truck",
            "Sample Truck",
            "1.0.0",
            &textures,
        ))
        .unwrap();
        let mut p = fleet_project("ACE", &[(&package, &ids(&["cabin", "chassis"]))]).unwrap();
        p.mod_settings.internal_name = Some("ace".into());
        p.add(Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(128.0, 128.0), Size::new(100.0, 60.0), 0.0),
        ));
        p
    }

    /// Starts exporting `project` to `path` and waits for the outcome.
    fn export(project: &Project, path: &Path) -> ExportOutcome {
        let plan = plan(project).unwrap();
        ModJob::start(
            project.clone(),
            plan,
            FontLibrary::bundled(),
            path.to_path_buf(),
            || {},
        )
        .wait()
    }

    fn archive(path: &Path) -> zip::ZipArchive<std::fs::File> {
        zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap()
    }

    fn read_entry(zip: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Vec<u8> {
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut zip.by_name(name).unwrap(), &mut out).unwrap();
        out
    }

    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }

    #[test]
    fn the_job_writes_the_planned_mod() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        let project = truck(&["standard", "side_skirts"]);
        assert_eq!(
            export(&project, &path),
            ExportOutcome::Written(path.clone())
        );
        let mut zip = archive(&path);
        let mut names: Vec<String> = zip.file_names().map(str::to_owned).collect();
        names.sort();
        let planned: Vec<String> = plan(&project).unwrap().files.into_keys().collect();
        assert_eq!(names, planned);
        let dds = read_entry(&mut zip, &format!("{TRUCK_TEX}/side_skirts.dds"));
        assert_eq!(&dds[84..88], b"DXT5");
        assert_eq!((u32_at(&dds, 12), u32_at(&dds, 16)), (1024, 1024));
        assert_eq!(u32_at(&dds, 28), 11, "mip levels");
        let icon = read_entry(&mut zip, "material/ui/accessory/ace_icon.dds");
        assert_eq!((u32_at(&icon, 16), u32_at(&icon, 12)), ICON_SIZE);
        let image = image::load_from_memory(&read_entry(&mut zip, "icon.jpg")).unwrap();
        assert_eq!((image.width(), image.height()), IMAGE_SIZE);
        // No temporary file left behind.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    /// mod-export "Rectangular texture in the mod": an 8192 × 1024 base is
    /// written at its own size with 14 mipmap levels.
    #[test]
    fn rectangular_texture_in_the_mod() {
        let base = tp_vehicles::sample::SampleTexture {
            id: "base",
            name: "Base",
            size: tp_vehicles::TextureSize::new(8192, 1024),
            layout: 1,
        };
        let package = Package::read(&tp_vehicles::sample::package_with(
            "community.jdoe.liner",
            "Liner",
            "1.0.0",
            "trailer",
            &[base],
            &[],
        ))
        .unwrap();
        let mut project = fleet_project("ACE", &[(&package, &ids(&["base"]))]).unwrap();
        project.mod_settings.internal_name = Some("ace".into());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        let plan = plan(&project).unwrap();
        let (name, _) = plan
            .files
            .iter()
            .find(|(_, e)| matches!(e, Entry::Texture { .. }))
            .unwrap();
        let name = name.clone();
        assert_eq!(
            export(&project, &path),
            ExportOutcome::Written(path.clone())
        );
        let dds = read_entry(&mut archive(&path), &name);
        assert_eq!(&dds[84..88], b"DXT5");
        assert_eq!((u32_at(&dds, 16), u32_at(&dds, 12)), (8192, 1024));
        assert_eq!(u32_at(&dds, 28), 14, "mip levels");
    }

    /// drop-shadow: Export Mod's textures go through the same renderer as
    /// Export Texture, so a shadow is the same in both.
    #[test]
    fn a_shadow_in_the_mods_texture_matches_the_render() {
        use tp_core::document::{CharStyle, Frame, Object, ObjectId, Rgba, Shadow, ShapeKind};
        use tp_core::kurbo::{Point, Size, Vec2};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        let mut project = small_truck();
        let mut text = Object::new(
            ObjectId(0),
            ShapeKind::Text,
            Frame::new(Point::new(100.0, 100.0), Size::new(120.0, 80.0), 0.0),
        );
        text.text = Some(tp_core::document::TextBlock::new(
            "I",
            CharStyle {
                size: 80.0,
                ..CharStyle::default()
            },
        ));
        text.shadow = Some(Shadow {
            color: Rgba::rgb(20, 40, 160),
            opacity: 1.0,
            offset: Vec2::new(40.0, 40.0),
            blur: 0.0,
            ..Shadow::DEFAULT
        });
        project.add(text);
        let size = 256;
        let options = RenderOptions {
            width: size,
            height: size,
            background: None,
        };
        let rendered = |p: &Project| {
            let pixmap =
                tp_render::render(p, 0, options, &mut FontLibrary::bundled(), &mut |_, _| true)
                    .unwrap();
            tp_render::to_rgba(&pixmap)
        };
        let with = rendered(&project);
        let mut plain = project.clone();
        let objects = plain.surface().objects.clone();
        let mut last = (*objects[objects.len() - 1]).clone();
        last.shadow = None;
        plain.surface_mut().replace(&[last]);
        let without = rendered(&plain);
        // A pixel of the shadow alone, inside a 4 × 4 block it fills.
        let (x, y) = (0..size / 4)
            .flat_map(|by| (0..size / 4).map(move |bx| (bx * 4, by * 4)))
            .find(|&(bx, by)| {
                (0..4).all(|dy| {
                    (0..4).all(|dx| {
                        with.get_pixel(bx + dx, by + dy).0[3] == 255
                            && without.get_pixel(bx + dx, by + dy).0[3] == 0
                    })
                })
            })
            .expect("a block of shadow");
        assert_eq!(
            export(&project, &path),
            ExportOutcome::Written(path.clone())
        );
        let mut zip = archive(&path);
        let name = zip
            .file_names()
            .find(|n| n.ends_with("/cabin.dds"))
            .unwrap()
            .to_owned();
        let dds = read_entry(&mut zip, &name);
        let mut pixels = vec![0u8; (size * size * 4) as usize];
        let top = (size * size) as usize;
        texpresso::Format::Bc3.decompress(&dds[128..128 + top], 256, 256, &mut pixels);
        let at = ((y * size + x) * 4) as usize;
        let exported = &pixels[at..at + 4];
        let expected = with.get_pixel(x, y).0;
        assert!(
            exported
                .iter()
                .zip(expected)
                .all(|(a, b)| a.abs_diff(b) <= 8),
            "{exported:?} vs {expected:?}"
        );
    }

    /// drop-shadow: the cost of shadows in an export. The sample truck's
    /// 4096 px Standard cab with 20 texts with a blur of 8 and a lettering
    /// across the texture with a blur of 200: the export stays under 3 s.
    /// Timings are printed (`--release -- --ignored --nocapture`).
    #[test]
    #[ignore = "performance measurement: run in release"]
    fn shadowed_export_performance() {
        use std::time::Instant;
        use tp_core::document::{CharStyle, Frame, Object, ObjectId, Shadow, ShapeKind};
        use tp_core::kurbo::{Point, Size};
        let mut project = truck(&["standard"]);
        let text = |content: &str, at: Point, size: f64, blur: f64| {
            let mut o = Object::new(
                ObjectId(0),
                ShapeKind::Text,
                Frame::new(at, Size::new(size * content.len() as f64, size * 1.3), 0.0),
            );
            o.text = Some(tp_core::document::TextBlock::new(
                content,
                CharStyle {
                    size,
                    ..CharStyle::default()
                },
            ));
            o.shadow = Some(Shadow {
                blur,
                ..Shadow::DEFAULT
            });
            o
        };
        for i in 0..20 {
            let at = Point::new(
                400.0 + f64::from(i % 4) * 1000.0,
                400.0 + f64::from(i / 4) * 800.0,
            );
            project.add(text("ACE 24/7", at, 120.0, 8.0));
        }
        project.add(text(
            "ACE LOGISTICS",
            Point::new(2048.0, 2048.0),
            560.0,
            200.0,
        ));
        let surface = project.surface().size;
        assert_eq!(surface, Size::new(4096.0, 4096.0));
        let options = RenderOptions {
            width: 4096,
            height: 4096,
            background: None,
        };
        let mut fonts = FontLibrary::bundled();
        let render = |p: &Project, fonts: &mut FontLibrary| {
            tp_render::render(p, 0, options, fonts, &mut |_, _| true).unwrap()
        };
        let mut plain = project.clone();
        let unshadowed: Vec<Object> = plain
            .surface()
            .objects
            .iter()
            .map(|o| Object {
                shadow: None,
                ..(**o).clone()
            })
            .collect();
        plain.surface_mut().replace(&unshadowed);
        render(&plain, &mut fonts);
        let start = Instant::now();
        render(&plain, &mut fonts);
        let without = start.elapsed();
        let start = Instant::now();
        let pixmap = render(&project, &mut fonts);
        let with = start.elapsed();
        // The lettering's shadow spreads far around it.
        let covered = |p: &Pixmap| p.pixels().iter().filter(|c| c.alpha() > 0).count();
        let plain_pixmap = render(&plain, &mut fonts);
        println!(
            "covered: {} vs {}",
            covered(&pixmap),
            covered(&plain_pixmap)
        );
        assert!(covered(&pixmap) > covered(&plain_pixmap) * 3);
        assert!(pixmap.pixel(2048, 2048 + 600).unwrap().alpha() > 0);
        let start = Instant::now();
        tp_render::encode_dds(&pixmap, DdsEncoding::Bc3, &mut |_, _| true).unwrap();
        let encode = start.elapsed();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        let start = Instant::now();
        assert_eq!(
            export(&project, &path),
            ExportOutcome::Written(path.clone())
        );
        let whole = start.elapsed();
        println!(
            "4096 px, 20 texts at blur 8 and a lettering at blur 200:\n  \
             render without shadows {without:?}\n  render with shadows {with:?}\n  \
             DDS encoding {encode:?}\n  Export Texture (render + DDS) {:?}\n  \
             Export Mod (whole mod) {whole:?}",
            with + encode
        );
        let limit = std::time::Duration::from_secs(3);
        assert!(
            with + encode < limit,
            "Export Texture took {:?}",
            with + encode
        );
        assert!(whole < limit, "Export Mod took {whole:?}");
    }

    #[test]
    fn cancelling_leaves_no_file_and_keeps_an_earlier_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        std::fs::write(&path, b"earlier export").unwrap();
        // 4096 px textures: the job can't end before it is cancelled.
        let project = truck(&["standard"]);
        let job = ModJob::start(
            project.clone(),
            plan(&project).unwrap(),
            FontLibrary::bundled(),
            path.clone(),
            || {},
        );
        job.cancel();
        assert_eq!(job.wait(), ExportOutcome::Cancelled);
        assert_eq!(std::fs::read(&path).unwrap(), b"earlier export");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn an_unwritable_destination_is_reported() {
        let path = PathBuf::from("/nonexistent-dir/ACE.scs");
        assert!(matches!(
            export(&small_truck(), &path),
            ExportOutcome::Failed { path: p, .. } if p == path
        ));
    }

    #[test]
    fn exports_are_reproducible() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (dir.path().join("a.scs"), dir.path().join("b.scs"));
        let project = small_truck();
        export(&project, &a);
        export(&project, &b);
        assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
    }

    #[test]
    fn a_chosen_image_is_used() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        let mut project = small_truck();
        let red = image::RgbaImage::from_pixel(1280, 720, image::Rgba([255, 0, 0, 255]));
        let mut png = Vec::new();
        red.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let (asset, _) = project.add_asset(
            "Picture",
            tp_core::AssetKind::Raster,
            Arc::from(png.as_slice()),
            tp_core::kurbo::Size::new(1280.0, 720.0),
        );
        project.mod_settings.image = Some(asset);
        export(&project, &path);
        let jpeg = read_entry(&mut archive(&path), "icon.jpg");
        let image = image::load_from_memory(&jpeg).unwrap().to_rgb8();
        let [r, g, b] = image.get_pixel(138, 81).0;
        assert!(r > 240 && g < 20 && b < 20, "{r} {g} {b}");
    }

    #[test]
    fn mod_folders_per_platform() {
        let documents = Path::new("/home/jane/Documents");
        let data = Path::new("/home/jane/data");
        let base = if cfg!(windows) { documents } else { data };
        assert_eq!(
            mod_folder_in("ets2", Some(documents), Some(data)),
            Some(base.join("Euro Truck Simulator 2").join("mod"))
        );
        assert_eq!(
            mod_folder_in("ats", Some(documents), Some(data)),
            Some(base.join("American Truck Simulator").join("mod"))
        );
        assert_eq!(mod_folder_in("other", Some(documents), Some(data)), None);
        assert_eq!(mod_folder_in("ets2", None, None), None);
    }

    #[test]
    fn only_an_existing_mod_folder_is_used() {
        let dir = tempfile::tempdir().unwrap();
        let some = Some(dir.path());
        assert_eq!(existing_mod_folder("ets2", some, some), None);
        let folder = mod_folder_in("ets2", some, some).unwrap();
        std::fs::create_dir_all(&folder).unwrap();
        assert_eq!(existing_mod_folder("ets2", some, some), Some(folder));
        assert_eq!(existing_mod_folder("ats", some, some), None);
    }

    #[test]
    fn names_and_extension() {
        assert_eq!(suggested_name("ACE Logistics"), "ACE Logistics.scs");
        assert_eq!(suggested_name("A/B: C"), "A-B- C.scs");
        assert_eq!(suggested_name(".."), "mod.scs");
        assert_eq!(with_extension("/a/ace".into()), PathBuf::from("/a/ace.scs"));
        assert_eq!(
            with_extension("/a/ace.SCS".into()),
            PathBuf::from("/a/ace.SCS")
        );
    }

    /// The sample truck's package id.
    const TRUCK_ID: &str = "community.truckpaint.sample_truck";

    #[test]
    fn each_problem_is_fixed_in_its_place() {
        use crate::game_versions::VehicleRange;
        let range = |id: &str| VehicleRange {
            vehicle: "V".into(),
            range: ">=1.56".into(),
            package_id: id.into(),
        };
        let field = ProblemPlace::Field;
        let cases = [
            (Problem::NameEmpty, field(ModField::Name)),
            (Problem::NameInvalid, field(ModField::Name)),
            (Problem::VersionInvalid, field(ModField::Version)),
            (Problem::AuthorInvalid, field(ModField::Author)),
            (Problem::InternalNameEmpty, field(ModField::InternalName)),
            (Problem::InternalNameInvalid, field(ModField::InternalName)),
            (
                Problem::InternalNameTooLong { max: 10 },
                field(ModField::InternalName),
            ),
            (Problem::PriceZero, field(ModField::Price)),
            (
                Problem::SamePath {
                    path: "p".into(),
                    first: "A".into(),
                    second: "B".into(),
                    package_id: "b".into(),
                },
                ProblemPlace::Vehicle("b".into()),
            ),
            (
                Problem::MissingGameData {
                    vehicle: "A".into(),
                    version: "1.0.0".into(),
                    package_id: "a".into(),
                },
                ProblemPlace::Vehicle("a".into()),
            ),
            (
                Problem::BadGameVersion {
                    version: "1.56.x".into(),
                },
                field(ModField::GameVersions),
            ),
            (
                Problem::UnsupportedGameVersion {
                    version: "1.55.*".into(),
                    vehicle: range("a"),
                },
                field(ModField::GameVersions),
            ),
            (
                Problem::NoCommonGameVersion {
                    first: range("first"),
                    second: range("second"),
                },
                ProblemPlace::Vehicle("first".into()),
            ),
        ];
        for (problem, place) in cases {
            assert_eq!(problem.place(), place, "{problem:?}");
        }
        // The messages are unchanged.
        assert_eq!(Problem::NameEmpty.message(), "The mod needs a name.");
        assert_eq!(
            Problem::BadGameVersion {
                version: "1.56.x".into()
            }
            .message(),
            "1.56.x isn't a game version: write it like 1.56.* or 1.56.2."
        );
    }

    #[test]
    fn each_setting_edit_is_one_step() {
        let mut ws = Workspace::new(truck(&["standard"]));
        let steps = ws.history.len();
        ws.set_mod_setting(1.0, |s| s.author = "Jane".into());
        assert_eq!(ws.project.mod_settings.author, "Jane");
        assert_eq!(ws.history.len(), steps + 1);
        assert_eq!(ws.history.undo_label(), Some("undo-edit-mod-settings"));
        // An unchanged value records nothing.
        let name = ws.project.mod_settings.name.clone();
        ws.set_mod_setting(2.0, |s| s.name = name);
        assert_eq!(ws.history.len(), steps + 1);
        ws.undo();
        assert_eq!(ws.project.mod_settings.author, "");
    }

    /// A PNG of `w` × `h` pixels of `rgb`.
    fn png(w: u32, h: u32, rgb: [u8; 3]) -> PictureFile {
        let image = image::RgbaImage::from_pixel(w, h, image::Rgba([rgb[0], rgb[1], rgb[2], 255]));
        let mut bytes = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        PictureFile::read("picture", bytes).unwrap()
    }

    #[test]
    fn choosing_a_picture_adds_an_asset_and_generated_removes_it() {
        let mut ws = Workspace::new(truck(&["standard"]));
        let assets = ws.project.assets.len();
        let steps = ws.history.len();
        let red = Picture::File(png(8, 4, [255, 0, 0]));
        ws.set_mod_picture(ModPicture::Image, &red, 1.0);
        let first = ws.project.mod_settings.image.expect("chosen");
        assert_eq!(ws.project.assets.len(), assets + 1);
        assert_eq!(ws.project.mod_settings.icon, None);
        assert_eq!(ws.history.len(), steps + 1);
        assert_eq!(ws.history.undo_label(), Some("undo-edit-mod-settings"));
        // Another picture replaces it: the first one is removed.
        ws.set_mod_picture(
            ModPicture::Image,
            &Picture::File(png(8, 4, [0, 0, 255])),
            2.0,
        );
        let second = ws.project.mod_settings.image.expect("chosen");
        assert_ne!(first, second);
        assert!(!ws.project.assets.contains_key(&first));
        assert_eq!(ws.project.assets.len(), assets + 1);
        // Undo restores the previous picture and its asset.
        ws.undo();
        assert_eq!(ws.project.mod_settings.image, Some(first));
        assert!(ws.project.assets.contains_key(&first));
        assert!(!ws.project.assets.contains_key(&second));
        // Back to the generated picture: the asset goes away.
        ws.set_mod_picture(ModPicture::Image, &Picture::Generated, 3.0);
        assert_eq!(ws.project.mod_settings.image, None);
        assert_eq!(ws.project.assets.len(), assets);
        // Generated again: nothing recorded.
        let steps = ws.history.len();
        ws.set_mod_picture(ModPicture::Image, &Picture::Generated, 4.0);
        assert_eq!(ws.history.len(), steps);
    }

    #[test]
    fn destination_of_a_mod() {
        let dir = tempfile::tempdir().unwrap();
        let folders = Folders {
            documents: Some(dir.path().join("Documents")),
            data: Some(dir.path().join("data")),
            home: Some(dir.path().to_path_buf()),
        };
        let none = BTreeMap::new();
        let mut project = truck(&["standard"]);
        project.mod_settings.name = "ACE Logistics".into();
        let last = dir.path().join("last");
        // No game folder: the last export's folder, else Documents, else
        // the home folder.
        assert_eq!(
            destination(&project, &none, Some(&last), &folders),
            last.join("ACE Logistics.scs")
        );
        assert_eq!(
            destination(&project, &none, None, &folders),
            dir.path().join("Documents").join("ACE Logistics.scs")
        );
        let homeless = Folders {
            documents: None,
            ..folders.clone()
        };
        assert_eq!(
            destination(&project, &none, None, &homeless),
            dir.path().join("ACE Logistics.scs")
        );
        // The game's mod folder when it exists.
        let game = mod_folder_in(
            "ets2",
            folders.documents.as_deref(),
            folders.data.as_deref(),
        )
        .unwrap();
        std::fs::create_dir_all(&game).unwrap();
        assert_eq!(
            destination(&project, &none, Some(&last), &folders),
            game.join("ACE Logistics.scs")
        );
        // Characters that can't be in a file name.
        project.mod_settings.name = "A/B: C".into();
        assert_eq!(
            destination(&project, &none, None, &folders),
            game.join("A-B- C.scs")
        );
        // The mod folder the player chose wins when it exists, else the
        // game's folder as before.
        project.mod_settings.name = "ACE Logistics".into();
        let mut chosen = BTreeMap::from([("ets2".to_owned(), dir.path().join("chosen"))]);
        assert_eq!(
            destination(&project, &chosen, None, &folders),
            game.join("ACE Logistics.scs")
        );
        std::fs::create_dir_all(dir.path().join("chosen")).unwrap();
        assert_eq!(
            destination(&project, &chosen, None, &folders),
            dir.path().join("chosen").join("ACE Logistics.scs")
        );
        chosen.clear();
    }

    const TRUCK_DEF: &str = "def/vehicle/truck/truckpaint.sample/paint_job";
    const TRUCK_TEX: &str = "vehicle/truck/upgrade/paintjob/ace/truckpaint.sample";

    #[test]
    fn a_valid_project_has_no_problem() {
        assert_eq!(problems(&truck(&["standard"])), []);
    }

    #[test]
    fn each_problem_is_reported() {
        let check = |edit: &dyn Fn(&mut Project), expected: Problem| {
            let mut p = truck(&["standard"]);
            edit(&mut p);
            assert_eq!(problems(&p), [expected]);
        };
        check(&|p| p.mod_settings.name = " ".into(), Problem::NameEmpty);
        check(
            &|p| p.mod_settings.name = "A \"B\"".into(),
            Problem::NameInvalid,
        );
        check(
            &|p| p.mod_settings.version = "1\\0".into(),
            Problem::VersionInvalid,
        );
        check(
            &|p| p.mod_settings.author = "A\nB".into(),
            Problem::AuthorInvalid,
        );
        check(
            &|p| p.mod_settings.internal_name = Some(String::new()),
            Problem::InternalNameEmpty,
        );
        check(
            &|p| p.mod_settings.internal_name = Some("Ace".into()),
            Problem::InternalNameInvalid,
        );
        // The sample truck has two main textures: 10 characters at most.
        check(
            &|p| p.mod_settings.internal_name = Some("ace_logistic".into()),
            Problem::InternalNameTooLong { max: 10 },
        );
        check(&|p| p.mod_settings.price = 0, Problem::PriceZero);
        check(
            &|p| p.vehicles[0].game_data = None,
            Problem::MissingGameData {
                vehicle: "TruckPaint Sample Truck".into(),
                version: "1.1.0".into(),
                package_id: TRUCK_ID.into(),
            },
        );
        let mut p = truck(&["standard"]);
        let mut twin = p.vehicles[0].clone();
        twin.name = "Twin".into();
        twin.package_id = "custom.twin".into();
        p.vehicles.push(twin);
        assert_eq!(
            problems(&p),
            [Problem::SamePath {
                path: "truckpaint.sample".into(),
                first: "TruckPaint Sample Truck".into(),
                second: "Twin".into(),
                package_id: "custom.twin".into(),
            }]
        );
        assert!(plan(&p).is_err());
    }

    #[test]
    fn summary_lists_painted_and_unpainted_cabins() {
        let package = example("sample_truck", "1.1.0");
        let p = truck(&["standard", "chassis", "cab_accessories"]);
        let manifest = package.manifest.clone();
        let list = summary(&p, &|_| Some(manifest.clone()));
        assert_eq!(
            list,
            [VehicleSummary {
                name: "TruckPaint Sample Truck".into(),
                trailer: false,
                mains: vec![
                    MainSummary {
                        name: "Standard cab".into(),
                        cabins: ids(&["standard"]),
                        painted: true,
                    },
                    MainSummary {
                        name: "High roof".into(),
                        cabins: ids(&["high_roof"]),
                        painted: false,
                    },
                ],
                accessories: ids(&["Chassis", "Cab accessories"]),
            }]
        );
        // Without the package, only what is painted.
        assert_eq!(summary(&p, &|_| None)[0].mains.len(), 1);
    }

    #[test]
    fn game_version_problems() {
        use crate::game_versions::VehicleRange;
        let truck_range = || VehicleRange {
            vehicle: "TruckPaint Sample Truck".into(),
            range: ">=1.56".into(),
            package_id: TRUCK_ID.into(),
        };
        let mut p = truck(&["standard"]);
        p.game_versions = vec!["1.56.x".into()];
        assert_eq!(
            problems(&p),
            [Problem::BadGameVersion {
                version: "1.56.x".into()
            }]
        );
        p.game_versions = vec!["1.55.*".into(), "1.56.*".into()];
        assert_eq!(
            problems(&p),
            [Problem::UnsupportedGameVersion {
                version: "1.55.*".into(),
                vehicle: truck_range(),
            }]
        );
        p.game_versions = vec!["1.56.*".into(), "1.57.*".into()];
        assert_eq!(problems(&p), []);
        // A second vehicle (another game path) that stops before 1.55.
        let mut old = p.vehicles[0].clone();
        old.name = "Old Hauler".into();
        old.package_id = "custom.old.hauler".into();
        let data = old.game_data.as_mut().unwrap();
        data.versions = "<1.55".into();
        data.path = "old.hauler".into();
        p.vehicles.push(old);
        p.game_versions.clear();
        assert_eq!(
            problems(&p),
            [Problem::NoCommonGameVersion {
                first: truck_range(),
                second: VehicleRange {
                    vehicle: "Old Hauler".into(),
                    range: "<1.55".into(),
                    package_id: "custom.old.hauler".into(),
                },
            }]
        );
        assert!(plan(&p).is_err());
    }

    #[test]
    fn game_versions_in_the_manifest() {
        let mut p = truck(&["standard"]);
        p.game_versions = vec!["1.56.*".into(), "1.57.*".into()];
        let manifest = plan(&p).unwrap().text("manifest.sii").unwrap().to_owned();
        assert!(
            manifest.contains(
                "\tdescription_file: \"description.txt\"\n\
             \tcompatible_versions[]: \"1.56.*\"\n\
             \tcompatible_versions[]: \"1.57.*\"\n}"
            ),
            "{manifest}"
        );
        p.game_versions.clear();
        let manifest = plan(&p).unwrap().text("manifest.sii").unwrap().to_owned();
        assert!(!manifest.contains("compatible_versions"), "{manifest}");
    }

    #[test]
    fn tobj_matches_paintjob_packer() {
        let reference = include_bytes!("../tests/fixtures/paintjob-packer.tobj");
        assert_eq!(
            tobj("/vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/standard.dds"),
            reference
        );
    }

    #[test]
    fn truck_with_two_cabin_layouts() {
        let plan = plan(&truck(&["standard", "high_roof", "chassis", "side_skirts"])).unwrap();
        let paths: Vec<&str> = plan.files.keys().map(String::as_str).collect();
        let mut expected = vec![
            "description.txt".to_owned(),
            "icon.jpg".into(),
            "manifest.sii".into(),
            "material/ui/accessory/ace_icon.dds".into(),
            "material/ui/accessory/ace_icon.mat".into(),
            "material/ui/accessory/ace_icon.tobj".into(),
            format!("{TRUCK_DEF}/ace_a.sii"),
            format!("{TRUCK_DEF}/ace_b.sii"),
            format!("{TRUCK_DEF}/ace_settings.sui"),
            format!("{TRUCK_DEF}/accessory/ace_a.sii"),
            format!("{TRUCK_DEF}/accessory/ace_b.sii"),
        ];
        for t in ["chassis", "high_roof", "side_skirts", "standard"] {
            expected.push(format!("{TRUCK_TEX}/{t}.dds"));
            expected.push(format!("{TRUCK_TEX}/{t}.tobj"));
        }
        expected.sort();
        assert_eq!(paths, expected);
        assert_eq!(
            plan.text(&format!("{TRUCK_DEF}/ace_a.sii")).unwrap(),
            "SiiNunit\n{\naccessory_paint_job_data : ace_a.truckpaint.sample.paint_job\n{\n\
             @include \"ace_settings.sui\"\n\
             \tsuitable_for[]: \"standard.truckpaint.sample.cabin\"\n\
             \tpaint_job_mask: \"/vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/standard.tobj\"\n\
             }\n}\n"
        );
        assert!(
            plan.text(&format!("{TRUCK_DEF}/ace_b.sii"))
                .unwrap()
                .contains("suitable_for[]: \"high_roof.truckpaint.sample.cabin\"")
        );
        let accessories = "SiiNunit\n{\n\
             \nsimple_paint_job_data : .ovr0\n{\n\
             \tpaint_job_mask: \"/vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/chassis.tobj\"\n\
             \tacc_list[]: \"chassis.sample\"\n}\n\
             \nsimple_paint_job_data : .ovr1\n{\n\
             \tpaint_job_mask: \"/vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/side_skirts.tobj\"\n\
             \tacc_list[]: \"sideskirt.sample\"\n}\n}\n";
        assert_eq!(
            plan.text(&format!("{TRUCK_DEF}/accessory/ace_a.sii")),
            Some(accessories)
        );
        assert_eq!(
            plan.text(&format!("{TRUCK_DEF}/accessory/ace_b.sii")),
            Some(accessories)
        );
        assert_eq!(
            plan.text(&format!("{TRUCK_DEF}/ace_settings.sui")).unwrap(),
            "\tname: \"ACE\"\n\tprice: 5000\n\tunlock: 0\n\ticon: \"ace_icon\"\n\tairbrush: true\n"
        );
        assert_eq!(
            plan.files[&format!("{TRUCK_TEX}/side_skirts.dds")],
            Entry::Texture {
                surface: 3,
                width: 1024,
                height: 1024
            }
        );
        assert_eq!(
            plan.files[&format!("{TRUCK_TEX}/standard.tobj")],
            Entry::Bytes(tobj(&format!("/{TRUCK_TEX}/standard.dds")))
        );
        assert_eq!(
            plan.text("manifest.sii").unwrap(),
            "SiiNunit\n{\nmod_package : .package_name\n{\n\
             \tpackage_version: \"1.0\"\n\tdisplay_name: \"ACE\"\n\tauthor: \"\"\n\
             \tcategory[]: \"paint_job\"\n\tmp_mod_optional: true\n\
             \ticon: \"icon.jpg\"\n\tdescription_file: \"description.txt\"\n}\n}\n"
        );
        assert_eq!(
            plan.text("material/ui/accessory/ace_icon.mat").unwrap(),
            "material: \"ui\"\n{\n\ttexture: \"ace_icon.tobj\"\n\ttexture_name: \"texture\"\n}\n"
        );
    }

    #[test]
    fn unpainted_cabin_gets_no_paint_job() {
        let plan = plan(&truck(&["high_roof", "chassis"])).unwrap();
        assert!(plan.files.contains_key(&format!("{TRUCK_DEF}/ace_b.sii")));
        assert!(!plan.files.contains_key(&format!("{TRUCK_DEF}/ace_a.sii")));
        assert!(
            !plan
                .files
                .contains_key(&format!("{TRUCK_TEX}/standard.dds"))
        );
    }

    #[test]
    fn trailer_paint_job() {
        let plan = plan(&trailer(&["base", "mudflaps"])).unwrap();
        let def = "def/vehicle/trailer_owned/truckpaint.sample_trailer/paint_job";
        let unit = plan.text(&format!("{def}/ace.sii")).unwrap();
        assert!(
            unit.contains("accessory_paint_job_data : ace.truckpaint.sample_trailer.paint_job")
        );
        assert!(!unit.contains("suitable_for"), "{unit}");
        assert!(unit.contains(
            "paint_job_mask: \"/vehicle/trailer_owned/upgrade/paintjob/ace/truckpaint.sample_trailer/base.tobj\""
        ));
        let acc = plan.text(&format!("{def}/accessory/ace.sii")).unwrap();
        assert!(acc.contains("acc_list[]: \"r_mudflap.sample\""), "{acc}");
        assert!(!acc.contains("body"), "{acc}");
    }

    /// A project "ace" holding a Krone Cool Liner-like trailer whose main
    /// texture "Body" also paints the body accessories, painting `textures`.
    fn krone(textures: &[&str]) -> Project {
        use tp_vehicles::sample::{self, SampleTexture};
        let tex = |id, name| SampleTexture {
            id,
            name,
            size: tp_vehicles::TextureSize::new(256, 64),
            layout: 1,
        };
        let main = [tex("body", "Body")];
        let accessories = [tex("chassis", "Chassis")];
        let mut m = sample::manifest_with(
            "official.ets2.krone.coolliner",
            "Krone Cool Liner",
            "1.0.0",
            "trailer",
            &main,
            &accessories,
        );
        m["game"]["path"] = "krone.coolliner".into();
        m["paint_job"]["main"][0]["also_paints"] = vec!["body.reefer", "body.reef_13ct"].into();
        let all = [tex("body", "Body"), tex("chassis", "Chassis")];
        let package = Package::read(&sample::zip(&m, &sample::templates(&all))).unwrap();
        let mut p = fleet_project("ACE", &[(&package, &ids(textures))]).unwrap();
        p.mod_settings.internal_name = Some("ace".into());
        p
    }

    #[test]
    fn main_texture_applied_to_accessories() {
        let def = "def/vehicle/trailer_owned/krone.coolliner/paint_job";
        let body_tobj = "/vehicle/trailer_owned/upgrade/paintjob/ace/krone.coolliner/body.tobj";
        // Only the Body painted: the accessory file is still written.
        let only_body = plan(&krone(&["body"])).unwrap();
        let acc = only_body.text(&format!("{def}/accessory/ace.sii")).unwrap();
        assert_eq!(
            acc,
            format!(
                "SiiNunit\n{{\n\nsimple_paint_job_data : .ovr0\n{{\n\
                 \tpaint_job_mask: \"{body_tobj}\"\n\
                 \tacc_list[]: \"body.reefer\"\n\tacc_list[]: \"body.reef_13ct\"\n}}\n}}\n"
            )
        );
        // With the chassis: its own entry first, then the body's.
        let with_chassis = plan(&krone(&["body", "chassis"])).unwrap();
        let acc = with_chassis
            .text(&format!("{def}/accessory/ace.sii"))
            .unwrap();
        let chassis = acc.find("acc_list[]: \"chassis.sample\"").unwrap();
        let body = acc.find("acc_list[]: \"body.reefer\"").unwrap();
        assert!(chassis < body, "{acc}");
        assert!(acc.contains(".ovr1"), "{acc}");
        assert!(
            acc.contains(&format!("paint_job_mask: \"{body_tobj}\"")),
            "{acc}"
        );
    }

    #[test]
    fn fleet_in_one_mod() {
        let mut ws = Workspace::new(truck(&["standard"]));
        let t = example("sample_trailer", "1.0.0");
        ws.add_vehicle(&t, &ids(&["base"]), 1.0).unwrap();
        let plan = plan(&ws.project).unwrap();
        assert_eq!(
            plan.files
                .keys()
                .filter(|k| k.ends_with("manifest.sii"))
                .count(),
            1
        );
        assert!(
            plan.files.contains_key(
                "def/vehicle/trailer_owned/truckpaint.sample_trailer/paint_job/ace.sii"
            )
        );
        assert!(plan.files.contains_key(&format!("{TRUCK_DEF}/ace_a.sii")));
        assert_eq!(
            plan.text("description.txt").unwrap(),
            "Vehicles supported:\n- TruckPaint Sample Truck\n- TruckPaint Sample Trailer\n"
        );
    }

    #[test]
    fn alternate_uv_set_colour_picker_and_required_mods() {
        let mut p = trailer(&["base"]);
        p.mod_settings.description = "Company colors.\n".into();
        let data = p.vehicles[0].game_data.as_mut().unwrap();
        data.alt_uv = true;
        data.colour_picker = true;
        data.requires = vec![
            RequiredMod {
                name: "Trailer Pack".into(),
                version: Some(">=2.1".into()),
            },
            RequiredMod {
                name: "Wheels".into(),
                version: None,
            },
        ];
        let plan = plan(&p).unwrap();
        let settings = plan
            .text("def/vehicle/trailer_owned/truckpaint.sample_trailer/paint_job/ace_settings.sui")
            .unwrap();
        assert!(settings.ends_with("\talternate_uvset: true\n\tbase_color_locked: false\n"));
        assert_eq!(
            plan.text("description.txt").unwrap(),
            "Company colors.\n\nVehicles supported:\n- TruckPaint Sample Trailer\n\n\
             Requires:\n- Trailer Pack >=2.1\n- Wheels\n"
        );
        // No accessories painted: no accessory file.
        assert!(
            !plan
                .files
                .keys()
                .any(|k| k.contains("paint_job/accessory/"))
        );
    }

    /// The sample truck (both cabins, every accessory) and the sample
    /// trailer (Base and every accessory), nothing drawn.
    fn fleet() -> Project {
        let package = example("sample_truck", "1.1.0");
        let mut textures = crate::vehicle_project::default_textures(&package.manifest);
        textures.push("high_roof".into());
        let mut ws = Workspace::new(fleet_project("ACE", &[(&package, &textures)]).unwrap());
        let trailer = example("sample_trailer", "1.0.0");
        let all = crate::vehicle_project::default_textures(&trailer.manifest);
        ws.add_vehicle(&trailer, &all, 1.0).unwrap();
        ws.project
    }

    fn surface_named(p: &Project, name: &str) -> usize {
        p.surfaces.iter().position(|s| s.name == name).unwrap()
    }

    /// Puts a rectangle on texture `name` (Modified).
    fn draw(p: &mut Project, name: &str) {
        use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
        use tp_core::kurbo::{Point, Size};
        let i = surface_named(p, name);
        let id = ObjectId(9000 + i as u64);
        let frame = Frame::new(Point::new(50.0, 50.0), Size::new(10.0, 10.0), 0.0);
        p.surfaces[i]
            .objects
            .push(Arc::new(Object::new(id, ShapeKind::rectangle(), frame)));
    }

    fn flag(p: &mut Project, name: &str, status: TemplateStatus) {
        let i = surface_named(p, name);
        p.surfaces[i].template.as_mut().unwrap().status = status;
    }

    fn messages(p: &Project) -> Vec<String> {
        warnings(p).iter().map(|w| w.message(p)).collect()
    }

    #[test]
    fn warnings_of_a_fleet_with_work_left() {
        let mut p = fleet();
        let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Standard cab",
                "High roof",
                "Chassis",
                "Cab accessories",
                "Side skirts",
                "Base",
                "Curtain body 13.6 m",
                "Curtain body 10.5 m",
                "Mudflaps"
            ]
        );
        for name in ["Chassis", "Cab accessories", "Side skirts", "Mudflaps"] {
            draw(&mut p, name);
        }
        flag(&mut p, "Curtain body 13.6 m", TemplateStatus::LayoutChanged);
        assert_eq!(
            warnings(&p),
            [
                Warning::ToCheck {
                    surface: 6,
                    reason: CheckReason::LayoutChanged
                },
                Warning::Empty {
                    surfaces: vec![0, 1, 5, 7]
                }
            ]
        );
        assert_eq!(
            messages(&p),
            [
                "Curtain body 13.6 m: layout changed",
                "4 textures empty, exported with the game's color"
            ]
        );
        // Warnings never block the export.
        p.mod_settings.internal_name = Some("ace".into());
        assert_eq!(problems(&p), []);
        flag(&mut p, "Mudflaps", TemplateStatus::Removed);
        assert_eq!(
            messages(&p)[1],
            "Mudflaps: not in this version, left out of the mod"
        );
        assert_eq!(problems(&p), []);
    }

    #[test]
    fn one_empty_texture_is_named() {
        let mut p = fleet();
        let names: Vec<String> = p.surfaces.iter().map(|s| s.name.clone()).collect();
        for name in names.iter().filter(|n| *n != "High roof") {
            draw(&mut p, name);
        }
        assert_eq!(
            messages(&p),
            ["High roof is empty, exported with the game's color"]
        );
    }

    #[test]
    fn same_name_in_two_vehicles_gets_the_vehicle() {
        let mut p = fleet();
        let names: Vec<String> = p.surfaces.iter().map(|s| s.name.clone()).collect();
        for name in names.iter().filter(|n| *n != "Base") {
            draw(&mut p, name);
        }
        assert_eq!(texture_label(&p, 5), "Base");
        // A second trailer painting its own "Base".
        let mut twin = p.surfaces[5].clone();
        twin.template.as_mut().unwrap().package_id = "other.trailer".into();
        p.surfaces.push(twin);
        let mut vehicle = p.vehicles[1].clone();
        vehicle.package_id = "other.trailer".into();
        vehicle.name = "Other Trailer".into();
        p.vehicles.push(vehicle);
        draw(&mut p, "Mudflaps");
        let last = p.surfaces.len() - 1;
        p.surfaces[last].objects = p.surfaces[0].objects.clone();
        assert_eq!(
            messages(&p),
            ["TruckPaint Sample Trailer › Base is empty, exported with the game's color"]
        );
    }

    #[test]
    fn nothing_left_has_no_warning() {
        let mut p = fleet();
        let names: Vec<String> = p.surfaces.iter().map(|s| s.name.clone()).collect();
        for name in &names {
            draw(&mut p, name);
        }
        assert_eq!(warnings(&p), []);
    }
}
