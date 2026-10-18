//! `tpv index`: builds the Marketplace catalog from a folder of packages.

use std::collections::HashMap;

use sha2::{Digest, Sha256};
use tp_vehicles::catalog::{self, Catalog, Entry};

/// What `tpv index` adds to the packages' own fields.
#[derive(Debug, Default)]
pub struct Options<'a> {
    /// Address of the folder; each entry's url is `<base_url>/<file name>`.
    pub base_url: &'a str,
    /// License of packages that set none.
    pub default_license: Option<&'a str>,
    /// Notice of every package (packages have no notice of their own).
    pub default_notice: Option<&'a str>,
    /// The published catalog: none of its versions may change or disappear.
    pub previous: Option<&'a Catalog>,
}

/// The catalog of `files` (file name, bytes), or every problem found.
pub fn build(files: &[(String, Vec<u8>)], options: &Options) -> Result<Catalog, Vec<String>> {
    let base = options.base_url.trim_end_matches('/');
    let mut problems = Vec::new();
    let mut packages: Vec<Entry> = Vec::new();
    for (name, bytes) in files {
        let manifest = match crate::check(bytes) {
            Ok(m) => m,
            Err(e) => {
                problems.push(format!("{name}: {e}"));
                continue;
            }
        };
        let Some(license) = manifest
            .license
            .as_deref()
            .or(options.default_license)
            .filter(|l| !l.trim().is_empty())
        else {
            problems.push(format!(
                "{name}: no license (set one or pass --default-license)"
            ));
            continue;
        };
        packages.push(Entry {
            id: manifest.id.clone(),
            version: manifest.version.clone(),
            name: manifest.name.clone(),
            brand: manifest.brand.clone(),
            kind: manifest.kind,
            game: manifest.game.id,
            size: bytes.len() as u64,
            sha256: sha256(bytes),
            url: format!("{base}/{name}"),
            license: license.to_owned(),
            min_app: catalog::min_app(manifest.format),
            game_versions: Some(manifest.game.versions.to_string()),
            // shortcut: the preview stays inside the package, upgrade when Pages serves images.
            preview: None,
            authors: manifest.authors.clone(),
            changelog: manifest.changelog.clone(),
            notice: options.default_notice.map(str::to_owned),
        });
    }
    packages.sort_by(|a, b| a.id.cmp(&b.id).then(a.version.cmp(&b.version)));
    for pair in packages.windows(2) {
        if pair[0].id == pair[1].id && pair[0].version == pair[1].version {
            problems.push(format!("{} {} appears twice", pair[0].id, pair[0].version));
        }
    }
    if let Some(previous) = options.previous {
        let now: HashMap<_, _> = packages
            .iter()
            .map(|e| ((&e.id, &e.version), &e.sha256))
            .collect();
        for old in &previous.packages {
            match now.get(&(&old.id, &old.version)) {
                None => problems.push(format!(
                    "{} {} is published and can't be removed",
                    old.id, old.version
                )),
                Some(sha) if **sha != old.sha256 => problems.push(format!(
                    "{} {} is published and can't change: raise its version",
                    old.id, old.version
                )),
                Some(_) => {}
            }
        }
    }
    if problems.is_empty() {
        Ok(Catalog {
            schema: catalog::SCHEMA,
            packages,
        })
    } else {
        Err(problems)
    }
}

/// The catalog as written to `catalog.json`.
pub fn to_json(catalog: &Catalog) -> String {
    let mut out = serde_json::to_string_pretty(catalog).expect("catalog serializes");
    out.push('\n');
    out
}

/// Lowercase hex SHA-256 of `bytes`.
pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tp_vehicles::sample;

    fn package(id: &str, version: &str, license: Option<&str>) -> Vec<u8> {
        let textures = sample::truck_textures();
        let mut m = sample::manifest(id, "Truck", version, &textures);
        match license {
            Some(l) => m["license"] = l.into(),
            None => {
                m.as_object_mut().expect("object").remove("license");
            }
        }
        let files: Vec<(String, Vec<u8>)> = textures
            .iter()
            .map(|t| (format!("templates/{}.png", t.id), sample::png(32)))
            .collect();
        sample::zip(&m, &files)
    }

    fn opts() -> Options<'static> {
        Options {
            base_url: "https://example.com/p/",
            ..Options::default()
        }
    }

    fn files() -> Vec<(String, Vec<u8>)> {
        vec![
            ("b-1.0.0.tpv".into(), package("x.b", "1.0.0", Some("MIT"))),
            ("a-1.0.0.tpv".into(), package("x.a", "1.0.0", Some("MIT"))),
        ]
    }

    #[test]
    fn builds_sorted_entries_with_size_and_hash() {
        let files = files();
        let c = build(&files, &opts()).expect("valid");
        let e = &c.packages[0];
        assert_eq!((e.id.as_str(), e.license.as_str()), ("x.a", "MIT"));
        assert_eq!(e.url, "https://example.com/p/a-1.0.0.tpv");
        assert_eq!(e.size, files[1].1.len() as u64);
        assert_eq!(e.sha256, sha256(&files[1].1));
        assert_eq!(e.sha256.len(), 64);
        assert_eq!(
            to_json(&c),
            to_json(&build(&files, &opts()).expect("valid"))
        );
    }

    #[test]
    fn default_license_and_notice_fill_packages() {
        let files = vec![("a.tpv".into(), package("x.a", "1.0.0", None))];
        let problems = build(&files, &opts()).expect_err("no license");
        assert!(problems[0].contains("no license"), "{problems:?}");
        let options = Options {
            default_license: Some("Personal modding only"),
            default_notice: Some("Belongs to SCS Software"),
            ..opts()
        };
        let e = &build(&files, &options).expect("valid").packages[0];
        assert_eq!(e.license, "Personal modding only");
        assert_eq!(e.notice.as_deref(), Some("Belongs to SCS Software"));
    }

    #[test]
    fn refuses_invalid_and_duplicate_packages() {
        let mut files = files();
        files.push(("copy.tpv".into(), files[0].1.clone()));
        files.push(("bad.tpv".into(), b"not a zip".to_vec()));
        let problems = build(&files, &opts()).expect_err("refused");
        assert!(
            problems.iter().any(|p| p.starts_with("bad.tpv")),
            "{problems:?}"
        );
        assert!(
            problems
                .iter()
                .any(|p| p.contains("x.b 1.0.0 appears twice"))
        );
    }

    #[test]
    fn published_versions_cannot_change_or_disappear() {
        let files = files();
        let mut previous = build(&files, &opts()).expect("valid");
        previous.packages[0].sha256 = "00".into();
        let mut gone = previous.packages[1].clone();
        gone.id = "x.gone".into();
        previous.packages.push(gone);
        let options = Options {
            previous: Some(&previous),
            ..opts()
        };
        let problems = build(&files, &options).expect_err("refused");
        assert!(
            problems
                .iter()
                .any(|p| p.contains("x.a 1.0.0 is published and can't change"))
        );
        assert!(
            problems
                .iter()
                .any(|p| p.contains("x.gone 1.0.0 is published and can't be removed"))
        );
        let newer = vec![("a-1.1.0.tpv".into(), package("x.a", "1.1.0", Some("MIT")))];
        let mut all = files.clone();
        all.extend(newer);
        let same = build(&files, &opts()).expect("valid");
        let options = Options {
            previous: Some(&same),
            ..opts()
        };
        assert_eq!(build(&all, &options).expect("valid").packages.len(), 3);
    }
}
