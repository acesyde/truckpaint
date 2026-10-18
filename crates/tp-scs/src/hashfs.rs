//! HashFS version 2 archives (`.scs`): a salted CityHash64 of each path indexes an entry table.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use flate2::read::ZlibDecoder;

use crate::city::city_hash64;

const HEADER_SIZE: usize = 53;
const ENTRY_SIZE: usize = 16;
const META_PLAIN: u8 = 0x80;
const META_DIRECTORY: u8 = 0x81;
const COMPRESSION_NONE: u32 = 0;
const COMPRESSION_ZLIB: u32 = 1;

struct Entry {
    hash: u64,
    meta_index: u32,
    meta_count: u16,
    directory: bool,
}

/// Where a file's bytes are, from its "plain" metadata record.
struct Location {
    compression: u32,
    stored: u64,
    size: u64,
    offset: u64,
}

pub struct Archive {
    path: PathBuf,
    name: String,
    length: u64,
    salt: u16,
    entries: Vec<Entry>,
    metadata: Vec<u32>,
}

fn le<const N: usize>(bytes: &[u8], at: usize) -> [u8; N] {
    bytes[at..at + N].try_into().unwrap()
}

impl Archive {
    pub fn open(path: &Path) -> Result<Self, String> {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let mut archive = Self {
            path: path.to_owned(),
            name,
            length: 0,
            salt: 0,
            entries: Vec::new(),
            metadata: Vec::new(),
        };
        archive.length = std::fs::metadata(path)
            .map_err(|e| format!("{}: {e}", archive.name))?
            .len();
        let header = archive.read_at(0, HEADER_SIZE as u64, "")?;
        if &header[..4] != b"SCS#" {
            return Err(format!(
                "{}: not an SCS archive (wrong magic)",
                archive.name
            ));
        }
        let version = u16::from_le_bytes(le(&header, 4));
        if version != 2 {
            return Err(format!(
                "{}: HashFS version {version} is not supported yet",
                archive.name
            ));
        }
        if &header[8..12] != b"CITY" {
            return Err(format!(
                "{}: hash method {:?} is not supported yet",
                archive.name,
                String::from_utf8_lossy(&header[8..12])
            ));
        }
        archive.salt = u16::from_le_bytes(le(&header, 6));
        let table = |count_at, size_at, offset_at, width| -> Result<Vec<u8>, String> {
            let count = u64::from(u32::from_le_bytes(le(&header, count_at)));
            let stored = u64::from(u32::from_le_bytes(le(&header, size_at)));
            let offset = u64::from_le_bytes(le(&header, offset_at));
            let size = count * width;
            let bytes = archive.read_at(offset, stored, "table")?;
            if stored == size {
                Ok(bytes)
            } else {
                archive.inflate(&bytes, size, "table")
            }
        };
        let entries = table(12, 16, 28, ENTRY_SIZE as u64)?;
        let metadata = table(20, 24, 36, 4)?;
        archive.entries = entries
            .as_chunks::<ENTRY_SIZE>()
            .0
            .iter()
            .map(|e| Entry {
                hash: u64::from_le_bytes(le(e, 0)),
                meta_index: u32::from_le_bytes(le(e, 8)),
                meta_count: u16::from_le_bytes(le(e, 12)),
                directory: e[14] & 1 != 0,
            })
            .collect();
        archive.metadata = metadata
            .as_chunks::<4>()
            .0
            .iter()
            .map(|m| u32::from_le_bytes(*m))
            .collect();
        Ok(archive)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn fail(&self, path: &str, cause: &str) -> String {
        if path.is_empty() {
            format!("{}: {cause}", self.name)
        } else {
            format!("{}: {path}: {cause}", self.name)
        }
    }

    /// Reads exactly `size` bytes; opens the file each time so readers share no cursor.
    fn read_at(&self, offset: u64, size: u64, path: &str) -> Result<Vec<u8>, String> {
        if offset.checked_add(size).is_none_or(|end| end > self.length) {
            return Err(self.fail(path, "data lies outside the archive"));
        }
        let mut bytes = vec![0; size as usize];
        File::open(&self.path)
            .and_then(|mut file| {
                file.seek(SeekFrom::Start(offset))?;
                file.read_exact(&mut bytes)
            })
            .map_err(|e| self.fail(path, &e.to_string()))?;
        Ok(bytes)
    }

    fn inflate(&self, bytes: &[u8], size: u64, path: &str) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        ZlibDecoder::new(bytes)
            .take(size + 1)
            .read_to_end(&mut out)
            .map_err(|e| self.fail(path, &format!("zlib: {e}")))?;
        if out.len() as u64 != size {
            return Err(self.fail(path, "size differs from the declared size after inflate"));
        }
        Ok(out)
    }

    fn entry(&self, path: &str) -> Option<&Entry> {
        let relative = path.strip_prefix('/')?;
        let key = if self.salt == 0 {
            relative.to_owned()
        } else {
            format!("{}{relative}", self.salt)
        };
        let hash = city_hash64(key.as_bytes());
        let index = self.entries.binary_search_by_key(&hash, |e| e.hash).ok()?;
        Some(&self.entries[index])
    }

    fn location(&self, entry: &Entry, path: &str) -> Result<Location, String> {
        let wanted = if entry.directory {
            META_DIRECTORY
        } else {
            META_PLAIN
        };
        let first = entry.meta_index as usize;
        let records = self
            .metadata
            .get(first..first + usize::from(entry.meta_count))
            .ok_or_else(|| self.fail(path, "metadata lies outside the table"))?;
        let record = records
            .iter()
            .find(|r| (*r >> 24) as u8 == wanted)
            .ok_or_else(|| self.fail(path, "entry has no data record"))?;
        let at = (record & 0xFF_FFFF) as usize;
        let words = self
            .metadata
            .get(at..at + 4)
            .ok_or_else(|| self.fail(path, "metadata lies outside the table"))?;
        // Top nibble of word 1 is a flag field this reader does not know.
        if words[1] >> 28 != 0 {
            return Err(self.fail(path, "entry uses flags this reader does not support"));
        }
        Ok(Location {
            compression: words[0] >> 28,
            stored: u64::from(words[0] & 0x0FFF_FFFF),
            size: u64::from(words[1] & 0x0FFF_FFFF),
            offset: u64::from(words[3]) * 16,
        })
    }

    fn contents(&self, entry: &Entry, path: &str) -> Result<Vec<u8>, String> {
        let at = self.location(entry, path)?;
        match at.compression {
            COMPRESSION_NONE => self.read_at(at.offset, at.size, path),
            COMPRESSION_ZLIB => {
                let bytes = self.read_at(at.offset, at.stored, path)?;
                self.inflate(&bytes, at.size, path)
            }
            other => Err(self.fail(
                path,
                &format!("compression method {other} is not supported yet"),
            )),
        }
    }

    /// Whether `path` is a file of this archive.
    pub fn contains(&self, path: &str) -> bool {
        self.entry(path).is_some_and(|e| !e.directory)
    }

    /// The bytes of a file, or `None` when this archive has no such file.
    pub fn read(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        match self.entry(path) {
            Some(entry) if !entry.directory => self.contents(entry, path).map(Some),
            _ => Ok(None),
        }
    }

    /// The `(name, is_folder)` entries of a folder, or `None` when it is absent.
    pub fn list(&self, folder: &str) -> Result<Option<Vec<(String, bool)>>, String> {
        let folder = if folder.len() > 1 {
            folder.trim_end_matches('/')
        } else {
            folder
        };
        let Some(entry) = self.entry(folder).filter(|e| e.directory) else {
            return Ok(None);
        };
        let bytes = self.contents(entry, folder)?;
        let bad = || self.fail(folder, "folder listing is malformed");
        let count = u32::from_le_bytes(le(bytes.get(..4).ok_or_else(bad)?, 0)) as usize;
        let lengths = bytes.get(4..4 + count).ok_or_else(bad)?;
        let mut at = 4 + count;
        let mut result = Vec::with_capacity(count);
        for &length in lengths {
            let name = bytes.get(at..at + usize::from(length)).ok_or_else(bad)?;
            at += usize::from(length);
            let name = String::from_utf8_lossy(name);
            result.push(match name.strip_prefix('/') {
                Some(folder) => (folder.to_owned(), true),
                None => (name.into_owned(), false),
            });
        }
        Ok(Some(result))
    }

    /// Every file below `folder`, with full game paths, sorted.
    pub fn walk(&self, folder: &str) -> Result<Vec<String>, String> {
        let mut files = Vec::new();
        let mut pending = vec![folder.trim_end_matches('/').to_owned()];
        while let Some(folder) = pending.pop() {
            let at = if folder.is_empty() { "/" } else { &folder };
            let Some(entries) = self.list(at)? else {
                continue;
            };
            for (name, is_folder) in entries {
                let full = format!("{folder}/{name}");
                if is_folder {
                    pending.push(full);
                } else {
                    files.push(full);
                }
            }
        }
        files.sort();
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::ArchiveBuilder;

    fn open(bytes: &[u8]) -> (tempfile::TempDir, Result<Archive, String>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.scs");
        std::fs::write(&path, bytes).unwrap();
        let archive = Archive::open(&path);
        (dir, archive)
    }

    fn sample() -> Vec<u8> {
        ArchiveBuilder::default()
            .file("/def/a.sii", b"stored")
            .compressed("/def/vehicle/b.sui", &b"zlib text ".repeat(50))
            .file("/root.txt", b"x")
            .build()
    }

    #[test]
    fn reads_stored_and_compressed_files_with_or_without_salt() {
        for salt in [0, 7, 1234] {
            let bytes = ArchiveBuilder::default()
                .salt(salt)
                .file("/def/a.sii", b"stored")
                .compressed("/def/vehicle/b.sui", &b"zlib text ".repeat(50))
                .build();
            let (_dir, archive) = open(&bytes);
            let archive = archive.unwrap();
            assert_eq!(archive.read("/def/a.sii").unwrap().unwrap(), b"stored");
            assert_eq!(
                archive.read("/def/vehicle/b.sui").unwrap().unwrap(),
                b"zlib text ".repeat(50)
            );
            assert!(archive.read("/def/missing.sii").unwrap().is_none());
            // A folder is not a file.
            assert!(archive.read("/def").unwrap().is_none());
            assert!(!archive.contains("/def/vehicle"));
        }
    }

    #[test]
    fn lists_one_level_and_recursively() {
        let (_dir, archive) = open(&sample());
        let archive = archive.unwrap();
        let mut root = archive.list("/").unwrap().unwrap();
        root.sort();
        assert_eq!(
            root,
            [("def".to_owned(), true), ("root.txt".to_owned(), false)]
        );
        let mut def = archive.list("/def/").unwrap().unwrap();
        def.sort();
        assert_eq!(
            def,
            [("a.sii".to_owned(), false), ("vehicle".to_owned(), true)]
        );
        assert!(archive.list("/nothing").unwrap().is_none());
        assert_eq!(
            archive.walk("/").unwrap(),
            ["/def/a.sii", "/def/vehicle/b.sui", "/root.txt"]
        );
        assert_eq!(
            archive.walk("/def/vehicle").unwrap(),
            ["/def/vehicle/b.sui"]
        );
        assert!(archive.walk("/nothing").unwrap().is_empty());
    }

    fn patched(offset: usize, bytes: &[u8]) -> Vec<u8> {
        let mut archive = sample();
        archive[offset..offset + bytes.len()].copy_from_slice(bytes);
        archive
    }

    #[test]
    fn refuses_unknown_headers_naming_the_archive_and_cause() {
        for (archive, cause) in [
            (patched(0, b"ZIP#"), "wrong magic"),
            (patched(4, &[3, 0]), "HashFS version 3 is not supported yet"),
            (patched(8, b"FNV1"), "hash method"),
            // Entry table beyond the end of the file.
            (patched(28, &u64::MAX.to_le_bytes()), "outside the archive"),
            (patched(16, &[0xff; 4]), "outside the archive"),
        ] {
            let (_dir, result) = open(&archive);
            let error = result.err().expect("must be refused");
            assert!(error.starts_with("test.scs: "), "{error}");
            assert!(error.contains(cause), "{error}");
        }
    }

    #[test]
    fn refuses_unsupported_compression_and_wrong_sizes_naming_the_path() {
        let bytes = ArchiveBuilder::default()
            .declared("/m/tex.tobj", b"data", 3)
            .declared("/m/headerless.sii", b"data", 2)
            .mismatched("/m/lie.sii", b"hello")
            .compressed("/m/short.sii", b"hello")
            .build();
        let (_dir, archive) = open(&bytes);
        let archive = archive.unwrap();
        for (path, cause) in [
            ("/m/tex.tobj", "compression method 3 is not supported yet"),
            ("/m/headerless.sii", "compression method 2"),
            ("/m/lie.sii", "size differs"),
        ] {
            let error = archive.read(path).unwrap_err();
            assert!(error.starts_with(&format!("test.scs: {path}: ")), "{error}");
            assert!(error.contains(cause), "{error}");
        }
        assert_eq!(archive.read("/m/short.sii").unwrap().unwrap(), b"hello");
    }

    #[test]
    fn damaged_data_fails_without_panic_on_every_truncation() {
        let bytes = sample();
        for length in (0..bytes.len()).step_by(3) {
            let (_dir, archive) = open(&bytes[..length]);
            if let Ok(archive) = archive {
                for path in ["/def/a.sii", "/def/vehicle/b.sui", "/root.txt"] {
                    let _ = archive.read(path);
                }
                let _ = archive.walk("/");
            }
        }
        // Any single damaged byte fails cleanly or reads something else, never panics.
        for at in 0..bytes.len() {
            let mut damaged = bytes.clone();
            damaged[at] ^= 0xA5;
            let (_dir, archive) = open(&damaged);
            if let Ok(archive) = archive {
                for path in ["/def/a.sii", "/def/vehicle/b.sui", "/root.txt"] {
                    let _ = archive.read(path);
                }
                let _ = archive.walk("/");
            }
        }
    }
}
