//! Per-OS application directories.

use std::path::PathBuf;

use directories::ProjectDirs;

/// Product name used for window titles and directory names.
pub const APP_NAME: &str = "TruckPaint";

/// Standard per-user directories for the application.
#[derive(Clone, Debug)]
pub struct AppDirs {
    /// Preferences and window state.
    pub config: PathBuf,
    /// Logs and crash-recovery copies.
    pub data: PathBuf,
}

impl AppDirs {
    /// Resolves the platform directories (`~/.config`, `%APPDATA%`,
    /// `~/Library/Application Support`, ...). `None` if no home directory.
    pub fn resolve() -> Option<Self> {
        let dirs = ProjectDirs::from("app", APP_NAME, APP_NAME)?;
        Some(Self {
            config: dirs.config_dir().to_path_buf(),
            data: dirs.data_dir().to_path_buf(),
        })
    }

    pub fn logs(&self) -> PathBuf {
        self.data.join("logs")
    }

    /// Installed vehicle packages.
    pub fn vehicles(&self) -> PathBuf {
        self.data.join("vehicles")
    }

    /// The personal library of symbols, swatches and styles.
    pub fn library(&self) -> PathBuf {
        self.data.join(tp_file::library::FILE_NAME)
    }

    /// Crash-recovery copies of unsaved projects.
    pub fn recovery(&self) -> PathBuf {
        self.data.join("recovery")
    }
}
