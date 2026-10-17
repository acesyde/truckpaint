//! Summaries of the recent projects for the Vehicle Library: which
//! vehicles and textures each uses, read from the file without opening it
//! (see [`tp_file::summary`]). Files are read one at a time in a background
//! thread and kept until they change on disk.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::SystemTime;

/// Delay before the UI picks up a finished summary.
const REPAINT_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

/// What is known of a project file.
#[derive(Clone, Debug, PartialEq)]
pub enum Known {
    Read(tp_file::Summary),
    Unreadable,
}

#[derive(Default)]
pub struct ProjectSummaries {
    /// By path: the file's modification time when read, and what was read.
    read: HashMap<PathBuf, (Option<SystemTime>, Known)>,
    job: Option<(PathBuf, Option<SystemTime>, Receiver<Known>)>,
}

impl ProjectSummaries {
    /// Picks up a finished summary, and starts reading the first of `paths`
    /// not read yet or changed since.
    pub fn update(&mut self, ctx: &egui::Context, paths: &[PathBuf]) {
        if let Some((path, modified, job)) = &self.job {
            let known = match job.try_recv() {
                Ok(known) => known,
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => Known::Unreadable,
            };
            self.read.insert(path.clone(), (*modified, known));
            self.job = None;
        }
        let stale = paths.iter().find_map(|path| {
            let modified = modified(path);
            match self.read.get(path) {
                Some((then, _)) if *then == modified => None,
                _ => Some((path.clone(), modified)),
            }
        });
        if let Some((path, modified)) = stale {
            self.job = start(ctx, path.clone()).map(|rx| (path, modified, rx));
        }
    }

    /// What was read of `path`; `None` while it is being read.
    pub fn get(&self, path: &Path) -> Option<&Known> {
        self.read.get(path).map(|(_, known)| known)
    }

    pub fn is_reading(&self) -> bool {
        self.job.is_some()
    }
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn start(ctx: &egui::Context, path: PathBuf) -> Option<Receiver<Known>> {
    let (tx, rx) = mpsc::channel();
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("project-summary".into())
        .spawn(move || {
            let known = tp_file::summary(&path).map_or(Known::Unreadable, Known::Read);
            let _ = tx.send(known);
            ctx.request_repaint_after(REPAINT_DELAY);
        });
    match spawned {
        Ok(_) => Some(rx),
        Err(err) => {
            tracing::error!(%err, "cannot read the project summary");
            None
        }
    }
}
