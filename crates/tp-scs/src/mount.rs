//! Archives mounted in order, as the game does: the last one holding a path wins.

use std::path::PathBuf;

use crate::hashfs::Archive;

pub struct Mount {
    archives: Vec<Archive>,
}

impl Mount {
    pub fn open(paths: &[PathBuf]) -> Result<Self, String> {
        let archives = paths
            .iter()
            .map(|path| Archive::open(path))
            .collect::<Result<_, _>>()?;
        Ok(Self { archives })
    }

    /// Indices of the archives holding the file `path`, in mount order.
    pub fn sources_of(&self, path: &str) -> Vec<usize> {
        (0..self.archives.len())
            .filter(|&i| self.archives[i].contains(path))
            .collect()
    }

    /// The file's bytes from the last archive holding it, or `None` when none does.
    pub fn read(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        match self.archives.iter().rposition(|a| a.contains(path)) {
            Some(index) => self.archives[index].read(path),
            None => Ok(None),
        }
    }

    /// Every file below `folder` in archive `index`, with full game paths.
    pub fn walk(&self, index: usize, folder: &str) -> Result<Vec<String>, String> {
        self.archives[index].walk(folder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::ArchiveBuilder;

    #[test]
    fn last_mounted_archive_wins_and_a_missing_root_listing_does_not_matter() {
        let dir = tempfile::tempdir().unwrap();
        let archives = [
            (
                "def.scs",
                ArchiveBuilder::default()
                    .file("/def/a.sii", b"base")
                    .file("/def/only.sii", b"x"),
            ),
            (
                "dlc.scs",
                ArchiveBuilder::default().compressed("/def/a.sii", b"dlc"),
            ),
            (
                "locale.scs",
                ArchiveBuilder::default()
                    .without_root()
                    .file("/locale/en.sui", b"hello"),
            ),
        ];
        let paths = archives
            .iter()
            .map(|(name, builder)| {
                let path = dir.path().join(name);
                std::fs::write(&path, builder.build()).unwrap();
                path
            })
            .collect::<Vec<_>>();
        let mount = Mount::open(&paths).unwrap();
        assert_eq!(mount.read("/def/a.sii").unwrap().unwrap(), b"dlc");
        assert_eq!(mount.sources_of("/def/a.sii"), [0, 1]);
        assert_eq!(mount.read("/def/only.sii").unwrap().unwrap(), b"x");
        assert!(mount.read("/def/none.sii").unwrap().is_none());
        // The archive without a root listing is still searched by path, and walked by folder.
        assert_eq!(mount.read("/locale/en.sui").unwrap().unwrap(), b"hello");
        assert!(mount.walk(2, "/").unwrap().is_empty());
        assert_eq!(mount.walk(2, "/locale").unwrap(), ["/locale/en.sui"]);
        assert_eq!(
            mount.walk(0, "/def").unwrap(),
            ["/def/a.sii", "/def/only.sii"]
        );
    }
}
