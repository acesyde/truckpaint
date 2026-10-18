//! The `tpv` command: exit codes and written files.

use std::path::Path;
use std::process::{Command, Output};

use tp_vehicles::sample;

fn tpv(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tpv"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run tpv")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A folder holding a valid sample manifest and its templates.
fn source(dir: &Path) {
    let manifest = sample::manifest(
        "community.jdoe.my_truck",
        "My Truck",
        "1.0.0",
        &sample::truck_textures(),
    );
    std::fs::create_dir_all(dir.join("src/templates")).unwrap();
    std::fs::write(dir.join("src/vehicle.json"), manifest.to_string()).unwrap();
    for t in sample::truck_textures() {
        std::fs::write(
            dir.join(format!("src/templates/{}.png", t.id)),
            sample::png(16),
        )
        .unwrap();
    }
}

#[test]
fn pack_writes_the_default_file_and_check_reads_it() {
    let dir = tempfile::tempdir().unwrap();
    source(dir.path());
    let out = tpv(dir.path(), &["pack", "src"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let file = "community.jdoe.my_truck-1.0.0.tpv";
    assert!(dir.path().join(file).is_file());
    assert!(text(&out.stdout).contains("packed    community.jdoe.my_truck-1.0.0.tpv"));

    let out = tpv(dir.path(), &["check", file]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("community.jdoe.my_truck 1.0.0"));

    let out = tpv(dir.path(), &["pack", "src", "-o", "custom.tpv"]);
    assert!(out.status.success());
    assert!(dir.path().join("custom.tpv").is_file());
}

#[test]
fn check_describes_a_trailer() {
    let dir = tempfile::tempdir().unwrap();
    let tex = |id, name, size| sample::SampleTexture {
        id,
        name,
        size: tp_vehicles::TextureSize::new(size, size),
        layout: 1,
    };
    let bytes = sample::package_with(
        "community.jdoe.trailer",
        "Trailer",
        "1.0.0",
        "trailer",
        &[tex("base", "Base", 2048)],
        &[
            tex("body", "Body 13.6 m", 4096),
            tex("mudflaps", "Mudflaps", 512),
        ],
    );
    std::fs::write(dir.path().join("t.tpv"), bytes).unwrap();
    let out = tpv(dir.path(), &["check", "t.tpv"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("Trailer (Sample trailer, ets2)"),
        "{stdout}"
    );
    assert!(stdout.contains("base (Base): 2048²"), "{stdout}");
    assert!(stdout.contains("[whole vehicle]"), "{stdout}");
    assert!(
        stdout.contains("body (Body 13.6 m): 4096², layout 1, templates/body.png [body.sample]"),
        "{stdout}"
    );
}

#[test]
fn check_prints_rectangular_sizes() {
    let dir = tempfile::tempdir().unwrap();
    let base = sample::SampleTexture {
        id: "base",
        name: "Base",
        size: tp_vehicles::TextureSize::new(8192, 1024),
        layout: 1,
    };
    let bytes = sample::package_with(
        "community.jdoe.liner",
        "Liner",
        "1.0.0",
        "trailer",
        &[base],
        &[],
    );
    std::fs::write(dir.path().join("t.tpv"), bytes).unwrap();
    let out = tpv(dir.path(), &["check", "t.tpv"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("base (Base): 8192 × 1024, layout 1"),
        "{stdout}"
    );
}

#[test]
fn pack_refuses_a_single_size() {
    let dir = tempfile::tempdir().unwrap();
    source(dir.path());
    let path = dir.path().join("src/vehicle.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    manifest["paint_job"]["main"][0]["texture"]["size"] = 4096.into();
    std::fs::write(&path, manifest.to_string()).unwrap();
    let out = tpv(dir.path(), &["pack", "src", "-o", "out.tpv"]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = text(&out.stderr);
    assert!(
        stderr.contains("Cabin texture size 4096 is invalid: it must be a width and a height"),
        "{stderr}"
    );
    assert!(!dir.path().join("out.tpv").exists());
}

/// Every example manifest of the package format page (website/…/package-format.mdx) packs and
/// passes `tpv check`.
#[test]
fn documented_examples_pass_check() {
    let doc = include_str!("../../../website/src/content/docs/en/vehicles/package-format.mdx");
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"/>"#;
    let mut count = 0;
    for (i, block) in doc.split("```json\n").skip(1).enumerate() {
        let json = &block[..block.find("```").expect("end of example")];
        let manifest: serde_json::Value = serde_json::from_str(json).expect("parses");
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(src.join("templates")).unwrap();
        std::fs::write(src.join("vehicle.json"), json).unwrap();
        let templates = ["main", "accessories"]
            .iter()
            .flat_map(|list| manifest["paint_job"][list].as_array().into_iter().flatten())
            .map(|part| part["texture"]["template"].as_str().unwrap())
            .chain(manifest["preview"].as_str());
        for template in templates {
            let bytes = if template.ends_with(".svg") {
                svg.to_vec()
            } else {
                sample::png(16)
            };
            std::fs::write(src.join(template), bytes).unwrap();
        }
        let out = tpv(dir.path(), &["pack", "src", "-o", "doc.tpv"]);
        assert!(out.status.success(), "example {i}: {}", text(&out.stderr));
        let out = tpv(dir.path(), &["check", "doc.tpv"]);
        assert!(out.status.success(), "example {i}: {}", text(&out.stderr));
        count += 1;
    }
    assert!(
        count >= 3,
        "a truck with several layouts, one layout, a trailer"
    );
}

#[test]
fn failed_pack_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    source(dir.path());
    std::fs::remove_file(dir.path().join("src/templates/cabin.png")).unwrap();
    let out = tpv(dir.path(), &["pack", "src", "-o", "out.tpv"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("Cabin template is missing"),
        "{}",
        text(&out.stderr)
    );
    assert!(!dir.path().join("out.tpv").exists());
}

#[test]
fn check_refuses_a_corrupt_file_and_usage_errors_fail() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("bad.tpv"), b"junk").unwrap();
    let out = tpv(dir.path(), &["check", "bad.tpv"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("not a ZIP file"));
    assert_eq!(tpv(dir.path(), &[]).status.code(), Some(1));
    assert_eq!(tpv(dir.path(), &["pack"]).status.code(), Some(1));
    let help = tpv(dir.path(), &["--help"]);
    assert!(help.status.success());
    assert!(text(&help.stdout).contains("Usage:"));
    assert!(text(&help.stdout).contains("[width, height]"));
}

#[test]
fn changelog_is_packed_checked_and_limited() {
    let dir = tempfile::tempdir().unwrap();
    source(dir.path());
    let manifest = dir.path().join("src/vehicle.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    value["changelog"] = serde_json::json!("Door layout redrawn.");
    std::fs::write(&manifest, value.to_string()).unwrap();
    for out in ["a.tpv", "b.tpv"] {
        let packed = tpv(dir.path(), &["pack", "src", "-o", out]);
        assert!(packed.status.success(), "{}", text(&packed.stderr));
    }
    // Repacking the same folder gives the same bytes.
    assert_eq!(
        std::fs::read(dir.path().join("a.tpv")).unwrap(),
        std::fs::read(dir.path().join("b.tpv")).unwrap()
    );
    let out = tpv(dir.path(), &["check", "a.tpv"]);
    assert!(text(&out.stdout).contains("changelog: Door layout redrawn."));

    value["changelog"] = serde_json::json!("x".repeat(tp_vehicles::MAX_CHANGELOG + 1));
    std::fs::write(&manifest, value.to_string()).unwrap();
    let out = tpv(dir.path(), &["pack", "src", "-o", "long.tpv"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("changelog is longer than 1000"),
        "{}",
        text(&out.stderr)
    );
    assert!(!dir.path().join("long.tpv").exists());
}

#[test]
fn index_writes_the_same_catalog_twice_and_check_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/vehicles");
    for entry in std::fs::read_dir(&samples).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "tpv") {
            std::fs::copy(&path, dir.path().join(path.file_name().unwrap())).unwrap();
        }
    }
    let args = ["index", ".", "--base-url", "https://example.com/p"];
    assert!(tpv(dir.path(), &args).status.success());
    let first = std::fs::read(dir.path().join("catalog.json")).unwrap();
    assert!(tpv(dir.path(), &args).status.success());
    assert_eq!(
        first,
        std::fs::read(dir.path().join("catalog.json")).unwrap()
    );
    let catalog = tp_vehicles::catalog::Catalog::parse(&first).unwrap();
    assert_eq!(catalog.packages.len(), 3);

    std::fs::remove_file(dir.path().join("catalog.json")).unwrap();
    let out = tpv(dir.path(), &[&args[..], &["--check"]].concat());
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!dir.path().join("catalog.json").exists());

    std::fs::write(dir.path().join("bad.tpv"), b"not a zip").unwrap();
    let out = tpv(dir.path(), &[&args[..], &["--check"]].concat());
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("bad.tpv"));
}
