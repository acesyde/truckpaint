use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde_json::{Value, json};
use tp_scs::Mount;

/// The mounted game archives and the vehicle definitions found in them; read-only once
/// built, so every worker of a batch shares one.
pub struct Sources {
    pub archives: Vec<PathBuf>,
    mount: Mount,
    /// Game files of the inventoried folders.
    paths: BTreeSet<String>,
}

pub fn files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err(format!("unexpected symlink {}", entry.path().display()));
        }
        if kind.is_dir() {
            result.extend(files(&entry.path())?);
        } else if kind.is_file() {
            result.push(entry.path());
        }
    }
    result.sort();
    Ok(result)
}

pub fn virtual_path(parent: &str, reference: &str) -> Result<String, String> {
    if reference.contains('\\') || reference.contains('\0') || reference.contains(':') {
        return Err(format!("unsafe game path {reference}"));
    }
    let full = if reference.starts_with('/') {
        reference.to_owned()
    } else {
        format!(
            "{}/{reference}",
            parent.rsplit_once('/').map_or("", |(p, _)| p)
        )
    };
    let mut parts = Vec::new();
    for component in Path::new(&full).components() {
        match component {
            Component::Normal(p) => parts.push(p.to_string_lossy().into_owned()),
            Component::ParentDir => {
                parts
                    .pop()
                    .ok_or_else(|| format!("path escapes game root: {reference}"))?;
            }
            Component::RootDir | Component::CurDir => {}
            Component::Prefix(_) => return Err(format!("unsafe game path {reference}")),
        }
    }
    if parts.is_empty() {
        return Err("empty game file path".into());
    }
    Ok(format!("/{}", parts.join("/")))
}

impl Sources {
    pub fn new(installation: &Path) -> Result<Self, String> {
        let installation = installation
            .canonicalize()
            .map_err(|e| format!("installation: {e}"))?;
        let mut archives = Vec::new();
        // Matches the source pools observed in the installed game's mount log.
        for name in [
            "base.scs",
            "base_map.scs",
            "base_vehicle.scs",
            "base_share.scs",
            "def.scs",
            "core.scs",
            "effect.scs",
            "locale.scs",
            "base_cfg.scs",
            "version.scs",
        ] {
            let path = installation.join(name);
            if path.is_file() {
                archives.push(path);
            } else if matches!(name, "base.scs" | "def.scs" | "version.scs") {
                return Err(format!("missing installed archive {name}"));
            }
        }
        let mut dlc = std::fs::read_dir(&installation)
            .map_err(|e| e.to_string())?
            .map(|e| e.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| n.starts_with("dlc_") && n.ends_with(".scs"))
            })
            .map(|e| e.path())
            .collect::<Vec<_>>();
        dlc.sort();
        archives.extend(dlc);
        let mount = Mount::open(&archives)?;
        Ok(Self {
            archives,
            mount,
            paths: BTreeSet::new(),
        })
    }

    /// Records the files below each of `folders` (`/def/vehicle/truck`...) of every archive.
    pub fn inventory(&mut self, folders: &[&str]) -> Result<(), String> {
        for index in 0..self.archives.len() {
            for folder in folders {
                self.paths.extend(self.mount.walk(index, folder)?);
            }
        }
        Ok(())
    }

    /// Known game files whose path starts with `prefix`.
    pub fn paths_under(&self, prefix: &str) -> Vec<String> {
        self.paths
            .range(prefix.to_owned()..)
            .take_while(|path| path.starts_with(prefix))
            .cloned()
            .collect()
    }

    /// Archive indices holding `path`, in mount order.
    fn sources_of(&self, path: &str) -> Vec<usize> {
        self.mount.sources_of(path)
    }

    /// A game file's bytes from the last archive holding it.
    pub fn bytes(&self, path: &str) -> Result<Vec<u8>, String> {
        let path = virtual_path("/", path)?;
        self.mount
            .read(&path)?
            .ok_or_else(|| format!("missing game file {path}"))
    }

    pub fn read(&self, path: &str) -> Result<String, String> {
        String::from_utf8(self.bytes(path)?).map_err(|_| format!("{path}: not UTF-8 text"))
    }

    pub fn resolved(
        &self,
        path: &str,
        active: &mut BTreeSet<String>,
        evidence: &mut BTreeSet<String>,
    ) -> Result<String, String> {
        let path = virtual_path("/", path)?;
        if active.len() >= 64 || !active.insert(path.clone()) {
            return Err(format!("cyclic or too deep include: {path}"));
        }
        evidence.insert(path.clone());
        let text = self.read(&path)?;
        let mut result = String::new();
        for line in text.lines() {
            if let Some(include) = line.trim().strip_prefix("@include") {
                let include = include.trim();
                let end = include
                    .strip_prefix('"')
                    .and_then(|v| v.find('"'))
                    .ok_or_else(|| format!("invalid include in {path}"))?;
                let tail = include[end + 2..].trim();
                if !tail.is_empty() && !tail.starts_with('#') && !tail.starts_with("//") {
                    return Err(format!("invalid include in {path}"));
                }
                let reference = virtual_path(&path, &include[1..end + 1])?;
                result.push_str(&self.resolved(&reference, active, evidence)?);
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        active.remove(&path);
        Ok(result)
    }

    pub fn provenance(&self, paths: &BTreeSet<String>) -> Value {
        json!(
            paths
                .iter()
                .map(|path| {
                    let sources = self
                        .sources_of(path)
                        .iter()
                        .map(|i| {
                            self.archives[*i]
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string()
                        })
                        .collect::<Vec<_>>();
                    json!({"path": path, "source": sources.last(), "mounted_sources": sources})
                })
                .collect::<Vec<_>>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tp_scs::synthetic::ArchiveBuilder;

    #[test]
    fn includes_normalize_inside_game_root_only() {
        assert_eq!(
            virtual_path("/def/truck/cabin/high.sii", "../common.sui").unwrap(),
            "/def/truck/common.sui"
        );
        assert_eq!(
            virtual_path("/def/a.sii", "/vehicle/shared/model.pmd").unwrap(),
            "/vehicle/shared/model.pmd"
        );
        for reference in ["../../outside", "C:/outside", "..\\outside", ""] {
            assert!(virtual_path("/a.sii", reference).is_err(), "{reference}");
        }
    }

    fn game(def: &ArchiveBuilder, dlc: &ArchiveBuilder) -> (tempfile::TempDir, Sources) {
        let temp = tempfile::tempdir().unwrap();
        let empty = ArchiveBuilder::default().file("/x", b"");
        for (name, archive) in [
            ("base.scs", &empty),
            ("def.scs", def),
            ("version.scs", &empty),
            ("dlc_tuning.scs", dlc),
        ] {
            std::fs::write(temp.path().join(name), archive.build()).unwrap();
        }
        let sources = Sources::new(temp.path()).unwrap();
        (temp, sources)
    }

    #[test]
    fn overlays_inclusions_and_missing_or_cyclic_references() {
        let def = ArchiveBuilder::default()
            .compressed("/def/main.sii", b"@include \"common.sui\"\nname: source0\n")
            .file("/def/common.sui", b"model: \"/shared/model.pmd\"\n");
        let dlc = ArchiveBuilder::default()
            .file("/def/main.sii", b"@include \"common.sui\"\nname: source1\n")
            .file("/def/common.sui", b"model: \"/shared/model.pmd\"\n");
        let (_temp, mut sources) = game(&def, &dlc);
        sources.inventory(&["/def"]).unwrap();
        assert_eq!(
            sources.paths_under("/def/"),
            ["/def/common.sui", "/def/main.sii"]
        );
        let mut evidence = BTreeSet::new();
        let text = sources
            .resolved("/def/main.sii", &mut BTreeSet::new(), &mut evidence)
            .unwrap();
        assert!(text.contains("source1"));
        assert!(text.contains("/shared/model.pmd"));
        assert_eq!(sources.provenance(&evidence)[0]["source"], "dlc_tuning.scs");
        assert_eq!(
            sources.provenance(&evidence)[0]["mounted_sources"],
            json!(["def.scs", "dlc_tuning.scs"])
        );
        assert_eq!(
            sources.read("/def/none.sii").unwrap_err(),
            "missing game file /def/none.sii"
        );
        // A cycle through the later archive is refused.
        let cyclic = ArchiveBuilder::default()
            .file("/def/main.sii", b"@include \"common.sui\"\n")
            .file("/def/common.sui", b"@include \"main.sii\"\n");
        let (_temp, sources) = game(&def, &cyclic);
        assert!(
            sources
                .resolved("/def/main.sii", &mut BTreeSet::new(), &mut BTreeSet::new())
                .unwrap_err()
                .contains("cyclic")
        );
        // A missing include is an error naming the file.
        let missing =
            ArchiveBuilder::default().file("/def/common.sui", b"@include \"nothing.sui\"\n");
        let (_temp, sources) = game(&def, &missing);
        assert!(
            sources
                .resolved("/def/main.sii", &mut BTreeSet::new(), &mut BTreeSet::new())
                .unwrap_err()
                .contains("missing game file /def/nothing.sui")
        );
    }
}
