//! A small made-up game folder, for tests here and in the app: no game data.

use std::collections::BTreeMap;
use std::path::Path;

use tp_scs::synthetic::{ArchiveBuilder, PieceSpec, mat, pmd, pmg};

/// One vehicle of a made-up game.
pub struct Vehicle<'a> {
    /// The game id (`example.alpha`); its name is `Example <name>`.
    pub id: &'a str,
    pub name: &'a str,
    pub trailer: bool,
    /// The archive holding it: `version` (the base game) or a DLC (`dlc_krone`).
    pub archive: &'a str,
    /// A trailer without paint masks cannot be painted.
    pub paintable: bool,
    /// Moves the UVs, as a game update that changes a template.
    pub shifted: bool,
    /// A model the reader cannot parse, so generating it fails.
    pub broken: bool,
}

impl<'a> Vehicle<'a> {
    pub fn truck(id: &'a str, name: &'a str) -> Self {
        Self {
            id,
            name,
            trailer: false,
            archive: "version",
            paintable: true,
            shifted: false,
            broken: false,
        }
    }

    pub fn trailer(id: &'a str, name: &'a str) -> Self {
        Self {
            trailer: true,
            ..Self::truck(id, name)
        }
    }

    pub fn in_archive(self, archive: &'a str) -> Self {
        Self { archive, ..self }
    }
}

fn add(
    files: &mut BTreeMap<String, Vec<(String, Vec<u8>)>>,
    archive: &str,
    path: String,
    bytes: impl Into<Vec<u8>>,
) {
    files
        .entry(archive.to_owned())
        .or_default()
        .push((path, bytes.into()));
}

fn add_vehicles(files: &mut BTreeMap<String, Vec<(String, Vec<u8>)>>, vehicles: &[Vehicle]) {
    for v in vehicles {
        let (kind, data) = if v.trailer {
            ("trailer_owned", "accessory_trailer_data")
        } else {
            ("truck", "accessory_truck_data")
        };
        let def = format!("/def/vehicle/{kind}/{}", v.id);
        let model = format!("/vehicle/shared/{}", v.id);
        let a = v.archive;
        add(
            files,
            a,
            format!("{def}/data.sii"),
            format!(
                "SiiNunit\n{{\n{data} : vehicle.{} {{\ninfo[]: \"Example\"\ninfo[]: \"{}\"\n}}\n}}\n",
                v.id, v.name
            ),
        );
        let unit = if v.trailer {
            format!("{def}/body/curtain.sii")
        } else {
            format!("{def}/cabin/high.sii")
        };
        let header = if v.trailer {
            format!("accessory_trailer_body_data : curtain.{}.body", v.id)
        } else {
            format!("accessory_cabin_data : high.{}.cabin", v.id)
        };
        add(
            files,
            a,
            unit,
            format!("SiiNunit\n{{\n{header} {{\nname: \"High\"\nmodel: \"{model}.pmd\"\n}}\n}}\n"),
        );
        if v.trailer && v.paintable {
            add(
                files,
                a,
                format!("{def}/paint_job/default.sii"),
                format!(
                    "SiiNunit\n{{\naccessory_paint_job_data : default.{}.paint_job {{\npaint_job_mask: \"/trailer/pjm_at_0x0_size_4x4.tobj\"\n}}\n}}\n",
                    v.id
                ),
            );
        }
        let second = if v.shifted { [0.5, 0.] } else { [1., 0.] };
        let piece = PieceSpec {
            material: 0,
            mask: 0,
            uv_sets: vec![vec![[0., 0.], second, [0., 1.]]],
            triangles: vec![[0, 1, 2]],
        };
        let mut geometry = pmg(&[("body", 0, 1)], &[piece]);
        if v.broken {
            geometry[0] = 0x16;
        }
        add(files, a, format!("{model}.pmg"), geometry);
        add(
            files,
            a,
            format!("{model}.pmd"),
            pmd(
                &[("default", &[&format!("{model}.mat")])],
                &[("default", &[1])],
                1,
            ),
        );
        add(
            files,
            a,
            format!("{model}.mat"),
            mat("eut2.truckpaint.tsnmapuv"),
        );
    }
}

/// The files of the vehicles as a mod package holds them (`def/vehicle/...`, no leading slash),
/// with a `manifest.sii` of the given categories.
pub fn mod_files(vehicles: &[Vehicle], categories: &[&str]) -> Vec<(String, Vec<u8>)> {
    let mut files = BTreeMap::new();
    add_vehicles(&mut files, vehicles);
    let manifest = categories
        .iter()
        .fold(String::from("SiiNunit\n{\nmod_package : .package_name\n{\npackage_version: \"2.2.00\"\nauthor: \"Erik\"\n"), |text, c| {
            text + &format!("category[]: \"{c}\"\n")
        })
        + "}\n}\n";
    let mut result = vec![("manifest.sii".to_owned(), manifest.into_bytes())];
    for (path, bytes) in files.into_values().flatten() {
        result.push((path.trim_start_matches('/').to_owned(), bytes));
    }
    result
}

/// Writes `base.scs`, `def.scs`, `version.scs` and the DLC archives named by the vehicles
/// into `folder`, for `application` (`eut2`, `ats`) at game `version` (`1.61.1.9`).
pub fn write_game(folder: &Path, application: &str, version: &str, vehicles: &[Vehicle]) {
    let mut files = BTreeMap::new();
    add(
        &mut files,
        "version",
        "/version.sii".into(),
        format!(
            "SiiNunit\n{{\nfs_pack_set : .version {{\napplication: {application}\nversion: \"{version}\"\n}}\n}}\n"
        ),
    );
    add_vehicles(&mut files, vehicles);
    let empty = ArchiveBuilder::default().file("/x", b"");
    for name in ["base", "def", "version"] {
        files.entry(name.to_owned()).or_default();
    }
    for (name, entries) in files {
        let archive = if entries.is_empty() {
            empty.build()
        } else {
            entries
                .iter()
                .fold(ArchiveBuilder::default(), |archive, (path, bytes)| {
                    archive.compressed(path, bytes)
                })
                .build()
        };
        std::fs::write(folder.join(format!("{name}.scs")), archive).unwrap();
    }
}

/// Writes Workshop mod `id` beside a game folder at `<library>/steamapps/common/<game>`: a
/// `versions.sii` for any game 1.x and one ZIP package, `content_161`, with the files of
/// `vehicles` under `base/`.
pub fn write_workshop_mod(game: &Path, id: &str, vehicles: &[Vehicle], categories: &[&str]) {
    let steamapps = game.parent().and_then(Path::parent).unwrap();
    let folder = steamapps.join("workshop/content/227300").join(id);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join("versions.sii"),
        "SiiNunit\n{\npackage_version_info : .v\n{\npackage_name: \"content_161\"\ncompatible_versions[]: \"1.*\"\n}\n}\n",
    )
    .unwrap();
    let files = mod_files(vehicles, categories);
    let entries = files
        .iter()
        .map(|(path, bytes)| {
            let name = if path == "manifest.sii" {
                path.clone()
            } else {
                format!("base/{path}")
            };
            (name, bytes.as_slice())
        })
        .collect::<Vec<_>>();
    let entries = entries
        .iter()
        .map(|(name, bytes)| (name.as_str(), *bytes))
        .collect::<Vec<_>>();
    std::fs::write(
        folder.join("content_161.zip"),
        tp_scs::synthetic::zip_package(&entries, true),
    )
    .unwrap();
}
