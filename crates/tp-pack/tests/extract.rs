//! No game required: synthetic HashFS archives built from folders of files.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tp_scs::synthetic::{ArchiveBuilder, PieceSpec, mat, pmd, pmg};

fn write(root: &Path, path: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

const CORNERS: [[f32; 2]; 3] = [[0., 0.], [1., 0.], [0., 1.]];
const INNER: [[f32; 2]; 3] = [[0.25, 0.25], [0.75, 0.25], [0.25, 0.75]];

/// A body of one piece whose first set is `_TEXCOORD1`; `second` adds a `_TEXCOORD2` set.
fn geometry(first: [[f32; 2]; 3], second: Option<[[f32; 2]; 3]>) -> Vec<u8> {
    let mut sets = vec![first.to_vec()];
    let mut mask = 0;
    if let Some(second) = second {
        sets.push(second.to_vec());
        mask = 0x100;
    }
    let piece = PieceSpec {
        material: 0,
        mask,
        uv_sets: sets,
        triangles: vec![[0, 1, 2]],
    };
    pmg(&[("body", 0, 1)], &[piece])
}

/// Writes `vehicle/shared/<name>` as a one-part, one-look model with the given shader.
fn write_model(source: &Path, name: &str, geometry: &[u8], effect: &str) {
    let material = format!("/vehicle/shared/{name}.mat");
    write(source, &format!("vehicle/shared/{name}.pmg"), geometry);
    write(
        source,
        &format!("vehicle/shared/{name}.pmd"),
        pmd(&[("default", &[&material])], &[("default", &[1])], 1),
    );
    write(source, &format!("vehicle/shared/{name}.mat"), mat(effect));
}

fn files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.extend(files(&path));
        } else {
            result.push(path);
        }
    }
    result.sort();
    result
}

/// Builds `game/<name>.scs` from the files under `game/<name>.scs.files`. A file named
/// `.noroot` makes the archive leave out its root listing, like `locale.scs`.
fn pack(root: &Path) {
    let game = root.join("game");
    for name in ["base", "def", "version"] {
        std::fs::create_dir_all(game.join(format!("{name}.scs.files"))).unwrap();
    }
    for entry in std::fs::read_dir(&game).unwrap() {
        let folder = entry.unwrap().path();
        let Some(name) = folder
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".files"))
            .map(str::to_owned)
        else {
            continue;
        };
        let mut archive = ArchiveBuilder::default();
        for file in files(&folder) {
            let relative = file.strip_prefix(&folder).unwrap().to_str().unwrap();
            let game_path = format!("/{relative}");
            let bytes = std::fs::read(&file).unwrap();
            archive = match relative {
                ".noroot" => archive.without_root(),
                _ if relative.ends_with(".mat") => archive.file(&game_path, &bytes),
                _ => archive.compressed(&game_path, &bytes),
            };
        }
        std::fs::write(game.join(name), archive.build()).unwrap();
    }
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let archive = root.join("game/version.scs.files");
    write(
        &archive,
        "version.sii",
        "SiiNunit\n{\nfs_pack_set : .version {\napplication: eut2\nversion: \"1.61.1.1\"\n}\n}\n",
    );
    for vehicle in ["example.good", "example.broken"] {
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/paint_job/accessory/flake.sii"),
            "SiiNunit\n{\nsimple_paint_job_data : .flake {\nacc_list[]: \"obsolete.accessory\"\nflake_uvscale: 2\n}\n}\n",
        );
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/data.sii"),
            format!(
                "SiiNunit\n{{\naccessory_truck_data : vehicle.{vehicle} {{\ninfo[]: \"Example\"\ninfo[]: \"Truck\"\n}}\n}}\n"
            ),
        );
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/cabin/high.sii"),
            format!(
                "SiiNunit\n{{\naccessory_cabin_data : high.{vehicle}.cabin {{\nname: \"High\"\n@include \"../common.sui\"\n}}\n}}\n"
            ),
        );
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/common.sui"),
            format!("model: \"/vehicle/shared/{vehicle}.pmd\"\n"),
        );
        let mut bytes = geometry(CORNERS, None);
        if vehicle == "example.broken" {
            // A geometry version the reader does not know.
            bytes[0] = 0x16;
        }
        write_model(&archive, vehicle, &bytes, "eut2.truckpaint.tsnmapuv");
    }
    temp
}

/// An `extract` command for the game folder of `root`, with its archives built.
fn command(root: &Path) -> Command {
    pack(root);
    let mut command = Command::new(env!("CARGO_BIN_EXE_tpv"));
    command
        .current_dir(root)
        .arg("extract")
        .arg(root.join("game"));
    command
}

fn run(root: &Path, vehicle: Option<&str>) -> std::process::Output {
    let mut command = command(root);
    if let Some(vehicle) = vehicle {
        command.args(["--vehicle", vehicle]);
    }
    command.output().unwrap()
}

fn batches(root: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = std::fs::read_dir(root.join("dist"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
fn isolated_failures_reports_and_unique_complete_outputs() {
    let fixture = fixture();
    let root = fixture.path();
    let result = run(root, None);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("incomplete batch"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let first = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(first.join("report.json")).unwrap()).unwrap();
    assert_eq!(report["failed"], 1, "{report:#}");
    // The report names the app's version: `dev` unless the build sets it.
    assert_eq!(report["tool_version"], tp_core::VERSION, "{report:#}");
    let results = report["results"].as_array().unwrap();
    let good = results
        .iter()
        .find(|r| r["vehicle"] == "example.good")
        .unwrap();
    assert_eq!(good["status"], "success");
    assert_eq!(good["technical_validation"], "passed");
    assert_eq!(good["in_game_validation"], "required");
    assert!(
        good["sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["path"].as_str().unwrap().ends_with("common.sui"))
    );
    let package = first.join(good["package"].as_str().unwrap());
    assert_eq!(
        tp_pack::check(&std::fs::read(&package).unwrap())
            .unwrap()
            .id,
        "official.ets2.example.good"
    );
    assert_eq!(std::fs::read_dir(&first).unwrap().count(), 2);
    let result = run(root, Some("example.good"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(batches(root).len(), 2);
    assert!(package.is_file());
    let other = batches(root).into_iter().find(|p| p != &first).unwrap();
    assert_eq!(
        std::fs::read(package).unwrap(),
        std::fs::read(other.join("official.ets2.example.good-1.61.1.tpv")).unwrap()
    );
}

#[test]
fn invalid_version_and_excluded_vehicle_publish_nothing() {
    let fixture = fixture();
    let root = fixture.path();
    let result = command(root)
        .args(["--previous", "missing-folder"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!root.join("dist").exists());
    let result = run(root, Some("trailer.example"));
    assert!(!result.status.success());
    assert!(!root.join("dist").exists());
    write(
        &root.join("game/version.scs.files"),
        "version.sii",
        "SiiNunit\n{\nfs_pack_set : .version {\napplication: ats\nversion: \"1.61.1.1\"\n}\n}\n",
    );
    let result = run(root, None);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("only ETS2"));
    assert!(!root.join("dist").exists());
}

#[test]
fn conflicting_sizes_use_the_largest_and_unsized_masks_stay_on_the_main_texture() {
    for (name, mask) in [
        ("missing-dimension", "/electric/shared_a4cfe8ff.tobj"),
        ("ambiguous-dimension", "/mask_size_128x64"),
    ] {
        let fixture = fixture();
        let root = fixture.path();
        let source = root.join("game/version.scs.files");
        write(
            &source,
            "def/vehicle/truck/example.good/accessory/mirror/high.sii",
            "SiiNunit\n{\naccessory_addon_data : high.example.good.mirror {\nmodel: \"/vehicle/shared/example.good.pmd\"\n}\n}\n",
        );
        write(
            &source,
            "def/vehicle/truck/example.good/paint_job/accessory/first.sii",
            format!(
                "SiiNunit\n{{\nsimple_paint_job_data : .first {{\nacc_list[]: \"mirror.high\"\npaint_job_mask: \"{mask}\"\n}}\n}}\n"
            ),
        );
        if name == "ambiguous-dimension" {
            write(
                &source,
                "def/vehicle/truck/example.good/paint_job/accessory/second.sii",
                "SiiNunit\n{\nsimple_paint_job_data : .second {\nacc_list[]: \"mirror.high\"\npaint_job_mask: \"/mask_size_256x64\"\n}\n}\n",
            );
        }
        write(
            &source,
            "def/vehicle/truck/example.good/accessory/mirror/other.sii",
            "SiiNunit\n{\naccessory_addon_data : other.example.good.mirror {\nmodel: \"/vehicle/shared/example.good.pmd\"\n}\n}\n",
        );
        write(
            &source,
            "def/vehicle/truck/example.good/paint_job/accessory/fill.sii",
            "SiiNunit\n{\nsimple_paint_job_data : .fill {\nacc_list[]: \"mirror.other\"\npaint_job_mask: \"/shared_00000000.tobj\"\n}\n}\n",
        );
        let result = run(root, Some("example.good"));
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let batch = batches(root).pop().unwrap();
        let report: Value =
            serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
        let vehicle = &report["results"][0];
        let diagnostics = vehicle["dimension_diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[1]["reference"], "mirror.other");
        assert_eq!(diagnostics[1]["disposition"], "main_texture");
        let problem = &diagnostics[0];
        assert_eq!(problem["reference"], "mirror.high");
        assert_eq!(problem["masks"][0]["path"], mask);
        assert_eq!(
            problem["masks"][0]["referenced_by"][0]["source"],
            "version.scs"
        );
        let package = tp_pack::check(
            &std::fs::read(batch.join(vehicle["package"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        let accessories = &package.paint_job.accessories;
        if name == "ambiguous-dimension" {
            assert_eq!(problem["reason"], "conflicting_crop_dimensions");
            assert_eq!(problem["disposition"], "resolved_largest");
            assert_eq!(problem["selected"], serde_json::json!([256, 64]));
            assert_eq!(
                problem["candidates"],
                serde_json::json!([[128, 64], [256, 64]])
            );
            assert_eq!(problem["masks"].as_array().unwrap().len(), 2);
            assert_eq!(accessories.len(), 1);
            assert_eq!(accessories[0].game_ids, ["mirror.high"]);
            assert_eq!(<[u32; 2]>::from(accessories[0].texture.size), [256, 64]);
        } else {
            assert_eq!(problem["reason"], "missing_crop_dimensions");
            assert_eq!(problem["disposition"], "main_texture");
            assert_eq!(problem["candidates"], serde_json::json!([]));
            assert!(problem["masks"][0]["crop_size"].is_null());
            assert!(accessories.is_empty());
        }
        let main = vehicle["textures"][0]["contributors"].as_array().unwrap();
        let in_main = |id: &str| main.iter().any(|c| c["id"] == id);
        assert!(in_main("mirror.other"));
        assert_eq!(in_main("mirror.high"), name == "missing-dimension");
        assert_eq!(std::fs::read_dir(batch).unwrap().count(), 2);
    }
}

#[test]
fn missing_model_outputs_refuse_whole_vehicle() {
    let fixture = fixture();
    let root = fixture.path();
    std::fs::remove_file(root.join("game/version.scs.files/vehicle/shared/example.good.mat"))
        .unwrap();
    assert!(!run(root, Some("example.good")).status.success());
    assert_eq!(
        std::fs::read_dir(batches(root).pop().unwrap())
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn missing_dependencies_are_reported_without_guessing_a_dlc() {
    let orphan_fixture = fixture();
    let root = orphan_fixture.path();
    write(
        &root.join("game/version.scs.files"),
        "def/vehicle/truck/example.good/paint_job/accessory/orphan.sii",
        "SiiNunit\n{\nsimple_paint_job_data : .orphan {\nacc_list[]: \"sideskirt.missing\"\nacc_list[]: \"mirror.missing\"\npaint_job_mask: \"/mask_size_256x64\"\n}\n}\n",
    );
    assert!(run(root, Some("example.good")).status.success());
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let missing = report["results"][0]["missing_dependencies"]
        .as_array()
        .unwrap();
    assert_eq!(missing.len(), 2);
    assert_eq!(missing[0]["kind"], "accessory_definition");
    assert_eq!(missing[0]["reference"], "mirror.missing");
    assert_eq!(missing[0]["referenced_by"][0]["source"], "version.scs");
    assert!(missing.iter().all(|d| d["expected_dlc"].is_null()
        && d["dlc_origin"] == "unknown"
        && d["disposition"] == "ignored_reference"));
    assert_eq!(report["results"][0]["status"], "success");
    assert_eq!(std::fs::read_dir(&batch).unwrap().count(), 2);
    let clean_fixture = fixture();
    assert!(
        run(clean_fixture.path(), Some("example.good"))
            .status
            .success()
    );
    let filename = report["results"][0]["package"].as_str().unwrap();
    assert_eq!(
        std::fs::read(batch.join(filename)).unwrap(),
        std::fs::read(batches(clean_fixture.path()).pop().unwrap().join(filename)).unwrap()
    );

    write(
        &root.join("game/version.scs.files"),
        "def/vehicle/truck/example.good/accessory/mirror/missing.sii",
        "SiiNunit\n{\naccessory_addon_data : missing.example.good.mirror {\n}\n}\n",
    );
    // A defined accessory with no model field has nothing to paint: excluded.
    assert!(run(root, Some("example.good")).status.success());
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let missing = &report["results"][0]["missing_dependencies"][0];
    assert_eq!(missing["reference"], "mirror.missing");
    assert_eq!(missing["kind"], "accessory_without_model");
    assert_eq!(missing["disposition"], "excluded");
    assert_eq!(std::fs::read_dir(&batch).unwrap().count(), 2);

    let fixture = fixture();
    let root = fixture.path();
    std::fs::remove_file(root.join("game/version.scs.files/vehicle/shared/example.good.pmd"))
        .unwrap();
    assert!(!run(root, Some("example.good")).status.success());
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    assert_eq!(
        report["results"][0]["missing_dependencies"][0]["kind"],
        "game_file"
    );
    assert_eq!(
        report["results"][0]["missing_dependencies"][0]["reference"],
        "/vehicle/shared/example.good.pmd"
    );
    assert_eq!(
        report["results"][0]["missing_dependencies"][0]["disposition"],
        "blocking"
    );
}

#[test]
fn every_material_uses_the_ordinary_paint_uvs() {
    for alternate in [false, true] {
        let fixture = fixture();
        let root = fixture.path();
        let source = root.join("game/version.scs.files");
        let both = geometry(CORNERS, Some(INNER));
        let cabin_shader = if alternate {
            "eut2.truckpaint.tsnmapuv.altuv"
        } else {
            "eut2.truckpaint"
        };
        let accessory_shader = if alternate {
            "eut2.truckpaint.tsnmap"
        } else {
            "eut2.truckpaint.tsnmap.airbrush.altuv"
        };
        for (model, shader) in [("example.good", cabin_shader), ("mirror", accessory_shader)] {
            write_model(&source, model, &both, shader);
        }
        write(
            &source,
            "def/vehicle/truck/example.good/common.sui",
            "detail_model: \"/vehicle/shared/example.good.pmd\"\nvariant_uk: default\n",
        );
        write(
            &source,
            "def/vehicle/truck/example.good/accessory/mirror/paint.sii",
            "SiiNunit\n{\naccessory_addon_data : paint.example.good.mirror {\nmodel: \"/vehicle/shared/mirror.pmd\"\n}\n}\n",
        );
        write(
            &source,
            "def/vehicle/truck/example.good/paint_job/accessory/mirror.sii",
            "SiiNunit\n{\nsimple_paint_job_data : .paint {\nacc_list[]: \"mirror.paint\"\npaint_job_mask: \"/mask_size_256x64\"\n}\n}\n",
        );
        let result = run(root, Some("example.good"));
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let batch = batches(root).pop().unwrap();
        let report: Value =
            serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
        let result = &report["results"][0];
        let bytes = std::fs::read(batch.join(result["package"].as_str().unwrap())).unwrap();
        let package = tp_vehicles::Package::read(&bytes).unwrap();
        assert!(!package.manifest.game.alt_uv);
        assert_eq!(package.manifest.paint_job.main.len(), 1);
        assert_eq!(package.manifest.paint_job.accessories.len(), 1);
        assert_eq!(
            package.manifest.paint_job.accessories[0].texture.size,
            [256, 64].into()
        );
        for texture in result["textures"].as_array().unwrap() {
            assert_eq!(texture["edges"], 3);
            for contributor in texture["contributors"].as_array().unwrap() {
                for selection in contributor["selections"].as_array().unwrap() {
                    for surface in selection["painted_surfaces"].as_array().unwrap() {
                        assert_eq!(surface["uv_alias"], "_TEXCOORD1");
                    }
                }
            }
        }
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut svg = String::new();
        std::io::Read::read_to_string(
            &mut archive.by_name("templates/mirror_paint.svg").unwrap(),
            &mut svg,
        )
        .unwrap();
        assert!(svg.contains("M0.000,0.000"), "{svg}");
        assert!(!svg.contains("M64.000,16.000"), "{svg}");
    }
}

#[test]
fn metallic_shaders_and_finite_uv_overflow_produce_traceable_packages() {
    let fixture = fixture();
    let root = fixture.path();
    let source = root.join("game/version.scs.files");
    write_model(
        &source,
        "example.good",
        &geometry([[-2.125, 1.75], [1., 0.], [0., 1.]], None),
        "eut2.truckpaint.tsnmapuv.airbrush.flipflake",
    );
    let result = run(root, Some("example.good"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let vehicle = &report["results"][0];
    let surface =
        &vehicle["textures"][0]["contributors"][0]["selections"][0]["painted_surfaces"][0];
    assert_eq!(surface["uv_alias"], "_TEXCOORD1");
    assert_eq!(surface["out_of_bounds_vertices"], 1);
    assert_eq!(
        surface["uv_bounds"],
        serde_json::json!({"min":[-2.125,0.0],"max":[1.0,1.75]})
    );
    let package = std::fs::read(batch.join(vehicle["package"].as_str().unwrap())).unwrap();
    tp_pack::check(&package).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(package)).unwrap();
    let manifest: Value = serde_json::from_reader(zip.by_name("vehicle.json").unwrap()).unwrap();
    assert_eq!(manifest["game"]["alt_uv"], false);
    assert_eq!(manifest["license"], tp_pack::extract::LICENSE);
    assert_eq!(manifest["authors"][0], "SCS Software");
    let mut svg = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("templates/high.svg").unwrap(), &mut svg)
        .unwrap();
    assert!(svg.contains("-8704.000,7168.000"));
}

#[test]
fn solid_colour_surface_is_excluded_without_inventing_a_paint_job_uv_mapping() {
    let fixture = fixture();
    let root = fixture.path();
    let source = root.join("game/version.scs.files");
    write_model(
        &source,
        "solid",
        &geometry(CORNERS, None),
        "eut2.dif.spec.weight.add.env.tsnmapuv.paint.a",
    );
    write(
        &source,
        "def/vehicle/truck/example.good/accessory/guard/painted.sii",
        "SiiNunit\n{\naccessory_addon_data : painted.example.good.guard {\nexterior_model: \"/vehicle/shared/solid.pmd\"\n}\n}\n",
    );
    write(
        &source,
        "def/vehicle/truck/example.good/paint_job/accessory/guard.sii",
        "SiiNunit\n{\nsimple_paint_job_data : .guard {\nacc_list[]: \"guard.painted\"\npaint_job_mask: \"/mask_size_128x64.tobj\"\n}\n}\n",
    );
    let result = run(root, Some("example.good"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let vehicle = &report["results"][0];
    let missing = &vehicle["missing_dependencies"][0];
    assert_eq!(missing["reference"], "guard.painted");
    assert_eq!(missing["kind"], "surface_without_livery_material");
    assert_eq!(missing["disposition"], "excluded");
    let package =
        tp_pack::check(&std::fs::read(batch.join(vehicle["package"].as_str().unwrap())).unwrap())
            .unwrap();
    assert!(package.paint_job.accessories.is_empty());
    let main = vehicle["textures"][0]["contributors"].as_array().unwrap();
    assert!(main.iter().all(|c| c["id"] != "guard.painted"));
}

#[test]
fn declared_sizes_below_the_package_minimum_are_scaled_up() {
    let fixture = fixture();
    let root = fixture.path();
    let source = root.join("game/version.scs.files");
    write(
        &source,
        "def/vehicle/truck/example.good/accessory/mudflap/small.sii",
        "SiiNunit\n{\naccessory_addon_data : small.example.good.mudflap {\nmodel: \"/vehicle/shared/example.good.pmd\"\n}\n}\n",
    );
    write(
        &source,
        "def/vehicle/truck/example.good/paint_job/accessory/mudflap.sii",
        "SiiNunit\n{\nsimple_paint_job_data : .mudflap {\nacc_list[]: \"mudflap.small\"\npaint_job_mask: \"/mask_size_32x16.tobj\"\n}\n}\n",
    );
    let result = run(root, Some("example.good"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let vehicle = &report["results"][0];
    let problem = &vehicle["dimension_diagnostics"][0];
    assert_eq!(problem["reference"], "mudflap.small");
    assert_eq!(problem["disposition"], "raised_to_minimum");
    assert_eq!(problem["candidates"], serde_json::json!([[32, 16]]));
    assert_eq!(problem["selected"], serde_json::json!([128, 64]));
    let package =
        tp_pack::check(&std::fs::read(batch.join(vehicle["package"].as_str().unwrap())).unwrap())
            .unwrap();
    let accessory = &package.paint_job.accessories[0];
    assert_eq!(<[u32; 2]>::from(accessory.texture.size), [128, 64]);
}

#[test]
fn names_come_from_the_game_localization() {
    let fixture = fixture();
    let root = fixture.path();
    let source = root.join("game/version.scs.files");
    write(
        &source,
        "def/vehicle/truck/example.good/data.sii",
        "SiiNunit\n{\naccessory_truck_data : vehicle.example.good {\ninfo[]: \"@@brand_example@@\"\ninfo[]: \"@@series_example_stralis@@\"\ninfo[]: \"\"\ninfo[]: \"@@brand_example@@\"\ninfo[]: \"@@series_example_hiway@@\"\n}\n}\n",
    );
    write(
        &source,
        "def/vehicle/truck/example.good/cabin/high.sii",
        "SiiNunit\n{\naccessory_cabin_data : high.example.good.cabin {\nname: \"@@cabin_high_roof@@\"\n@include \"../common.sui\"\n}\n}\n",
    );
    write(
        &source,
        "locale/common.sui",
        "\tkey[]: \"brand_example\"\n\tval[]: \"Example\"\n\n\tkey[]: \"series_example_stralis\"\n\tval[]: \"Stralis\"\n\tkey[]: \"series_example_hiway\"\n\tval[]: \"Hi-Way\"\n",
    );
    write(
        &source,
        "locale/en_us/accessories.sui",
        "\tkey[]: \"cabin_high_roof\"\n\tval[]: \"High Roof\" #+ Translator note \"quoted\"\n",
    );
    let result = run(root, Some("example.good"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    assert_eq!(report["name_sources"][0]["source"], "version.scs");
    let package = tp_pack::check(
        &std::fs::read(batch.join(report["results"][0]["package"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    assert_eq!(package.name, "Example Stralis Hi-Way");
    assert_eq!(package.brand, "Example");
    assert_eq!(package.paint_job.main[0].name, "High Roof");

    // Cabins the game names alike get their game id appended.
    write(
        &source,
        "def/vehicle/truck/example.good/cabin/high_8x4.sii",
        "SiiNunit\n{\naccessory_cabin_data : high_8x4.example.good.cabin {\nname: \"@@cabin_high_roof@@\"\n@include \"../common.sui\"\n}\n}\n",
    );
    assert!(run(root, Some("example.good")).status.success());
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let package = tp_pack::check(
        &std::fs::read(batch.join(report["results"][0]["package"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    let names = package
        .paint_job
        .main
        .iter()
        .map(|p| p.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["High Roof (high)", "High Roof (high_8x4)"]);
}

#[test]
fn changed_templates_get_the_next_layout_version() {
    let fixture = fixture();
    let root = fixture.path();
    // The first main texture's layout version and the package version; `None`
    // when nothing was written because nothing changed.
    let layout = |previous: Option<&Path>| {
        let mut command = command(root);
        command.args(["--vehicle", "example.good"]);
        if let Some(previous) = previous {
            command.arg("--previous").arg(previous);
        }
        let result = command.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let batch = batches(root).pop().unwrap();
        let package = files(&batch)
            .into_iter()
            .find(|p| p.extension().is_some_and(|e| e == "tpv"))
            .map(|p| tp_pack::check(&std::fs::read(p).unwrap()).unwrap());
        (
            batch,
            package.map(|p| {
                (
                    p.paint_job.main[0].texture.layout_version,
                    p.version.to_string(),
                )
            }),
        )
    };
    let (first, version) = layout(None);
    assert_eq!(version, Some((1, "1.61.1".into())));
    // Same template, same game: nothing to install.
    let (_, version) = layout(Some(&first));
    assert_eq!(version, None);
    // A patch moves the UVs: the next layout version, under a higher package version.
    write_model(
        &root.join("game/version.scs.files"),
        "example.good",
        &geometry([[0., 0.], [0.5, 0.], [0., 1.]], None),
        "eut2.truckpaint.tsnmapuv",
    );
    let (third, version) = layout(Some(&first));
    assert_eq!(version, Some((2, "1.61.2".into())));
    let (_, version) = layout(Some(&third));
    assert_eq!(version, None);
}

/// An owned trailer `id` with one chassis and one body, both painted, using
/// the shared painted model; `brand` is its first `info[]` value.
fn trailer(root: &Path, id: &str, brand: &str) {
    let source = root.join("game/version.scs.files");
    let def = format!("def/vehicle/trailer_owned/{id}");
    write(
        &source,
        &format!("{def}/data.sii"),
        format!(
            "SiiNunit\n{{\naccessory_trailer_data : vehicle.{id} {{\ninfo[]: \"{brand}\"\ninfo[]: \"@@series_trailer@@\"\n}}\n}}\n"
        ),
    );
    for (group, unit, data) in [
        ("chassis", "ch_3", "accessory_chassis_data"),
        ("body", "curtain", "accessory_trailer_body_data"),
    ] {
        write(
            &source,
            &format!("{def}/{group}/{unit}.sii"),
            format!(
                "SiiNunit\n{{\n{data} : {unit}.{id}.{group} {{\nmodel: \"/vehicle/shared/example.good.pmd\"\n}}\n}}\n"
            ),
        );
    }
    write(
        &source,
        &format!("{def}/paint_job/default.sii"),
        format!(
            "SiiNunit\n{{\naccessory_paint_job_data : default.{id}.paint_job {{\npaint_job_mask: \"/trailer/pjm_at_0x0_size_4x4.tobj\"\n}}\n}}\n"
        ),
    );
}

fn report(batch: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap()
}

#[test]
fn owned_trailers_are_discovered_and_can_be_requested_alone() {
    let fixture = fixture();
    let root = fixture.path();
    trailer(root, "example.trailer", "");
    run(root, None);
    let all = report(&batches(root).pop().unwrap());
    let discovered = all["discovered_vehicles"].as_array().unwrap();
    assert!(discovered.iter().any(|v| v == "example.trailer"), "{all:#}");
    // Trucks first, then trailers.
    let kinds = all["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(kinds, ["truck", "truck", "trailer"]);
    run(root, Some("example.trailer"));
    let one = report(&batches(root).pop().unwrap());
    assert_eq!(
        one["processed_vehicles"],
        serde_json::json!(["example.trailer"])
    );
    assert_eq!(one["results"][0]["kind"], "trailer");
}

#[test]
fn an_id_defined_as_truck_and_trailer_is_refused() {
    let fixture = fixture();
    let root = fixture.path();
    trailer(root, "example.good", "");
    assert!(!run(root, Some("example.good")).status.success());
    let batch = batches(root).pop().unwrap();
    let result = &report(&batch)["results"][0];
    assert_eq!(result["stage"], "discovery");
    assert_eq!(result["kinds"], serde_json::json!(["truck", "trailer"]));
    assert_eq!(std::fs::read_dir(batch).unwrap().count(), 1);
}

#[test]
fn help_mentions_trailers() {
    let help = Command::new(env!("CARGO_BIN_EXE_tpv"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&help.stdout).contains("trailers"));
}

/// Extracts `vehicle` alone and returns its report entry and validated manifest.
fn extract_one(root: &Path, vehicle: &str) -> (Value, tp_vehicles::Manifest) {
    let result = run(root, Some(vehicle));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let batch = batches(root).pop().unwrap();
    let entry = report(&batch)["results"][0].clone();
    let bytes = std::fs::read(batch.join(entry["package"].as_str().unwrap())).unwrap();
    (entry, tp_pack::check(&bytes).unwrap())
}

#[test]
fn trailers_get_one_main_texture_and_game_names() {
    let fixture = fixture();
    let root = fixture.path();
    trailer(root, "scs.box", "");
    trailer(root, "krone.coolliner", "Krone");
    write(
        &root.join("game/version.scs.files"),
        "locale/en_us/localization.sui",
        "\tkey[]: \"series_trailer\"\n\tval[]: \"Cool Liner\"\n",
    );
    let (entry, scs) = extract_one(root, "scs.box");
    assert_eq!(entry["kind"], "trailer");
    assert_eq!((scs.name.as_str(), scs.brand.as_str()), ("SCS Box", "SCS"));
    assert_eq!(scs.kind, tp_vehicles::Kind::Trailer);
    assert_eq!(scs.game.path, "scs.box");
    let [main] = scs.paint_job.main.as_slice() else {
        panic!("one main texture expected: {:?}", scs.paint_job.main);
    };
    assert_eq!(main.id, "trailer");
    assert!(main.game_ids.is_empty());
    assert_eq!(<[u32; 2]>::from(main.texture.size), [4096, 4096]);
    // The chassis and the body both paint the main texture.
    let contributors = entry["textures"][0]["contributors"].as_array().unwrap();
    let ids = contributors
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["body.curtain", "chassis.ch_3"]);
    let (_, krone) = extract_one(root, "krone.coolliner");
    assert_eq!(krone.name, "Krone Cool Liner");
}

#[test]
fn trailer_bodies_sharing_a_mask_in_every_paint_job_share_one_texture() {
    let fixture = fixture();
    let root = fixture.path();
    trailer(root, "scs.box", "");
    let source = root.join("game/version.scs.files");
    let def = "def/vehicle/trailer_owned/scs.box";
    for body in ["dry_van", "ins", "reefer"] {
        write(
            &source,
            &format!("{def}/body/{body}.sii"),
            format!(
                "SiiNunit\n{{\naccessory_trailer_body_data : {body}.scs.box.body {{\nmodel: \"/vehicle/shared/example.good.pmd\"\n}}\n}}\n"
            ),
        );
    }
    let ovr = |mask: &str, ids: &[&str]| {
        let list = ids
            .iter()
            .map(|id| format!("acc_list[]: \"{id}\"\n"))
            .collect::<String>();
        format!(
            "simple_paint_job_data : .o{} {{\npaint_job_mask: \"{mask}\"\n{list}}}\n",
            ids.len()
        )
    };
    let shared = "/t/pjm_at_0x0_size_512x256.tobj";
    write(
        &source,
        &format!("{def}/paint_job/accessory/first.sii"),
        format!(
            "SiiNunit\n{{\n{}}}\n",
            ovr(shared, &["body.dry_van", "body.ins", "body.reefer"])
        ),
    );
    // The second paint job paints the reefer from another region.
    write(
        &source,
        &format!("{def}/paint_job/accessory/second.sii"),
        format!(
            "SiiNunit\n{{\n{}{}}}\n",
            ovr(shared, &["body.dry_van", "body.ins"]),
            ovr("/t/pjm_at_512x0_size_512x256.tobj", &["body.reefer"])
        ),
    );
    let (entry, manifest) = extract_one(root, "scs.box");
    let accessories = &manifest.paint_job.accessories;
    let ids = accessories
        .iter()
        .map(|a| a.game_ids.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        [vec!["body.dry_van", "body.ins"], vec!["body.reefer"]],
        "{entry:#}"
    );
    assert_eq!(<[u32; 2]>::from(accessories[0].texture.size), [512, 256]);
    let grouped = entry["textures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["part"] == "body_dry_van")
        .unwrap();
    assert_eq!(grouped["contributors"].as_array().unwrap().len(), 2);
    assert_eq!(grouped["shared_masks"], serde_json::json!([shared]));
    // The ungrouped bodies still paint the main texture with the chassis.
    let main = &entry["textures"][0]["contributors"];
    assert_eq!(main.as_array().unwrap().len(), 2, "{main:#}");
}

#[test]
fn trailers_follow_the_truck_rules_and_need_a_paint_job() {
    let fixture = fixture();
    let root = fixture.path();
    let source = root.join("game/version.scs.files");
    let def = "def/vehicle/trailer_owned/scs.box";
    trailer(root, "scs.box", "");
    // Only plain colours, no mask and no override: skipped, not a failure.
    write(
        &source,
        &format!("{def}/paint_job/default.sii"),
        "SiiNunit\n{\naccessory_paint_job_data : default.scs.box.paint_job {\nbase_color: (0.1, 0.2, 0.3)\nbase_color_locked: true\n}\n}\n",
    );
    assert!(run(root, Some("scs.box")).status.success());
    let batch = batches(root).pop().unwrap();
    let skipped = &report(&batch)["results"][0];
    assert_eq!(skipped["status"], "skipped");
    assert_eq!(skipped["reason"], "not_paintable");
    assert_eq!(report(&batch)["failed"], 0);
    assert_eq!(std::fs::read_dir(batch).unwrap().count(), 1);
    // An override alone is enough; its 32×32 size is scaled up, and a part without
    // livery material is left out.
    write_model(
        &source,
        "solid",
        &geometry(CORNERS, None),
        "eut2.dif.spec.weight.add.env.tsnmapuv.paint.a",
    );
    for (unit, model) in [("mudflap", "example.good"), ("guard", "solid")] {
        write(
            &source,
            &format!("{def}/accessory/{unit}.sii"),
            format!(
                "SiiNunit\n{{\naccessory_addon_data : {unit}.scs.box.r_part {{\nmodel: \"/vehicle/shared/{model}.pmd\"\n}}\n}}\n"
            ),
        );
    }
    write(
        &source,
        &format!("{def}/paint_job/accessory/first.sii"),
        "SiiNunit\n{\nsimple_paint_job_data : .o {\npaint_job_mask: \"/t/pjm_size_32x32.tobj\"\nacc_list[]: \"r_part.mudflap\"\nacc_list[]: \"r_part.guard\"\n}\n}\n",
    );
    let (entry, manifest) = extract_one(root, "scs.box");
    let mudflap = &manifest.paint_job.accessories[0];
    assert_eq!(mudflap.game_ids, ["r_part.mudflap"]);
    assert_eq!(<[u32; 2]>::from(mudflap.texture.size), [64, 64]);
    assert_eq!(
        entry["dimension_diagnostics"][0]["disposition"],
        "raised_to_minimum"
    );
    let excluded = &entry["missing_dependencies"][0];
    assert_eq!(
        (&excluded["reference"], &excluded["disposition"]),
        (
            &serde_json::json!("r_part.guard"),
            &serde_json::json!("excluded")
        )
    );
    // Rebuilt against that batch: nothing changed, so nothing is written.
    let first = batches(root).pop().unwrap();
    let result = command(root)
        .args(["--vehicle", "scs.box", "--previous"])
        .arg(&first)
        .output()
        .unwrap();
    assert!(result.status.success());
    let again = batches(root).pop().unwrap();
    assert!(!again.join("official.ets2.scs.box-1.61.1.tpv").exists());
    assert_eq!(report(&again)["results"][0]["reason"], "up_to_date");
    // An overridden accessory with no model field is left out (Krone Box Liner's
    // container body); a model that exists but cannot be read still refuses the trailer.
    write(
        &source,
        &format!("{def}/accessory/mudflap.sii"),
        "SiiNunit\n{\naccessory_addon_data : mudflap.scs.box.r_part {\n}\n}\n",
    );
    let (entry, _) = extract_one(root, "scs.box");
    let mudflap = entry["missing_dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["reference"] == "r_part.mudflap")
        .unwrap();
    assert_eq!(mudflap["kind"], "accessory_without_model");
    assert_eq!(mudflap["disposition"], "excluded");
    write(
        &source,
        &format!("{def}/accessory/mudflap.sii"),
        "SiiNunit\n{\naccessory_addon_data : mudflap.scs.box.r_part {\nmodel: \"/vehicle/shared/absent.pmd\"\n}\n}\n",
    );
    assert!(!run(root, Some("scs.box")).status.success());
}

#[test]
fn a_trailer_whose_parts_all_have_their_own_region_gets_a_plain_main_texture() {
    let fixture = fixture();
    let root = fixture.path();
    trailer(root, "krone.coolliner", "Krone");
    write(
        &root.join("game/version.scs.files"),
        "def/vehicle/trailer_owned/krone.coolliner/paint_job/accessory/default.sii",
        "SiiNunit\n{\nsimple_paint_job_data : .o {\npaint_job_mask: \"/t/pjm_size_1024x512.tobj\"\nacc_list[]: \"body.curtain\"\n}\nsimple_paint_job_data : .p {\npaint_job_mask: \"/t/pjm_size_512x512.tobj\"\nacc_list[]: \"chassis.ch_3\"\n}\n}\n",
    );
    let (entry, manifest) = extract_one(root, "krone.coolliner");
    let [main] = manifest.paint_job.main.as_slice() else {
        panic!("one main texture expected");
    };
    assert_eq!(<[u32; 2]>::from(main.texture.size), [64, 64]);
    assert!(main.game_ids.is_empty());
    assert_eq!(entry["textures"][0]["edges"], 0);
    assert_eq!(manifest.paint_job.accessories.len(), 2);
}

#[test]
fn an_archive_without_a_root_listing_is_still_searched_by_path() {
    let fixture = fixture();
    let root = fixture.path();
    // The model also lives in def.scs, whose root cannot be listed (like locale.scs):
    // both sources stay recorded, last mounted wins.
    let def = root.join("game/def.scs.files");
    write(&def, "vehicle/shared/example.good.pmd", "shadowed");
    write(&def, ".noroot", "");
    let output = run(root, Some("example.good"));
    assert!(output.status.success(), "{output:?}");
    let report = report(&batches(root)[0]);
    let model = report["results"][0]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["path"] == "/vehicle/shared/example.good.pmd")
        .unwrap();
    assert_eq!(
        model["mounted_sources"],
        serde_json::json!(["def.scs", "version.scs"])
    );
    assert_eq!(model["source"], "version.scs");
}

#[test]
fn the_reader_replaces_the_converter() {
    let fixture = fixture();
    let root = fixture.path();
    assert!(run(root, Some("example.good")).status.success());
    let report = report(&batches(root)[0]);
    assert_eq!(report["reader"]["name"], "tp-scs");
    assert!(report["reader"]["version"].is_string());
    assert!(report.get("converter").is_none() && report.get("model_cache").is_none());
    assert!(!root.join("target/game-extraction/cache").exists());
    // The old option is refused before anything is read.
    let result = command(root)
        .args(["--converter", "converter"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("--converter"));
}

#[test]
fn an_unsupported_archive_version_stops_the_batch_before_any_vehicle() {
    let fixture = fixture();
    let root = fixture.path();
    pack(root);
    let path = root.join("game/def.scs");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[4] = 3;
    std::fs::write(&path, bytes).unwrap();
    // Not repacked: run the binary directly on the damaged archive.
    let result = Command::new(env!("CARGO_BIN_EXE_tpv"))
        .current_dir(root)
        .arg("extract")
        .arg(root.join("game"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(
        error.contains("def.scs") && error.contains("HashFS version 3"),
        "{error}"
    );
    assert!(!root.join("dist").exists());
}

#[test]
fn an_unknown_geometry_version_refuses_only_the_vehicles_using_it() {
    let fixture = fixture();
    let root = fixture.path();
    assert!(!run(root, None).status.success());
    let report = report(&batches(root)[0]);
    let broken = report["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["vehicle"] == "example.broken")
        .unwrap();
    assert_eq!(broken["status"], "failure");
    let error = broken["error"].as_str().unwrap();
    assert!(
        error.contains("/vehicle/shared/example.broken.pmg") && error.contains("0x16"),
        "{error}"
    );
    assert_eq!(report["results"][1]["status"], "success");
}

#[test]
fn successful_batches_remove_their_work_folder() {
    let fixture = fixture();
    let root = fixture.path();
    trailer(root, "scs.box", "");
    write(
        &root.join("game/version.scs.files"),
        "def/vehicle/trailer_owned/scs.box/paint_job/default.sii",
        "SiiNunit\n{\naccessory_paint_job_data : default.scs.box.paint_job {\nbase_color: (0.1, 0.2, 0.3)\n}\n}\n",
    );
    let work = || {
        std::fs::read_dir(root.join("target/game-extraction"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>()
    };
    // A built truck, then a skipped colour-only trailer: both batches succeed.
    for vehicle in ["example.good", "scs.box"] {
        assert!(run(root, Some(vehicle)).status.success());
        let report = report(&batches(root).pop().unwrap());
        assert_eq!(report["work_directory_kept"], false);
        assert!(work().is_empty(), "{vehicle}: {:?}", work());
    }
    // A failed vehicle keeps the folder, and the report names it.
    assert!(!run(root, Some("example.broken")).status.success());
    let report = report(&batches(root).pop().unwrap());
    assert_eq!(report["work_directory_kept"], true);
    assert_eq!(work().len(), 1);
    assert!(
        root.join(report["work_directory"].as_str().unwrap())
            .is_dir()
    );
}

#[test]
fn concurrent_builds_match_single_vehicle_batches() {
    let fixture = fixture();
    let root = fixture.path();
    trailer(root, "scs.box", "");
    trailer(root, "krone.coolliner", "Krone");
    // example.broken fails: the batch fails, the others are still published.
    assert!(!run(root, None).status.success());
    let all = batches(root)[0].clone();
    let order = report(&all)["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["vehicle"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        order,
        [
            "example.broken",
            "example.good",
            "krone.coolliner",
            "scs.box"
        ]
    );
    for vehicle in ["example.good", "krone.coolliner", "scs.box"] {
        assert!(run(root, Some(vehicle)).status.success(), "{vehicle}");
        let alone = batches(root).pop().unwrap();
        let name = format!("official.ets2.{vehicle}-1.61.1.tpv");
        assert_eq!(
            std::fs::read(all.join(&name)).unwrap(),
            std::fs::read(alone.join(&name)).unwrap(),
            "{vehicle}"
        );
    }
}
