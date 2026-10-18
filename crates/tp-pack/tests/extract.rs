//! No game required: synthetic archives and a converter stand-in.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

const CONVERTER: &str = r#"#!/bin/sh
set -eu
mount=''
export=''
action=''
path=''
while [ "$#" -gt 0 ]; do
    case "$1" in
        -b) mount="$2"; shift 2 ;;
        -e) export="$2"; shift 2 ;;
        *) action="$1"; path="$2"; shift 2 ;;
    esac
done
root="$mount.files"
printf '%s %s %s\n' "$action" "$path" "${mount##*/}" >> "${0%/*}/calls.log"
case "$action" in
    --list-directory)
        # Like locale.scs, an archive marked `.noroot` cannot list its root.
        if [ ! -d "$root$path" ] || { [ "$path" = / ] && [ -e "$root/.noroot" ]; }; then
            echo '<error> [system] : readDir returned null!'
            exit 1
        fi
        for file in "$root${path%/}"/*; do
            [ -e "$file" ] || continue
            relative="${file#"$root"}"
            if [ -d "$file" ]; then echo "[D] $relative"; else echo "[F] $relative"; fi
        done
        ;;
    --list-directory-recursive)
        if [ ! -d "$root$path" ] || { [ "$path" = / ] && [ -e "$root/.noroot" ]; }; then
            echo '<error> [system] : readDir returned null!'
            exit 1
        fi
        find "$root${path%/}" -mindepth 1 | sort | while read -r file; do
            relative="${file#"$root"}"
            if [ -d "$file" ]; then echo "[D] $relative"; else echo "[F] $relative"; fi
        done
        ;;
    --extract-directory|--extract-file)
        mkdir -p "$(dirname "$export$path")"
        cp -R "$root$path" "$export$path"
        ;;
    -m)
        case "$path" in
            *broken*) echo '[err] model conversion failed'; exit 0 ;;
        esac
        mkdir -p "$(dirname "$export$path")"
        cp "$root$path.pim" "$export$path.pim"
        cp "$root$path.pit" "$export$path.pit"
        ;;
    *) echo '[err] unknown action'; exit 1 ;;
esac
"#;

const PIM: &str = "Material {\nAlias: \"paint\"\n}\nPiece {\nIndex: 0\nMaterial: 0\nVertexCount: 3\nTriangleCount: 1\nStream {\nAliases: \"_TEXCOORD1\"\n0 ( 0 0 )\n1 ( 1 0 )\n2 ( 0 1 )\n}\nTriangles {\n0 ( 0 1 2 )\n}\n}\nPart {\nName: \"body\"\nPieces: 0\n}\n";
const PIT: &str = "Look {\nName: \"default\"\nMaterial {\nAlias: \"paint\"\nEffect: \"eut2.truckpaint.tsnmapuv\"\n}\n}\nVariant {\nName: \"default\"\nPart {\nName: \"body\"\nAttribute {\nTag: \"visible\"\nValue: ( 1 )\n}\n}\n}\n";

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "converter", CONVERTER);
    std::fs::set_permissions(
        root.join("converter"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    for archive in ["base.scs", "def.scs", "version.scs"] {
        write(root, &format!("game/{archive}"), "fake archive");
    }
    // The stand-in reads the last mounted archive; mirror a complete mounted view here.
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
            &format!(
                "SiiNunit\n{{\naccessory_truck_data : vehicle.{vehicle} {{\ninfo[]: \"Example\"\ninfo[]: \"Truck\"\n}}\n}}\n"
            ),
        );
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/cabin/high.sii"),
            &format!(
                "SiiNunit\n{{\naccessory_cabin_data : high.{vehicle}.cabin {{\nname: \"High\"\n@include \"../common.sui\"\n}}\n}}\n"
            ),
        );
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/common.sui"),
            &format!("model: \"/vehicle/shared/{vehicle}.pmd\"\n"),
        );
        for (extension, content) in [("pmd", "binary"), ("pim", PIM), ("pit", PIT)] {
            write(
                &archive,
                &format!("vehicle/shared/{vehicle}.{extension}"),
                content,
            );
        }
    }
    temp
}

/// Converter calls of every batch run in `root`: (action, path, last mounted archive).
fn calls(root: &Path) -> Vec<[String; 3]> {
    std::fs::read_to_string(root.join("calls.log"))
        .unwrap_or_default()
        .lines()
        .map(|line| {
            let mut fields = line.splitn(3, ' ').map(str::to_owned);
            [(); 3].map(|()| fields.next().unwrap_or_default())
        })
        .collect()
}

fn run(root: &Path, vehicle: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tpv"));
    command
        .current_dir(root)
        .arg("extract")
        .arg(root.join("game"))
        .arg("--converter")
        .arg(root.join("converter"))
        .args(["--package-version", "2.0.0"]);
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
        std::fs::read(other.join("official.ets2.example.good-2.0.0.tpv")).unwrap()
    );
}

#[test]
fn invalid_version_and_excluded_vehicle_publish_nothing() {
    let fixture = fixture();
    let root = fixture.path();
    let result = Command::new(env!("CARGO_BIN_EXE_tpv"))
        .current_dir(root)
        .arg("extract")
        .arg(root.join("game"))
        .arg("--converter")
        .arg(root.join("converter"))
        .args(["--package-version", "not-a-version"])
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
            &format!(
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
    let script = CONVERTER.replace("cp \"$root$path.pit\" \"$export$path.pit\"", ":");
    write(root, "converter", &script);
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
        let pim = PIM.replace("Triangles {", "Stream {\nTag: \"_UV7\"\nAliases: \"_TEXCOORD2\"\n0 ( 0.25 0.25 )\n1 ( 0.75 0.25 )\n2 ( 0.25 0.75 )\n}\nTriangles {");
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
            write(&source, &format!("vehicle/shared/{model}.pmd"), "binary");
            write(&source, &format!("vehicle/shared/{model}.pim"), &pim);
            write(
                &source,
                &format!("vehicle/shared/{model}.pit"),
                &PIT.replace("eut2.truckpaint.tsnmapuv", shader),
            );
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
    write(
        &source,
        "vehicle/shared/example.good.pim",
        &PIM.replace("0 ( 0 0 )", "0 ( -2.125 1.75 )"),
    );
    write(
        &source,
        "vehicle/shared/example.good.pit",
        &PIT.replace(
            "truckpaint.tsnmapuv",
            "truckpaint.tsnmapuv.airbrush.flipflake",
        ),
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
    for (extension, text) in [
        ("pmd", "binary"),
        ("pim", PIM),
        (
            "pit",
            &PIT.replace(
                "eut2.truckpaint.tsnmapuv",
                "eut2.dif.spec.weight.add.env.tsnmapuv.paint.a",
            ),
        ),
    ] {
        write(&source, &format!("vehicle/shared/solid.{extension}"), text);
    }
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
    let layout = |previous: Option<&Path>| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_tpv"));
        command
            .current_dir(root)
            .arg("extract")
            .arg(root.join("game"))
            .arg("--converter")
            .arg(root.join("converter"))
            .args(["--package-version", "2.0.0", "--vehicle", "example.good"]);
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
        let package = tp_pack::check(
            &std::fs::read(batch.join("official.ets2.example.good-2.0.0.tpv")).unwrap(),
        )
        .unwrap();
        (batch, package.paint_job.main[0].texture.layout_version)
    };
    let (first, version) = layout(None);
    assert_eq!(version, 1);
    // Same template: the version is kept.
    let (second, version) = layout(Some(&first));
    assert_eq!(version, 1);
    // A game update moves the UVs: the next version.
    let source = root.join("game/version.scs.files/vehicle/shared/example.good.pim");
    let moved = std::fs::read_to_string(&source)
        .unwrap()
        .replace("1 ( 1 0 )", "1 ( 0.5 0 )");
    std::fs::write(&source, moved).unwrap();
    // The update also changes the archive itself, which invalidates converted models.
    write(&root.join("game"), "version.scs", "updated fake archive");
    let (third, version) = layout(Some(&second));
    assert_eq!(version, 2);
    let (_, version) = layout(Some(&third));
    assert_eq!(version, 2);
}

/// An owned trailer `id` with one chassis and one body, both painted, using
/// the shared painted model; `brand` is its first `info[]` value.
fn trailer(root: &Path, id: &str, brand: &str) {
    let source = root.join("game/version.scs.files");
    let def = format!("def/vehicle/trailer_owned/{id}");
    write(
        &source,
        &format!("{def}/data.sii"),
        &format!(
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
            &format!(
                "SiiNunit\n{{\n{data} : {unit}.{id}.{group} {{\nmodel: \"/vehicle/shared/example.good.pmd\"\n}}\n}}\n"
            ),
        );
    }
    write(
        &source,
        &format!("{def}/paint_job/default.sii"),
        &format!(
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
            &format!(
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
        &format!(
            "SiiNunit\n{{\n{}}}\n",
            ovr(shared, &["body.dry_van", "body.ins", "body.reefer"])
        ),
    );
    // The second paint job paints the reefer from another region.
    write(
        &source,
        &format!("{def}/paint_job/accessory/second.sii"),
        &format!(
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
    for (extension, text) in [
        ("pmd", "binary"),
        ("pim", PIM),
        (
            "pit",
            &PIT.replace(
                "eut2.truckpaint.tsnmapuv",
                "eut2.dif.spec.weight.add.env.tsnmapuv.paint.a",
            ),
        ),
    ] {
        write(&source, &format!("vehicle/shared/solid.{extension}"), text);
    }
    for (unit, model) in [("mudflap", "example.good"), ("guard", "solid")] {
        write(
            &source,
            &format!("{def}/accessory/{unit}.sii"),
            &format!(
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
    // Rebuilt against that batch: unchanged layouts keep version 1.
    let first = batches(root).pop().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_tpv"))
        .current_dir(root)
        .arg("extract")
        .arg(root.join("game"))
        .arg("--converter")
        .arg(root.join("converter"))
        .args([
            "--package-version",
            "2.0.0",
            "--vehicle",
            "scs.box",
            "--previous",
        ])
        .arg(&first)
        .output()
        .unwrap();
    assert!(result.status.success());
    let again = batches(root).pop().unwrap();
    assert_eq!(
        std::fs::read(again.join("official.ets2.scs.box-2.0.0.tpv")).unwrap(),
        std::fs::read(first.join("official.ets2.scs.box-2.0.0.tpv")).unwrap()
    );
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
fn archives_are_listed_once_whole_or_folder_by_folder() {
    let fixture = fixture();
    let root = fixture.path();
    // base.scs lists its root, without /vehicle.
    write(&root.join("game/base.scs.files"), "material/a.mat", "");
    // The model also lives in def.scs, whose root cannot be listed (like locale.scs):
    // both sources stay recorded, last mounted wins.
    let def = root.join("game/def.scs.files");
    write(&def, "vehicle/shared/example.good.pmd", "binary");
    write(&def, ".noroot", "");
    let output = run(root, Some("example.good"));
    assert!(output.status.success(), "{output:?}");
    let listings = calls(root)
        .into_iter()
        .filter(|[action, ..]| action.starts_with("--list-directory"))
        .collect::<Vec<_>>();
    let unique = listings.iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), listings.len(), "{listings:?}");
    for archive in ["base.scs", "def.scs", "version.scs"] {
        let mine = listings
            .iter()
            .filter(|[.., a]| a == archive)
            .map(|[action, path, _]| (action.as_str(), path.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(mine[0], ("--list-directory-recursive", "/"), "{archive}");
        if archive == "def.scs" {
            // No whole listing: its folders are listed one by one, where needed.
            assert!(
                mine.contains(&("--list-directory", "/vehicle/shared")),
                "{mine:?}"
            );
        } else {
            assert_eq!(mine.len(), 1, "{archive}: {mine:?}");
        }
    }
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
fn converted_models_are_reused_until_an_archive_changes() {
    let fixture = fixture();
    let root = fixture.path();
    let conversions = |root: &Path| {
        calls(root)
            .iter()
            .filter(|[action, ..]| action == "-m")
            .count()
    };
    let cached = root.join("target/game-extraction/cache/models/vehicle/shared");
    // example.broken fails its conversion: nothing of it enters the cache.
    assert!(!run(root, None).status.success());
    assert_eq!(conversions(root), 2);
    let first = report(&batches(root)[0]);
    assert_eq!(first["model_cache"]["converted"], 1);
    assert_eq!(first["model_cache"]["reused"], 0);
    assert!(cached.join("example.good.pim").is_file());
    assert!(!cached.join("example.broken.pim").exists());
    assert!(!cached.join("example.broken.pit").exists());

    assert!(run(root, Some("example.good")).status.success());
    assert_eq!(conversions(root), 2, "the second batch converts nothing");
    let package =
        |batch: &Path| std::fs::read(batch.join("official.ets2.example.good-2.0.0.tpv")).unwrap();
    let all = batches(root);
    assert_eq!(package(&all[0]), package(&all[1]));
    let second = report(&all[1]);
    assert_eq!(second["model_cache"]["converted"], 0);
    assert_eq!(second["model_cache"]["reused"], 1);

    // A game update changes an archive: the cache is cleared and the model converted again.
    write(&root.join("game"), "def.scs", "updated fake archive");
    assert!(run(root, Some("example.good")).status.success());
    assert_eq!(conversions(root), 3);
    assert_eq!(report(&batches(root)[2])["model_cache"]["converted"], 1);
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
            .filter(|p| p.file_name().unwrap() != "cache")
            .collect::<Vec<_>>()
    };
    // A built truck, then a skipped colour-only trailer: both batches succeed.
    for vehicle in ["example.good", "scs.box"] {
        assert!(run(root, Some(vehicle)).status.success());
        let report = report(&batches(root).pop().unwrap());
        assert_eq!(report["work_directory_kept"], false);
        assert!(work().is_empty(), "{vehicle}: {:?}", work());
    }
    // A failed vehicle keeps the folder and the converter log its error names.
    assert!(!run(root, Some("example.broken")).status.success());
    let report = report(&batches(root).pop().unwrap());
    assert_eq!(report["work_directory_kept"], true);
    assert_eq!(work().len(), 1);
    let error = report["results"][0]["error"].as_str().unwrap();
    let log = error.rsplit_once("see ").unwrap().1;
    assert!(root.join(log).is_file(), "{error}");
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
        let name = format!("official.ets2.{vehicle}-2.0.0.tpv");
        assert_eq!(
            std::fs::read(all.join(&name)).unwrap(),
            std::fs::read(alone.join(&name)).unwrap(),
            "{vehicle}"
        );
    }
}
