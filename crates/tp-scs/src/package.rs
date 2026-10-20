//! Mod packages that are not HashFS archives: a ZIP file, or a folder.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use zip::ZipArchive;
use zip::result::ZipError;

/// A ZIP package: the list of files is read once, an entry only when asked for.
pub struct ZipPackage {
    name: String,
    archive: Mutex<ZipArchive<File>>,
    /// Lowercase game path -> (index in the archive, path as written).
    entries: BTreeMap<String, (usize, String)>,
}

/// Whether `name` is a safe relative entry name; the reason when it is not.
fn unsafe_name(name: &str) -> Option<&'static str> {
    if name.starts_with('/') {
        Some("absolute entry name")
    } else if name.contains('\\') || name.contains('\0') || name.contains(':') {
        Some("entry name with a backslash, NUL or colon")
    } else if name
        .trim_end_matches('/')
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        Some("entry name that leaves the package")
    } else {
        None
    }
}

impl ZipPackage {
    pub fn open(path: &Path) -> Result<Self, String> {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let file = File::open(path).map_err(|e| format!("{name}: {e}"))?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| format!("{name}: not a readable ZIP: {e}"))?;
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let entry = archive
                .by_index_raw(index)
                .map_err(|e| format!("{name}: entry {index}: {e}"))?;
            let entry_name = entry.name().to_owned();
            if let Some(reason) = unsafe_name(&entry_name) {
                return Err(format!("{name}: {entry_name}: {reason}"));
            }
            if entry.is_dir() {
                continue;
            }
            let game_path = format!("/{entry_name}");
            entries.insert(game_path.to_lowercase(), (index, game_path));
        }
        Ok(Self {
            name,
            archive: Mutex::new(archive),
            entries,
        })
    }

    pub fn contains(&self, path: &str) -> bool {
        self.entries.contains_key(&path.to_lowercase())
    }

    pub fn read(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        let Some(&(index, _)) = self.entries.get(&path.to_lowercase()) else {
            return Ok(None);
        };
        let fail = |cause: &str| format!("{}: {path}: {cause}", self.name);
        let mut archive = self.archive.lock().map_err(|_| fail("reader poisoned"))?;
        let mut entry = archive.by_index(index).map_err(|e| match e {
            ZipError::UnsupportedArchive(why) => fail(&format!("{why} is not supported")),
            ZipError::InvalidPassword => fail("entry is encrypted"),
            other => fail(&other.to_string()),
        })?;
        let size = entry.size();
        // The declared size is not trusted for the allocation: a lying header must not
        // reserve memory, and a stream longer than declared is an error.
        let mut bytes = Vec::new();
        (&mut entry)
            .take(size + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| fail(&e.to_string()))?;
        if bytes.len() as u64 != size {
            return Err(fail("size differs from the declared size after unpacking"));
        }
        Ok(Some(bytes))
    }

    /// The names of the folders at the top of the package, as written.
    pub fn folders(&self) -> Vec<String> {
        let tops: std::collections::BTreeSet<_> = self
            .entries
            .values()
            .filter_map(|(_, written)| written[1..].split_once('/').map(|(top, _)| top.to_owned()))
            .collect();
        tops.into_iter().collect()
    }

    /// Every file below `folder` (`/` for all), with its path as written, sorted.
    pub fn walk(&self, folder: &str) -> Vec<String> {
        let prefix = format!("{}/", folder.trim_end_matches('/').to_lowercase());
        self.entries
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(_, (_, written))| written.clone())
            .collect()
    }
}

/// A folder package.
pub struct DirPackage {
    name: String,
    root: PathBuf,
}

impl DirPackage {
    pub fn open(path: &Path) -> Result<Self, String> {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let root = path.canonicalize().map_err(|e| format!("{name}: {e}"))?;
        Ok(Self { name, root })
    }

    fn fail(&self, path: &str, cause: &str) -> String {
        format!("{}: {path}: {cause}", self.name)
    }

    /// The file for a game path, when it exists and does not leave the folder.
    fn file(&self, path: &str) -> Result<Option<PathBuf>, String> {
        let relative = path.trim_start_matches('/');
        let mut file = self.root.clone();
        for part in Path::new(relative).components() {
            match part {
                Component::Normal(part) => file.push(part),
                _ => return Err(self.fail(path, "path leaves the package")),
            }
        }
        match file.canonicalize() {
            Ok(real) if !real.starts_with(&self.root) => {
                Err(self.fail(path, "link leaves the package"))
            }
            Ok(real) => Ok(Some(real)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(self.fail(path, &e.to_string())),
        }
    }

    pub fn contains(&self, path: &str) -> bool {
        matches!(self.file(path), Ok(Some(file)) if file.is_file())
    }

    pub fn read(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        match self.file(path)? {
            Some(file) if file.is_file() => std::fs::read(file)
                .map(Some)
                .map_err(|e| self.fail(path, &e.to_string())),
            _ => Ok(None),
        }
    }

    pub fn walk(&self, folder: &str) -> Result<Vec<String>, String> {
        let mut files = Vec::new();
        let Some(start) = self.file(folder)? else {
            return Ok(files);
        };
        let mut pending = vec![(start, folder.trim_end_matches('/').to_owned())];
        while let Some((dir, game)) = pending.pop() {
            if !dir.is_dir() {
                continue;
            }
            let entries = std::fs::read_dir(&dir).map_err(|e| self.fail(&game, &e.to_string()))?;
            for entry in entries {
                let entry = entry.map_err(|e| self.fail(&game, &e.to_string()))?;
                let full = format!("{game}/{}", entry.file_name().to_string_lossy());
                // A link is resolved (and refused when it leaves the folder) by `file`.
                let Some(real) = self.file(&full)? else {
                    continue;
                };
                if real.is_dir() {
                    pending.push((real, full));
                } else {
                    files.push(full);
                }
            }
        }
        files.sort();
        Ok(files)
    }

    /// The names of the folders at the top of the package.
    pub fn folders(&self) -> Result<Vec<String>, String> {
        let entries = std::fs::read_dir(&self.root).map_err(|e| self.fail("/", &e.to_string()))?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| self.fail("/", &e.to_string()))?;
            if entry.path().is_dir() {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        names.sort();
        Ok(names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::zip_package;

    fn zip_file(files: &[(&str, &[u8])], deflate: bool) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mod.zip");
        std::fs::write(&path, zip_package(files, deflate)).unwrap();
        (dir, path)
    }

    #[test]
    fn zip_reads_stored_and_deflated_entries_by_game_path() {
        for deflate in [false, true] {
            let (_dir, path) = zip_file(
                &[
                    ("manifest.sii", b"hello"),
                    ("base/def/vehicle/truck/x/data.sii", &b"truck ".repeat(100)),
                ],
                deflate,
            );
            let zip = ZipPackage::open(&path).unwrap();
            assert_eq!(zip.read("/manifest.sii").unwrap().unwrap(), b"hello");
            assert_eq!(
                zip.read("/base/def/vehicle/truck/x/data.sii")
                    .unwrap()
                    .unwrap(),
                b"truck ".repeat(100)
            );
            // Game paths are matched without regard to case.
            assert!(zip.contains("/BASE/def/vehicle/truck/X/data.sii"));
            assert!(zip.read("/missing").unwrap().is_none());
            assert_eq!(zip.folders(), ["base"]);
            assert_eq!(
                zip.walk("/base/def"),
                ["/base/def/vehicle/truck/x/data.sii"]
            );
        }
    }

    #[test]
    fn opening_a_zip_reads_no_entry() {
        let (_dir, path) = zip_file(&[("a.sii", b"abc")], true);
        // Spoil the entry's data (after its 30-byte local header and 5-byte name): the
        // list of files at the end is intact, so opening works and only reading fails.
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[36] ^= 0xFF;
        std::fs::write(&path, bytes).unwrap();
        let zip = ZipPackage::open(&path).unwrap();
        assert!(zip.contains("/a.sii"));
        assert!(zip.read("/a.sii").unwrap_err().contains("mod.zip: /a.sii"));
    }

    #[test]
    fn zip_refuses_unsafe_entry_names_naming_package_and_entry() {
        for name in [
            "../x.sii",
            "a/../../x.sii",
            "/abs.sii",
            "a\\b.sii",
            "c:/x.sii",
        ] {
            let (_dir, path) = zip_file(&[(name, b"x")], false);
            let error = ZipPackage::open(&path).err().expect(name);
            assert!(
                error.starts_with("mod.zip: ") && error.contains(name),
                "{error}"
            );
        }
    }

    #[test]
    fn zip_with_a_cut_list_of_files_fails_naming_the_package() {
        let (_dir, path) = zip_file(&[("a.sii", b"abc")], false);
        let bytes = std::fs::read(&path).unwrap();
        std::fs::write(&path, &bytes[..bytes.len() - 10]).unwrap();
        assert!(
            ZipPackage::open(&path)
                .err()
                .unwrap()
                .starts_with("mod.zip: ")
        );
    }

    #[test]
    fn zip_with_an_unsupported_method_names_package_path_and_cause() {
        let (_dir, path) = zip_file(&[("a.sii", b"abcabcabc")], false);
        // Method 12 (bzip2, not built in) in the central directory: 10 bytes after the signature.
        let mut bytes = std::fs::read(&path).unwrap();
        let central = bytes.windows(4).rposition(|w| w == b"PK\x01\x02").unwrap();
        bytes[central + 10] = 12;
        std::fs::write(&path, bytes).unwrap();
        let zip = ZipPackage::open(&path).unwrap();
        let error = zip.read("/a.sii").unwrap_err();
        assert!(error.starts_with("mod.zip: /a.sii: "), "{error}");
    }

    #[test]
    fn folder_reads_by_game_path_and_lists_recursively() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("pkg");
        std::fs::create_dir_all(root.join("def/vehicle/truck/x")).unwrap();
        std::fs::write(root.join("def/vehicle/truck/x/data.sii"), b"data").unwrap();
        std::fs::write(root.join("manifest.sii"), b"m").unwrap();
        let package = DirPackage::open(&root).unwrap();
        assert_eq!(package.read("/manifest.sii").unwrap().unwrap(), b"m");
        assert!(package.contains("/def/vehicle/truck/x/data.sii"));
        assert!(!package.contains("/def"));
        assert!(package.read("/nothing").unwrap().is_none());
        assert_eq!(
            package.walk("/def").unwrap(),
            ["/def/vehicle/truck/x/data.sii"]
        );
        assert!(package.walk("/nothing").unwrap().is_empty());
        assert_eq!(package.folders().unwrap(), ["def"]);
        assert!(package.read("/../x").unwrap_err().contains("leaves"));
    }

    #[cfg(unix)]
    #[test]
    fn folder_refuses_a_link_that_leaves_it() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("pkg");
        std::fs::create_dir_all(root.join("def")).unwrap();
        std::fs::write(dir.path().join("secret.sii"), b"secret").unwrap();
        std::os::unix::fs::symlink(dir.path().join("secret.sii"), root.join("def/link.sii"))
            .unwrap();
        let package = DirPackage::open(&root).unwrap();
        assert!(
            package
                .read("/def/link.sii")
                .unwrap_err()
                .contains("link leaves the package")
        );
        assert!(package.walk("/def").is_err());
    }
}
