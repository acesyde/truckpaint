//! The installed vehicle packages: stored unchanged as
//! `<root>/<id>/<version>.tpv`, listed from their manifests.

use std::path::{Path, PathBuf};

use tp_core::ProjectVehicle;
use tp_i18n::tr;
use tp_pack::PackError;
use tp_pack::custom::TemplateError;
use tp_vehicles::{Manifest, Package, PackageError};

/// Id of the built-in sample truck (picked after installing the samples).
pub const SAMPLE_ID: &str = "community.truckpaint.sample_truck";
/// Id of the built-in sample trailer.
pub const SAMPLE_TRAILER_ID: &str = "community.truckpaint.sample_trailer";

/// A built-in sample vehicle: its file name and bytes (newest version).
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub file: &'static str,
    pub bytes: &'static [u8],
}

/// The built-in sample vehicles, offered while the library is empty.
pub const SAMPLES: [Sample; 2] = [
    Sample {
        file: "community.truckpaint.sample_truck-1.1.0.tpv",
        bytes: include_bytes!(
            "../../../examples/vehicles/community.truckpaint.sample_truck-1.1.0.tpv"
        ),
    },
    Sample {
        file: "community.truckpaint.sample_trailer-1.0.0.tpv",
        bytes: include_bytes!(
            "../../../examples/vehicles/community.truckpaint.sample_trailer-1.0.0.tpv"
        ),
    },
];

/// One installed version of a vehicle.
#[derive(Clone, Debug)]
pub struct InstalledVersion {
    pub manifest: Manifest,
    pub path: PathBuf,
}

/// An installed vehicle and its versions, newest first.
#[derive(Clone, Debug)]
pub struct InstalledVehicle {
    pub id: String,
    pub versions: Vec<InstalledVersion>,
}

impl InstalledVehicle {
    pub fn newest(&self) -> &InstalledVersion {
        &self.versions[0]
    }
}

/// Why a package could not be installed or loaded.
#[derive(Debug)]
pub enum InstallError {
    Package(PackageError),
    Io(std::io::Error),
    /// No library folder (no home directory).
    NoLibrary,
}

impl From<PackageError> for InstallError {
    fn from(e: PackageError) -> Self {
        InstallError::Package(e)
    }
}

impl From<std::io::Error> for InstallError {
    fn from(e: std::io::Error) -> Self {
        InstallError::Io(e)
    }
}

/// Why `file` could not be installed, in the current language.
pub fn install_error_message(err: &InstallError, file: &str) -> String {
    let reason = match err {
        InstallError::NoLibrary => tr("pkg-no-library"),
        InstallError::Io(e) => tr!("pkg-io", reason = e.to_string()),
        InstallError::Package(e) => package_error_reason(e),
    };
    tr!("pkg-install-failed", file = file, reason = reason)
}

/// Why a package is invalid, in the current language ("it has no main
/// texture.").
pub fn package_error_reason(e: &PackageError) -> String {
    match e {
        PackageError::NotAZip => tr("pkg-not-a-zip"),
        PackageError::NoManifest => tr("pkg-no-manifest"),
        PackageError::BadManifest(why) => tr!("pkg-bad-manifest", reason = why.as_str()),
        PackageError::NewerFormat(_) => tr("pkg-newer-format"),
        PackageError::BadId(id) => tr!("pkg-bad-id", id = id.as_str()),
        PackageError::BadGamePath(path) => tr!("pkg-bad-game-path", path = path.as_str()),
        PackageError::NoMainTexture => tr("pkg-no-main-texture"),
        PackageError::DuplicatePart(id) => tr!("pkg-duplicate-part", id = id.as_str()),
        PackageError::MissingGameIds(name) => {
            tr!("pkg-missing-game-ids", texture = name.as_str())
        }
        PackageError::BadGameId(id) => tr!("pkg-bad-game-id", id = id.as_str()),
        PackageError::DuplicateGameId(id) => tr!("pkg-duplicate-game-id", id = id.as_str()),
        PackageError::BadAlsoPaints(name) => {
            tr!("pkg-bad-also-paints", texture = name.as_str())
        }
        PackageError::BadSize { texture, size } => {
            tr!(
                "pkg-bad-size",
                texture = texture.as_str(),
                size = size.as_str()
            )
        }
        PackageError::UnsafePath(path) => tr!("pkg-unsafe-path", path = path.as_str()),
        PackageError::TooLarge => tr("pkg-too-large"),
        PackageError::MissingTemplate { texture, .. } => {
            tr!("pkg-missing-template", texture = texture.as_str())
        }
        PackageError::BadTemplate { texture, .. } => {
            tr!("pkg-bad-template", texture = texture.as_str())
        }
        PackageError::TemplateTooLarge { texture } => {
            tr!("pkg-template-too-large", texture = texture.as_str())
        }
        PackageError::BadChangelog => tr("pkg-bad-changelog"),
        PackageError::BadPreview { path, cause } => {
            tr!(
                "pkg-bad-preview",
                path = path.as_str(),
                cause = cause.as_str()
            )
        }
    }
}

/// Why the custom vehicle `name` could not be built, in the current
/// language.
pub fn pack_error_message(err: &PackError, name: &str) -> String {
    let reason = match err {
        PackError::Package(e) => package_error_reason(e),
        PackError::UnsupportedDds { texture, .. } => {
            tr!("custom-template-unconvertible", texture = texture.as_str())
        }
        other => tr!("pkg-bad-manifest", reason = other.to_string()),
    };
    tr!("custom-build-failed", name = name, reason = reason)
}

/// Why `file` can't be a template, in the current language.
pub fn template_error_message(err: &TemplateError, file: &str) -> String {
    match err {
        TemplateError::NotAnImage => tr!("custom-file-not-image", file = file),
        TemplateError::UnsupportedDds(format) => tr!(
            "custom-file-unsupported-dds",
            file = file,
            format = format.as_str()
        ),
        TemplateError::Damaged => tr!("custom-file-damaged", file = file),
        TemplateError::TooLarge => tr!(
            "custom-file-too-large",
            file = file,
            size = tp_vehicles::MAX_IMAGE_SIDE
        ),
    }
}

/// The installed packages.
#[derive(Debug, Default)]
pub struct VehicleLibrary {
    root: Option<PathBuf>,
    vehicles: Vec<InstalledVehicle>,
}

impl VehicleLibrary {
    /// The library stored in `root` (created on first install).
    pub fn open(root: &Path) -> Self {
        let mut library = Self {
            root: Some(root.to_path_buf()),
            vehicles: Vec::new(),
        };
        library.refresh();
        library
    }

    /// Installed vehicles, sorted by name.
    pub fn vehicles(&self) -> &[InstalledVehicle] {
        &self.vehicles
    }

    pub fn get(&self, id: &str) -> Option<&InstalledVehicle> {
        self.vehicles.iter().find(|v| v.id == id)
    }

    /// Lists the library folder again (unreadable files are skipped).
    pub fn refresh(&mut self) {
        self.vehicles.clear();
        let Some(root) = &self.root else {
            return;
        };
        let Ok(dirs) = std::fs::read_dir(root) else {
            return;
        };
        for dir in dirs.flatten().filter(|d| d.path().is_dir()) {
            let mut versions = Vec::new();
            for file in std::fs::read_dir(dir.path())
                .into_iter()
                .flatten()
                .flatten()
            {
                let path = file.path();
                if path.extension().is_none_or(|e| e != tp_vehicles::EXTENSION) {
                    continue;
                }
                match std::fs::read(&path)
                    .map_err(InstallError::from)
                    .and_then(|b| Package::read_manifest(&b).map_err(InstallError::from))
                {
                    Ok(manifest) => versions.push(InstalledVersion { manifest, path }),
                    Err(err) => tracing::warn!(path = %path.display(), ?err, "skipped package"),
                }
            }
            if versions.is_empty() {
                continue;
            }
            versions.sort_by(|a, b| b.manifest.version.cmp(&a.manifest.version));
            self.vehicles.push(InstalledVehicle {
                id: versions[0].manifest.id.clone(),
                versions,
            });
        }
        self.vehicles
            .sort_by_key(|v| v.newest().manifest.name.to_lowercase());
    }

    /// Validates and installs a package; returns its manifest. The same
    /// version is replaced; other versions are kept.
    pub fn install_bytes(&mut self, bytes: &[u8]) -> Result<Manifest, InstallError> {
        let root = self.root.clone().ok_or(InstallError::NoLibrary)?;
        let package = Package::read(bytes)?;
        let m = package.manifest;
        let dir = root.join(&m.id);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.{}", m.version, tp_vehicles::EXTENSION));
        tp_file::write_atomic(&path, bytes).map_err(|e| match e {
            tp_file::Error::Io(e) => InstallError::Io(e),
            other => InstallError::Io(std::io::Error::other(other.to_string())),
        })?;
        self.refresh();
        Ok(m)
    }

    /// Installs the built-in sample vehicles.
    pub fn install_samples(&mut self) -> Result<Vec<Manifest>, InstallError> {
        SAMPLES
            .iter()
            .map(|s| self.install_bytes(s.bytes))
            .collect()
    }

    pub fn install_file(&mut self, path: &Path) -> Result<Manifest, InstallError> {
        let bytes = std::fs::read(path)?;
        self.install_bytes(&bytes)
    }

    /// Removes one installed version.
    pub fn remove(&mut self, id: &str, version: &semver::Version) -> std::io::Result<()> {
        if let Some(path) = self.path_of(id, version) {
            std::fs::remove_file(&path)?;
            if let Some(dir) = path.parent()
                && std::fs::read_dir(dir).is_ok_and(|mut d| d.next().is_none())
            {
                let _ = std::fs::remove_dir(dir);
            }
        }
        self.refresh();
        Ok(())
    }

    fn path_of(&self, id: &str, version: &semver::Version) -> Option<PathBuf> {
        self.get(id)?
            .versions
            .iter()
            .find(|v| v.manifest.version == *version)
            .map(|v| v.path.clone())
    }

    /// Reads an installed version completely (templates included).
    pub fn load(&self, id: &str, version: &semver::Version) -> Result<Package, InstallError> {
        let path = self
            .path_of(id, version)
            .ok_or_else(|| InstallError::Io(std::io::ErrorKind::NotFound.into()))?;
        Ok(Package::read(&std::fs::read(path)?)?)
    }

    /// The newest installed version newer than the one a project's vehicle
    /// records (textures it no longer has are reported by the update plan).
    pub fn update_for(&self, vehicle: &ProjectVehicle) -> Option<&InstalledVersion> {
        let current = semver::Version::parse(&vehicle.version).ok()?;
        self.get(&vehicle.package_id)?
            .versions
            .iter()
            .find(|v| v.manifest.version > current)
    }

    /// The installed version a project's vehicle records.
    pub fn recorded(&self, vehicle: &ProjectVehicle) -> Option<&InstalledVersion> {
        self.get(&vehicle.package_id)?
            .versions
            .iter()
            .find(|v| v.manifest.version.to_string() == vehicle.version)
    }
}

#[cfg(test)]
mod tests {
    use tp_vehicles::sample;

    use super::*;

    fn pkg(version: &str) -> Vec<u8> {
        sample::package(
            "scs.sample.truck",
            "Sample Truck",
            version,
            &sample::truck_textures(),
        )
    }

    #[test]
    fn install_versions_and_remove() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = VehicleLibrary::open(dir.path());
        assert!(lib.vehicles().is_empty());
        lib.install_bytes(&pkg("1.2.0")).unwrap();
        lib.install_bytes(&pkg("1.2.0")).unwrap();
        lib.install_bytes(&pkg("1.3.0")).unwrap();
        let v = lib.get("scs.sample.truck").unwrap();
        let versions: Vec<String> = v
            .versions
            .iter()
            .map(|v| v.manifest.version.to_string())
            .collect();
        assert_eq!(
            versions,
            ["1.3.0", "1.2.0"],
            "newest first, reinstall replaces"
        );
        // Persisted: a new library sees the same.
        let lib2 = VehicleLibrary::open(dir.path());
        assert_eq!(lib2.get("scs.sample.truck").unwrap().versions.len(), 2);
        let package = lib
            .load("scs.sample.truck", &"1.2.0".parse().unwrap())
            .unwrap();
        assert_eq!(package.templates.len(), 3);
        lib.remove("scs.sample.truck", &"1.3.0".parse().unwrap())
            .unwrap();
        assert_eq!(lib.get("scs.sample.truck").unwrap().versions.len(), 1);
        lib.remove("scs.sample.truck", &"1.2.0".parse().unwrap())
            .unwrap();
        assert!(lib.vehicles().is_empty());
        assert!(!dir.path().join("scs.sample.truck").exists());
    }

    fn example(file: &str) -> Vec<u8> {
        let path = format!(
            "{}/../../examples/vehicles/{file}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(path).unwrap()
    }

    #[test]
    fn sample_vehicles_install() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = VehicleLibrary::open(dir.path());
        let installed: Vec<(String, String)> = lib
            .install_samples()
            .unwrap()
            .iter()
            .map(|m| (m.id.clone(), m.version.to_string()))
            .collect();
        assert_eq!(
            installed,
            [
                (SAMPLE_ID.to_owned(), "1.1.0".to_owned()),
                (SAMPLE_TRAILER_ID.to_owned(), "1.0.0".to_owned())
            ]
        );
        // The embedded bytes are the committed packages.
        for sample in SAMPLES {
            assert_eq!(
                sample.bytes,
                example(sample.file).as_slice(),
                "{}",
                sample.file
            );
        }
        lib.install_bytes(&example("community.truckpaint.sample_truck-1.0.0.tpv"))
            .unwrap();
        let v = lib.get(SAMPLE_ID).unwrap();
        assert_eq!(v.versions.len(), 2);
        assert_eq!(v.newest().manifest.name, "TruckPaint Sample Truck");
        assert_eq!(lib.vehicles().len(), 2);
    }

    #[test]
    fn invalid_packages_are_not_installed_or_listed() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = VehicleLibrary::open(dir.path());
        let err = lib.install_bytes(b"nope").unwrap_err();
        assert!(matches!(err, InstallError::Package(PackageError::NotAZip)));
        assert!(install_error_message(&err, "x.tpv").contains("x.tpv"));
        // A damaged file dropped in the folder is skipped.
        std::fs::create_dir_all(dir.path().join("a.b")).unwrap();
        std::fs::write(dir.path().join("a.b/1.0.0.tpv"), b"junk").unwrap();
        lib.refresh();
        assert!(lib.vehicles().is_empty());
        let mut none = VehicleLibrary::default();
        assert!(matches!(
            none.install_bytes(&pkg("1.0.0")),
            Err(InstallError::NoLibrary)
        ));
    }

    #[test]
    fn updates_need_a_newer_version() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = VehicleLibrary::open(dir.path());
        lib.install_bytes(&pkg("1.2.0")).unwrap();
        let mut vehicle = ProjectVehicle {
            package_id: "scs.sample.truck".into(),
            version: "1.2.0".into(),
            name: "Sample Truck".into(),
            brand: "Sample".into(),
            kind: "truck".into(),
            game: "ets2".into(),
            game_data: None,
        };
        assert!(lib.update_for(&vehicle).is_none());
        lib.install_bytes(&pkg("1.3.0")).unwrap();
        assert_eq!(
            lib.update_for(&vehicle)
                .unwrap()
                .manifest
                .version
                .to_string(),
            "1.3.0"
        );
        assert_eq!(
            lib.recorded(&vehicle).unwrap().manifest.version.to_string(),
            "1.2.0"
        );
        vehicle.version = "1.3.0".into();
        assert!(lib.update_for(&vehicle).is_none());
    }
}
