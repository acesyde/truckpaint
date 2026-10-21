//! Builds vehicle packages (`.tpv`) from folders and checks them, with the
//! same validation as the application. Messages are in English: this is a
//! tool for package authors.

use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use serde_json::Value;
use tp_vehicles::{MANIFEST, Manifest, Package, PackageError};

pub mod custom;
pub mod dds;
pub mod extract;
pub mod synthetic;

/// Why a folder could not be packed or a package is invalid.
#[derive(Debug)]
pub enum PackError {
    NoManifest(PathBuf),
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
    /// Invalid JSON or a missing / mistyped field.
    BadManifest(String),
    UnsafePath(String),
    MissingFile {
        texture: String,
        path: String,
    },
    TooLarge {
        path: String,
    },
    UnsupportedDds {
        texture: String,
        detail: String,
    },
    /// Two referenced files end up at the same path in the package.
    DuplicateEntry(String),
    /// The package was built but does not pass validation.
    Package(PackageError),
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackError::NoManifest(dir) => {
                write!(f, "no {MANIFEST} in {}", dir.display())
            }
            PackError::Io { path, error } => write!(f, "{}: {error}", path.display()),
            PackError::BadManifest(why) => write!(f, "invalid {MANIFEST}: {why}"),
            PackError::UnsafePath(path) => {
                write!(
                    f,
                    "unsafe path \"{path}\": paths must be relative, without \"..\""
                )
            }
            PackError::MissingFile { texture, path } => {
                write!(
                    f,
                    "the {texture} template is missing: no file at \"{path}\""
                )
            }
            PackError::TooLarge { path } => write!(f, "\"{path}\" is too large"),
            PackError::UnsupportedDds { texture, detail } => {
                write!(f, "the {texture} template cannot be converted: {detail}")
            }
            PackError::DuplicateEntry(path) => {
                write!(f, "two files would be packed as \"{path}\"")
            }
            PackError::Package(e) => f.write_str(&describe(e)),
        }
    }
}

impl std::error::Error for PackError {}

/// A validation error in English.
pub fn describe(e: &PackageError) -> String {
    match e {
        PackageError::NotAZip => "not a ZIP file".into(),
        PackageError::NoManifest => format!("no {MANIFEST} in the package"),
        PackageError::BadManifest(why) => format!("invalid {MANIFEST}: {why}"),
        PackageError::NewerFormat(n) => {
            format!(
                "package format {n} is newer than this tool supports ({})",
                tp_vehicles::FORMAT
            )
        }
        PackageError::BadId(id) => {
            format!("invalid id \"{id}\": use lowercase words separated by dots, at least two")
        }
        PackageError::BadGamePath(path) => {
            format!("invalid game path \"{path}\": use words of a-z, 0-9 and _ separated by dots")
        }
        PackageError::NoMainTexture => "the paint job has no main texture".into(),
        PackageError::DuplicatePart(id) => format!("two parts have the id \"{id}\""),
        PackageError::MissingGameIds(name) => {
            format!("\"{name}\" has no game id (an accessory, or one of several main textures)")
        }
        PackageError::BadGameId(id) => {
            format!("invalid game id \"{id}\": use words of a-z, 0-9 and _ separated by dots")
        }
        PackageError::DuplicateGameId(id) => format!("the game id \"{id}\" is listed twice"),
        PackageError::BadAlsoPaints(name) => {
            format!("the accessory \"{name}\" has also_paints: only a main texture can")
        }
        PackageError::BadSize { texture, size } => format!(
            "the {texture} texture size {size} is invalid: it must be a width and a height, \
             [width, height], each a power of two from {} to {}",
            tp_vehicles::MIN_SIDE,
            tp_vehicles::MAX_SIDE
        ),
        PackageError::UnsafePath(path) => {
            format!("unsafe path \"{path}\": paths must be relative, without \"..\"")
        }
        PackageError::TooLarge => "the package is larger than 512 MB uncompressed".into(),
        PackageError::MissingTemplate { texture, path } => {
            format!("the {texture} template is missing: no file at \"{path}\"")
        }
        PackageError::BadTemplate { texture, path } => {
            format!("the {texture} template \"{path}\" is not a readable PNG or SVG")
        }
        PackageError::TemplateTooLarge { texture } => format!(
            "the {texture} template is larger than {} px",
            tp_vehicles::MAX_IMAGE_SIDE
        ),
        PackageError::BadPreview { path, cause } => {
            format!("the preview mesh \"{path}\" is invalid: {cause}")
        }
        PackageError::BadChangelog => format!(
            "the changelog is longer than {} characters",
            tp_vehicles::MAX_CHANGELOG
        ),
    }
}

/// A package built from a folder.
#[derive(Debug)]
pub struct Packed {
    pub bytes: Vec<u8>,
    pub manifest: Manifest,
    /// Files of the folder left out of the package (relative, `/`-separated).
    pub ignored: Vec<String>,
    /// DDS templates converted to PNG: (source path, packaged path).
    pub converted: Vec<(String, String)>,
}

impl Packed {
    /// The default file name: `<id>-<version>.tpv`.
    pub fn file_name(&self) -> String {
        format!(
            "{}-{}.{}",
            self.manifest.id,
            self.manifest.version,
            tp_vehicles::EXTENSION
        )
    }
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> PackError + '_ {
    move |error| PackError::Io {
        path: path.to_path_buf(),
        error,
    }
}

fn is_dds(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".dds")
}

/// The files a manifest references (templates, preview mesh and preview image), each with the
/// name of the part it is first used by.
fn referenced(manifest: &Manifest) -> BTreeMap<String, String> {
    let mut referenced: BTreeMap<String, String> = BTreeMap::new();
    for (_, part) in manifest.paint_job.parts() {
        referenced
            .entry(part.texture.template.clone())
            .or_insert_with(|| part.name.clone());
        if let Some(mesh) = &part.preview_mesh {
            referenced
                .entry(mesh.clone())
                .or_insert_with(|| part.name.clone());
        }
    }
    if let Some(preview) = &manifest.preview {
        referenced
            .entry(preview.clone())
            .or_insert_with(|| "preview".into());
    }
    referenced
}

fn parse(value: &Value) -> Result<Manifest, PackError> {
    tp_vehicles::manifest_from_value(value).map_err(|e| match e {
        PackageError::BadManifest(why) => PackError::BadManifest(why),
        other => PackError::Package(other),
    })
}

/// Packs `folder` (a `vehicle.json` and the files it references) and
/// validates the result. Nothing is written to disk.
pub fn pack_folder(folder: &Path) -> Result<Packed, PackError> {
    let manifest_path = folder.join(MANIFEST);
    if !manifest_path.is_file() {
        return Err(PackError::NoManifest(folder.to_path_buf()));
    }
    let text = std::fs::read(&manifest_path).map_err(io(&manifest_path))?;
    let value: Value =
        serde_json::from_slice(&text).map_err(|e| PackError::BadManifest(e.to_string()))?;
    let referenced = referenced(&parse(&value)?);

    let mut files = BTreeMap::new();
    for (path, texture) in &referenced {
        if !tp_vehicles::is_safe_path(path) || path == MANIFEST {
            return Err(PackError::UnsafePath(path.clone()));
        }
        let file = folder.join(path);
        let missing = || PackError::MissingFile {
            texture: texture.clone(),
            path: path.clone(),
        };
        let meta = std::fs::metadata(&file).map_err(|_| missing())?;
        if !meta.is_file() {
            return Err(missing());
        }
        if meta.len() > tp_vehicles::MAX_UNCOMPRESSED {
            return Err(PackError::TooLarge { path: path.clone() });
        }
        files.insert(path.clone(), std::fs::read(&file).map_err(io(&file))?);
    }
    let mut packed = pack_entries(value, files)?;
    list_ignored(folder, folder, &referenced, &mut packed.ignored);
    packed.ignored.sort();
    Ok(packed)
}

/// Packs a manifest and the files it references, given by their path in
/// the package, and validates the result. DDS templates are converted to
/// PNG. Files the manifest doesn't reference are left out.
pub fn pack_entries(
    manifest: Value,
    files: BTreeMap<String, Vec<u8>>,
) -> Result<Packed, PackError> {
    pack_entries_with(manifest, files, &mut || {})
}

/// [`pack_entries`], calling `converted_one` after each DDS template is
/// converted.
pub fn pack_entries_with(
    mut value: Value,
    mut files: BTreeMap<String, Vec<u8>>,
    converted_one: &mut dyn FnMut(),
) -> Result<Packed, PackError> {
    let referenced = referenced(&parse(&value)?);
    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut converted = Vec::new();
    for (path, texture) in &referenced {
        if !tp_vehicles::is_safe_path(path) || path == MANIFEST {
            return Err(PackError::UnsafePath(path.clone()));
        }
        let bytes = files.remove(path).ok_or_else(|| PackError::MissingFile {
            texture: texture.clone(),
            path: path.clone(),
        })?;
        if bytes.len() as u64 > tp_vehicles::MAX_UNCOMPRESSED {
            return Err(PackError::TooLarge { path: path.clone() });
        }
        let (name, bytes) = if is_dds(path) {
            let image = dds::decode(&bytes).map_err(|e| PackError::UnsupportedDds {
                texture: texture.clone(),
                detail: e.to_string(),
            })?;
            drop(bytes);
            let mut png = Vec::new();
            image
                .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
                .map_err(|e| PackError::UnsupportedDds {
                    texture: texture.clone(),
                    detail: e.to_string(),
                })?;
            let name = format!("{}.png", &path[..path.len() - 4]);
            converted.push((path.clone(), name.clone()));
            converted_one();
            (name, png)
        } else {
            (path.clone(), bytes)
        };
        if entries.insert(name.clone(), bytes).is_some() {
            return Err(PackError::DuplicateEntry(name));
        }
    }
    rewrite_templates(&mut value, &converted);

    let bytes = write_zip(&value, &entries);
    let manifest = Package::read(&bytes).map_err(PackError::Package)?.manifest;
    Ok(Packed {
        bytes,
        manifest,
        ignored: Vec::new(),
        converted,
    })
}

/// Points the converted templates at their PNG.
fn rewrite_templates(value: &mut Value, converted: &[(String, String)]) {
    for list in ["main", "accessories"] {
        let Some(parts) = value
            .pointer_mut(&format!("/paint_job/{list}"))
            .and_then(Value::as_array_mut)
        else {
            continue;
        };
        for part in parts {
            if let Some(template) = part.pointer_mut("/texture/template")
                && let Some((_, png)) = converted
                    .iter()
                    .find(|(dds, _)| template.as_str() == Some(dds.as_str()))
            {
                *template = Value::String(png.clone());
            }
        }
    }
}

/// A ZIP with the manifest first, then `entries` in path order, every
/// entry with the same date and options so the output is reproducible.
fn write_zip(manifest: &Value, entries: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644);
    let mut out = Vec::new();
    {
        let mut w = zip::ZipWriter::new(Cursor::new(&mut out));
        let mut json = serde_json::to_vec_pretty(manifest).expect("JSON value");
        json.push(b'\n');
        // Writing into memory cannot fail.
        w.start_file(MANIFEST, options).expect("zip");
        w.write_all(&json).expect("zip");
        for (name, bytes) in entries {
            w.start_file(name.as_str(), options).expect("zip");
            w.write_all(bytes).expect("zip");
        }
        w.finish().expect("zip");
    }
    out
}

/// Files under `dir` that are neither the manifest nor referenced; hidden
/// files and folders are skipped.
fn list_ignored(
    root: &Path,
    dir: &Path,
    referenced: &BTreeMap<String, String>,
    out: &mut Vec<String>,
) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            list_ignored(root, &path, referenced, out);
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if relative != MANIFEST && !referenced.contains_key(&relative) {
            out.push(relative);
        }
    }
}

/// Validates a package as the application does when installing.
pub fn check(bytes: &[u8]) -> Result<Manifest, PackError> {
    Package::read(bytes)
        .map(|p| p.manifest)
        .map_err(PackError::Package)
}

/// A readable summary of a manifest.
pub fn summary(m: &Manifest) -> String {
    let g = &m.game;
    let mut out = format!(
        "{} {}\n  {} ({} {}, {})\n  game path: {}\n  game versions: {}\n",
        m.id,
        m.version,
        m.name,
        m.brand,
        m.kind.code(),
        g.id.code(),
        g.path,
        g.versions
    );
    if g.alt_uv {
        out.push_str("  alternate UV set\n");
    }
    if g.colour_picker {
        out.push_str("  colour picker\n");
    }
    if let Some(changelog) = &m.changelog {
        out.push_str(&format!(
            "  changelog: {}\n",
            changelog.replace('\n', "\n    ")
        ));
    }
    for (title, parts) in [
        ("main textures", &m.paint_job.main),
        ("accessories", &m.paint_job.accessories),
    ] {
        if parts.is_empty() {
            continue;
        }
        out.push_str(&format!("  {title}:\n"));
        for p in parts {
            let t = &p.texture;
            let mut covers = if p.game_ids.is_empty() {
                "whole vehicle".to_owned()
            } else {
                p.game_ids.join(", ")
            };
            if !p.also_paints.is_empty() {
                covers.push_str(&format!("; also paints {}", p.also_paints.join(", ")));
            }
            out.push_str(&format!(
                "    {} ({}): {}, layout {}, {} [{}]\n",
                p.id,
                p.name,
                t.size.label(),
                t.layout_version,
                t.template,
                covers
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests;
