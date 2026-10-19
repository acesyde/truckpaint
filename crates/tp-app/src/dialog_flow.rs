//! What a file dialog is for. Sites that need a file set a request (the
//! dialog and its [`Purpose`]) during the frame; the application starts it
//! after the frame, and the answer comes back here, once, to what asked for
//! it. While a dialog is open nothing else can ask for one (see
//! `AppState::dialog_pending`).

use std::path::PathBuf;

use tp_core::kurbo::Point;
use tp_i18n::tr;
use tp_vehicles::Game;

use crate::file_dialogs::{DialogAnswer, DialogRequest, with_extension};
use crate::mod_export::ModPicture;
use crate::project_io::PendingAction;
use crate::state::{AppState, Modal};

/// The dialog that holds the packages to install.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallInto {
    NewProject,
    Library,
    AddVehicle,
}

/// What to do with the answer of a dialog.
#[derive(Clone, Debug)]
pub enum Purpose {
    OpenProject,
    /// Save the project of `session` to the chosen file, then run `then`.
    SaveProject {
        then: Option<PendingAction>,
        session: u64,
    },
    PlaceImages {
        at: Option<Point>,
        session: u64,
    },
    ModPicture(ModPicture),
    /// The chosen file is handed to the open Export Texture dialog.
    ExportTexture,
    /// The chosen file becomes the open Export Mod dialog's destination.
    ExportMod,
    ExportPackage {
        from: PathBuf,
    },
    InstallPackages(InstallInto),
    PickTemplates {
        replace: Option<usize>,
    },
    GameFolder(Game),
    ModFolder(String),
}

/// A dialog to start and what it is for.
#[derive(Clone, Debug)]
pub struct Wanted {
    pub request: DialogRequest,
    pub purpose: Purpose,
}

/// The dialog asked for during the frame and the one that is open.
#[derive(Debug, Default)]
pub struct DialogSlots {
    wanted: Option<Wanted>,
    pending: Option<Purpose>,
}

impl DialogSlots {
    /// Asks for a dialog, started after the frame. Ignored while a dialog is
    /// open or already asked for.
    pub fn want(&mut self, request: DialogRequest, purpose: Purpose) {
        if self.pending.is_none() && self.wanted.is_none() {
            self.wanted = Some(Wanted { request, purpose });
        }
    }

    /// Whether a file dialog is open.
    pub fn is_open(&self) -> bool {
        self.pending.is_some()
    }
}

impl AppState {
    /// Asks for a dialog, started after the frame (see [`DialogSlots::want`]).
    pub fn want_dialog(&mut self, request: DialogRequest, purpose: Purpose) {
        self.dialog.want(request, purpose);
    }

    /// Whether a file dialog is open.
    pub fn dialog_pending(&self) -> bool {
        self.dialog.is_open()
    }

    /// Starts the dialog asked for during the frame.
    pub fn start_wanted_dialog(&mut self, ctx: &egui::Context) {
        let Some(wanted) = self.dialog.wanted.take() else {
            return;
        };
        if self.dialog.pending.is_some() || self.dialogs.busy() {
            return;
        }
        let wake = ctx.clone();
        self.dialogs
            .start(wanted.request, Box::new(move || wake.request_repaint()));
        self.dialog.pending = Some(wanted.purpose);
    }

    /// Applies the answer of the open dialog, if it came.
    pub fn poll_dialog(&mut self, ctx: &egui::Context) {
        let Some(answer) = self.dialogs.poll() else {
            return;
        };
        if let Some(purpose) = self.dialog.pending.take() {
            self.apply_answer(ctx, purpose, answer);
        }
    }

    /// Whether the game and mod folders are on screen (Preferences, or the
    /// first launch).
    fn folders_are_shown(&self) -> bool {
        matches!(self.modal, Some(Modal::Preferences)) || self.first_run.is_some()
    }

    /// Whether the open project is the one of `session`.
    fn is_session(&self, session: u64) -> bool {
        self.workspace().is_some_and(|ws| ws.session == session)
    }

    fn apply_answer(&mut self, ctx: &egui::Context, purpose: Purpose, answer: DialogAnswer) {
        let now = ctx.input(|i| i.time);
        match purpose {
            Purpose::OpenProject => {
                if let Some(path) = answer.path() {
                    self.open_file(ctx, &path);
                }
            }
            Purpose::SaveProject { then, session } => {
                if let Some(path) = answer.path()
                    && self.is_session(session)
                {
                    self.start_save_then(ctx, with_extension(path), then);
                }
            }
            Purpose::PlaceImages { at, session } => {
                let paths = answer.paths();
                if !paths.is_empty()
                    && self.is_session(session)
                    && let Some(ws) = self.workspace_mut()
                {
                    ws.place_files(crate::import::read_files(&paths), at, now);
                }
            }
            Purpose::ModPicture(which) => {
                if let Some(path) = answer.path()
                    && let Some(ws) = self.workspace_mut()
                {
                    let name = path.file_name().map_or_else(
                        || path.display().to_string(),
                        |n| n.to_string_lossy().into_owned(),
                    );
                    let bytes = std::fs::read(&path).map_err(|e| e.to_string());
                    ws.use_picture_file(which, &name, bytes, now);
                }
            }
            Purpose::ExportTexture => {
                if let Some(path) = answer.path()
                    && let Some(Modal::Export(dialog)) = &mut self.modal
                {
                    dialog.chosen = Some(path);
                }
            }
            Purpose::ExportMod => {
                if let Some(path) = answer.path()
                    && let Some(Modal::ExportMod(dialog)) = &mut self.modal
                {
                    let path = crate::mod_export::with_extension(path);
                    self.last_mod_folder = path.parent().map(std::path::Path::to_path_buf);
                    dialog.choose_destination(path);
                }
            }
            Purpose::ExportPackage { from } => {
                if let Some(to) = answer.path()
                    && let Some(Modal::VehicleLibrary(dialog)) = &mut self.modal
                {
                    let written =
                        std::fs::read(&from)
                            .map_err(|e| e.to_string())
                            .and_then(|bytes| {
                                tp_file::write_atomic(&to, &bytes).map_err(|e| e.to_string())
                            });
                    let file = to.display().to_string();
                    dialog.messages = vec![match written {
                        Ok(()) => Ok(tr!("vehicles-exported", file = file.as_str())),
                        Err(reason) => Err(tr!(
                            "vehicles-export-failed",
                            file = file.as_str(),
                            reason = reason.as_str()
                        )),
                    }];
                }
            }
            Purpose::InstallPackages(into) => {
                let paths = answer.paths();
                let open = matches!(
                    (&self.modal, into),
                    (Some(Modal::NewProject(_)), InstallInto::NewProject)
                        | (Some(Modal::VehicleLibrary(_)), InstallInto::Library)
                        | (Some(Modal::AddVehicle(_)), InstallInto::AddVehicle)
                );
                if open && !paths.is_empty() {
                    let results = crate::ui::vehicle_dialogs::install(self, &paths);
                    match &mut self.modal {
                        Some(Modal::NewProject(draft)) => draft.messages = results,
                        Some(Modal::VehicleLibrary(dialog)) => dialog.messages = results,
                        Some(Modal::AddVehicle(dialog)) => dialog.messages = results,
                        _ => {}
                    }
                }
            }
            Purpose::PickTemplates { replace } => {
                let paths = answer.paths();
                if !paths.is_empty()
                    && let Some(Modal::CustomVehicle(dialog)) = &mut self.modal
                {
                    dialog.add_paths(&paths, replace);
                }
            }
            Purpose::GameFolder(game) => {
                if let Some(path) = answer.path()
                    && self.folders_are_shown()
                {
                    self.game
                        .set_folder(&mut self.prefs.game_folders, game, &path);
                }
            }
            Purpose::ModFolder(code) => {
                if let Some(path) = answer.path()
                    && self.folders_are_shown()
                {
                    self.prefs.mod_folders.insert(code, path);
                }
            }
        }
    }
}
