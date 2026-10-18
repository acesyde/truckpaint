//! Builds small HashFS v2 archives in memory, for tests here and in `tp-pack`: no game asset is needed.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::city::city_hash64;

fn deflate(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

#[derive(Default)]
pub struct ArchiveBuilder {
    salt: u16,
    no_root: bool,
    /// Path -> (stored bytes, declared compression id, declared size).
    files: BTreeMap<String, (Vec<u8>, u32, usize)>,
}

impl ArchiveBuilder {
    pub fn salt(mut self, salt: u16) -> Self {
        self.salt = salt;
        self
    }

    /// Leaves the root folder out, as `locale.scs` does: files stay reachable by path.
    pub fn without_root(mut self) -> Self {
        self.no_root = true;
        self
    }

    /// A file stored as it is.
    pub fn file(mut self, path: &str, bytes: &[u8]) -> Self {
        self.files
            .insert(path.to_owned(), (bytes.to_vec(), 0, bytes.len()));
        self
    }

    /// A zlib-compressed file.
    pub fn compressed(mut self, path: &str, bytes: &[u8]) -> Self {
        self.files
            .insert(path.to_owned(), (deflate(bytes), 1, bytes.len()));
        self
    }

    /// A file whose record declares compression `id` over the given bytes.
    pub fn declared(mut self, path: &str, bytes: &[u8], id: u32) -> Self {
        self.files
            .insert(path.to_owned(), (bytes.to_vec(), id, bytes.len()));
        self
    }

    /// A zlib file whose record declares one byte more than it inflates to.
    pub fn mismatched(mut self, path: &str, bytes: &[u8]) -> Self {
        self.files
            .insert(path.to_owned(), (deflate(bytes), 1, bytes.len() + 1));
        self
    }

    /// The archive bytes: folders are derived from the file paths; both tables are zlib-compressed.
    pub fn build(&self) -> Vec<u8> {
        let mut folders = BTreeMap::<String, BTreeSet<(String, bool)>>::new();
        folders.entry(String::new()).or_default();
        for path in self.files.keys() {
            let mut parts = path.trim_start_matches('/').split('/').collect::<Vec<_>>();
            let name = parts.pop().unwrap();
            let mut folder = String::new();
            for part in parts {
                folders
                    .entry(folder.clone())
                    .or_default()
                    .insert((part.to_owned(), true));
                folder = format!("{folder}/{part}");
                folders.entry(folder.clone()).or_default();
            }
            folders
                .entry(folder)
                .or_default()
                .insert((name.to_owned(), false));
        }
        // (relative path, is folder, stored bytes, compression id, size); a folder's data is its listing.
        let mut items = Vec::new();
        for (folder, children) in &folders {
            if self.no_root && folder.is_empty() {
                continue;
            }
            let mut listing = (children.len() as u32).to_le_bytes().to_vec();
            let names = children
                .iter()
                .map(|(name, is_folder)| format!("{}{name}", if *is_folder { "/" } else { "" }))
                .collect::<Vec<_>>();
            listing.extend(names.iter().map(|n| n.len() as u8));
            names.iter().for_each(|n| listing.extend(n.as_bytes()));
            let size = listing.len();
            items.push((
                folder.trim_start_matches('/').to_owned(),
                true,
                listing,
                0,
                size,
            ));
        }
        for (path, (bytes, id, size)) in &self.files {
            items.push((
                path.trim_start_matches('/').to_owned(),
                false,
                bytes.clone(),
                *id,
                *size,
            ));
        }
        let mut data = vec![0u8; 64];
        let mut rows = Vec::new();
        for (relative, directory, stored, id, size) in items {
            while !data.len().is_multiple_of(16) {
                data.push(0);
            }
            let offset = data.len() as u64;
            data.extend(&stored);
            let key = if self.salt == 0 {
                relative
            } else {
                format!("{}{relative}", self.salt)
            };
            rows.push((
                city_hash64(key.as_bytes()),
                directory,
                id,
                stored.len(),
                size,
                offset,
            ));
        }
        rows.sort_by_key(|row| row.0);
        let count = rows.len();
        let mut entries = Vec::new();
        let mut metadata = (0..count as u32)
            .map(|i| {
                ((if rows[i as usize].1 { 0x81u32 } else { 0x80 }) << 24) | (count as u32 + 4 * i)
            })
            .collect::<Vec<_>>();
        for (i, (hash, directory, id, stored, size, offset)) in rows.iter().enumerate() {
            entries.extend(hash.to_le_bytes());
            entries.extend((i as u32).to_le_bytes());
            entries.extend(1u16.to_le_bytes());
            entries.extend([u8::from(*directory), 0]);
            metadata.extend([
                (*id << 28) | *stored as u32,
                *size as u32,
                0,
                (*offset / 16) as u32,
            ]);
        }
        let metadata = metadata
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .collect::<Vec<_>>();
        // Tables whose compressed size equals their size are stored raw, as the format says.
        let table = |raw: &[u8]| {
            let packed = deflate(raw);
            if packed.len() == raw.len() {
                raw.to_vec()
            } else {
                packed
            }
        };
        let (entries_z, metadata_z) = (table(&entries), table(&metadata));
        let mut out = Vec::new();
        out.extend(b"SCS#");
        out.extend(2u16.to_le_bytes());
        out.extend(self.salt.to_le_bytes());
        out.extend(b"CITY");
        let entries_at = data.len() as u64;
        let metadata_at = entries_at + entries_z.len() as u64;
        out.extend((count as u32).to_le_bytes());
        out.extend((entries_z.len() as u32).to_le_bytes());
        out.extend(((metadata.len() / 4) as u32).to_le_bytes());
        out.extend((metadata_z.len() as u32).to_le_bytes());
        out.extend(entries_at.to_le_bytes());
        out.extend(metadata_at.to_le_bytes());
        out.extend(0u64.to_le_bytes());
        out.push(0);
        assert_eq!(out.len(), 53);
        data[..53].copy_from_slice(&out);
        data.extend(entries_z);
        data.extend(metadata_z);
        data
    }
}

/// The 64-bit token of a lowercase name (`default`, `visible`).
pub fn token_of(name: &str) -> u64 {
    name.bytes().enumerate().fold(0, |sum, (i, c)| {
        let digit = match c {
            b'0'..=b'9' => c - b'0' + 1,
            b'a'..=b'z' => c - b'a' + 11,
            _ => 37,
        };
        sum + u64::from(digit) * 38u64.pow(i as u32)
    })
}

/// A piece of a synthetic `.pmg`.
pub struct PieceSpec {
    pub material: u32,
    /// Texture-coordinate mask: nibble `n` is the set that `_TEXCOORDn` aliases.
    pub mask: u32,
    pub uv_sets: Vec<Vec<[f32; 2]>>,
    pub triangles: Vec<[u16; 3]>,
}

/// A version 0x15 `.pmg`: parts as `(name, first piece, piece count)`, no skeleton or locators.
pub fn pmg(parts: &[(&str, i32, i32)], pieces: &[PieceSpec]) -> Vec<u8> {
    let parts_at = 112;
    let pieces_at = parts_at + 24 * parts.len();
    let mut data = vec![0u8; pieces_at + 100 * pieces.len()];
    data[0] = 0x15;
    data[1..4].copy_from_slice(b"gmP");
    let put = |data: &mut Vec<u8>, at: usize, value: i32| {
        data[at..at + 4].copy_from_slice(&value.to_le_bytes())
    };
    put(&mut data, 4, pieces.len() as i32);
    put(&mut data, 8, parts.len() as i32);
    put(&mut data, 76, parts_at as i32);
    put(&mut data, 84, pieces_at as i32);
    for (i, (name, first, count)) in parts.iter().enumerate() {
        let at = parts_at + 24 * i;
        data[at..at + 8].copy_from_slice(&token_of(name).to_le_bytes());
        put(&mut data, at + 8, *count);
        put(&mut data, at + 12, *first);
    }
    for (i, piece) in pieces.iter().enumerate() {
        let at = pieces_at + 100 * i;
        let width = piece.uv_sets.len();
        let vertices = piece.uv_sets.first().map_or(0, Vec::len);
        // Absent streams are -1; the position stream makes the stride 12 + 8 * width.
        for field in [64, 68, 72, 76, 80, 84, 88, 92, 96] {
            put(&mut data, at + field, -1);
        }
        put(&mut data, at, piece.triangles.len() as i32 * 3);
        put(&mut data, at + 4, vertices as i32);
        data[at + 8..at + 12].copy_from_slice(&piece.mask.to_le_bytes());
        put(&mut data, at + 12, width as i32);
        put(&mut data, at + 16, piece.material as i32);
        let first = data.len();
        put(&mut data, at + 64, first as i32);
        put(&mut data, at + 72, first as i32 + 12);
        for v in 0..vertices {
            data.extend([0u8; 12]);
            for set in &piece.uv_sets {
                data.extend(set[v][0].to_le_bytes());
                data.extend(set[v][1].to_le_bytes());
            }
        }
        let indices = data.len();
        put(&mut data, at + 96, indices as i32);
        for triangle in &piece.triangles {
            triangle.iter().for_each(|i| data.extend(i.to_le_bytes()));
        }
    }
    data
}

/// A version 4 `.pmd`: looks as `(name, material files)`, variants as `(name, visible per part)`.
pub fn pmd(looks: &[(&str, &[&str])], variants: &[(&str, &[i32])], parts: usize) -> Vec<u8> {
    let materials = looks.first().map_or(0, |l| l.1.len());
    let mut data = vec![0u8; 64];
    let place = |data: &mut Vec<u8>, bytes: &[u8]| {
        let at = data.len();
        data.extend(bytes);
        at as u32
    };
    let words = |values: Vec<u32>| {
        values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>()
    };
    let look_at = place(
        &mut data,
        &looks
            .iter()
            .flat_map(|l| token_of(l.0).to_le_bytes())
            .collect::<Vec<_>>(),
    );
    let variant_at = place(
        &mut data,
        &variants
            .iter()
            .flat_map(|v| token_of(v.0).to_le_bytes())
            .collect::<Vec<_>>(),
    );
    let links_at = place(
        &mut data,
        &words((0..parts as u32).flat_map(|p| [p, p + 1]).collect()),
    );
    let mut defs = Vec::new();
    for part in 0..parts as u32 {
        defs.extend(token_of("visible").to_le_bytes());
        defs.extend(0u32.to_le_bytes());
        defs.extend((part * 4).to_le_bytes());
    }
    let defs_at = place(&mut data, &defs);
    let values = variants
        .iter()
        .flat_map(|v| words(v.1.iter().map(|x| *x as u32).collect()))
        .collect::<Vec<_>>();
    let values_at = place(&mut data, &values);
    let table_at = place(&mut data, &vec![0u8; 4 * looks.len() * materials]);
    for (i, file) in looks.iter().flat_map(|l| l.1.iter()).enumerate() {
        let mut text = file.as_bytes().to_vec();
        text.push(0);
        let at = place(&mut data, &text);
        data[table_at as usize + 4 * i..][..4].copy_from_slice(&at.to_le_bytes());
    }
    let header = [
        4,
        materials as u32,
        looks.len() as u32,
        0,
        variants.len() as u32,
        parts as u32,
        parts as u32,
        parts as u32 * 4,
        0,
        look_at,
        variant_at,
        links_at,
        values_at,
        defs_at,
        table_at,
        0,
    ];
    data[..64].copy_from_slice(&words(header.to_vec()));
    data
}

/// A `.mat` file of the current format.
pub fn mat(effect: &str) -> Vec<u8> {
    format!("effect : \"{effect}\" {{\n\tattribute : 1\n}}\n").into_bytes()
}
