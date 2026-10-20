//! One source of game files in a [`Mount`](crate::Mount): an archive, a ZIP package or a
//! folder, read from a root inside it.

use std::io::Read;
use std::path::Path;

use crate::hashfs::Archive;
use crate::package::{DirPackage, ZipPackage};

enum Backing {
    Hash(Archive),
    Zip(ZipPackage),
    Dir(DirPackage),
}

pub struct Layer {
    backing: Backing,
    /// A folder of the source whose content is the game root: empty, or `/base`, `/dlc_x`.
    root: String,
    name: String,
}

impl Layer {
    /// Opens a HashFS archive, a ZIP package (by its first bytes, whatever its extension) or
    /// a folder, with the game root at `root` (`""`, `"base"`, `"dlc_toys"`).
    pub fn open(path: &Path, root: &str) -> Result<Self, String> {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let backing = if path.is_dir() {
            Backing::Dir(DirPackage::open(path)?)
        } else {
            let mut magic = [0; 4];
            std::fs::File::open(path)
                .and_then(|mut file| file.read_exact(&mut magic))
                .map_err(|e| format!("{name}: {e}"))?;
            match &magic {
                b"SCS#" => Backing::Hash(Archive::open(path)?),
                b"PK\x03\x04" | b"PK\x05\x06" => Backing::Zip(ZipPackage::open(path)?),
                _ => return Err(format!("{name}: not an SCS archive or a ZIP package")),
            }
        };
        let root = match root.trim_matches('/') {
            "" => String::new(),
            root => format!("/{root}"),
        };
        Ok(Self {
            backing,
            root,
            name,
        })
    }

    /// The file name of the source, with the root when there is one (`mod.zip/base`).
    pub fn name(&self) -> String {
        format!("{}{}", self.name, self.root)
    }

    fn inner(&self, path: &str) -> String {
        format!("{}{path}", self.root)
    }

    pub fn contains(&self, path: &str) -> bool {
        let path = self.inner(path);
        match &self.backing {
            Backing::Hash(archive) => archive.contains(&path),
            Backing::Zip(zip) => zip.contains(&path),
            Backing::Dir(dir) => dir.contains(&path),
        }
    }

    pub fn read(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        let path = self.inner(path);
        match &self.backing {
            Backing::Hash(archive) => archive.read(&path),
            Backing::Zip(zip) => zip.read(&path),
            Backing::Dir(dir) => dir.read(&path),
        }
    }

    /// Every file below `folder`, with full game paths (the root removed).
    pub fn walk(&self, folder: &str) -> Result<Vec<String>, String> {
        let inner = self.inner(folder);
        let files = match &self.backing {
            Backing::Hash(archive) => archive.walk(&inner)?,
            Backing::Zip(zip) => zip.walk(&inner),
            Backing::Dir(dir) => dir.walk(&inner)?,
        };
        Ok(files
            .into_iter()
            .filter_map(|file| file.strip_prefix(&self.root).map(str::to_owned))
            .collect())
    }

    /// The names of the folders at the top of the source, ignoring the root.
    pub fn folders(&self) -> Result<Vec<String>, String> {
        match &self.backing {
            Backing::Hash(archive) => Ok(archive
                .list("/")?
                .unwrap_or_default()
                .into_iter()
                .filter(|(_, folder)| *folder)
                .map(|(name, _)| name)
                .collect()),
            Backing::Zip(zip) => Ok(zip.folders()),
            Backing::Dir(dir) => dir.folders(),
        }
    }
}
