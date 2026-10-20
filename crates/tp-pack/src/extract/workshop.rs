//! Trucks of Steam Workshop mods: which package of a mod fits the game, whether it is a truck
//! mod, which trucks it adds, and the files to mount over the game for it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tp_scs::Layer;

use super::{syntax, units};

/// The Steam Workshop id of ETS2.
const APP_ID: &str = "227300";

/// The Workshop folder of a game folder: `<library>/steamapps/workshop/content/227300` when
/// the game is at `<library>/steamapps/common/<game>` and that folder exists.
pub fn folder_of(game_folder: &Path) -> Option<PathBuf> {
    let common = game_folder.parent()?;
    let steamapps = common.parent()?;
    if common.file_name()? != "common" || steamapps.file_name()? != "steamapps" {
        return None;
    }
    let folder = steamapps.join("workshop").join("content").join(APP_ID);
    folder.is_dir().then_some(folder)
}

/// Whether `version` (`1.61.1.9`) matches `pattern` (`1.61.*`): `*` stands for any text.
fn matches(pattern: &str, version: &str) -> bool {
    let mut parts = pattern.split('*');
    let first = parts.next().unwrap_or_default();
    let Some(mut rest) = version.strip_prefix(first) else {
        return false;
    };
    let mut parts = parts.peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            // The last part must end the version (an empty one, after a final `*`, always does).
            return rest.ends_with(part);
        }
        let Some(at) = rest.find(part) else {
            return false;
        };
        rest = &rest[at + part.len()..];
    }
    rest.is_empty()
}

/// The name of the package of a mod that fits `game_version`, from its `versions.sii`.
/// The first listed package with a matching `compatible_versions` is used; else the first
/// package without any (the mod's "latest"); an informational package never.
pub fn pick_package(versions: &str, game_version: &str) -> Result<Option<String>, String> {
    let root = syntax::parse(versions)?;
    let mut packages = Vec::new();
    for unit in units(&root) {
        if !unit.header.starts_with("package_version_info") {
            continue;
        }
        let informational = unit
            .fields
            .get("informational")
            .is_some_and(|v| v.iter().any(|v| syntax::unquote(v) == "true"));
        if informational {
            continue;
        }
        let patterns = unit
            .fields
            .get("compatible_versions[]")
            .map(|v| v.iter().map(|v| syntax::unquote(v).to_owned()).collect())
            .unwrap_or_else(Vec::new);
        packages.push((unit.field("package_name")?.to_owned(), patterns));
    }
    let fit = packages
        .iter()
        .find(|(_, patterns): &&(String, Vec<String>)| {
            patterns.iter().any(|p| matches(p, game_version))
        });
    let latest = || packages.iter().find(|(_, patterns)| patterns.is_empty());
    Ok(fit.or_else(latest).map(|(name, _)| name.clone()))
}

/// The folder, ZIP or `.scs` of that name in a mod folder.
pub fn locate(mod_folder: &Path, package: &str) -> Option<PathBuf> {
    if package.is_empty() || package.contains(['/', '\\']) || package == ".." {
        return None;
    }
    [
        mod_folder.join(package),
        mod_folder.join(format!("{package}.zip")),
        mod_folder.join(format!("{package}.scs")),
    ]
    .into_iter()
    .find(|path| path.exists())
}

/// What a mod's `manifest.sii` says; read leniently, as mod authors write it by hand.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    pub categories: Vec<String>,
    pub name: String,
    pub author: String,
    pub version: String,
    pub dlc_dependencies: Vec<String>,
}

/// The quoted values of every `key:` in `text`, comments left out. Several can share a line.
fn values(text: &str, key: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let mut quoted = false;
        let mut code = String::new();
        for c in line.chars() {
            if c == '"' {
                quoted = !quoted;
            } else if !quoted && c == '#' {
                break;
            }
            code.push(c);
        }
        let mut rest = code.as_str();
        while let Some(at) = rest.find(key) {
            let before = rest[..at].chars().next_back();
            rest = &rest[at + key.len()..];
            if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let Some(value) = rest.trim_start().strip_prefix(':') else {
                continue;
            };
            let Some(value) = value.trim_start().strip_prefix('"') else {
                continue;
            };
            if let Some(end) = value.find('"') {
                found.push(value[..end].to_owned());
                rest = &value[end..];
            }
        }
    }
    found
}

pub fn manifest(text: &str) -> Manifest {
    let first = |key| values(text, key).into_iter().next().unwrap_or_default();
    Manifest {
        categories: values(text, "category[]")
            .into_iter()
            .map(|c| c.to_lowercase())
            .collect(),
        name: first("display_name"),
        author: first("author"),
        version: first("package_version"),
        dlc_dependencies: values(text, "dlc_dependencies[]"),
    }
}

/// The DLC of a game folder, by the name of their archives (`dlc_toys`).
pub fn owned_dlc(game_folder: &Path) -> BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(game_folder) else {
        return BTreeSet::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.strip_suffix(".scs")
                .filter(|n| n.starts_with("dlc_"))
                .map(str::to_owned)
        })
        .collect()
}

/// The layers to mount over the game for a package: its game root (`def/` at its top, else
/// `base/`), then one per `dlc_*` folder whose DLC the player owns, sorted. A package with
/// neither root has no layer.
pub fn layers(package: &Path, owned: &BTreeSet<String>) -> Result<Vec<Layer>, String> {
    let folders = Layer::open(package, "")?.folders()?;
    let has = |name: &str| folders.iter().any(|f| f.eq_ignore_ascii_case(name));
    let root = if has("def") {
        ""
    } else if has("base") {
        "base"
    } else {
        // Nothing the game would read: no layer, so no truck.
        return Ok(Vec::new());
    };
    let mut layers = vec![Layer::open(package, root)?];
    let mut dlc = folders
        .iter()
        .filter(|f| owned.contains(&f.to_lowercase()))
        .collect::<Vec<_>>();
    dlc.sort();
    for folder in dlc {
        layers.push(Layer::open(package, folder)?);
    }
    Ok(layers)
}

/// A truck added by a Workshop mod.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModTruck {
    pub workshop_id: String,
    pub mod_name: String,
    pub mod_version: String,
    pub author: String,
    pub dlc_dependencies: Vec<String>,
    /// The package (ZIP, `.scs` or folder) holding the truck.
    pub package: PathBuf,
    /// The truck's game id (`actros.mp6`).
    pub id: String,
}

impl ModTruck {
    /// The id of the package made from this truck: `mod.ets2.3815870196.actros.mp6`.
    pub fn package_id(&self, game: &str) -> String {
        format!("mod.{game}.{}.{}", self.workshop_id, self.id)
    }
}

/// What a scan of the Workshop found.
#[derive(Debug, Default)]
pub struct Workshop {
    pub trucks: Vec<ModTruck>,
    /// Mods that could not be read, with the reason; the others are unaffected.
    pub unreadable: Vec<(String, String)>,
    /// Mods with no package for this game, or no `versions.sii`: nothing to report.
    pub left_out: Vec<(String, String)>,
}

fn truck_ids(layers: &[Layer]) -> Result<BTreeSet<String>, String> {
    let mut ids = BTreeSet::new();
    for layer in layers {
        for file in layer.walk("/def/vehicle/truck")? {
            let id = file
                .strip_prefix("/def/vehicle/truck/")
                .and_then(|p| p.strip_suffix("/data.sii"))
                .filter(|id| !id.contains('/') && tp_vehicles::is_valid_game_name(id));
            ids.extend(id.map(str::to_owned));
        }
    }
    Ok(ids)
}

/// Reads one mod folder: no trucks when the mod is not a truck mod or adds none.
fn read_mod(
    folder: &Path,
    game_version: &str,
    owned: &BTreeSet<String>,
    base_trucks: &BTreeSet<String>,
) -> Result<Vec<ModTruck>, String> {
    let versions = std::fs::read_to_string(folder.join("versions.sii"))
        .map_err(|e| format!("versions.sii: {e}"))?;
    let name = pick_package(&versions, game_version)?.ok_or("no package for this game version")?;
    let package = locate(folder, &name).ok_or_else(|| format!("package {name} is missing"))?;
    let text = Layer::open(&package, "")?.read("/manifest.sii")?;
    let manifest = manifest(&String::from_utf8_lossy(&text.unwrap_or_default()));
    if !manifest.categories.iter().any(|c| c == "truck") {
        return Ok(Vec::new());
    }
    let layers = layers(&package, owned)?;
    let workshop_id = folder.file_name().unwrap_or_default().to_string_lossy();
    Ok(truck_ids(&layers)?
        .into_iter()
        .filter(|id| !base_trucks.contains(id))
        .map(|id| ModTruck {
            workshop_id: workshop_id.clone().into_owned(),
            mod_name: manifest.name.clone(),
            mod_version: manifest.version.clone(),
            author: manifest.author.clone(),
            dlc_dependencies: manifest.dlc_dependencies.clone(),
            package: package.clone(),
            id,
        })
        .collect())
}

/// Scans the Workshop folder for the trucks mods add to the game; reads each mod's
/// `versions.sii`, manifest and file table, never a model.
pub fn scan(
    workshop: &Path,
    game_folder: &Path,
    game_version: &str,
    base_trucks: &BTreeSet<String>,
) -> Result<Workshop, String> {
    let owned = owned_dlc(game_folder);
    let mut folders = std::fs::read_dir(workshop)
        .map_err(|e| format!("{}: {e}", workshop.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect::<Vec<_>>();
    folders.sort();
    let mut result = Workshop::default();
    for folder in folders {
        let id = folder
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        match read_mod(&folder, game_version, &owned, base_trucks) {
            Ok(trucks) => result.trucks.extend(trucks),
            Err(reason)
                if reason.starts_with("versions.sii")
                    || reason.starts_with("no package")
                    || reason.ends_with("is missing") =>
            {
                result.left_out.push((id, reason));
            }
            Err(reason) => result.unreadable.push((id, reason)),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tp_scs::synthetic::{ArchiveBuilder, zip_package};

    const MANIFEST: &str = "SiiNunit\n{\nmod_package : .package_name\n{\n    package_version: \"2.2.00\"\n    author: \"Erik\"\n    category[]: \"truck\"\n    dlc_dependencies[]: \"dlc_daf_2021\"\n}\n}\n";

    fn versions(packages: &[(&str, &[&str], bool)]) -> String {
        let mut text = String::from("SiiNunit\n{\n");
        for (name, compatible, informational) in packages {
            text += &format!("package_version_info : .{name}\n{{\n    package_name: \"{name}\"\n");
            for c in *compatible {
                text += &format!("    compatible_versions[]: \"{c}\"\n");
            }
            if *informational {
                text += "    informational: true\n";
            }
            text += "}\n";
        }
        text + "}\n"
    }

    #[test]
    fn workshop_folder_is_next_to_the_games_library() {
        let dir = tempfile::tempdir().unwrap();
        let game = dir
            .path()
            .join("SteamLibrary/steamapps/common/Euro Truck Simulator 2");
        std::fs::create_dir_all(&game).unwrap();
        assert_eq!(folder_of(&game), None, "no subscription yet");
        let workshop = dir
            .path()
            .join("SteamLibrary/steamapps/workshop/content/227300");
        std::fs::create_dir_all(&workshop).unwrap();
        assert_eq!(folder_of(&game), Some(workshop));
        let outside = dir.path().join("games/ets2");
        std::fs::create_dir_all(&outside).unwrap();
        assert_eq!(folder_of(&outside), None);
        assert_eq!(folder_of(Path::new("/")), None);
    }

    #[test]
    fn patterns_use_star_as_the_only_wildcard() {
        for (pattern, version, expected) in [
            ("1.61.*", "1.61.1.9", true),
            ("1.61.*", "1.62.0.1", false),
            ("1.61.*", "1.61.", true),
            ("1.6*", "1.61.1.9", true),
            ("*", "1.61.1.9", true),
            ("1.61.1.9", "1.61.1.9", true),
            ("1.61.1", "1.61.1.9", false),
            ("1.*.9", "1.61.1.9", true),
            ("1.*.8", "1.61.1.9", false),
            ("1.61.1.9.*", "1.61.1.9", false),
        ] {
            assert_eq!(matches(pattern, version), expected, "{pattern} {version}");
        }
    }

    #[test]
    fn the_package_is_chosen_by_the_games_version() {
        let pick = |packages: &[(&str, &[&str], bool)], version| {
            pick_package(&versions(packages), version).unwrap()
        };
        let both = [
            ("content_160", &["1.60.*"][..], false),
            ("content_161", &["1.61.*"][..], false),
        ];
        assert_eq!(pick(&both, "1.61.1.9").as_deref(), Some("content_161"));
        assert_eq!(pick(&both, "1.59.0.0"), None);
        // The mod's latest package has no versions; a real match wins over it.
        let latest = [
            ("latest", &[][..], false),
            ("156_content", &["1.56.*"][..], false),
        ];
        assert_eq!(pick(&latest, "1.61.1.9").as_deref(), Some("latest"));
        assert_eq!(pick(&latest, "1.56.2.0").as_deref(), Some("156_content"));
        // Several fit: the first listed.
        let several = [("a", &["1.61.*"][..], false), ("b", &["1.61.*"][..], false)];
        assert_eq!(pick(&several, "1.61.1.9").as_deref(), Some("a"));
        // An informational package is never used, with or without versions.
        let info = [("universal", &[][..], true), ("v", &["1.61.*"][..], true)];
        assert_eq!(pick(&info, "1.61.1.9"), None);
        assert!(pick_package("SiiNunit {", "1.61").is_err());
    }

    #[test]
    fn a_package_is_a_folder_a_zip_or_an_scs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("a")).unwrap();
        std::fs::write(dir.path().join("b.zip"), b"x").unwrap();
        std::fs::write(dir.path().join("c.scs"), b"x").unwrap();
        for (name, expected) in [("a", "a"), ("b", "b.zip"), ("c", "c.scs")] {
            assert_eq!(locate(dir.path(), name), Some(dir.path().join(expected)));
        }
        assert_eq!(locate(dir.path(), "missing"), None);
        assert_eq!(locate(dir.path(), "../a"), None);
    }

    #[test]
    fn manifests_are_read_the_way_authors_write_them() {
        let read = manifest(MANIFEST);
        assert_eq!(read.categories, ["truck"]);
        assert_eq!(read.author, "Erik");
        assert_eq!(read.version, "2.2.00");
        assert_eq!(read.dlc_dependencies, ["dlc_daf_2021"]);
        // Several categories, one after the other or on one line; case is not significant.
        for text in [
            "category[]: \"ui\"\ncategory[]: \"Truck\"",
            "category[]:\"tuning_parts\"category[]:\"truck\"",
        ] {
            assert!(
                manifest(text).categories.contains(&"truck".to_owned()),
                "{text}"
            );
        }
        // A commented line says nothing, and a name that only ends like a key is not it.
        let commented =
            manifest("# display_name: \"Fast Money\"\nmy_author: \"x\"\ncategory[]: \"other\"");
        assert!(commented.name.is_empty() && commented.author.is_empty());
        assert_eq!(commented.categories, ["other"]);
        assert_eq!(manifest(""), Manifest::default());
    }

    /// A game folder with one DLC.
    fn game(dir: &Path) -> PathBuf {
        let game = dir.join("lib/steamapps/common/Euro Truck Simulator 2");
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("dlc_toys.scs"), ArchiveBuilder::default().build()).unwrap();
        std::fs::write(game.join("base.scs"), ArchiveBuilder::default().build()).unwrap();
        game
    }

    fn mod_folder(dir: &Path, id: &str, versions: &str) -> PathBuf {
        let folder = dir.join("lib/steamapps/workshop/content/227300").join(id);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("versions.sii"), versions).unwrap();
        folder
    }

    fn truck_zip(manifest: &str, trucks: &[&str]) -> Vec<u8> {
        let mut files = vec![("manifest.sii".to_owned(), manifest.as_bytes().to_vec())];
        for truck in trucks {
            files.push((
                format!("base/def/vehicle/truck/{truck}/data.sii"),
                b"d".to_vec(),
            ));
            files.push((
                format!("base/def/vehicle/truck/{truck}/cabin/cab.sii"),
                b"c".to_vec(),
            ));
        }
        let files = files
            .iter()
            .map(|(n, b)| (n.as_str(), b.as_slice()))
            .collect::<Vec<_>>();
        zip_package(&files, true)
    }

    #[test]
    fn roots_and_dlc_folders_become_layers() {
        let dir = tempfile::tempdir().unwrap();
        let owned = BTreeSet::from(["dlc_toys".to_owned()]);
        let package = dir.path().join("mod.zip");
        std::fs::write(
            &package,
            zip_package(
                &[
                    ("base/def/a.sii", b"a"),
                    ("dlc_toys/def/b.sii", b"b"),
                    ("dlc_krone/def/c.sii", b"c"),
                ],
                false,
            ),
        )
        .unwrap();
        let layers = layers(&package, &owned).unwrap();
        let names = layers.iter().map(Layer::name).collect::<Vec<_>>();
        assert_eq!(names, ["mod.zip/base", "mod.zip/dlc_toys"]);
        assert!(layers[0].contains("/def/a.sii") && layers[1].contains("/def/b.sii"));
        // `def/` at the top: no root to remove.
        let folder = dir.path().join("content_161");
        std::fs::create_dir_all(folder.join("def")).unwrap();
        assert_eq!(layers_names(&folder, &owned), ["content_161"]);
        // Neither: nothing to mount.
        let other = dir.path().join("other.zip");
        std::fs::write(&other, zip_package(&[("readme.txt", b"x")], false)).unwrap();
        assert!(super::layers(&other, &owned).unwrap().is_empty());
    }

    fn layers_names(package: &Path, owned: &BTreeSet<String>) -> Vec<String> {
        layers(package, owned)
            .unwrap()
            .iter()
            .map(Layer::name)
            .collect()
    }

    #[test]
    fn scan_lists_new_trucks_of_truck_mods_only() {
        let dir = tempfile::tempdir().unwrap();
        let game = game(dir.path());
        // A truck mod in a ZIP: one new truck and one the game defines.
        let mp6 = mod_folder(
            dir.path(),
            "111",
            &versions(&[
                ("content_161", &["1.61.*"], false),
                ("informational", &[], true),
            ]),
        );
        std::fs::write(
            mp6.join("content_161.zip"),
            truck_zip(MANIFEST, &["actros.mp6", "daf.2021"]),
        )
        .unwrap();
        // The same shape in a folder, of another category: not examined.
        let other = mod_folder(dir.path(), "222", &versions(&[("latest", &[], false)]));
        std::fs::create_dir_all(other.join("latest/def/vehicle/truck/ui.truck")).unwrap();
        std::fs::write(other.join("latest/manifest.sii"), "category[]: \"ui\"").unwrap();
        std::fs::write(
            other.join("latest/def/vehicle/truck/ui.truck/data.sii"),
            "d",
        )
        .unwrap();
        // A truck mod with no truck definition, one with only an informational package, and a damaged one.
        let empty = mod_folder(dir.path(), "333", &versions(&[("v", &["1.61.*"], false)]));
        std::fs::write(empty.join("v.zip"), truck_zip(MANIFEST, &[])).unwrap();
        mod_folder(dir.path(), "444", &versions(&[("info", &[], true)]));
        let damaged = mod_folder(dir.path(), "555", &versions(&[("v", &["1.61.*"], false)]));
        std::fs::write(damaged.join("v.zip"), &truck_zip(MANIFEST, &["x.y"])[..60]).unwrap();
        // A truck mod in a folder with a DLC root the player owns.
        let folder = mod_folder(dir.path(), "666", &versions(&[("v", &["1.61.*"], false)]));
        std::fs::create_dir_all(folder.join("v/def/vehicle/truck/at.top")).unwrap();
        std::fs::write(folder.join("v/manifest.sii"), MANIFEST).unwrap();
        std::fs::write(folder.join("v/def/vehicle/truck/at.top/data.sii"), "d").unwrap();
        std::fs::create_dir_all(folder.join("v/dlc_toys/def/vehicle/truck/toy.truck")).unwrap();
        std::fs::write(
            folder.join("v/dlc_toys/def/vehicle/truck/toy.truck/data.sii"),
            "d",
        )
        .unwrap();
        std::fs::create_dir_all(folder.join("v/dlc_krone/def/vehicle/truck/krone.truck")).unwrap();
        std::fs::write(
            folder.join("v/dlc_krone/def/vehicle/truck/krone.truck/data.sii"),
            "d",
        )
        .unwrap();

        let workshop = folder_of(&game).unwrap();
        let base = BTreeSet::from(["daf.2021".to_owned()]);
        let found = scan(&workshop, &game, "1.61.1.9", &base).unwrap();
        let listed = found
            .trucks
            .iter()
            .map(|t| (t.workshop_id.as_str(), t.id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            listed,
            [
                ("111", "actros.mp6"),
                ("666", "at.top"),
                ("666", "toy.truck")
            ]
        );
        let first = &found.trucks[0];
        assert_eq!(
            (
                first.author.as_str(),
                first.mod_version.as_str(),
                first.dlc_dependencies.as_slice()
            ),
            ("Erik", "2.2.00", ["dlc_daf_2021".to_owned()].as_slice())
        );
        assert!(first.package.ends_with("111/content_161.zip"));
        // The damaged mod is named with a reason; the others are unaffected.
        assert_eq!(found.unreadable.len(), 1);
        assert_eq!(found.unreadable[0].0, "555");
        // No package for this game: left out, not an error.
        assert!(found.left_out.iter().any(|(id, _)| id == "444"));
        assert!(!found.unreadable.iter().any(|(id, _)| id == "444"));
    }

    /// Run by hand against a real game: `TP_GAME=<game folder> cargo test -p tp-pack
    /// real_workshop -- --ignored --nocapture`.
    #[test]
    #[ignore = "needs a real game folder in TP_GAME"]
    fn real_workshop() {
        let game = PathBuf::from(std::env::var("TP_GAME").expect("TP_GAME"));
        let scan_game = super::super::scan(&game).unwrap();
        let base = scan_game
            .vehicles
            .iter()
            .filter(|v| v.kind == tp_vehicles::Kind::Truck)
            .map(|v| v.id.clone())
            .collect();
        let workshop = folder_of(&game).expect("a Workshop folder");
        let started = std::time::Instant::now();
        let found = super::scan(&workshop, &game, &scan_game.info.version, &base).unwrap();
        eprintln!("{:?} in {:?}", found, started.elapsed());
        for truck in &found.trucks {
            let started = std::time::Instant::now();
            let named = super::super::describe_mod_truck(&scan_game, truck);
            let made = super::super::generate_mod_truck(&scan_game, truck, None);
            let peak = std::fs::read_to_string("/proc/self/status")
                .unwrap_or_default()
                .lines()
                .find(|l| l.starts_with("VmHWM"))
                .map(str::to_owned);
            match made {
                Ok(super::super::Outcome::Package(packed)) => {
                    let package = tp_vehicles::Package::read(&packed.bytes).unwrap();
                    eprintln!(
                        "{} {named:?}: {} v{} {} bytes, {} textures, in {:?}, {peak:?}",
                        truck.id,
                        package.manifest.id,
                        package.manifest.version,
                        packed.bytes.len(),
                        package.manifest.paint_job.accessories.len() + 1,
                        started.elapsed()
                    );
                }
                Ok(_) => eprintln!("{} up to date", truck.id),
                Err(e) => eprintln!("{} FAILED: {e}", truck.id),
            }
        }
    }
}
