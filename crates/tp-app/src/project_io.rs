//! Saving, opening, the unsaved-changes prompt and crash recovery.

use std::path::{Path, PathBuf};
use tp_i18n::tr;

use egui::ViewportCommand;
use tp_core::Project;

use crate::file_dialogs::{suggested_file_name, with_extension};
use crate::prefs::now_unix;
use crate::recovery::{self, RecoveryMeta, RecoveryStore};
use crate::saver::{Done, JobKind, Saver};
use crate::state::{AppState, Modal, NewProjectDraft, Screen};
use crate::workspace::{PendingSave, Workspace};

/// An action that would discard the open project, run once the user has
/// answered the unsaved-changes prompt.
#[derive(Clone, Debug, PartialEq)]
pub enum PendingAction {
    CloseProject,
    Quit,
    NewProject,
    /// Show the Open dialog, then open the chosen file.
    OpenDialog,
    Open(PathBuf),
    Restore(String),
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

impl AppState {
    fn saver(&mut self, ctx: &egui::Context) -> &mut Saver {
        self.saver.get_or_insert_with(|| {
            let ctx = ctx.clone();
            Saver::new(move || ctx.request_repaint())
        })
    }

    /// The recovery folder, when crash recovery is enabled.
    pub fn recovery_dir(&self) -> Option<&Path> {
        self.recovery.as_ref().map(RecoveryStore::dir)
    }

    /// Starts recovery for this session in `dir` and lists copies left by
    /// earlier sessions.
    pub fn enable_recovery(&mut self, dir: &Path) {
        match RecoveryStore::open(dir) {
            Ok(store) => {
                self.recovered = store.scan();
                self.recovery = Some(store);
            }
            Err(err) => tracing::error!(%err, "crash recovery is disabled"),
        }
    }

    fn message(&mut self, title: String, text: String) {
        self.modal = Some(Modal::Message { title, text });
    }

    /// Runs `action`, first asking to save unsaved changes.
    pub fn guard(&mut self, ctx: &egui::Context, action: PendingAction) {
        let unsaved = self.workspace().is_some_and(Workspace::has_unsaved_changes);
        if unsaved {
            self.modal = Some(Modal::UnsavedChanges(action));
        } else {
            self.run_action(ctx, action);
        }
    }

    /// Answer "Save" to the prompt: the action continues once saved.
    pub fn save_then(&mut self, ctx: &egui::Context, action: PendingAction) {
        if self.save(ctx, false) {
            self.after_save = Some(action);
        }
    }

    pub fn run_action(&mut self, ctx: &egui::Context, action: PendingAction) {
        match action {
            PendingAction::CloseProject => self.close_project(),
            PendingAction::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            PendingAction::NewProject => {
                self.modal = Some(Modal::NewProject(NewProjectDraft::new(
                    &self.vehicles,
                    &self.prefs.games,
                )));
            }
            PendingAction::OpenDialog => {
                if let Some(path) = self.dialogs.open_project() {
                    self.open_file(ctx, &path);
                }
            }
            PendingAction::Open(path) => self.open_file(ctx, &path),
            PendingAction::Restore(session) => self.restore(ctx, &session),
        }
    }

    /// Save (or Save As when `save_as` or never saved). Returns whether a
    /// save started.
    pub fn save(&mut self, ctx: &egui::Context, save_as: bool) -> bool {
        let now = ctx.input(|i| i.time);
        let Some(ws) = self.workspace_mut() else {
            return false;
        };
        ws.commit_pending(now);
        let path = match (&ws.path, save_as) {
            (Some(path), false) => path.clone(),
            _ => {
                let suggested = suggested_file_name(&ws.project.name);
                match self.dialogs.save_project(&suggested) {
                    Some(path) => with_extension(path),
                    None => return false,
                }
            }
        };
        self.start_save(ctx, path)
    }

    fn start_save(&mut self, ctx: &egui::Context, path: PathBuf) -> bool {
        let Some(ws) = self.workspace_mut() else {
            return false;
        };
        if ws.saving.is_some() {
            ws.save_queued = Some(path);
            return true;
        }
        let snapshot = ws.snapshot();
        let project = ws.project.clone();
        match self
            .saver(ctx)
            .write(JobKind::Save, project, path.clone(), None)
        {
            Some(id) => {
                if let Some(ws) = self.workspace_mut() {
                    ws.saving = Some(PendingSave { id, snapshot, path });
                }
                true
            }
            None => {
                self.message(
                    tr("file-save-failed-title"),
                    tr!("file-save-failed", file = file_name(&path)),
                );
                false
            }
        }
    }

    /// Applies finished writes (called every frame).
    pub fn poll_saves(&mut self, ctx: &egui::Context) {
        let done = self.saver.as_ref().map(Saver::finished).unwrap_or_default();
        for d in done {
            self.on_written(ctx, d);
        }
    }

    fn on_written(&mut self, ctx: &egui::Context, done: Done) {
        // Failures of background copies are logged by the saver.
        if matches!(done.kind, JobKind::Recovery | JobKind::Library) {
            return;
        }
        let Some(ws) = self.workspace_mut() else {
            return;
        };
        if ws.saving.as_ref().is_none_or(|p| p.id != done.id) {
            return;
        }
        let pending = ws.saving.take().expect("checked above");
        match done.result {
            Ok(()) => {
                ws.saved = Some(pending.snapshot);
                ws.settings_changed = false;
                ws.path = Some(pending.path.clone());
                ws.recovery_written = None;
                let name = ws.project.name.clone();
                let queued = ws.save_queued.take();
                if let Some(store) = &self.recovery {
                    store.clear();
                }
                self.prefs.push_recent(&name, &pending.path);
                self.refresh_recent_availability();
                if let Some(path) = queued {
                    self.start_save(ctx, path);
                } else if let Some(action) = self.after_save.take() {
                    self.run_action(ctx, action);
                }
            }
            Err(reason) => {
                ws.save_queued = None;
                self.after_save = None;
                self.message(
                    tr("file-save-failed-title"),
                    tr!(
                        "file-save-failed-reason",
                        file = file_name(&pending.path),
                        reason = reason.to_string()
                    ),
                );
            }
        }
    }

    /// Waits for pending writes and applies them (before closing).
    fn finish_writes(&mut self, ctx: Option<&egui::Context>) {
        if let Some(mut saver) = self.saver.take() {
            let done = saver.flush();
            if let Some(ctx) = ctx {
                for d in done {
                    self.on_written(ctx, d);
                }
            }
        }
    }

    /// Opens a project file, replacing the open project.
    pub fn open_file(&mut self, ctx: &egui::Context, path: &Path) {
        if self
            .workspace()
            .and_then(|ws| ws.path.as_deref())
            .is_some_and(|open| same_file(open, path))
        {
            return;
        }
        let opened = match tp_file::read(path) {
            Ok(opened) => opened,
            Err(err) => {
                tracing::warn!(path = %path.display(), %err, "cannot open project");
                self.message(
                    tr("file-open-failed-title"),
                    file_error(&err, &file_name(path)),
                );
                return;
            }
        };
        let name = opened.project.name.clone();
        let mut project = opened.project;
        let filled = self.fill_game_data(&mut project);
        self.open_document(
            ctx,
            project,
            Some(path.to_path_buf()),
            !opened.migrated && !filled,
        );
        self.prefs.push_recent(&name, path);
        self.refresh_recent_availability();
    }

    /// Fills the game data `project` lacks (files written before it was
    /// recorded) from the installed package versions its vehicles record.
    /// Returns whether anything was filled.
    fn fill_game_data(&self, project: &mut Project) -> bool {
        let manifests: Vec<_> = project
            .vehicles
            .iter()
            .filter(|v| v.game_data.is_none())
            .filter_map(|v| self.vehicles.recorded(v))
            .map(|installed| installed.manifest.clone())
            .collect();
        let mut filled = false;
        for m in &manifests {
            filled |= crate::vehicle_project::fill_game_data(project, m);
        }
        filled
    }

    /// Replaces the open project with `project` read from a file.
    fn open_document(
        &mut self,
        ctx: &egui::Context,
        project: Project,
        path: Option<PathBuf>,
        saved: bool,
    ) {
        self.finish_writes(Some(ctx));
        if self.has_project() {
            self.close_project();
        }
        self.open_project(project);
        if let Some(ws) = self.workspace_mut() {
            ws.relayout_all_texts();
            ws.path = path;
            if saved {
                ws.saved = Some(ws.snapshot());
            }
        }
    }

    /// Queues the library for writing when it changed (called every frame).
    pub fn write_library(&mut self, ctx: &egui::Context) {
        if let Some((library, path)) = self.library.take_write() {
            self.saver(ctx).write(JobKind::Library, library, path, None);
        }
    }

    /// Writes a recovery copy when due (called every frame).
    pub fn write_recovery(&mut self, ctx: &egui::Context, now: f64) {
        let Some(store) = &self.recovery else {
            return;
        };
        let Screen::Workspace(ws) = &mut self.screen else {
            return;
        };
        if ws.recovery_at <= 0.0 {
            ws.recovery_at = now;
        }
        if now - ws.recovery_at < recovery::INTERVAL || !ws.has_unsaved_changes() {
            return;
        }
        ws.recovery_at = now;
        let snapshot = ws.snapshot();
        if ws
            .recovery_written
            .as_ref()
            .is_some_and(|w| w.same_saved(&snapshot))
        {
            return;
        }
        let meta = store.meta_file(&RecoveryMeta {
            name: ws.project.name.clone(),
            original: ws.path.clone(),
            saved_at: now_unix(),
        });
        let path = store.copy_path();
        let project = ws.project.clone();
        ws.recovery_written = Some(snapshot);
        self.saver(ctx)
            .write(JobKind::Recovery, project, path, Some(meta));
    }

    /// Deletes this session's recovery copy (project closed or saved).
    pub fn clear_recovery(&mut self) {
        self.finish_writes(None);
        if let Some(store) = &self.recovery {
            store.clear();
        }
    }

    /// Opens a recovered project, unsaved, linked to its original file.
    pub fn restore(&mut self, ctx: &egui::Context, session: &str) {
        let Some(store) = &self.recovery else {
            return;
        };
        let original = self
            .recovered
            .iter()
            .find(|r| r.session == session)
            .and_then(|r| r.meta.as_ref())
            .and_then(|m| m.original.clone());
        match store.read(session) {
            Ok(opened) => {
                let mut project = opened.project;
                self.fill_game_data(&mut project);
                self.open_document(ctx, project, original, false);
                if let Some(store) = &self.recovery {
                    store.adopt(session);
                }
                self.recovered.retain(|r| r.session != session);
                if let Some(ws) = self.workspace_mut() {
                    ws.recovery_written = Some(ws.snapshot());
                }
            }
            Err(err) => {
                if let Some(r) = self.recovered.iter_mut().find(|r| r.session == session) {
                    r.meta = None;
                }
                self.message(
                    tr("file-restore-failed-title"),
                    file_error(&err, &tr("file-recovered-copy")),
                );
            }
        }
    }

    pub fn discard_recovered(&mut self, session: &str) {
        if let Some(store) = &self.recovery {
            store.discard(session);
        }
        self.recovered.retain(|r| r.session != session);
    }

    /// Normal exit: pending saves finish and the recovery copy is removed.
    pub fn shutdown(&mut self) {
        self.mcp.stop();
        // A library change not yet queued is written before quitting.
        if let Some((library, path)) = self.library.take_write()
            && let Err(err) = tp_file::library::write(&library, &path)
        {
            tracing::error!(%err, "the library could not be written");
        }
        self.clear_recovery();
    }

    /// Intercepts the window's close button when work is unsaved.
    pub fn handle_close_request(&mut self, ctx: &egui::Context) {
        if !ctx.input(|i| i.viewport().close_requested()) || self.allow_close {
            return;
        }
        let unsaved = self.workspace().is_some_and(Workspace::has_unsaved_changes);
        if unsaved {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.modal = Some(Modal::UnsavedChanges(PendingAction::Quit));
        }
    }
}

/// Why a project file could not be read or written, about `file`, in the
/// current language.
pub fn file_error(err: &tp_file::Error, file: &str) -> String {
    match err {
        tp_file::Error::NotAProject => tr!("file-not-a-project", file = file),
        tp_file::Error::Damaged(_) => tr!("file-damaged", file = file),
        tp_file::Error::NewerVersion { .. } => tr!("file-newer-version", file = file),
        tp_file::Error::Unsupported { .. } => tr!("file-dev-format", file = file),
        tp_file::Error::Io(e) => tr!("file-io", file = file, reason = e.to_string()),
    }
}
