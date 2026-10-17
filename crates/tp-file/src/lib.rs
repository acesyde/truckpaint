//! The `.truckpaint` project file: a ZIP archive holding
//! - `mimetype`: `application/x-truckpaint` (identifies the file),
//! - `project.ron`: the document, starting with its `format` version,
//! - `assets/<id>.<ext>`: imported files, byte for byte.
//!
//! Every format version keeps a frozen module (`v1`, …). Reading parses a
//! file with the module of its own version and migrates it step by step to
//! the current one; writing always uses the current version.

pub mod library;
mod summary;
pub mod v1;

use std::io::{Cursor, Read, Write};
use std::path::Path;

use serde::Deserialize;
use tp_core::Project;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Current file format version.
pub const FORMAT_VERSION: u32 = 1;
/// File extension, without the dot.
pub const EXTENSION: &str = "truckpaint";
const MIMETYPE: &str = "application/x-truckpaint";
const DOCUMENT: &str = "project.ron";

/// The current format's module.
pub use v1 as current;

pub use summary::{Summary, SummaryTexture, SummaryVehicle};

/// Why a file could not be read or written.
#[derive(Debug)]
pub enum Error {
    /// Not a TruckPaint project at all.
    NotAProject,
    /// A project, but incomplete or inconsistent.
    Damaged(String),
    /// Written by an unreleased development build.
    Unsupported {
        found: u32,
    },
    /// Written by a newer TruckPaint.
    NewerVersion {
        found: u32,
        supported: u32,
    },
    Io(std::io::Error),
}

impl Error {
    /// Message for the user, about the file named `file`.
    pub fn message(&self, file: &str) -> String {
        match self {
            Self::NotAProject => format!("{file} is not a TruckPaint project."),
            Self::Damaged(_) => format!("{file} is damaged and cannot be opened."),
            Self::Unsupported { .. } => {
                format!("{file} uses a development format that this version cannot open.")
            }
            Self::NewerVersion { .. } => {
                format!("{file} was created with a newer version of TruckPaint.")
            }
            Self::Io(err) => format!("{file} could not be read or written ({err})."),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAProject => write!(f, "not a TruckPaint project"),
            Self::Damaged(why) => write!(f, "damaged project: {why}"),
            Self::Unsupported { found } => write!(f, "development format {found}"),
            Self::NewerVersion { found, supported } => {
                write!(f, "format {found} is newer than {supported}")
            }
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// A project read from a file.
#[derive(Debug)]
pub struct Opened {
    pub project: Project,
    /// The file used an older format and was converted.
    pub migrated: bool,
}

/// Only the version, read before anything else so it is known even when
/// the rest of the document does not match the current structures.
#[derive(Deserialize)]
struct Header {
    format: u32,
}

/// What development builds wrote in format 1 before it was released:
/// projects without a vehicle (blank textures), a single `vehicle`,
/// vehicles with `variants`, or surfaces with a single (square) `size`
/// instead of a width and a height. Read before the full document, which
/// they don't match.
fn is_development_v1(text: &str) -> bool {
    /// True when the field is present, whatever its value.
    fn present<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
        serde::de::IgnoredAny::deserialize(d).map(|_| true)
    }
    /// True when the field is a single number.
    fn number<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
        struct IsNumber;
        impl<'de> serde::de::Visitor<'de> for IsNumber {
            type Value = bool;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a size")
            }
            fn visit_f64<E>(self, _: f64) -> Result<bool, E> {
                Ok(true)
            }
            fn visit_i64<E>(self, _: i64) -> Result<bool, E> {
                Ok(true)
            }
            fn visit_u64<E>(self, _: u64) -> Result<bool, E> {
                Ok(true)
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<bool, A::Error> {
                while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                Ok(false)
            }
        }
        d.deserialize_any(IsNumber)
    }
    #[derive(Deserialize)]
    struct Vehicle {
        #[serde(default, deserialize_with = "present")]
        variants: bool,
    }
    #[derive(Deserialize)]
    struct Surface {
        #[serde(default, deserialize_with = "number")]
        size: bool,
    }
    #[derive(Deserialize)]
    struct Markers {
        #[serde(default)]
        vehicles: Vec<Vehicle>,
        #[serde(default, deserialize_with = "present")]
        vehicle: bool,
        #[serde(default)]
        surfaces: Vec<Surface>,
    }
    match ron_options().from_str::<Markers>(text) {
        Ok(m) => {
            m.vehicles.is_empty()
                || m.vehicle
                || m.vehicles.iter().any(|v| v.variants)
                || m.surfaces.iter().any(|s| s.size)
        }
        // Not even the markers parse: the full parse reports the damage.
        Err(_) => false,
    }
}

fn ron_options() -> ron::Options {
    ron::Options::default()
}

/// Serializes a project to `.truckpaint` bytes.
pub fn to_bytes(project: &Project) -> Result<Vec<u8>, Error> {
    zip_bytes(project, MIMETYPE, DOCUMENT)
}

/// A ZIP of `mimetype`, the document as `document` and the assets.
fn zip_bytes(project: &Project, mimetype: &str, document: &str) -> Result<Vec<u8>, Error> {
    let zip_err = |e: zip::result::ZipError| Error::Io(std::io::Error::other(e));
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("mimetype", stored).map_err(zip_err)?;
    zip.write_all(mimetype.as_bytes())?;

    let file = current::from_project(project);
    let text = ron_options()
        .to_string_pretty(&file, ron::ser::PrettyConfig::default())
        .map_err(|e| Error::Io(std::io::Error::other(e)))?;
    zip.start_file(document, deflated).map_err(zip_err)?;
    zip.write_all(text.as_bytes())?;

    for asset in project.assets.values() {
        let options = match asset.kind {
            tp_core::AssetKind::Svg => deflated,
            // Already compressed.
            tp_core::AssetKind::Raster => stored,
        };
        zip.start_file(current::asset_entry(asset), options)
            .map_err(zip_err)?;
        zip.write_all(&asset.bytes)?;
    }
    Ok(zip.finish().map_err(zip_err)?.into_inner())
}

/// Reads a project from `.truckpaint` bytes.
pub fn from_bytes(bytes: &[u8]) -> Result<Opened, Error> {
    let (mut zip, text) = open_zip(bytes, MIMETYPE, DOCUMENT)?;
    let header = header(&text)?;
    if header.format == 1 && is_development_v1(&text) {
        return Err(Error::Unsupported {
            found: header.format,
        });
    }
    let (document, migrated) = migrate(header.format, &text)?;
    let project = current::into_project(&document, |entry| entry_bytes(&mut zip, entry))
        .map_err(Error::Damaged)?;
    Ok(Opened { project, migrated })
}

type Zip<'a> = ZipArchive<Cursor<&'a [u8]>>;

/// The archive and its document, once `mimetype` is checked.
fn open_zip<'a>(
    bytes: &'a [u8],
    mimetype: &str,
    document: &str,
) -> Result<(Zip<'a>, String), Error> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|e| {
        // A ZIP that cannot be read (e.g. truncated) is a damaged project;
        // anything else is not a project at all.
        if bytes.starts_with(b"PK\x03\x04") {
            Error::Damaged(e.to_string())
        } else {
            Error::NotAProject
        }
    })?;
    let found = entry_bytes(&mut zip, "mimetype").ok_or(Error::NotAProject)?;
    if String::from_utf8_lossy(&found).trim() != mimetype {
        return Err(Error::NotAProject);
    }
    // Saving has no size limit, so the cap grows with the file: stored images
    // unpack 1:1 and the deflated document far below 64:1, a zip bomb far above.
    let limit = (bytes.len() as u64)
        .saturating_mul(64)
        .saturating_add(64 << 20);
    let mut total: u64 = 0;
    for i in 0..zip.len() {
        let entry = zip
            .by_index_raw(i)
            .map_err(|e| Error::Damaged(e.to_string()))?;
        total = total.saturating_add(entry.size());
    }
    if total > limit {
        return Err(Error::Damaged(format!(
            "unpacks to {total} bytes, more than {limit}"
        )));
    }
    let text =
        entry_bytes(&mut zip, document).ok_or_else(|| Error::Damaged("missing document".into()))?;
    let text = String::from_utf8(text).map_err(|e| Error::Damaged(e.to_string()))?;
    Ok((zip, text))
}

fn header(text: &str) -> Result<Header, Error> {
    ron_options()
        .from_str(text)
        .map_err(|e| Error::Damaged(e.to_string()))
}

/// An entry's bytes, never more than its declared size (the total of which
/// `open_zip` bounds).
fn entry_bytes(zip: &mut Zip<'_>, entry: &str) -> Option<Vec<u8>> {
    let entry = zip.by_name(entry).ok()?;
    let size = entry.size();
    let mut out = Vec::new();
    entry.take(size).read_to_end(&mut out).ok()?;
    Some(out)
}

/// Parses the document with its own version's structures and converts it
/// to the current version. Returns whether a conversion happened.
fn migrate(format: u32, text: &str) -> Result<(current::FileProject, bool), Error> {
    let parse_err = |e: ron::error::SpannedError| Error::Damaged(e.to_string());
    match format {
        // The next format adds its module and converts from the older ones
        // here, step by step (`vN → vN+1`).
        1 => Ok((ron_options().from_str(text).map_err(parse_err)?, false)),
        // Formats of unreleased development builds.
        2 | 3 => Err(Error::Unsupported { found: format }),
        0 => Err(Error::Damaged("invalid format version 0".into())),
        found => Err(Error::NewerVersion {
            found,
            supported: FORMAT_VERSION,
        }),
    }
}

/// Reads a project file.
pub fn read(path: &Path) -> Result<Opened, Error> {
    from_bytes(&std::fs::read(path)?)
}

/// The vehicles and textures of a project file, read from its document
/// only: no image is decoded and the file is not changed.
pub fn summary(path: &Path) -> Result<Summary, Error> {
    let bytes = std::fs::read(path)?;
    let (_, text) = open_zip(&bytes, MIMETYPE, DOCUMENT)?;
    let header = header(&text)?;
    if header.format == 1 && is_development_v1(&text) {
        return Err(Error::Unsupported {
            found: header.format,
        });
    }
    let (document, _) = migrate(header.format, &text)?;
    Ok(Summary::of_file(&document))
}

/// Writes a project file atomically: the content goes to a temporary file
/// next to `path`, flushed to disk, then renamed over `path`, so an error
/// never leaves a partial file behind.
pub fn write(project: &Project, path: &Path) -> Result<(), Error> {
    let bytes = to_bytes(project)?;
    write_atomic(path, &bytes)
}

/// Replaces `path` with `bytes` atomically.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".into());
    let temp = path.with_file_name(format!(".{name}.tmp-{}", std::process::id()));
    let result = (|| {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    Ok(result?)
}
