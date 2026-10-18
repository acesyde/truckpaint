//! The Marketplace: downloads the public catalog of vehicle packages, keeps
//! the last good one on disk, and downloads packages to install. The
//! network is only used when the player asks (opening the tab, Refresh,
//! Install), always off the UI thread.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, SystemTime};

use tp_vehicles::catalog::{Catalog, CatalogError, Entry};

/// The official catalog.
pub const OFFICIAL_URL: &str = "https://truckpaint-marketplace.ace-dev.me/catalog.json";
/// Environment variable naming another catalog (wins over Preferences).
pub const ENV: &str = "TRUCKPAINT_CATALOG_URL";
/// Largest catalog read.
const MAX_CATALOG: u64 = 8 * 1024 * 1024;
/// Largest package downloaded.
const MAX_PACKAGE: u64 = 256 * 1024 * 1024;

/// Whether `url` may be a catalog address: HTTPS, or a local `file://`
/// path for testing.
pub fn is_valid_address(url: &str) -> bool {
    let url = url.trim();
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("file://"))
        .is_some_and(|rest| !rest.is_empty())
}

/// The address set by the environment, if any.
pub fn env_address() -> Option<String> {
    std::env::var(ENV).ok().filter(|v| !v.trim().is_empty())
}

/// The catalog address: the environment, then Preferences, then the
/// official one.
pub fn address(preference: Option<&str>) -> String {
    env_address()
        .or_else(|| preference.map(str::to_owned))
        .unwrap_or_else(|| OFFICIAL_URL.to_owned())
}

/// Why something could not be downloaded.
#[derive(Clone, Debug, PartialEq)]
pub enum Failure {
    /// Network, server or file error.
    Unreachable(String),
    /// The response isn't a catalog.
    Invalid(String),
    /// A catalog format this version doesn't know.
    NewerApp,
    /// The package's size or SHA-256 differs from the catalog's.
    Damaged,
}

/// Reads `url` (HTTPS or `file://`), at most `limit` bytes.
fn fetch(url: &str, limit: u64) -> Result<Vec<u8>, Failure> {
    let unreachable = |e: &dyn std::fmt::Display| Failure::Unreachable(e.to_string());
    if let Some(path) = url.strip_prefix("file://") {
        let bytes = std::fs::read(path).map_err(|e| unreachable(&e))?;
        if bytes.len() as u64 > limit {
            return Err(Failure::Unreachable(format!("larger than {limit} bytes")));
        }
        return Ok(bytes);
    }
    if !url.starts_with("https://") {
        return Err(Failure::Unreachable("not an HTTPS address".into()));
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_global(Some(Duration::from_secs(300)))
        .user_agent(format!("TruckPaint/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    let mut response = agent.get(url).call().map_err(|e| unreachable(&e))?;
    response
        .body_mut()
        .with_config()
        .limit(limit)
        .read_to_vec()
        .map_err(|e| unreachable(&e))
}

/// Downloads and reads the catalog at `url`.
pub fn fetch_catalog(url: &str) -> Result<(Vec<u8>, Catalog), Failure> {
    let bytes = fetch(url, MAX_CATALOG)?;
    match Catalog::parse(&bytes) {
        Ok(catalog) => Ok((bytes, catalog)),
        Err(CatalogError::Schema(_)) => Err(Failure::NewerApp),
        Err(CatalogError::Invalid(e)) => Err(Failure::Invalid(e)),
    }
}

/// Downloads `entry`'s package and checks it against the catalog.
pub fn download(entry: &Entry) -> Result<Vec<u8>, Failure> {
    let bytes = fetch(&entry.url, entry.size.min(MAX_PACKAGE))?;
    if bytes.len() as u64 != entry.size || tp_pack::index::sha256(&bytes) != entry.sha256 {
        return Err(Failure::Damaged);
    }
    Ok(bytes)
}

/// The cached catalog and when it was saved.
pub fn read_cache(path: &Path) -> Option<(Catalog, SystemTime)> {
    let bytes = std::fs::read(path).ok()?;
    let saved = std::fs::metadata(path).and_then(|m| m.modified()).ok()?;
    Some((Catalog::parse(&bytes).ok()?, saved))
}

/// What a catalog download gave.
struct Fetched {
    result: Result<Catalog, Failure>,
    /// The cache, when the download failed.
    cached: Option<(Catalog, SystemTime)>,
}

/// Packages being downloaded, one at a time.
pub struct Installing {
    pub total: usize,
    pub done: usize,
    cancel: Arc<AtomicBool>,
    results: Receiver<(Entry, Result<Vec<u8>, Failure>)>,
}

/// The Marketplace's state for the session.
#[derive(Default)]
pub struct Marketplace {
    /// Where the last good catalog is kept.
    pub cache: Option<PathBuf>,
    pub catalog: Option<Catalog>,
    /// The shown catalog comes from the cache, saved then: offline.
    pub offline_since: Option<SystemTime>,
    /// Why the last download failed.
    pub failure: Option<Failure>,
    pub loading: bool,
    /// The catalog was asked for in this session.
    asked: bool,
    job: Option<Receiver<Fetched>>,
    pub installing: Option<Installing>,
}

impl std::fmt::Debug for Marketplace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Marketplace")
            .field("loading", &self.loading)
            .field("failure", &self.failure)
            .finish_non_exhaustive()
    }
}

impl Marketplace {
    /// Downloads the catalog the first time the tab opens in a session.
    pub fn open(&mut self, ctx: &egui::Context, url: &str) {
        if !self.asked {
            self.refresh(ctx, url);
        }
    }

    /// Downloads the catalog again (unless a download is running).
    pub fn refresh(&mut self, ctx: &egui::Context, url: &str) {
        if self.loading {
            return;
        }
        self.asked = true;
        self.loading = true;
        let (tx, rx) = mpsc::channel();
        let (url, cache, ctx) = (url.to_owned(), self.cache.clone(), ctx.clone());
        let spawned = std::thread::Builder::new()
            .name("marketplace-catalog".into())
            .spawn(move || {
                let fetched = match fetch_catalog(&url) {
                    Ok((bytes, catalog)) => {
                        // Only a good catalog replaces the cache.
                        if let Some(path) = &cache
                            && let Err(err) = tp_file::write_atomic(path, &bytes)
                        {
                            tracing::warn!(%err, "cannot cache the catalog");
                        }
                        Fetched {
                            result: Ok(catalog),
                            cached: None,
                        }
                    }
                    Err(failure) => Fetched {
                        result: Err(failure),
                        cached: cache.as_deref().and_then(read_cache),
                    },
                };
                let _ = tx.send(fetched);
                ctx.request_repaint();
            });
        match spawned {
            Ok(_) => self.job = Some(rx),
            Err(err) => {
                self.loading = false;
                self.failure = Some(Failure::Unreachable(err.to_string()));
            }
        }
    }

    /// Picks up a finished catalog download, and returns the packages
    /// downloaded since the last call.
    pub fn poll(&mut self) -> Vec<(Entry, Result<Vec<u8>, Failure>)> {
        if let Some(job) = &self.job
            && let Ok(fetched) = job.try_recv()
        {
            self.job = None;
            self.loading = false;
            match fetched.result {
                Ok(catalog) => {
                    self.catalog = Some(catalog);
                    self.offline_since = None;
                    self.failure = None;
                }
                Err(failure) => {
                    // A catalog already shown stays; else the cache.
                    if (self.catalog.is_none() || self.offline_since.is_some())
                        && let Some((catalog, saved)) = fetched.cached
                    {
                        self.catalog = Some(catalog);
                        self.offline_since = Some(saved);
                    }
                    self.failure = Some(failure);
                }
            }
        }
        let mut out = Vec::new();
        if let Some(job) = &mut self.installing {
            // The worker drops its sender when it stops (done or cancelled).
            let finished = loop {
                match job.results.try_recv() {
                    Ok(result) => out.push(result),
                    Err(mpsc::TryRecvError::Empty) => break false,
                    Err(mpsc::TryRecvError::Disconnected) => break true,
                }
            };
            job.done += out.len();
            if finished {
                self.installing = None;
            }
        }
        out
    }

    /// Whether packages can be installed: a catalog downloaded in this
    /// session and no installation running.
    pub fn can_install(&self) -> bool {
        self.catalog.is_some() && self.offline_since.is_none() && self.installing.is_none()
    }

    /// Downloads `entries`, one after the other, until done or cancelled.
    pub fn install(&mut self, ctx: &egui::Context, entries: Vec<Entry>) {
        if entries.is_empty() || self.installing.is_some() {
            return;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let total = entries.len();
        let (stop, ctx) = (cancel.clone(), ctx.clone());
        let spawned = std::thread::Builder::new()
            .name("marketplace-install".into())
            .spawn(move || {
                for entry in entries {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let result = download(&entry);
                    if tx.send((entry, result)).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
                ctx.request_repaint();
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot download packages");
            return;
        }
        self.installing = Some(Installing {
            total,
            done: 0,
            cancel,
            results: rx,
        });
    }

    /// Stops after the package being downloaded; it is still installed.
    pub fn cancel(&mut self) {
        if let Some(job) = &mut self.installing {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Waits for the running downloads (tests).
    pub fn wait(&mut self) -> Vec<(Entry, Result<Vec<u8>, Failure>)> {
        let mut out = Vec::new();
        while self.loading || self.installing.is_some() {
            out.extend(self.poll());
            std::thread::sleep(Duration::from_millis(5));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tp_vehicles::sample;

    /// A folder with one package and a catalog listing it.
    fn catalog_folder(dir: &Path) -> (String, Vec<u8>) {
        let bytes = sample::package(
            "community.test.truck",
            "Test Truck",
            "1.0.0",
            &sample::truck_textures(),
        );
        std::fs::write(dir.join("truck.tpv"), &bytes).unwrap();
        let base = format!("file://{}", dir.display());
        let files = vec![("truck.tpv".to_owned(), bytes.clone())];
        let options = tp_pack::index::Options {
            base_url: &base,
            default_license: Some("MIT"),
            ..Default::default()
        };
        let catalog = tp_pack::index::build(&files, &options).unwrap();
        std::fs::write(dir.join("catalog.json"), tp_pack::index::to_json(&catalog)).unwrap();
        (format!("{base}/catalog.json"), bytes)
    }

    fn market(dir: &Path) -> Marketplace {
        Marketplace {
            cache: Some(dir.join("cache.json")),
            ..Default::default()
        }
    }

    #[test]
    fn addresses() {
        assert!(is_valid_address("https://example.com/catalog.json"));
        assert!(is_valid_address("file:///tmp/catalog.json"));
        assert!(!is_valid_address("http://example.com/catalog.json"));
        assert!(!is_valid_address("https://"));
        assert_eq!(
            address(Some("https://a/b")),
            env_address().unwrap_or("https://a/b".into())
        );
    }

    #[test]
    fn downloads_caches_and_installs() {
        let dir = tempfile::tempdir().unwrap();
        let (url, bytes) = catalog_folder(dir.path());
        let ctx = egui::Context::default();
        let mut m = market(dir.path());
        m.open(&ctx, &url);
        m.wait();
        let catalog = m.catalog.clone().expect("catalog");
        assert_eq!(m.failure, None);
        assert!(dir.path().join("cache.json").exists());
        m.install(&ctx, catalog.packages.clone());
        let got = m.wait();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].1.as_ref().ok(), Some(&bytes));
    }

    #[test]
    fn checksum_mismatch_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (url, _) = catalog_folder(dir.path());
        let (_, catalog) = fetch_catalog(&url).unwrap();
        let mut entry = catalog.packages[0].clone();
        entry.sha256 = "0".repeat(64);
        assert_eq!(download(&entry), Err(Failure::Damaged));
    }

    #[test]
    fn a_bad_catalog_keeps_the_cache_and_goes_offline() {
        let dir = tempfile::tempdir().unwrap();
        let (url, _) = catalog_folder(dir.path());
        let ctx = egui::Context::default();
        let mut m = market(dir.path());
        m.refresh(&ctx, &url);
        m.wait();
        std::fs::write(dir.path().join("catalog.json"), b"<html>").unwrap();
        let mut later = market(dir.path());
        later.refresh(&ctx, &url);
        later.wait();
        assert!(matches!(later.failure, Some(Failure::Invalid(_))));
        assert!(later.catalog.is_some() && later.offline_since.is_some());
        assert!(!later.can_install());
        assert!(read_cache(&dir.path().join("cache.json")).is_some());
    }

    #[test]
    fn unreachable_without_cache_shows_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = egui::Context::default();
        let mut m = market(dir.path());
        m.refresh(
            &ctx,
            &format!("file://{}/missing.json", dir.path().display()),
        );
        m.wait();
        assert!(matches!(m.failure, Some(Failure::Unreachable(_))));
        assert!(m.catalog.is_none());
    }

    #[test]
    fn cancel_stops_before_the_next_package() {
        let dir = tempfile::tempdir().unwrap();
        let (url, _) = catalog_folder(dir.path());
        let (_, catalog) = fetch_catalog(&url).unwrap();
        let ctx = egui::Context::default();
        let mut m = market(dir.path());
        m.catalog = Some(catalog.clone());
        let many = vec![catalog.packages[0].clone(); 50];
        m.install(&ctx, many);
        m.cancel();
        let got = m.wait();
        assert!(got.len() < 50, "cancelled after {}", got.len());
    }
}
