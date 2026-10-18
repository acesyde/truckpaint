//! User preferences: versioned, stored as RON in the config directory, written
//! atomically, and robust to missing or corrupted files.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tp_ui::ThemeSettings;

use crate::layout::WorkspaceLayout;

/// Current preferences schema version.
pub const PREFS_VERSION: u32 = 1;
const FILE_NAME: &str = "prefs.ron";
/// Maximum number of entries kept in the recent projects list.
pub const MAX_RECENT: usize = 12;
/// Maximum number of recent colors kept.
pub const MAX_RECENT_COLORS: usize = 12;

/// A previously opened project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: PathBuf,
    /// Seconds since the Unix epoch.
    pub last_opened: u64,
}

/// Default grid spacing, in texture pixels.
pub const DEFAULT_GRID_SPACING: f64 = 64.0;
/// Allowed grid spacing, in texture pixels.
pub const GRID_SPACING_RANGE: std::ops::RangeInclusive<f64> = 4.0..=1024.0;

/// Grid, guides and snapping settings (editor state, not part of projects).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewAids {
    pub grid: bool,
    pub guides: bool,
    pub snapping: bool,
    /// Grid spacing in texture pixels.
    pub grid_spacing: f64,
}

impl Default for ViewAids {
    fn default() -> Self {
        Self {
            grid: false,
            guides: true,
            snapping: true,
            grid_spacing: DEFAULT_GRID_SPACING,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub version: u32,
    pub ui_scale: f32,
    pub text_scale: f32,
    pub layout: WorkspaceLayout,
    pub recent: Vec<RecentProject>,
    /// Recently applied colors as RGBA, most recent first.
    pub recent_colors: Vec<[u8; 4]>,
    pub view_aids: ViewAids,
    /// Interface language chosen by the user (`en`, `fr`, `es`, `de`), or
    /// `None` to follow the system.
    pub language: Option<String>,
    /// Use the system title bar instead of the drawn one (Windows and
    /// Linux; Reset to defaults keeps it).
    pub system_title_bar: bool,
    /// The local MCP server is on (Reset to defaults keeps it).
    pub mcp_enabled: bool,
    /// The port of the local MCP server (Reset to defaults keeps it).
    pub mcp_port: u16,
    /// Address of the Marketplace catalog, or `None` for the official one.
    pub catalog_url: Option<String>,
}

/// Ports the MCP server may use (no privileged port).
pub const MCP_PORT_RANGE: std::ops::RangeInclusive<u16> = 1024..=65535;

impl Default for Prefs {
    fn default() -> Self {
        Self {
            version: PREFS_VERSION,
            ui_scale: 1.0,
            text_scale: 1.0,
            layout: WorkspaceLayout::default(),
            recent: Vec::new(),
            recent_colors: Vec::new(),
            view_aids: ViewAids::default(),
            language: None,
            system_title_bar: false,
            mcp_enabled: false,
            mcp_port: tp_mcp::DEFAULT_PORT,
            catalog_url: None,
        }
    }
}

impl Prefs {
    /// The language chosen by the user, if any (unknown codes are ignored).
    pub fn language(&self) -> Option<tp_i18n::Language> {
        self.language
            .as_deref()
            .and_then(tp_i18n::Language::from_code)
    }

    pub fn set_language(&mut self, language: Option<tp_i18n::Language>) {
        self.language = language.map(|l| l.code().to_owned());
    }

    pub fn theme(&self) -> ThemeSettings {
        ThemeSettings {
            ui_scale: self.ui_scale,
            text_scale: self.text_scale,
        }
        .clamped()
    }

    /// Moves `rgba` to the front of the recent colors (no duplicates, capped).
    pub fn push_recent_color(&mut self, rgba: [u8; 4]) {
        self.recent_colors.retain(|c| *c != rgba);
        self.recent_colors.insert(0, rgba);
        self.recent_colors.truncate(MAX_RECENT_COLORS);
    }

    /// Moves a project file to the top of the recent projects (no
    /// duplicates, capped).
    pub fn push_recent(&mut self, name: &str, path: &Path) {
        self.recent.retain(|r| r.path != path);
        self.recent.insert(
            0,
            RecentProject {
                name: name.to_owned(),
                path: path.to_path_buf(),
                last_opened: now_unix(),
            },
        );
        self.recent.truncate(MAX_RECENT);
    }

    /// Restores UI scale and text size to 100%.
    /// "Reset to defaults" of the Preferences dialog: scaling and grid
    /// spacing.
    pub fn reset_scaling(&mut self) {
        self.ui_scale = 1.0;
        self.text_scale = 1.0;
        self.view_aids.grid_spacing = DEFAULT_GRID_SPACING;
    }

    /// Repairs values coming from disk.
    fn sanitized(mut self) -> Self {
        let theme = self.theme();
        self.ui_scale = theme.ui_scale;
        self.text_scale = theme.text_scale;
        self.layout = self.layout.sanitized();
        self.recent.truncate(MAX_RECENT);
        self.recent_colors.truncate(MAX_RECENT_COLORS);
        if !MCP_PORT_RANGE.contains(&self.mcp_port) {
            self.mcp_port = tp_mcp::DEFAULT_PORT;
        }
        let spacing = self.view_aids.grid_spacing;
        self.view_aids.grid_spacing = if spacing.is_finite() {
            spacing.clamp(*GRID_SPACING_RANGE.start(), *GRID_SPACING_RANGE.end())
        } else {
            DEFAULT_GRID_SPACING
        };
        self
    }
}

/// Why loading fell back to defaults (for logging).
#[derive(Debug)]
pub enum LoadIssue {
    Unreadable(io::Error),
    Invalid(String),
    UnknownVersion(u32),
}

/// Result of [`PrefsStore::load`].
pub struct Loaded {
    pub prefs: Prefs,
    /// Set when the file existed but could not be used.
    pub issue: Option<LoadIssue>,
    /// Where the unusable file was moved to.
    pub backup: Option<PathBuf>,
}

/// Reads and writes preferences in one directory.
#[derive(Clone, Debug)]
pub struct PrefsStore {
    dir: PathBuf,
}

impl PrefsStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    /// Loads preferences. Never fails: problems yield defaults, and an unusable
    /// file is preserved as `prefs.ron.bak-<timestamp>`.
    pub fn load(&self) -> Loaded {
        let path = self.path();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Loaded {
                    prefs: Prefs::default(),
                    issue: None,
                    backup: None,
                };
            }
            Err(err) => {
                return Loaded {
                    prefs: Prefs::default(),
                    issue: Some(LoadIssue::Unreadable(err)),
                    backup: None,
                };
            }
        };

        let issue = match ron::from_str::<Prefs>(&text) {
            Ok(prefs) if prefs.version == PREFS_VERSION => {
                return Loaded {
                    prefs: prefs.sanitized(),
                    issue: None,
                    backup: None,
                };
            }
            Ok(prefs) => LoadIssue::UnknownVersion(prefs.version),
            Err(err) => LoadIssue::Invalid(err.to_string()),
        };

        let backup = self.backup_path();
        let backup = fs::rename(&path, &backup).ok().map(|()| backup);
        Loaded {
            prefs: Prefs::default(),
            issue: Some(issue),
            backup,
        }
    }

    /// Saves atomically: write a temporary file next to the target, flush it
    /// to disk, then rename over the previous file.
    pub fn save(&self, prefs: &Prefs) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let text = ron::ser::to_string_pretty(prefs, ron::ser::PrettyConfig::default())
            .map_err(io::Error::other)?;
        let tmp = self.dir.join(format!("{FILE_NAME}.tmp"));
        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
        }
        fs::rename(&tmp, self.path())
    }

    fn backup_path(&self) -> PathBuf {
        let stamp = now_unix();
        let mut candidate = self.dir.join(format!("{FILE_NAME}.bak-{stamp}"));
        let mut n = 1;
        while candidate.exists() {
            candidate = self.dir.join(format!("{FILE_NAME}.bak-{stamp}-{n}"));
            n += 1;
        }
        candidate
    }
}

/// Seconds since the Unix epoch (0 if the clock is before 1970).
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Whether a recent project's file is still present.
pub fn recent_exists(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LeftTab, WorkspaceLayout};

    #[test]
    fn missing_file_gives_defaults_without_issue() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = PrefsStore::new(dir.path()).load();
        assert_eq!(loaded.prefs, Prefs::default());
        assert!(loaded.issue.is_none());
    }

    #[test]
    fn view_aids_default_and_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        // A file written before view aids existed gets the defaults.
        fs::write(store.path(), "(version: 1, ui_scale: 1.25)").unwrap();
        let loaded = store.load();
        assert!(loaded.issue.is_none());
        assert_eq!(loaded.prefs.view_aids, ViewAids::default());
        let mut prefs = Prefs::default();
        assert_eq!(prefs.view_aids, ViewAids::default());
        prefs.view_aids = ViewAids {
            grid: true,
            guides: false,
            snapping: false,
            grid_spacing: 128.0,
        };
        store.save(&prefs).unwrap();
        assert_eq!(store.load().prefs.view_aids, prefs.view_aids);
        prefs.reset_scaling();
        assert_eq!(prefs.view_aids.grid_spacing, DEFAULT_GRID_SPACING);
    }

    #[test]
    fn files_from_builds_with_the_3d_preview_load() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        fs::write(
            store.path(),
            "(version: 1, ui_scale: 1.25, layout: (column_width: 300.0, \
             view_mode: ThreeD, split_fraction: 0.42, vehicles_open: false))",
        )
        .unwrap();
        let loaded = store.load();
        assert!(loaded.issue.is_none());
        assert!(loaded.backup.is_none());
        assert_eq!(loaded.prefs.ui_scale, 1.25);
        // The panel column and the sidebar of that build are gone too.
        assert_eq!(loaded.prefs.layout, WorkspaceLayout::default());
    }

    #[test]
    fn language_defaults_to_the_system_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        // A file written before languages existed follows the system.
        fs::write(store.path(), "(version: 1, ui_scale: 1.25)").unwrap();
        let loaded = store.load();
        assert!(loaded.issue.is_none());
        assert_eq!(loaded.prefs.language(), None);
        let mut prefs = loaded.prefs;
        prefs.set_language(Some(tp_i18n::Language::German));
        store.save(&prefs).unwrap();
        assert_eq!(
            store.load().prefs.language(),
            Some(tp_i18n::Language::German)
        );
        // Unknown codes (a newer version's language) fall back to the system.
        prefs.language = Some("it".into());
        assert_eq!(prefs.language(), None);
        prefs.set_language(None);
        assert_eq!(prefs.language, None);
    }

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path().join("nested"));
        let mut prefs = Prefs {
            ui_scale: 1.25,
            text_scale: 1.1,
            ..Prefs::default()
        };
        prefs.layout = WorkspaceLayout {
            left_tab: LeftTab::Resources,
            left_width: 320.0,
            inspector_width: 360.0,
            panels_hidden: true,
            generation: 0,
        };
        prefs.recent.push(RecentProject {
            name: "ACE".into(),
            path: "/tmp/ace.truckpaint".into(),
            last_opened: 42,
        });
        store.save(&prefs).unwrap();
        let loaded = store.load();
        assert!(loaded.issue.is_none());
        assert_eq!(loaded.prefs, prefs);
        assert!(!dir.path().join("nested/prefs.ron.tmp").exists());
    }

    #[test]
    fn recent_colors_are_deduplicated_capped_and_saved() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        let mut prefs = Prefs::default();
        for i in 0..20u8 {
            prefs.push_recent_color([i, 0, 0, 255]);
        }
        prefs.push_recent_color([10, 0, 0, 255]);
        assert_eq!(prefs.recent_colors.len(), MAX_RECENT_COLORS);
        assert_eq!(prefs.recent_colors[0], [10, 0, 0, 255]);
        assert_eq!(prefs.recent_colors.iter().filter(|c| c[0] == 10).count(), 1);
        store.save(&prefs).unwrap();
        assert_eq!(store.load().prefs.recent_colors, prefs.recent_colors);
    }

    #[test]
    fn version_1_file_without_recent_colors_still_loads() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        fs::write(store.path(), "(version: 1, ui_scale: 1.25)").unwrap();
        let loaded = store.load();
        assert!(loaded.issue.is_none());
        assert_eq!(loaded.prefs.ui_scale, 1.25);
        assert!(loaded.prefs.recent_colors.is_empty());
    }

    #[test]
    fn file_without_the_title_bar_choice_uses_the_drawn_bar() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        fs::write(store.path(), "(version: 1, ui_scale: 1.25)").unwrap();
        let loaded = store.load();
        assert!(loaded.issue.is_none());
        assert!(!loaded.prefs.system_title_bar);
        let mut prefs = Prefs {
            system_title_bar: true,
            ..Prefs::default()
        };
        store.save(&prefs).unwrap();
        assert!(store.load().prefs.system_title_bar);
        // Reset to defaults keeps it.
        prefs.reset_scaling();
        assert!(prefs.system_title_bar);
    }

    #[test]
    fn retired_template_pack_fields_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        fs::write(
            store.path(),
            r#"(version: 1, ui_scale: 1.25, template_packs: {"ets2": "/packs/old"})"#,
        )
        .unwrap();
        let loaded = store.load();
        assert!(loaded.issue.is_none());
        assert!(loaded.backup.is_none());
        assert_eq!(loaded.prefs.ui_scale, 1.25);
        store.save(&loaded.prefs).unwrap();
        assert!(
            !fs::read_to_string(store.path())
                .unwrap()
                .contains("template_packs")
        );
    }

    #[test]
    fn corrupted_file_falls_back_and_is_backed_up() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        fs::write(store.path(), "this is ( not ron").unwrap();
        let loaded = store.load();
        assert_eq!(loaded.prefs, Prefs::default());
        assert!(matches!(loaded.issue, Some(LoadIssue::Invalid(_))));
        let backup = loaded.backup.expect("backup created");
        assert_eq!(fs::read_to_string(backup).unwrap(), "this is ( not ron");
        assert!(!store.path().exists());
    }

    #[test]
    fn unknown_version_falls_back() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        let future = Prefs {
            version: PREFS_VERSION + 1,
            ..Prefs::default()
        };
        store.save(&future).unwrap();
        let loaded = store.load();
        assert!(
            matches!(loaded.issue, Some(LoadIssue::UnknownVersion(v)) if v == PREFS_VERSION + 1)
        );
        assert!(loaded.backup.is_some());
    }

    #[test]
    fn out_of_range_values_are_clamped() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        store
            .save(&Prefs {
                ui_scale: 9.0,
                text_scale: 0.0,
                ..Prefs::default()
            })
            .unwrap();
        let prefs = store.load().prefs;
        assert_eq!(prefs.ui_scale, 2.0);
        assert_eq!(prefs.text_scale, 0.85);
    }
}
