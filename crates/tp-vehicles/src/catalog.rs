//! The Marketplace catalog (`catalog.json`): one entry per published version
//! of a vehicle package. Built by `tpv index`, read by the app.

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{Game, Kind};

/// Highest catalog format this version understands.
pub const SCHEMA: u32 = 1;

/// The catalog file (unknown fields are ignored).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Catalog {
    pub schema: u32,
    pub packages: Vec<Entry>,
}

/// One published version of a vehicle.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Entry {
    pub id: String,
    pub version: Version,
    pub name: String,
    pub brand: String,
    pub kind: Kind,
    pub game: Game,
    /// Size of the `.tpv`, in bytes.
    pub size: u64,
    /// SHA-256 of the `.tpv`, lowercase hex.
    pub sha256: String,
    /// HTTPS address of the `.tpv`.
    pub url: String,
    pub license: String,
    /// Oldest TruckPaint that can install it.
    pub min_app: Version,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_versions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
    /// A short rights statement shown with the vehicle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
}

/// Why a catalog could not be read.
#[derive(Debug, PartialEq)]
pub enum CatalogError {
    /// Not a catalog: invalid JSON or missing fields.
    Invalid(String),
    /// A catalog format this version doesn't know.
    Schema(u32),
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CatalogError::Invalid(e) => write!(f, "not a valid catalog: {e}"),
            CatalogError::Schema(n) => write!(f, "unknown catalog format {n}"),
        }
    }
}

/// The oldest TruckPaint that reads packages of format `format`.
pub fn min_app(_format: u32) -> Version {
    Version::new(0, 1, 0)
}

impl Catalog {
    /// Reads a catalog; the format is checked before the entries.
    pub fn parse(bytes: &[u8]) -> Result<Catalog, CatalogError> {
        #[derive(Deserialize)]
        struct Head {
            schema: u32,
        }
        let head: Head =
            serde_json::from_slice(bytes).map_err(|e| CatalogError::Invalid(e.to_string()))?;
        if head.schema != SCHEMA {
            return Err(CatalogError::Schema(head.schema));
        }
        serde_json::from_slice(bytes).map_err(|e| CatalogError::Invalid(e.to_string()))
    }

    /// The vehicle ids, each once, in catalog order.
    pub fn ids(&self) -> Vec<&str> {
        let mut seen = std::collections::HashSet::new();
        self.packages
            .iter()
            .map(|e| e.id.as_str())
            .filter(|id| seen.insert(*id))
            .collect()
    }

    /// Every version of `id`, newest first.
    pub fn versions(&self, id: &str) -> Vec<&Entry> {
        let mut out: Vec<&Entry> = self.packages.iter().filter(|e| e.id == id).collect();
        out.sort_by(|a, b| b.version.cmp(&a.version));
        out
    }

    /// The newest version of `id` that `app` can install.
    pub fn newest_compatible(&self, id: &str, app: &Version) -> Option<&Entry> {
        self.versions(id).into_iter().find(|e| e.min_app <= *app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, version: &str, min_app: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id, "version": version, "name": "FH", "brand": "Volvo",
            "kind": "truck", "game": "ets2", "size": 10, "sha256": "ab",
            "url": "https://example.com/a.tpv", "license": "MIT", "min_app": min_app,
            "future_field": true
        })
    }

    fn catalog(entries: Vec<serde_json::Value>) -> Catalog {
        let json = serde_json::json!({"schema": 1, "packages": entries});
        Catalog::parse(json.to_string().as_bytes()).expect("valid")
    }

    #[test]
    fn parses_and_ignores_unknown_fields() {
        let c = catalog(vec![entry("a.b", "1.0.0", "0.1.0")]);
        assert_eq!(c.packages[0].game, Game::Ets2);
        assert_eq!(c.packages[0].notice, None);
    }

    #[test]
    fn refuses_unknown_schema_and_garbage() {
        let json = br#"{"schema": 2, "packages": [{"anything": 1}]}"#;
        assert_eq!(Catalog::parse(json), Err(CatalogError::Schema(2)));
        assert!(matches!(
            Catalog::parse(b"<html>"),
            Err(CatalogError::Invalid(_))
        ));
    }

    #[test]
    fn newest_compatible_skips_versions_too_new_for_the_app() {
        let c = catalog(vec![
            entry("a.b", "1.0.0", "0.1.0"),
            entry("a.b", "1.10.0", "9.0.0"),
            entry("a.b", "1.2.0", "0.1.0"),
            entry("c.d", "1.0.0", "9.0.0"),
        ]);
        let app = Version::new(0, 1, 0);
        assert_eq!(c.ids(), ["a.b", "c.d"]);
        assert_eq!(
            c.newest_compatible("a.b", &app)
                .map(|e| e.version.to_string()),
            Some("1.2.0".into())
        );
        assert_eq!(c.versions("a.b")[0].version, Version::new(1, 10, 0));
        assert!(c.newest_compatible("c.d", &app).is_none());
    }
}
