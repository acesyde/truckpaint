//! Sources of game files mounted in order, as the game does: the last one holding a path wins.

use std::path::PathBuf;

use crate::layer::Layer;

pub struct Mount {
    layers: Vec<Layer>,
}

impl Mount {
    /// Mounts HashFS archives.
    pub fn open(paths: &[PathBuf]) -> Result<Self, String> {
        let layers = paths
            .iter()
            .map(|path| Layer::open(path, ""))
            .collect::<Result<_, _>>()?;
        Ok(Self { layers })
    }

    /// Mounts `layer` after the ones already there.
    pub fn push(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// The name of layer `index` (the file name of its source, with its root).
    pub fn name(&self, index: usize) -> String {
        self.layers[index].name()
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Indices of the layers holding the file `path`, in mount order.
    pub fn sources_of(&self, path: &str) -> Vec<usize> {
        (0..self.layers.len())
            .filter(|&i| self.layers[i].contains(path))
            .collect()
    }

    /// The file's bytes from the last layer holding it, or `None` when none does.
    pub fn read(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        match self.layers.iter().rposition(|a| a.contains(path)) {
            Some(index) => self.layers[index].read(path),
            None => Ok(None),
        }
    }

    /// Every file below `folder` in layer `index`, with full game paths.
    pub fn walk(&self, index: usize, folder: &str) -> Result<Vec<String>, String> {
        self.layers[index].walk(folder)
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

    #[test]
    fn layers_of_every_kind_mount_with_roots_and_the_last_wins() {
        let dir = tempfile::tempdir().unwrap();
        let game = dir.path().join("base.scs");
        std::fs::write(
            &game,
            ArchiveBuilder::default()
                .file("/def/a.sii", b"game")
                .file("/model/m.pmg", b"game model")
                .build(),
        )
        .unwrap();
        let zip = dir.path().join("mod.zip");
        std::fs::write(
            &zip,
            crate::synthetic::zip_package(
                &[
                    ("manifest.sii", b"manifest"),
                    ("base/def/a.sii", b"mod"),
                    ("base/def/b.sii", b"mod only"),
                    ("dlc_toys/def/a.sii", b"dlc"),
                ],
                true,
            ),
        )
        .unwrap();
        let mut mount = Mount::open(&[game]).unwrap();
        mount.push(Layer::open(&zip, "base").unwrap());
        assert_eq!(mount.read("/def/a.sii").unwrap().unwrap(), b"mod");
        assert_eq!(mount.read("/def/b.sii").unwrap().unwrap(), b"mod only");
        assert_eq!(mount.read("/model/m.pmg").unwrap().unwrap(), b"game model");
        // The root's own folder is not a game path; files outside it are not seen.
        assert!(mount.read("/manifest.sii").unwrap().is_none());
        assert!(mount.read("/base/def/a.sii").unwrap().is_none());
        assert_eq!(mount.sources_of("/def/a.sii"), [0, 1]);
        assert_eq!(mount.name(1), "mod.zip/base");
        assert_eq!(mount.walk(1, "/def").unwrap(), ["/def/a.sii", "/def/b.sii"]);
        // A DLC folder of the package is a layer of its own, mounted after the main one.
        mount.push(Layer::open(&zip, "dlc_toys").unwrap());
        assert_eq!(mount.read("/def/a.sii").unwrap().unwrap(), b"dlc");
        assert_eq!(mount.len(), 3);
    }

    #[test]
    fn a_folder_with_def_at_its_root_is_mounted_with_no_root() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("content_161");
        std::fs::create_dir_all(pkg.join("def")).unwrap();
        std::fs::write(pkg.join("def/x.sii"), b"x").unwrap();
        let layer = Layer::open(&pkg, "").unwrap();
        assert_eq!(layer.folders().unwrap(), ["def"]);
        assert_eq!(layer.read("/def/x.sii").unwrap().unwrap(), b"x");
        let other = dir.path().join("not-a-package.txt");
        std::fs::write(&other, b"text").unwrap();
        assert!(
            Layer::open(&other, "")
                .err()
                .unwrap()
                .contains("not-a-package.txt")
        );
    }
}
