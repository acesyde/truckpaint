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
            &mut files,
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
            &mut files,
            a,
            unit,
            format!("SiiNunit\n{{\n{header} {{\nname: \"High\"\nmodel: \"{model}.pmd\"\n}}\n}}\n"),
        );
        if v.trailer && v.paintable {
            add(
                &mut files,
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
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            uv_sets: vec![vec![[0., 0.], second, [0., 1.]]],
            triangles: vec![[0, 1, 2]],
        };
        let mut geometry = pmg(&[("body", 0, 1)], &[piece]);
        if v.broken {
            geometry[0] = 0x16;
        }
        add(&mut files, a, format!("{model}.pmg"), geometry);
        add(
            &mut files,
            a,
            format!("{model}.pmd"),
            pmd(
                &[("default", &[&format!("{model}.mat")])],
                &[("default", &[1])],
                1,
            ),
        );
        add(
            &mut files,
            a,
            format!("{model}.mat"),
            mat("eut2.truckpaint.tsnmapuv"),
        );
    }
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
