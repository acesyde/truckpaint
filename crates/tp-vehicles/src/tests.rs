use serde_json::json;

use super::sample::{self, SampleTexture};
use super::*;

fn ok_package() -> Vec<u8> {
    sample::package(
        "scs.sample.truck",
        "Sample Truck",
        "1.2.0",
        &sample::truck_textures(),
    )
}

fn with_manifest(edit: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
    let textures = sample::truck_textures();
    let mut m = sample::manifest("scs.sample.truck", "Sample Truck", "1.2.0", &textures);
    edit(&mut m);
    let files: Vec<(String, Vec<u8>)> = textures
        .iter()
        .map(|t| (format!("templates/{}.png", t.id), sample::png(32)))
        .collect();
    sample::zip(&m, &files)
}

fn err(bytes: &[u8]) -> PackageError {
    Package::read(bytes).expect_err("refused")
}

#[test]
fn reads_a_valid_package() {
    let p = Package::read(&ok_package()).unwrap();
    let m = &p.manifest;
    assert_eq!(m.id, "scs.sample.truck");
    assert_eq!(m.version, semver::Version::new(1, 2, 0));
    assert_eq!(m.kind, Kind::Truck);
    assert_eq!(m.game.id, Game::Ets2);
    assert_eq!(m.game.path, "sample.vehicle");
    assert!(!m.game.alt_uv && !m.game.colour_picker);
    assert!(m.game.versions.matches(&semver::Version::new(1, 53, 0)));
    assert!(!m.game.versions.matches(&semver::Version::new(1, 60, 0)));
    let roles: Vec<(Role, &str)> = m
        .paint_job
        .parts()
        .map(|(r, p)| (r, p.id.as_str()))
        .collect();
    assert_eq!(
        roles,
        [
            (Role::Main, "cabin"),
            (Role::Accessory, "chassis"),
            (Role::Accessory, "accessories")
        ]
    );
    assert_eq!(m.paint_job.position("chassis"), Some(1));
    let cabin = p.template("cabin").unwrap();
    assert_eq!((cabin.kind, cabin.width), (ImageKind::Png, 64.0));
    assert_eq!(Package::read_manifest(&ok_package()).unwrap(), p.manifest);
}

#[test]
fn trucks_with_several_main_textures_and_trailers() {
    let tex = |id, name| SampleTexture {
        id,
        name,
        size: TextureSize::new(4096, 4096),
        layout: 1,
    };
    let truck = sample::package_with(
        "a.truck",
        "Truck",
        "1.0.0",
        "truck",
        &[
            tex("standard", "Standard cab"),
            tex("high_roof", "High roof"),
        ],
        &[tex("chassis", "Chassis")],
    );
    let p = Package::read(&truck).unwrap();
    assert_eq!(p.manifest.paint_job.main.len(), 2);
    assert_eq!(p.manifest.paint_job.main[1].game_ids, ["high_roof"]);
    let trailer = sample::package_with(
        "a.trailer",
        "Trailer",
        "1.0.0",
        "trailer",
        &[tex("base", "Base")],
        &[tex("body", "Body 13.6 m"), tex("mudflaps", "Mudflaps")],
    );
    let p = Package::read(&trailer).unwrap();
    assert_eq!(p.manifest.kind, Kind::Trailer);
    assert!(p.manifest.paint_job.main[0].game_ids.is_empty());
    assert_eq!(p.templates.len(), 3);
}

#[test]
fn unknown_fields_and_optional_data() {
    let bytes = with_manifest(|m| {
        m["mirror"] = json!({ "axis": "x" });
        m["game"]["requires"] = json!([{ "name": "Some mod", "version": ">=2" }]);
        m["game"]["alt_uv"] = json!(true);
        m["paint_job"]["main"][0]["texture"]["mask"] = json!("later");
        // Real accessory ids can be longer than a 12-character token.
        m["paint_job"]["accessories"][0]["game_ids"] = json!(["fenders_a.parlok_small_p"]);
    });
    let p = Package::read(&bytes).unwrap();
    assert_eq!(p.manifest.game.requires[0].name, "Some mod");
    assert!(p.manifest.game.alt_uv);
}

#[test]
fn svg_templates() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="200"><rect width="10" height="10"/></svg>"#;
    let t = [SampleTexture {
        id: "cabin",
        name: "Cabin",
        size: TextureSize::new(4096, 4096),
        layout: 1,
    }];
    let mut m = sample::manifest("a.b", "A", "1.0.0", &t);
    m["paint_job"]["main"][0]["texture"]["template"] = json!("templates/cabin.svg");
    let bytes = sample::zip(&m, &[("templates/cabin.svg".into(), svg.to_vec())]);
    let p = Package::read(&bytes).unwrap();
    let i = p.template("cabin").unwrap();
    assert_eq!((i.kind, i.width, i.height), (ImageKind::Svg, 300.0, 200.0));
}

#[test]
fn invalid_packages_are_refused() {
    assert_eq!(err(b"not a zip"), PackageError::NotAZip);
    assert_eq!(
        err(&sample::zip_without_manifest()),
        PackageError::NoManifest
    );
    for edit in [
        (|m: &mut serde_json::Value| m["name"] = json!(3)) as fn(&mut serde_json::Value),
        |m| {
            m.as_object_mut().unwrap().remove("game");
        },
        |m| {
            m["game"].as_object_mut().unwrap().remove("path");
        },
        |m| m["version"] = json!("1.2"),
        |m| m["game"]["versions"] = json!("about 1.5"),
        |m| m["game"]["id"] = json!("fs22"),
        |m| m["kind"] = json!("boat"),
        |m| {
            m["paint_job"]["accessories"][0]
                .as_object_mut()
                .unwrap()
                .remove("texture");
        },
    ] {
        assert!(matches!(
            err(&with_manifest(edit)),
            PackageError::BadManifest(_)
        ));
    }
    assert_eq!(
        err(&with_manifest(|m| m["format"] = json!(2))),
        PackageError::NewerFormat(2)
    );
    assert_eq!(
        err(&with_manifest(|m| m["id"] = json!("Volvo"))),
        PackageError::BadId("Volvo".into())
    );
    assert_eq!(
        err(&with_manifest(|m| m["game"]["path"] = json!("Scania.R"))),
        PackageError::BadGamePath("Scania.R".into())
    );
    assert_eq!(
        err(&with_manifest(|m| m["paint_job"]["main"] = json!([]))),
        PackageError::NoMainTexture
    );
    assert_eq!(
        err(&with_manifest(|m| {
            m["paint_job"].as_object_mut().unwrap().remove("main");
        })),
        PackageError::NoMainTexture
    );
    assert_eq!(
        err(&with_manifest(
            |m| m["paint_job"]["accessories"][0]["id"] = json!("cabin")
        )),
        PackageError::DuplicatePart("cabin".into())
    );
    assert_eq!(
        err(&with_manifest(
            |m| m["paint_job"]["accessories"][1]["game_ids"] = json!([])
        )),
        PackageError::MissingGameIds("Accessories".into())
    );
    // Two main textures: each says which cabins it is for.
    assert_eq!(
        err(&with_manifest(|m| {
            let mut second = m["paint_job"]["main"][0].clone();
            second["id"] = json!("high_roof");
            second["name"] = json!("High roof");
            m["paint_job"]["main"][0]["game_ids"] = json!(["standard"]);
            m["paint_job"]["main"].as_array_mut().unwrap().push(second);
        })),
        PackageError::MissingGameIds("High roof".into())
    );
    assert_eq!(
        err(&with_manifest(
            |m| m["paint_job"]["accessories"][0]["game_ids"] = json!(["Mirror.painted"])
        )),
        PackageError::BadGameId("Mirror.painted".into())
    );
    assert_eq!(
        err(&with_manifest(
            |m| m["paint_job"]["accessories"][1]["game_ids"] = json!(["chassis.sample"])
        )),
        PackageError::DuplicateGameId("chassis.sample".into())
    );
    // The same id among main textures and among accessories is fine: they
    // are different things in the game.
    assert!(
        Package::read(&with_manifest(|m| {
            m["paint_job"]["main"][0]["game_ids"] = json!(["chassis.sample"]);
        }))
        .is_ok()
    );
    assert_eq!(
        err(&with_manifest(
            |m| m["paint_job"]["main"][0]["texture"]["size"] = json!([3000, 1024])
        )),
        PackageError::BadSize {
            texture: "Cabin".into(),
            size: "[3000, 1024]".into()
        }
    );
    assert_eq!(
        err(&with_manifest(
            |m| m["paint_job"]["accessories"][1]["texture"]["size"] = json!([1024, 32])
        )),
        PackageError::BadSize {
            texture: "Accessories".into(),
            size: "[1024, 32]".into()
        }
    );
    assert_eq!(
        err(&with_manifest(
            |m| m["paint_job"]["accessories"][0]["texture"]["template"] =
                json!("templates/missing.png")
        )),
        PackageError::MissingTemplate {
            texture: "Chassis".into(),
            path: "templates/missing.png".into()
        }
    );
    assert!(matches!(
        err(&with_manifest(
            |m| m["paint_job"]["main"][0]["texture"]["template"] = json!("templates/cabin.jpg")
        )),
        PackageError::MissingTemplate { .. } | PackageError::BadTemplate { .. }
    ));
    assert!(is_valid_game_name("scania.r_2016") && is_valid_game_name("highline"));
    assert!(!is_valid_game_name("") && !is_valid_game_name("a..b") && !is_valid_game_name("a-b"));
}

/// A trailer whose main texture "Body" also paints `also_paints`, with a
/// "Chassis" accessory covering `chassis.sample`.
fn trailer_also_painting(also_paints: serde_json::Value) -> Vec<u8> {
    with_manifest(|m| {
        m["kind"] = json!("trailer");
        m["paint_job"]["main"][0]["name"] = json!("Body");
        m["paint_job"]["main"][0]["also_paints"] = also_paints;
    })
}

#[test]
fn main_texture_that_also_paints_accessories() {
    let p = Package::read(&trailer_also_painting(json!([
        "body.reefer",
        "body.reef_13ct"
    ])))
    .unwrap();
    let job = &p.manifest.paint_job;
    assert_eq!(job.main[0].also_paints, ["body.reefer", "body.reef_13ct"]);
    assert!(job.accessories.iter().all(|a| a.also_paints.is_empty()));
    // Absent: empty.
    let p = Package::read(&ok_package()).unwrap();
    assert!(p.manifest.paint_job.main[0].also_paints.is_empty());
}

#[test]
fn accessory_id_painted_twice() {
    // Also an accessory's game id.
    assert_eq!(
        err(&trailer_also_painting(json!(["chassis.sample"]))),
        PackageError::DuplicateGameId("chassis.sample".into())
    );
    // Twice in also_paints, on one main texture or across two.
    assert_eq!(
        err(&trailer_also_painting(json!([
            "body.reefer",
            "body.reefer"
        ]))),
        PackageError::DuplicateGameId("body.reefer".into())
    );
    assert_eq!(
        err(&with_manifest(|m| {
            let mut second = m["paint_job"]["main"][0].clone();
            second["id"] = json!("high_roof");
            second["game_ids"] = json!(["high_roof"]);
            second["also_paints"] = json!(["body.reefer"]);
            m["paint_job"]["main"][0]["game_ids"] = json!(["standard"]);
            m["paint_job"]["main"][0]["also_paints"] = json!(["body.reefer"]);
            m["paint_job"]["main"].as_array_mut().unwrap().push(second);
        })),
        PackageError::DuplicateGameId("body.reefer".into())
    );
}

#[test]
fn bad_also_paints_refused() {
    assert_eq!(
        err(&with_manifest(
            |m| m["paint_job"]["accessories"][0]["also_paints"] = json!(["body.reefer"])
        )),
        PackageError::BadAlsoPaints("Chassis".into())
    );
    assert_eq!(
        err(&trailer_also_painting(json!(["Body.Reefer"]))),
        PackageError::BadGameId("Body.Reefer".into())
    );
    assert!(matches!(
        err(&trailer_also_painting(json!("body.reefer"))),
        PackageError::BadManifest(_)
    ));
}

#[test]
fn rectangular_texture() {
    let bytes = with_manifest(|m| {
        m["paint_job"]["main"][0]["texture"]["size"] = json!([8192, 1024]);
        m["paint_job"]["accessories"][0]["texture"]["size"] = json!([64, 8192]);
    });
    let p = Package::read(&bytes).unwrap();
    let job = &p.manifest.paint_job;
    assert_eq!(job.main[0].texture.size, TextureSize::new(8192, 1024));
    assert_eq!(job.main[0].texture.size.label(), "8192 × 1024");
    assert_eq!(job.accessories[0].texture.size, TextureSize::new(64, 8192));
    assert_eq!(job.accessories[1].texture.size.label(), "1024²");
}

#[test]
fn single_size_refused() {
    // A single number, as packages wrote before textures had a width and
    // a height.
    for size in [
        json!(4096),
        json!([4096]),
        json!([4096, 4096, 4096]),
        json!("4096"),
    ] {
        let bytes = with_manifest(|m| m["paint_job"]["main"][0]["texture"]["size"] = size.clone());
        assert_eq!(
            err(&bytes),
            PackageError::BadSize {
                texture: "Cabin".into(),
                size: size.to_string()
            }
        );
        assert!(matches!(
            Package::read_manifest(&bytes),
            Err(PackageError::BadSize { .. })
        ));
    }
}

#[test]
fn allowed_sides() {
    let sides: Vec<u32> = TextureSize::allowed_sides().collect();
    assert_eq!(sides, [64, 128, 256, 512, 1024, 2048, 4096, 8192]);
    assert!(TextureSize::new(2048, 256).is_allowed());
    assert!(!TextureSize::new(32, 64).is_allowed());
    assert!(!TextureSize::new(16384, 1024).is_allowed());
    assert!(!TextureSize::new(4000, 1024).is_allowed());
}

#[test]
fn wrong_image_type_and_unsafe_paths() {
    let t = &sample::truck_textures()[..1];
    let m = sample::manifest("a.b", "A", "1.0.0", t);
    // A PNG name holding something else.
    let bad = sample::zip(&m, &[("templates/cabin.png".into(), b"GIF89a".to_vec())]);
    assert!(matches!(err(&bad), PackageError::BadTemplate { .. }));
    // Any entry escaping the package, even unused.
    let evil = sample::zip(
        &m,
        &[
            ("templates/cabin.png".into(), sample::png(8)),
            ("../../evil.png".into(), sample::png(8)),
        ],
    );
    assert_eq!(
        err(&evil),
        PackageError::UnsafePath("../../evil.png".into())
    );
    assert!(!is_safe_path("/etc/passwd") && !is_safe_path("C:/x") && !is_safe_path("a/../b"));
    assert!(is_safe_path("templates/cabin.png"));
}

#[test]
fn oversized_images_and_packages() {
    let t = &sample::truck_textures()[..1];
    let m = sample::manifest("a.b", "A", "1.0.0", t);
    // A PNG header claiming 20000 × 20000 pixels.
    let mut huge = sample::png(8);
    huge[16..20].copy_from_slice(&20_000u32.to_be_bytes());
    huge[20..24].copy_from_slice(&20_000u32.to_be_bytes());
    let bytes = sample::zip(&m, &[("templates/cabin.png".into(), huge)]);
    assert!(matches!(
        err(&bytes),
        PackageError::TemplateTooLarge { .. } | PackageError::BadTemplate { .. }
    ));
    // Declared sizes beyond the limit (a zip bomb) are refused before reading.
    let big = sample::zip_with_declared_size(&m, MAX_UNCOMPRESSED + 1);
    assert_eq!(err(&big), PackageError::TooLarge);
}

#[test]
fn ids() {
    assert!(is_valid_id("scs.volvo.fh16_2012"));
    assert!(is_valid_id("community.jdoe.mighty-hauler"));
    assert!(!is_valid_id("volvo"));
    assert!(!is_valid_id("SCS.volvo"));
    assert!(!is_valid_id("scs..volvo"));
}

#[test]
fn documented_example_manifests_are_valid() {
    let doc = include_str!("../../../website/src/content/docs/en/vehicles/package-format.mdx");
    let mut count = 0;
    for block in doc.split("```json\n").skip(1) {
        let json = &block[..block.find("```").expect("end of example")];
        let manifest: Manifest = serde_json::from_str(json).expect("parses");
        validate(&manifest).expect("valid");
        count += 1;
    }
    assert!(
        count >= 3,
        "a truck with several layouts, one layout, a trailer"
    );
}

#[test]
fn changelog_is_optional_text_of_limited_length() {
    // A manifest without the field reads as before.
    assert_eq!(
        Package::read(&with_manifest(|_| {}))
            .unwrap()
            .manifest
            .changelog,
        None
    );
    let note = "Door layout redrawn.";
    let package = Package::read(&with_manifest(|m| m["changelog"] = json!(note))).unwrap();
    assert_eq!(package.manifest.changelog.as_deref(), Some(note));
    // The limit counts characters, not bytes.
    let longest = "é".repeat(MAX_CHANGELOG);
    assert!(Package::read(&with_manifest(|m| m["changelog"] = json!(longest))).is_ok());
    assert_eq!(
        err(&with_manifest(
            |m| m["changelog"] = json!("é".repeat(MAX_CHANGELOG + 1))
        )),
        PackageError::BadChangelog
    );
    assert!(matches!(
        err(&with_manifest(|m| m["changelog"] = json!(12))),
        PackageError::BadManifest(_)
    ));
}

#[test]
fn diff_lists_layout_size_added_and_removed_parts() {
    let manifest = |textures: &[SampleTexture]| -> Manifest {
        serde_json::from_value(sample::manifest(
            "scs.sample.truck",
            "Sample Truck",
            "1.0.0",
            textures,
        ))
        .unwrap()
    };
    let old = sample::truck_textures();
    assert!(diff(&manifest(&old), &manifest(&old)).is_empty());

    let mut new = old.clone();
    new[0].layout = 2; // main texture
    new[1].size = TextureSize::new(4096, 4096); // accessory
    new[2].layout = 3;
    new.push(SampleTexture {
        id: "side_skirts",
        name: "Side skirts",
        size: TextureSize::new(1024, 512),
        layout: 1,
    });
    let d = diff(&manifest(&old), &manifest(&new));
    assert_eq!(d.layout_changed, ["Cabin", "Accessories"]);
    assert_eq!(d.resized, ["Chassis"]);
    assert_eq!(d.added, ["Side skirts"]);
    assert!(d.removed.is_empty());

    let d = diff(&manifest(&new), &manifest(&old));
    assert_eq!(d.removed, ["Side skirts"]);
    assert!(d.added.is_empty());
}

fn with_preview(mesh: serde_json::Value, named: bool) -> Vec<u8> {
    let textures = sample::truck_textures();
    let mut m = sample::manifest("scs.sample.truck", "Sample Truck", "1.2.0", &textures);
    m["paint_job"]["main"][0]["preview_mesh"] = json!("preview/cabin.json");
    let mut files = sample::templates(&textures);
    if named {
        files.push(("preview/cabin.json".into(), mesh.to_string().into_bytes()));
    }
    sample::zip(&m, &files)
}

fn mesh(edit: impl FnOnce(&mut serde_json::Value)) -> serde_json::Value {
    let mut m = json!({"format": 1, "pieces": [{
        "texture": "cabin",
        "positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0]],
        "uv": [[0, 0], [1, 0], [0, 1]],
        "triangles": [[0, 1, 2]],
    }]});
    edit(&mut m);
    m
}

#[test]
fn reads_a_preview_mesh_and_accepts_packages_without_one() {
    let p = Package::read(&with_preview(mesh(|_| {}), true)).unwrap();
    assert_eq!(p.preview("cabin").unwrap().pieces[0].triangles, [[0, 1, 2]]);
    assert!(Package::read(&ok_package()).unwrap().previews.is_empty());
    let bytes = with_preview(mesh(|_| {}), true);
    assert_eq!(
        Package::read_preview(&bytes, "cabin").unwrap(),
        p.preview("cabin").cloned()
    );
    assert_eq!(Package::read_preview(&bytes, "chassis").unwrap(), None);
    assert_eq!(Package::read_preview(&ok_package(), "cabin").unwrap(), None);
}

#[test]
fn broken_preview_meshes_are_refused_naming_the_file() {
    let broken = [
        with_preview(mesh(|_| {}), false),
        with_preview(mesh(|m| m["pieces"][0]["texture"] = json!("nope")), true),
        with_preview(mesh(|m| m["pieces"][0]["uv"] = json!([[0, 0]])), true),
        with_preview(
            mesh(|m| m["pieces"][0]["triangles"] = json!([[0, 1, 120]])),
            true,
        ),
        with_preview(mesh(|m| m["format"] = json!(2)), true),
        with_preview(
            mesh(|m| m["pieces"][0]["positions"][0][0] = json!("x")),
            true,
        ),
        with_preview(
            mesh(|m| m["pieces"][0]["positions"][0][0] = json!(1e300)),
            true,
        ),
    ];
    for bytes in broken {
        match err(&bytes) {
            PackageError::BadPreview { path, .. } => assert_eq!(path, "preview/cabin.json"),
            other => panic!("{other:?}"),
        }
    }
}

fn piece(x0: f64, x1: f64) -> serde_json::Value {
    json!({
        "texture": "cabin",
        "positions": [[x0, 0, 0], [x1, 0, 0], [x0, 1, 0]],
        "uv": [[0, 0], [1, 0], [0, 1]],
        "triangles": [[0, 1, 2]],
    })
}

fn locator(x: f64) -> serde_json::Value {
    json!({"position": [x, 0, 0], "rotation": [1, 0, 0, 0], "scale": 1})
}

/// A cab with a door slot on the cabin and a bumper slot on the chassis.
fn slotted(edit: impl FnOnce(&mut serde_json::Value)) -> serde_json::Value {
    mesh(|m| {
        m["locators"] = json!({"cab_door": [locator(1.075)]});
        m["slots"] = json!([
            {"id": "chassis", "options": [
                {"id": "4x2", "name": "4x2", "default": true, "pieces": [piece(0.0, 1.0)],
                 "locators": {"f_bumper": [locator(0.5)]}},
            ]},
            {"id": "cab_door", "parent": "cabin", "options": [
                {"id": "std", "name": "Standard", "default": true, "pieces": [piece(-2.32, 0.17)]},
                {"id": "alt", "name": "Alternate", "pieces": [piece(-2.0, 0.0)]},
            ]},
            {"id": "f_bumper", "parent": "chassis", "options": [
                {"id": "std", "name": "Standard", "pieces": [piece(0.0, 1.0)]},
            ]},
        ]);
        edit(m);
    })
}

#[test]
fn reads_slots_and_refuses_each_broken_one() {
    let p = Package::read(&with_preview(slotted(|_| {}), true)).unwrap();
    let mesh = p.preview("cabin").unwrap();
    assert_eq!(mesh.slots.len(), 3);
    assert_eq!(mesh.slots[0].parent, None);
    assert_eq!(mesh.slots[1].parent, Some(SlotParent::Cabin));
    assert!(mesh.slots[1].options[0].default && !mesh.slots[1].options[1].default);
    // Only a slot with no recorded parent is drawn as it is; the others need the right locators.
    let door = mesh.slot("cab_door").unwrap();
    assert_eq!(mesh.mounts(door, None).len(), 1);
    let chassis = &mesh.slot("chassis").unwrap().options[0];
    let bumper = mesh.slot("f_bumper").unwrap();
    assert_eq!(mesh.mounts(bumper, Some(chassis))[0].position[0], 0.5);
    assert!(mesh.mounts(bumper, None).is_empty());
    // Two locators of one name place two copies.
    let twin = slotted(|m| m["locators"]["cab_door"] = json!([locator(1.0), locator(-1.0)]));
    let twin = Package::read(&with_preview(twin, true)).unwrap();
    let mesh = twin.preview("cabin").unwrap();
    assert_eq!(mesh.mounts(mesh.slot("cab_door").unwrap(), None).len(), 2);

    let broken = [
        slotted(|m| m["slots"][1]["id"] = json!("chassis")),
        slotted(|m| m["slots"][1]["options"][1]["id"] = json!("std")),
        slotted(|m| m["slots"][1]["options"][1]["default"] = json!(true)),
        slotted(|m| m["locators"] = json!({})),
        slotted(|m| drop(m["slots"].as_array_mut().unwrap().remove(0))),
        slotted(|m| m["slots"][1]["options"][0]["pieces"][0]["texture"] = json!("nope")),
        slotted(|m| m["locators"]["cab_door"][0]["scale"] = json!(1e300)),
    ];
    for mesh in broken {
        match err(&with_preview(mesh, true)) {
            PackageError::BadPreview { path, .. } => assert_eq!(path, "preview/cabin.json"),
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn a_locator_places_the_model_of_its_accessory() {
    // The DAF XD door spans x -2.32 to 0.17 in its model and sits at x 1.075 in the cab.
    let at = PreviewLocator {
        position: [1.075, 0.0, 0.0],
        ..PreviewLocator::IDENTITY
    };
    let x = [-2.32, 0.17].map(|x| at.place([x, 0.0, 0.0])[0]);
    assert!(
        (x[0] + 1.245).abs() < 1e-5 && (x[1] - 1.245).abs() < 1e-5,
        "{x:?}"
    );
    // The left chassis locator of a trailer, as stored: a tool box (long in x, thin in +z) must
    // lie along the vehicle and stick out to the left.
    let half = std::f32::consts::FRAC_1_SQRT_2;
    let left = PreviewLocator {
        position: [-1.285, 0.6, -0.7],
        rotation: [half, 0.0, half, 0.0],
        scale: 1.0,
    };
    let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-5);
    assert!(close(left.place([0.0, 0.0, 0.5]), [-1.785, 0.6, -0.7]));
    assert!(close(left.place([1.0, 0.0, 0.0]), [-1.285, 0.6, 0.3]));
    // A rear locator turns the model around.
    let rear = PreviewLocator {
        rotation: [0.0, 0.0, 1.0, 0.0],
        scale: 2.0,
        ..PreviewLocator::IDENTITY
    };
    assert!(close(rear.place([1.0, 1.0, 1.0]), [-2.0, 2.0, -2.0]));
}
