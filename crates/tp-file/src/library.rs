//! The personal library file: a ZIP like a project's, holding
//! - `mimetype`: `application/x-truckpaint-library`,
//! - `library.ron`: a format 1 document without vehicles, with one
//!   placeholder surface (the library's elements are its palette, styles
//!   and symbols),
//! - `assets/<id>.<ext>`: the images its symbols use.

use std::path::Path;

use tp_core::{Project, TextureResolution};

use crate::{Error, current};

/// File name of the library in the application's data folder.
pub const FILE_NAME: &str = "library.tplib";
const MIMETYPE: &str = "application/x-truckpaint-library";
const DOCUMENT: &str = "library.ron";

/// An empty library.
pub fn empty() -> Project {
    Project::new("Library", TextureResolution::R2048)
}

/// Serializes the library, leaving out the assets nothing uses anymore.
pub fn to_bytes(library: &Project) -> Result<Vec<u8>, Error> {
    let mut library = library.clone();
    let unused: Vec<_> = library
        .assets
        .keys()
        .copied()
        .filter(|a| library.asset_usage(*a) == 0)
        .collect();
    for a in unused {
        library.remove_asset(a);
    }
    crate::zip_bytes(&library, MIMETYPE, DOCUMENT)
}

/// Reads a library from its bytes.
pub fn from_bytes(bytes: &[u8]) -> Result<Project, Error> {
    let (mut zip, format, text) = crate::open_zip(bytes, MIMETYPE, DOCUMENT)?;
    let (document, _) = crate::migrate(format, &text)?;
    current::into_document(&document, |entry| crate::entry_bytes(&mut zip, entry))
        .map_err(Error::Damaged)
}

/// Reads a library file.
pub fn read(path: &Path) -> Result<Project, Error> {
    from_bytes(&std::fs::read(path)?)
}

/// Writes a library file atomically (see [`crate::write`]).
pub fn write(library: &Project, path: &Path) -> Result<(), Error> {
    crate::write_atomic(path, &to_bytes(library)?)
}
