use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde_json::{Value, json};

/// What the inventory found, read by every worker of a batch.
#[derive(Default)]
pub struct Shared {
    pub files: BTreeMap<String, Vec<usize>>,
    pub listed: BTreeMap<(usize, String), Option<Vec<String>>>,
}

pub struct Sources {
    pub converter: PathBuf,
    pub archives: Vec<PathBuf>,
    pub shared: Arc<Shared>,
    /// Archive indices of each file found here, beyond `shared`.
    pub files: BTreeMap<String, Vec<usize>>,
    pub work: PathBuf,
    /// The batch work folder, where the inventory extracted definitions before workers
    /// started; workers only read it.
    pub inventory_work: PathBuf,
    pub log_index: usize,
    /// `[D]`/`[F]` entries of each (archive, folder) seen here, beyond `shared`; `None`
    /// when known absent.
    pub listed: BTreeMap<(usize, String), Option<Vec<String>>>,
    /// Archives listed recursively: every file they hold is already in `files`.
    pub whole: BTreeSet<usize>,
    /// Folders whose files were looked up in the other archives.
    pub scanned: BTreeSet<String>,
    /// Converted `.pim`/`.pit` models; shared by batches once `share_models` succeeds.
    pub models: PathBuf,
    pub converted: usize,
    pub reused: usize,
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
    pub fn new(installation: &Path, converter: &Path, work: &Path) -> Result<Self, String> {
        let installation = installation
            .canonicalize()
            .map_err(|e| format!("installation: {e}"))?;
        let converter = converter
            .canonicalize()
            .map_err(|e| format!("converter: {e}"))?;
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
        std::fs::create_dir_all(work).map_err(|e| e.to_string())?;
        Ok(Self {
            converter,
            archives,
            shared: Arc::default(),
            files: BTreeMap::new(),
            work: work.to_owned(),
            inventory_work: work.to_owned(),
            log_index: 0,
            listed: BTreeMap::new(),
            whole: BTreeSet::new(),
            scanned: BTreeSet::new(),
            models: work.join("models"),
            converted: 0,
            reused: 0,
        })
    }

    /// Uses `cache` for converted models, clearing it when a mounted archive or the
    /// converter differs (name or path, size, modification time) from what filled it.
    pub fn share_models(&mut self, cache: &Path) -> Result<(), String> {
        let identity = |path: &Path, name: String| {
            let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos().to_string());
            Ok::<_, String>(json!([name, meta.len(), modified]))
        };
        let mut archives = Vec::new();
        for path in &self.archives {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            archives.push(identity(path, name)?);
        }
        let converter = identity(&self.converter, self.converter.display().to_string())?;
        let key = serde_json::to_vec_pretty(&json!({"archives": archives, "converter": converter}))
            .map_err(|e| e.to_string())?;
        let models = cache.join("models");
        let key_path = cache.join("key.json");
        if std::fs::read(&key_path).ok().as_deref() != Some(key.as_slice()) {
            if models.exists() {
                std::fs::remove_dir_all(&models)
                    .map_err(|e| format!("{}: {e}", models.display()))?;
            }
            std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
            std::fs::write(&key_path, &key).map_err(|e| format!("{}: {e}", key_path.display()))?;
        }
        self.models = models;
        Ok(())
    }

    pub fn command(
        &mut self,
        mounts: &[usize],
        export: &Path,
        args: &[&str],
    ) -> Result<String, String> {
        let mut command = Command::new(&self.converter);
        for index in mounts {
            command.arg("-b").arg(&self.archives[*index]);
        }
        command.arg("-e").arg(export).args(args);
        let result = command.output().map_err(|e| format!("converter: {e}"))?;
        let stdout =
            String::from_utf8(result.stdout).map_err(|_| "converter output is not UTF-8")?;
        let stderr = String::from_utf8_lossy(&result.stderr);
        let diagnostic = format!("{stdout}\n{stderr}");
        let log = self
            .work
            .join(format!("converter-{:06}.log", self.log_index));
        self.log_index += 1;
        std::fs::write(&log, &diagnostic).map_err(|e| format!("{}: {e}", log.display()))?;
        let absent_directory = args
            .first()
            .is_some_and(|a| a.starts_with("--list-directory"))
            && diagnostic.contains("<error> [system] : readDir returned null!");
        let checked = if absent_directory {
            diagnostic.replace("<error> [system] : readDir returned null!", "")
        } else {
            diagnostic.clone()
        };
        if (!result.status.success() && !absent_directory)
            || ["Unable to", "[error]", "[err]", "[warn]", "<error>"]
                .iter()
                .any(|marker| checked.contains(marker))
        {
            return Err(format!(
                "converter failed for {}: see {}",
                args.join(" "),
                log.display()
            ));
        }
        Ok(stdout)
    }

    /// Extracts the vehicle definitions under each of `folders` (`/def/vehicle/truck`…)
    /// from every archive and records which archives provide each file.
    pub fn inventory(&mut self, folders: &[&str]) -> Result<(), String> {
        for index in 0..self.archives.len() {
            self.list_whole(index)?;
            for folder in folders {
                self.inventory_folder(index, folder)?;
            }
        }
        Ok(())
    }

    /// Lists a whole archive once, filling `files` and the folder listings. An archive
    /// that cannot list its root (`locale.scs`) keeps per-folder listings.
    fn list_whole(&mut self, index: usize) -> Result<(), String> {
        let export = self.work.join(format!("archive-{index}"));
        std::fs::create_dir_all(&export).map_err(|e| e.to_string())?;
        let listing = self.command(&[index], &export, &["--list-directory-recursive", "/"])?;
        let entries = parse_listing(&listing)?;
        if entries.is_empty() {
            return Ok(());
        }
        self.listed
            .insert((index, "/".to_owned()), Some(Vec::new()));
        for entry in entries {
            let path = &entry[4..];
            if entry.starts_with("[D] ") {
                self.listed
                    .insert((index, path.to_owned()), Some(Vec::new()));
            } else {
                self.record(path, index);
            }
            let parent = path.rsplit_once('/').map_or("/", |(p, _)| p);
            let parent = if parent.is_empty() { "/" } else { parent };
            if let Some(Some(siblings)) = self.listed.get_mut(&(index, parent.to_owned())) {
                siblings.push(entry.clone());
            }
        }
        self.whole.insert(index);
        Ok(())
    }

    /// Moves what was found so far into `shared`, for `worker` views.
    pub fn share(&mut self) {
        self.shared = Arc::new(Shared {
            files: std::mem::take(&mut self.files),
            listed: std::mem::take(&mut self.listed),
        });
    }

    /// A view for worker `n`: shared findings, its own work folder and converter logs.
    pub fn worker(&self, n: usize) -> Sources {
        Sources {
            converter: self.converter.clone(),
            archives: self.archives.clone(),
            shared: Arc::clone(&self.shared),
            files: self.files.clone(),
            work: self.work.join(format!("worker-{n}")),
            inventory_work: self.inventory_work.clone(),
            log_index: 0,
            listed: self.listed.clone(),
            whole: self.whole.clone(),
            scanned: self.scanned.clone(),
            models: self.models.clone(),
            converted: 0,
            reused: 0,
        }
    }

    /// Archive indices holding `path`, in mount order.
    fn sources_of(&self, path: &str) -> Vec<usize> {
        let mut indices = [&self.shared.files, &self.files]
            .iter()
            .filter_map(|files| files.get(path))
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    /// Known game files whose path starts with `prefix`.
    pub fn paths_under(&self, prefix: &str) -> Vec<String> {
        let paths = [&self.shared.files, &self.files].map(|files| {
            files
                .range(prefix.to_owned()..)
                .map(|(path, _)| path)
                .take_while(|path| path.starts_with(prefix))
        });
        let [shared, own] = paths;
        shared
            .chain(own)
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn memo(&self, key: &(usize, String)) -> Option<&Option<Vec<String>>> {
        self.listed.get(key).or_else(|| self.shared.listed.get(key))
    }

    fn record(&mut self, path: &str, index: usize) {
        if self
            .shared
            .files
            .get(path)
            .is_some_and(|i| i.contains(&index))
        {
            return;
        }
        let indices = self.files.entry(path.to_owned()).or_default();
        if !indices.contains(&index) {
            indices.push(index);
            indices.sort_unstable();
        }
    }

    /// Lists `folder` in one archive at most once per batch. A folder missing from its
    /// parent's listing is absent without a call; an empty listing proves nothing, since
    /// some archives (`locale.scs`) cannot list their root, so children are then listed.
    fn listing(&mut self, index: usize, folder: &str) -> Result<Vec<String>, String> {
        let key = (index, folder.to_owned());
        if let Some(entries) = self.memo(&key) {
            return Ok(entries.clone().unwrap_or_default());
        }
        let present = match folder.rsplit_once('/') {
            Some((parent, _)) if folder != "/" => {
                let parent = if parent.is_empty() { "/" } else { parent };
                self.listing(index, parent)?;
                match self
                    .memo(&(index, parent.to_owned()))
                    .ok_or("unlisted parent")?
                {
                    Some(entries) => {
                        entries.is_empty() || entries.contains(&format!("[D] {folder}"))
                    }
                    None => false,
                }
            }
            _ => true,
        };
        let mut entries = None;
        if present {
            let export = self.work.join(format!("archive-{index}"));
            std::fs::create_dir_all(&export).map_err(|e| e.to_string())?;
            // Listing absent directories is allowed; actual extraction errors are not.
            let listing = self.command(&[index], &export, &["--list-directory", folder])?;
            entries = Some(parse_listing(&listing)?);
        }
        self.listed.insert(key, entries.clone());
        Ok(entries.unwrap_or_default())
    }

    fn inventory_folder(&mut self, index: usize, folder: &str) -> Result<(), String> {
        if self.listing(index, folder)?.is_empty() {
            return Ok(());
        }
        let export = self.work.join(format!("archive-{index}"));
        self.command(&[index], &export, &["--extract-directory", folder])?;
        let root = export.join(folder.trim_start_matches('/'));
        if !root.is_dir() {
            return Err(format!(
                "converter did not extract listed {folder} definitions"
            ));
        }
        for path in files(&root)? {
            let relative = path.strip_prefix(&export).map_err(|e| e.to_string())?;
            let virtual_name = format!("/{}", relative.to_string_lossy().replace('\\', "/"));
            self.record(&virtual_name, index);
        }
        Ok(())
    }

    pub fn read(&mut self, path: &str) -> Result<String, String> {
        let local = self.locate(path)?;
        std::fs::read_to_string(&local).map_err(|e| format!("{path}: {e}"))
    }

    pub fn locate(&mut self, path: &str) -> Result<PathBuf, String> {
        let path = virtual_path("/", path)?;
        let parent = path.rsplit_once('/').unwrap().0;
        let parent = if parent.is_empty() { "/" } else { parent };
        // Wholly listed archives are already in `files`; the others may override them.
        if self.scanned.insert(parent.to_owned()) {
            let partial = (0..self.archives.len()).filter(|i| !self.whole.contains(i));
            for index in partial.collect::<Vec<_>>() {
                let listing = self.listing(index, parent)?;
                for entry in listing.iter().filter_map(|line| line.strip_prefix("[F] ")) {
                    self.record(entry, index);
                }
            }
        }
        let Some(&index) = self.sources_of(&path).last() else {
            return Err(format!("missing game file {path}"));
        };
        let inventoried = self
            .inventory_work
            .join(format!("archive-{index}"))
            .join(path.trim_start_matches('/'));
        if inventoried.is_file() {
            return Ok(inventoried);
        }
        let local = self
            .work
            .join(format!("archive-{index}"))
            .join(path.trim_start_matches('/'));
        if !local.is_file() {
            let export = self.work.join(format!("archive-{index}"));
            self.command(&[index], &export, &["--extract-file", &path])?;
            if !local.is_file() {
                return Err(format!("converter did not extract {path}"));
            }
        }
        Ok(local)
    }

    pub fn resolved(
        &mut self,
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

/// The `[D] /path` and `[F] /path` entries of a converter listing, paths normalized.
fn parse_listing(listing: &str) -> Result<Vec<String>, String> {
    listing
        .lines()
        .filter_map(|line| {
            let (kind, path) = line.split_at_checked(4)?;
            matches!(kind, "[D] " | "[F] ")
                .then(|| virtual_path("/", path.trim()).map(|path| format!("{kind}{path}")))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn overlays_inclusions_and_missing_or_cyclic_references() {
        let temp = tempfile::tempdir().unwrap();
        for index in 0..2 {
            let dir = temp.path().join(format!("archive-{index}/def"));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("main.sii"),
                format!("@include \"common.sui\"\nname: source{index}\n"),
            )
            .unwrap();
            std::fs::write(dir.join("common.sui"), "model: \"/shared/model.pmd\"\n").unwrap();
        }
        let mut sources = Sources {
            converter: "unused".into(),
            archives: vec!["def.scs".into(), "dlc_tuning.scs".into()],
            shared: Arc::default(),
            files: BTreeMap::from([
                ("/def/main.sii".into(), vec![0, 1]),
                ("/def/common.sui".into(), vec![0, 1]),
            ]),
            work: temp.path().to_owned(),
            inventory_work: temp.path().to_owned(),
            log_index: 0,
            listed: BTreeMap::new(),
            // Both archives are fully known from `files`: no converter call.
            whole: BTreeSet::from([0, 1]),
            scanned: BTreeSet::new(),
            models: temp.path().join("models"),
            converted: 0,
            reused: 0,
        };
        let mut evidence = BTreeSet::new();
        let text = sources
            .resolved("/def/main.sii", &mut BTreeSet::new(), &mut evidence)
            .unwrap();
        assert!(text.contains("source1"));
        assert!(text.contains("/shared/model.pmd"));
        assert_eq!(sources.provenance(&evidence)[0]["source"], "dlc_tuning.scs");
        // Worker views read the shared findings and never share a work folder.
        sources.share();
        let (first, second) = (sources.worker(0), sources.worker(1));
        assert_eq!(first.provenance(&evidence)[0]["source"], "dlc_tuning.scs");
        assert_eq!(
            first.provenance(&evidence)[0]["mounted_sources"],
            json!(["def.scs", "dlc_tuning.scs"])
        );
        assert_ne!(first.work, second.work);
        assert!(first.work.starts_with(&sources.work));
        std::fs::write(
            temp.path().join("archive-1/def/common.sui"),
            "@include \"main.sii\"\n",
        )
        .unwrap();
        assert!(
            sources
                .resolved("/def/main.sii", &mut BTreeSet::new(), &mut BTreeSet::new())
                .unwrap_err()
                .contains("cyclic")
        );
        std::fs::remove_file(temp.path().join("archive-1/def/common.sui")).unwrap();
        assert!(
            sources
                .resolved("/def/main.sii", &mut BTreeSet::new(), &mut BTreeSet::new())
                .is_err()
        );
    }
}
