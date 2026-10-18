//! Application state machine and per-frame orchestration.

use std::sync::Arc;

use egui::{Key, Ui, ViewportCommand};
use tp_core::Project;
use tp_core::document::Object;
use tp_i18n::tr;
use tp_ui::ThemeSettings;

use crate::commands::{self, Availability, CommandId, EditContext};
use crate::file_dialogs::{FileDialogs, NativeDialogs};
use crate::layout::Space;
use crate::mod_export::ModPicture;
use crate::paths::APP_NAME;
use crate::prefs::{Prefs, PrefsStore, recent_exists};
pub use crate::project_io::PendingAction;
use crate::recovery::{Recovered, RecoveryStore};
use crate::saver::Saver;
use crate::text_engine::TextEngine;
use crate::title_bar::{TitleBarMode, title_bar_mode};
use crate::tool::Tool;
use crate::ui;
use crate::viewport::Viewport;
pub use crate::workspace::{COPY_OFFSET, ColorTarget, SaveState, Workspace};

/// Delay after the last preference change before it is written to disk.
const PREFS_SAVE_DELAY: f64 = 1.0;

/// The top-level screen.
pub enum Screen {
    Home,
    Workspace(Box<Workspace>),
}

/// State of the New Project dialog.
#[derive(Clone, Debug, Default)]
pub struct NewProjectDraft {
    pub name: String,
    pub focus_requested: bool,
    /// The checked vehicles and their checked textures, in the order they
    /// were checked (the fleet order).
    pub vehicles: Vec<crate::ui::vehicle_dialogs::VehicleChoice>,
    /// The package id of the open card of "Your fleet".
    pub open: Option<String>,
    /// Search, kind and game of the vehicle list; the game is the
    /// project's (ETS2 when unset).
    pub filter: crate::ui::vehicle_dialogs::VehicleFilter,
    /// Results of installing packages from the dialog.
    pub messages: Vec<Result<String, String>>,
}

impl NewProjectDraft {
    /// The dialog as it opens: the first game the player plays (`played`)
    /// that has installed vehicles, else the game of the installed
    /// vehicles, or ETS2 when vehicles of both games or none are installed.
    pub fn new(library: &crate::vehicles::VehicleLibrary, played: &[String]) -> Self {
        let installed = |code: &String| {
            library
                .vehicles()
                .iter()
                .any(|v| v.newest().manifest.game.id.code() == code)
        };
        if let Some(game) = played
            .iter()
            .find(|g| installed(g))
            .and_then(|g| tp_vehicles::Game::ALL.into_iter().find(|x| x.code() == g))
        {
            let mut draft = Self::default();
            draft.filter.game = Some(game);
            return draft;
        }
        let mut games = library
            .vehicles()
            .iter()
            .map(|v| v.newest().manifest.game.id);
        let first = games.next();
        let game = match first {
            Some(g) if games.all(|other| other == g) => g,
            _ => tp_vehicles::Game::Ets2,
        };
        let mut draft = Self::default();
        draft.filter.game = Some(game);
        draft
    }

    /// The project's game.
    pub fn game(&self) -> tp_vehicles::Game {
        self.filter.game.unwrap_or(tp_vehicles::Game::Ets2)
    }

    /// Checks `choice` after the checked vehicles (where it is when already
    /// checked) and opens its card.
    pub fn check(&mut self, choice: crate::ui::vehicle_dialogs::VehicleChoice) {
        self.open = Some(choice.id.clone());
        crate::ui::vehicle_dialogs::push_choice(&mut self.vehicles, choice);
    }

    /// Unchecks vehicle `id`; when its card was open, the last checked
    /// vehicle's card opens.
    pub fn uncheck(&mut self, id: &str) {
        crate::ui::vehicle_dialogs::remove_choice(&mut self.vehicles, id);
        self.keep_a_card_open();
    }

    /// Checks `vehicle` or unchecks it (a click on its row).
    pub fn toggle(&mut self, vehicle: &crate::vehicles::InstalledVehicle) {
        if self.vehicles.iter().any(|c| c.id == vehicle.id) {
            self.uncheck(&vehicle.id);
        } else {
            self.check(crate::ui::vehicle_dialogs::VehicleChoice::of(vehicle));
        }
    }

    /// Sets the project's game: the vehicles of the other game are
    /// unchecked.
    pub fn set_game(&mut self, game: tp_vehicles::Game, library: &crate::vehicles::VehicleLibrary) {
        self.filter.game = Some(game);
        crate::ui::vehicle_dialogs::retain_game(&mut self.vehicles, library, game);
        self.keep_a_card_open();
    }

    /// The checked vehicle whose card is open.
    pub fn open_choice(&self) -> Option<&crate::ui::vehicle_dialogs::VehicleChoice> {
        let open = self.open.as_deref()?;
        self.vehicles.iter().find(|c| c.id == open)
    }

    /// Opens the last checked vehicle's card when the open one is gone.
    fn keep_a_card_open(&mut self) {
        if self.open_choice().is_none() {
            self.open = self.vehicles.last().map(|c| c.id.clone());
        }
    }
}

pub enum Modal {
    NewProject(NewProjectDraft),
    Preferences,
    KeyboardShortcuts,
    About,
    /// "Save changes before closing?", then the action.
    UnsavedChanges(PendingAction),
    /// An error or information message.
    Message {
        title: String,
        text: String,
    },
    /// Export Texture dialog.
    Export(Box<crate::ui::export_dialog::ExportDialog>),
    /// Export Mod dialog.
    ExportMod(Box<crate::ui::mod_export_dialog::ModExportDialog>),
    /// Vehicle Library dialog.
    VehicleLibrary(crate::ui::vehicle_dialogs::LibraryDialog),
    /// Import from Library dialog.
    ImportFromLibrary(Box<crate::ui::library_dialog::LibraryDialog>),
    /// Update Template confirmation.
    UpdateTemplate(Box<crate::ui::vehicle_dialogs::UpdateDialog>),
    /// Add Vehicle dialog.
    AddVehicle(crate::ui::vehicle_dialogs::AddVehicleDialog),
    /// The textures a project's vehicle paints.
    Textures(crate::ui::vehicle_dialogs::TexturesDialog),
    /// Copy from Cabin: the main texture to copy the artwork of.
    CopyFromCabin(crate::ui::vehicle_dialogs::CopyFromCabinDialog),
    /// Custom Vehicle dialog (or New Version…), over the dialog it came
    /// from.
    CustomVehicle(Box<crate::ui::custom_vehicle::CustomVehicleDialog>),
    /// Confirmation before removing a vehicle with artwork from the project.
    RemoveVehicle {
        package_id: String,
        name: String,
    },
    /// The command palette.
    CommandPalette(crate::ui::palette::Palette),
}

/// Opening of the command palette asked during the frame (the Textures
/// tab's Search textures field); handled after it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteOpen {
    /// Lists only the project's textures.
    pub textures_only: bool,
    /// What was typed in the field.
    pub query: String,
}

/// Objects copied or cut, and where they come from.
#[derive(Clone, Debug, Default)]
pub struct Clipboard {
    pub objects: Vec<Object>,
    /// The session of the workspace they were copied in and its document
    /// then, which holds what they use (images, swatches, styles, symbols)
    /// even after it closes.
    pub source: Option<(u64, Arc<Project>)>,
}

/// An action on one vehicle of the project, asked from the Vehicles panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VehicleRequest {
    Update(String),
    Textures(String),
    Remove(String),
}

pub struct AppState {
    pub prefs: Prefs,
    store: Option<PrefsStore>,
    /// Last version written to (or read from) disk.
    saved_prefs: Prefs,
    prefs_changed_at: Option<f64>,
    applied_theme: Option<ThemeSettings>,
    fonts_installed: bool,
    pub screen: Screen,
    pub modal: Option<Modal>,
    /// Asked from the Vehicles panel during the frame; handled after it.
    pub vehicle_request: Option<VehicleRequest>,
    /// Asked from the Search textures field during the frame; handled
    /// after it.
    pub palette_request: Option<PaletteOpen>,
    /// The commands last run from the command palette, most recent first
    /// (this session only).
    pub palette_recent: Vec<CommandId>,
    pub show_gallery: bool,
    /// Commands triggered this frame, executed at the end of the frame.
    pub queue: Vec<CommandId>,
    /// Whether each recent project's file exists (aligned with `prefs.recent`).
    pub recent_available: Vec<bool>,
    /// Thumbnails of the recent projects, read in the background.
    pub recent_thumbnails: crate::recent_thumbnails::RecentThumbnails,
    /// Objects copied or cut, shared by every project in this session.
    pub clipboard: Clipboard,
    /// Session id of the next workspace opened.
    next_session: u64,
    /// Pastes since the last copy, for the repeated-paste offset.
    pub paste_count: u32,
    /// A text field had focus at the end of the previous frame. egui drops
    /// focus at the start of the frame where Escape is pressed, so this keeps
    /// that Escape for the field instead of canvas shortcuts.
    text_focus_last_frame: bool,
    title: String,
    /// Fonts kept between projects (system fonts load once).
    fonts: Option<tp_text::FontLibrary>,
    /// Load the fonts installed on the computer (off in tests, so results
    /// do not depend on the machine).
    pub system_fonts: bool,
    /// Installed vehicle packages.
    pub vehicles: crate::vehicles::VehicleLibrary,
    /// Previews of installed packages (Vehicle Library), read in the
    /// background.
    pub package_previews: crate::package_previews::PackagePreviews,
    pub project_summaries: crate::project_summaries::ProjectSummaries,
    /// Language of the operating system (English unless detected).
    pub system_language: tp_i18n::Language,
    /// Open / Save / Place dialogs (scripted in tests).
    pub dialogs: Box<dyn FileDialogs>,
    /// Background writer of project files and recovery copies.
    pub(crate) saver: Option<Saver>,
    /// This session's crash-recovery files.
    pub(crate) recovery: Option<RecoveryStore>,
    /// Copies left by earlier sessions, offered on the home screen.
    pub recovered: Vec<Recovered>,
    /// Action to run once the save started from the prompt succeeds.
    pub(crate) after_save: Option<PendingAction>,
    /// The window may close (the user already answered the prompt).
    pub(crate) allow_close: bool,
    /// Last export choices, for the session.
    pub export_settings: crate::export::ExportSettings,
    /// Folder of the last mod export of the session.
    pub last_mod_folder: Option<std::path::PathBuf>,
    /// The user's folders where a mod is proposed to be exported (a test
    /// sets its own).
    pub mod_folders: crate::mod_export::Folders,
    /// The personal library of symbols, swatches and styles.
    pub library: crate::library::LibraryStore,
    /// The platform is macOS (`new` reads it; tests set it to draw either
    /// title bar on any system).
    pub macos: bool,
    /// `TRUCKPAINT_SYSTEM_TITLE_BAR=1` was set at startup.
    pub force_system_title_bar: bool,
    /// The menus are in the macOS menu bar, so the window draws no menu
    /// row.
    pub native_menu_installed: bool,
    /// What a double-click on the title bar does (the system's setting).
    pub double_click: crate::title_bar::DoubleClickAction,
    /// The title bar mode of the last frame, to switch the window's frame
    /// when it changes.
    shown_title_bar_mode: Option<TitleBarMode>,
    /// The games' folders, scan and generation run.
    pub game: crate::game::GameState,
    /// The local MCP server and its calls.
    pub mcp: crate::mcp::McpHost,
    /// Preferences should scroll to the AI assistant (MCP) section when
    /// it opens (the status bar's MCP item was clicked).
    pub reveal_mcp_preferences: bool,
}

impl AppState {
    /// Creates the state, loading preferences from `store` when given.
    pub fn new(store: Option<PrefsStore>) -> Self {
        let prefs = match &store {
            Some(store) => {
                let loaded = store.load();
                if let Some(issue) = &loaded.issue {
                    tracing::warn!(?issue, backup = ?loaded.backup, "preferences reset to defaults");
                }
                loaded.prefs
            }
            None => Prefs::default(),
        };
        // No preferences file yet: the first run looks for the games once.
        let first_run = store.as_ref().is_some_and(|s| !s.path().exists());
        let mut state = Self::with_prefs(prefs, store);
        if first_run {
            crate::game::detect_first_run(&mut state.prefs.game_folders);
        }
        state.system_fonts = true;
        state.dialogs = Box::new(NativeDialogs);
        state.macos = cfg!(target_os = "macos");
        state.force_system_title_bar = crate::title_bar::forced_by_environment();
        state.double_click = crate::title_bar::DoubleClickAction::of_system();
        state
    }

    /// Creates the state from in-memory preferences (tests).
    pub fn with_prefs(prefs: Prefs, store: Option<PrefsStore>) -> Self {
        let mut state = Self {
            saved_prefs: prefs.clone(),
            prefs,
            store,
            prefs_changed_at: None,
            applied_theme: None,
            fonts_installed: false,
            screen: Screen::Home,
            modal: None,
            vehicle_request: None,
            palette_request: None,
            palette_recent: Vec::new(),
            show_gallery: false,
            queue: Vec::new(),
            recent_available: Vec::new(),
            recent_thumbnails: Default::default(),
            clipboard: Clipboard::default(),
            next_session: 1,
            paste_count: 0,
            text_focus_last_frame: false,
            title: String::new(),
            fonts: None,
            system_fonts: false,
            system_language: tp_i18n::Language::English,
            vehicles: crate::vehicles::VehicleLibrary::default(),
            package_previews: Default::default(),
            project_summaries: Default::default(),
            // Tests never open real dialogs; `new` installs the native ones.
            dialogs: Box::new(crate::file_dialogs::ScriptedDialogs::default()),
            saver: None,
            recovery: None,
            recovered: Vec::new(),
            after_save: None,
            allow_close: false,
            export_settings: crate::export::ExportSettings::default(),
            last_mod_folder: None,
            mod_folders: crate::mod_export::Folders::of_user(),
            // In memory; `TruckPaintApp::new` gives it its file.
            library: crate::library::LibraryStore::default(),
            macos: false,
            force_system_title_bar: false,
            native_menu_installed: false,
            double_click: Default::default(),
            shown_title_bar_mode: None,
            game: Default::default(),
            mcp: Default::default(),
            reveal_mcp_preferences: false,
        };
        state.refresh_recent_availability();
        state
    }

    /// File names proposed by scripted dialogs (tests).
    pub fn dialogs_suggested(&self) -> Vec<String> {
        self.dialogs.suggested()
    }

    pub fn has_project(&self) -> bool {
        matches!(self.screen, Screen::Workspace(_))
    }

    pub fn workspace(&self) -> Option<&Workspace> {
        match &self.screen {
            Screen::Workspace(ws) => Some(ws),
            Screen::Home => None,
        }
    }

    pub fn workspace_mut(&mut self) -> Option<&mut Workspace> {
        match &mut self.screen {
            Screen::Workspace(ws) => Some(ws),
            Screen::Home => None,
        }
    }

    pub fn refresh_recent_availability(&mut self) {
        self.recent_available = self
            .prefs
            .recent
            .iter()
            .map(|r| recent_exists(&r.path))
            .collect();
        self.recent_thumbnails.invalidate();
    }

    /// Opens the editor on a project (new, read from a file or recovered),
    /// in the Project space.
    pub fn open_project(&mut self, project: Project) {
        tracing::info!(name = %project.name, side = project.resolution.side(), "project created");
        let fonts = self
            .fonts
            .take()
            .unwrap_or_else(tp_text::FontLibrary::bundled);
        let mut ws = Workspace::with_text_engine(project, TextEngine::new(fonts));
        ws.session = self.next_session;
        ws.space = Space::Project;
        self.next_session += 1;
        self.screen = Screen::Workspace(Box::new(ws));
    }

    pub fn close_project(&mut self) {
        self.clear_recovery();
        if let Screen::Workspace(ws) = std::mem::replace(&mut self.screen, Screen::Home) {
            self.fonts = Some(ws.text.fonts);
        }
        self.refresh_recent_availability();
    }

    /// Runs one frame of the whole application.
    /// Opens Import from Library…; a library that could not be read says
    /// so in the dialog.
    pub fn open_import_from_library(&mut self) {
        if !self.has_project() {
            return;
        }
        let mut dialog = crate::ui::library_dialog::LibraryDialog::default();
        self.library.library();
        if let Some(issue) = self.library.take_issue() {
            dialog.issue = Some(crate::ui::library_dialog::issue_message(&issue));
        }
        self.modal = Some(Modal::ImportFromLibrary(Box::new(dialog)));
    }

    /// Reports a library that could not be read, once, when it was first
    /// used outside the Import from Library dialog.
    fn report_library_issue(&mut self) {
        if self.modal.is_some() {
            return;
        }
        if let Some(issue) = self.library.take_issue() {
            self.modal = Some(Modal::Message {
                title: tr("library-unreadable-title"),
                text: crate::ui::library_dialog::issue_message(&issue),
            });
        }
    }

    /// Opens Update Template for the newest installed version of the
    /// project's vehicle `package_id` (`None`: the active surface's).
    pub fn open_update_dialog(&mut self, package_id: Option<&str>) {
        let Some(vehicle) = self.workspace().and_then(|ws| {
            match package_id {
                Some(id) => ws.project.vehicle(id),
                None => ws.project.vehicle_of(ws.project.active_surface),
            }
            .cloned()
        }) else {
            return;
        };
        let Some(version) = self
            .vehicles
            .update_for(&vehicle)
            .map(|v| v.manifest.version.clone())
        else {
            return;
        };
        let loaded = self.vehicles.load(&vehicle.package_id, &version);
        let Some(ws) = self.workspace() else {
            return;
        };
        match loaded {
            Ok(package) => {
                if let Some(plan) = crate::vehicle_project::plan(&ws.project, &package) {
                    self.modal = Some(Modal::UpdateTemplate(Box::new(
                        crate::ui::vehicle_dialogs::UpdateDialog::new(package, plan),
                    )));
                }
            }
            Err(err) => {
                let file = format!("{} {version}", vehicle.name);
                self.modal = Some(Modal::Message {
                    title: tr("undo-update-template"),
                    text: crate::vehicles::install_error_message(&err, &file),
                });
            }
        }
    }

    /// Tells in a message that a generation ended, unless the Vehicle Library
    /// is open (it shows the result) or another dialog is.
    fn announce_generation(&mut self) {
        if !std::mem::take(&mut self.game.announce) {
            return;
        }
        let Some(last) = &self.game.last else {
            return;
        };
        if self.modal.is_none() {
            self.modal = Some(Modal::Message {
                title: tp_i18n::tr("game-finished-title"),
                text: crate::game::summary_text(last),
            });
        }
    }

    /// Opens Preferences, checking the game folders again.
    pub fn open_preferences(&mut self) {
        self.game.refresh_folders(&self.prefs.game_folders);
        self.modal = Some(Modal::Preferences);
    }

    /// Runs an action asked from the Vehicles panel.
    pub fn handle_vehicle_request(&mut self, request: VehicleRequest, now: f64) {
        match request {
            VehicleRequest::Update(id) => self.open_update_dialog(Some(&id)),
            VehicleRequest::Textures(id) => {
                let Some(dialog) = self.workspace().and_then(|ws| {
                    let vehicle = ws.project.vehicle(&id)?;
                    Some(crate::ui::vehicle_dialogs::TexturesDialog::new(
                        &ws.project,
                        vehicle,
                    ))
                }) else {
                    return;
                };
                self.modal = Some(Modal::Textures(dialog));
            }
            VehicleRequest::Remove(id) => {
                let Some(ws) = self.workspace() else {
                    return;
                };
                let Some(vehicle) = ws.project.vehicle(&id) else {
                    return;
                };
                let name = vehicle.name.clone();
                if ws.project.has_artwork(ws.project.vehicle_range(&id)) {
                    self.modal = Some(Modal::RemoveVehicle {
                        package_id: id,
                        name,
                    });
                } else if let Some(ws) = self.workspace_mut() {
                    let _ = ws.remove_vehicle(&id, now);
                }
            }
        }
    }

    /// How the top of the window is drawn: the platform's, the
    /// preference's or the environment's choice.
    pub fn title_bar_mode(&self) -> TitleBarMode {
        title_bar_mode(
            self.macos,
            self.prefs.system_title_bar,
            self.force_system_title_bar,
        )
    }

    /// Whether the window draws the menus as a row: with the system title
    /// bar, and on macOS until the menus are in the menu bar (the drawn
    /// title bar holds them).
    pub fn draws_menu_row(&self) -> bool {
        match self.title_bar_mode() {
            TitleBarMode::MacNative => !self.native_menu_installed,
            TitleBarMode::Drawn => false,
            TitleBarMode::System => true,
        }
    }

    /// Gives the window its system frame or takes it away when the mode
    /// changes between the drawn and the system title bar (the preference
    /// was toggled); the window opened with the right one.
    fn sync_decorations(&mut self, ctx: &egui::Context) {
        let mode = self.title_bar_mode();
        let shown = self.shown_title_bar_mode.replace(mode);
        if let Some(shown) = shown
            && shown != mode
            && shown != TitleBarMode::MacNative
            && mode != TitleBarMode::MacNative
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(
                mode == TitleBarMode::System,
            ));
        }
    }

    /// The interface language: the user's choice, else the system's.
    pub fn language(&self) -> tp_i18n::Language {
        self.prefs.language().unwrap_or(self.system_language)
    }

    pub fn show(&mut self, ui: &mut Ui) {
        tp_i18n::set_language(self.language());
        let ctx = ui.ctx().clone();
        if !self.fonts_installed {
            // Fonts registered now are only usable from the next pass.
            self.install_theme(&ctx);
            ctx.request_repaint();
            return;
        }
        if self.sync_theme(&ctx) {
            // Context style changes apply from the next frame; style this
            // frame's root `Ui` directly so the first frame is themed too.
            tp_ui::theme::configure_style(ui.style_mut(), self.prefs.theme().text_scale);
        }
        tp_ui::theme::follow_display(&ctx);
        self.mcp
            .sync(self.prefs.mcp_enabled, self.prefs.mcp_port, &ctx);
        // Calls run before the frame draws, so it shows their result.
        self.run_mcp_calls(ctx.input(|i| i.time));
        if let Some(next) = self.mcp.next_change() {
            ctx.request_repaint_after(next);
        }
        self.game.poll(&mut self.vehicles);
        self.announce_generation();
        self.handle_close_request(&ctx);
        self.handle_keyboard(&ctx);
        let aids = self.prefs.view_aids;
        if let Some(ws) = self.workspace_mut() {
            ws.aids = aids;
        }

        match self.screen {
            Screen::Home => ui::home::show(ui, self),
            Screen::Workspace(_) => ui::workspace::show(ui, self),
        }
        if let Some(request) = self.vehicle_request.take() {
            self.handle_vehicle_request(request, ctx.input(|i| i.time));
        }
        if let Some(request) = self.palette_request.take()
            && self.modal.is_none()
        {
            self.modal = Some(Modal::CommandPalette(crate::ui::palette::Palette::new(
                request.textures_only,
                &request.query,
            )));
        }
        ui::dialogs::show_modal(&ctx, self);
        if self.show_gallery {
            ui::gallery::show(&ctx, &mut self.show_gallery);
        }
        if self.title_bar_mode() == TitleBarMode::Drawn {
            ui::title_bar::resize_grips(&ctx);
        }

        for id in std::mem::take(&mut self.queue) {
            self.dispatch(&ctx, id);
        }
        self.after_frame(&ctx);
        self.flush_recent_color(&ctx);
        self.text_focus_last_frame = ctx.text_edit_focused();
        self.sync_title(&ctx);
        self.sync_decorations(&ctx);
        let now = ctx.input(|i| i.time);
        self.persist_prefs(now, false);
        if self.prefs_changed_at.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(PREFS_SAVE_DELAY + 0.1));
        }
    }

    /// Installs fonts and theme. Call before the first frame when possible
    /// (fonts become usable from the next pass).
    pub fn install_theme(&mut self, ctx: &egui::Context) {
        let theme = self.prefs.theme();
        tp_ui::theme::install(ctx, theme);
        self.fonts_installed = true;
        self.applied_theme = Some(theme);
        // Cmd/Ctrl +/-/0 zoom the canvas, not the whole interface.
        ctx.options_mut(|o| o.zoom_with_keyboard = false);
    }

    /// What the editing commands can do right now.
    pub fn edit_context(&self) -> EditContext {
        match self.workspace() {
            Some(ws) => EditContext {
                has_project: true,
                texture_count: ws.project.surfaces.len(),
                has_selection: !ws.selection.is_empty(),
                can_undo: ws.history.can_undo() || ws.is_drawing_pen(),
                can_redo: ws.history.can_redo(),
                has_clipboard: !self.clipboard.objects.is_empty(),
                gesture_active: ws.gesture.is_active(),
                undo_label: ws.history.undo_label(),
                redo_label: ws.history.redo_label(),
                selection_has_group: ws
                    .selection
                    .iter()
                    .any(|id| ws.project.surface().get(*id).is_some_and(|o| o.is_group())),
                single_text: matches!(ws.selection.as_slice(), [id] if ws.can_edit_text(*id)),
                editing_text: ws.is_editing_text(),
                selection_has_convertible: ws.selection_has_convertible(),
                has_guides: !ws.project.surface().guides.is_empty(),
                selection_count: ws.selection.len(),
                selection_has_text: ws.selection_has_text(),
                combine_block: ws.combine_block(),
                align_to_key: ws.panels.align_to == crate::arrange::AlignTo::KeyObject,
                has_template: ws
                    .project
                    .surface()
                    .template
                    .as_ref()
                    .is_some_and(|t| t.status != tp_core::TemplateStatus::Removed),
                template_visible: ws
                    .project
                    .surface()
                    .template
                    .as_ref()
                    .is_some_and(|t| t.visible),
                update_available: ws
                    .project
                    .vehicle_of(ws.project.active_surface)
                    .is_some_and(|v| self.vehicles.update_for(v).is_some()),
                cabin_sources: crate::vehicle_project::cabin_sources(
                    &ws.project,
                    ws.project.active_surface,
                )
                .len(),
                editing_symbol: ws.is_editing_symbol(),
                selection_has_instance: ws.selection_has_instance(),
                only_instances: !ws.selection.is_empty()
                    && ws.selected_objects().iter().all(|o| o.is_instance()),
                single_instance: ws.selected_instance_symbol().is_some(),
                space: Some(ws.space),
            },
            None => EditContext::default(),
        }
    }

    /// Re-applies the theme when scaling preferences changed; returns true
    /// when it did.
    fn sync_theme(&mut self, ctx: &egui::Context) -> bool {
        let theme = self.prefs.theme();
        if self.applied_theme == Some(theme) {
            return false;
        }
        tp_ui::theme::apply(ctx, theme);
        self.applied_theme = Some(theme);
        true
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let field_focused = ctx.text_edit_focused()
            || self.text_focus_last_frame
            || self.modal.is_some()
            || tp_ui::widgets::keyboard_claimed(ctx)
            || ui::workspace::panels::vehicle::search_field_focused(ctx);
        let now = ctx.input(|i| i.time);
        let mut editing_text = false;
        if let Some(ws) = self.workspace_mut()
            && ws.is_editing_text()
        {
            editing_text = true;
            if !field_focused {
                crate::text_input::handle(ctx, ws, now);
            }
        }
        if !field_focused && let Some(ws) = self.workspace_mut() {
            crate::path_edit::handle_pen_keys(ctx, ws, now);
        }
        let typing = field_focused || editing_text;
        // Escape closes an open popover (color, stroke) and does nothing
        // else, such as deselecting.
        if !typing {
            tp_ui::widgets::Popover::close_on_escape(ctx);
        }
        let has_project = self.has_project();
        let edit = self.edit_context();
        // `Tab` and `G` act in the Workshop only; elsewhere Tab moves the
        // keyboard focus.
        let in_workshop = edit.space == Some(Space::Workshop);
        let triggered = ctx.input_mut(|i| {
            commands::take_triggered(i, typing, has_project, |id| {
                is_enabled(id, &edit) && (in_workshop || !id.key_needs_workshop())
            })
        });
        if triggered.contains(&CommandId::TogglePanels) {
            // egui read Tab at the start of the frame: keep it from also
            // moving the keyboard focus.
            ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        }
        self.queue.extend(triggered);

        // Holding Space temporarily activates the Hand tool.
        let space_down = !typing && ctx.input(|i| i.key_down(Key::Space));
        if let Some(ws) = self.workspace_mut() {
            match (space_down, ws.tool_before_space) {
                (true, None) => {
                    ws.tool_before_space = Some(ws.tool);
                    ws.tool = Tool::Hand;
                }
                (false, Some(previous)) => {
                    ws.tool = previous;
                    ws.tool_before_space = None;
                }
                _ => {}
            }
        }
    }

    /// Executes a command. Disabled commands are ignored.
    pub fn dispatch(&mut self, ctx: &egui::Context, id: CommandId) {
        if !is_enabled(id, &self.edit_context()) {
            return;
        }
        let palette_open = matches!(self.modal, Some(Modal::CommandPalette(_)));
        // The palette's shortcut opens and closes it, never over another
        // dialog; it changes nothing else (a text edit, a pen path or a
        // panel edit in progress go on).
        if id == CommandId::CommandPalette {
            if palette_open {
                self.modal = None;
            } else if self.modal.is_none() {
                self.modal = Some(Modal::CommandPalette(crate::ui::palette::Palette::new(
                    false, "",
                )));
            }
            return;
        }
        // Any other command closes the palette first.
        if palette_open {
            self.modal = None;
        }
        // Export Mod's key works while a mod setting is typed in (the
        // dialog proposes the typed Name), not over another dialog.
        if id == CommandId::ExportMod && self.modal.is_some() {
            return;
        }
        tracing::debug!(?id, "command");
        let now = ctx.input(|i| i.time);
        let ppp = ctx.pixels_per_point();
        // A panel interaction in progress becomes one step before any command.
        if let Some(ws) = self.workspace_mut() {
            ws.commit_pending(now);
            if ws.is_editing_text() {
                match id {
                    CommandId::Undo => {
                        ws.text_undo(now);
                        return;
                    }
                    CommandId::Redo => {
                        ws.text_redo(now);
                        return;
                    }
                    _ if !keeps_text_session(id) => ws.end_text_session(now),
                    _ => {}
                }
            }
            // While a pen path is drawn, Undo removes its last point; other
            // commands (except view changes) finish it first.
            if ws.is_drawing_pen() {
                match id {
                    CommandId::Undo => {
                        ws.pen_pop();
                        return;
                    }
                    CommandId::Redo => return,
                    _ if !keeps_text_session(id) => {
                        ws.finish_pen(false, now);
                    }
                    _ => {}
                }
            }
        }
        match id {
            CommandId::Place => self.with_workspace(|ws| ws.place_request = Some(None)),
            CommandId::EditText => self.with_workspace(|ws| {
                if let [id] = ws.selection[..] {
                    ws.start_editing(id, None, now);
                }
            }),
            CommandId::NewProject => self.guard(ctx, PendingAction::NewProject),
            CommandId::OpenProject => self.guard(ctx, PendingAction::OpenDialog),
            CommandId::ExportTexture => {
                self.modal = Some(Modal::Export(Box::new(
                    crate::ui::export_dialog::ExportDialog::new(self.export_settings),
                )));
            }
            CommandId::ExportMod => {
                if let Some(ws) = self.workspace_mut() {
                    ws.commit_mod_draft(now);
                }
                if let Some(id) = ctx.memory(|m| m.focused()) {
                    ctx.memory_mut(|m| m.surrender_focus(id));
                }
                if let Some(ws) = self.workspace() {
                    let library = &self.vehicles;
                    let summary = crate::mod_export::summary(&ws.project, &|v| {
                        library.recorded(v).map(|i| i.manifest.clone())
                    });
                    let destination = crate::mod_export::destination(
                        &ws.project,
                        &self.prefs.mod_folders,
                        self.last_mod_folder.as_deref(),
                        &self.mod_folders,
                    );
                    let dialog =
                        crate::ui::mod_export_dialog::ModExportDialog::new(summary, destination);
                    self.modal = Some(Modal::ExportMod(Box::new(dialog)));
                }
            }
            CommandId::Save => {
                self.save(ctx, false);
            }
            CommandId::SaveAs => {
                self.save(ctx, true);
            }
            CommandId::CloseProject => self.guard(ctx, PendingAction::CloseProject),
            CommandId::Preferences => self.open_preferences(),
            CommandId::Quit => self.guard(ctx, PendingAction::Quit),
            CommandId::ShowSpace(space) => self.with_workspace(|ws| ws.show_space(space)),
            CommandId::ShowLeftTab(tab) => {
                self.prefs.layout.show_tab(tab);
                self.with_workspace(|ws| ws.show_space(Space::Workshop));
            }
            CommandId::TogglePanels => {
                self.prefs.layout.panels_hidden = !self.prefs.layout.panels_hidden;
            }
            CommandId::ResetWorkspace => self.prefs.layout.reset(),
            CommandId::DesignGallery => self.show_gallery = !self.show_gallery,
            CommandId::CommandPalette => {}
            CommandId::KeyboardShortcuts => self.modal = Some(Modal::KeyboardShortcuts),
            CommandId::About => self.modal = Some(Modal::About),
            CommandId::SelectTool(tool) => {
                if let Some(ws) = self.workspace_mut() {
                    if tool != Tool::DirectSelect {
                        ws.points.clear();
                    }
                    ws.tool = tool;
                    ws.tool_before_space = None;
                }
            }
            CommandId::Undo => self.with_workspace(Workspace::undo),
            CommandId::Redo => self.with_workspace(Workspace::redo),
            CommandId::SelectAll => self.with_workspace(Workspace::select_all),
            CommandId::Deselect => self.with_workspace(|ws| {
                if ws.selection.is_empty() && ws.points.is_empty() {
                    ws.finish_symbol_edit(now);
                } else {
                    ws.deselect();
                }
            }),
            CommandId::Delete => self.with_workspace(|ws| {
                if ws.points.is_empty() {
                    ws.delete_selection(now);
                } else {
                    ws.delete_points(now);
                }
            }),
            CommandId::DeleteLayer => self.with_workspace(|ws| ws.delete_selection(now)),
            CommandId::ConvertToPath => self.with_workspace(|ws| ws.convert_selection_to_path(now)),
            CommandId::CreateOutlines => self.with_workspace(|ws| ws.create_outlines(now)),
            CommandId::Combine(op) => self.with_workspace(|ws| ws.combine_selection(op, now)),
            CommandId::Align(edge) => self.with_workspace(|ws| ws.align_selection(edge, now)),
            CommandId::Flip(axis) => self.with_workspace(|ws| ws.flip_selection(axis, now)),
            CommandId::Distribute(axis, mode) => {
                self.with_workspace(|ws| ws.distribute_selection(axis, mode, now));
            }
            CommandId::ShowGrid => self.prefs.view_aids.grid = !self.prefs.view_aids.grid,
            CommandId::ShowGuides => self.prefs.view_aids.guides = !self.prefs.view_aids.guides,
            CommandId::ShowTemplate => self.with_workspace(|ws| {
                if let Some(t) = ws.project.surface_mut().template.as_mut() {
                    t.visible = !t.visible;
                    ws.settings_changed = true;
                }
            }),
            CommandId::NextTexture | CommandId::PreviousTexture => {
                let next = id == CommandId::NextTexture;
                self.with_workspace(|ws| {
                    let n = ws.project.surfaces.len();
                    let i = ws.project.active_surface;
                    ws.set_active_surface(if next { (i + 1) % n } else { (i + n - 1) % n });
                });
            }
            CommandId::VehicleLibrary => {
                self.modal = Some(Modal::VehicleLibrary(Default::default()));
            }
            CommandId::AddVehicle => {
                let game = self
                    .workspace()
                    .and_then(|ws| ws.project.game().map(str::to_owned));
                self.modal = Some(Modal::AddVehicle(
                    crate::ui::vehicle_dialogs::AddVehicleDialog::for_game(game.as_deref()),
                ));
            }
            CommandId::UpdateTemplate => self.open_update_dialog(None),
            CommandId::CopyFromCabin => {
                if let Some(ws) = self.workspace() {
                    let dialog = crate::ui::vehicle_dialogs::CopyFromCabinDialog::new(&ws.project);
                    if !dialog.sources.is_empty() {
                        self.modal = Some(Modal::CopyFromCabin(dialog));
                    }
                }
            }
            CommandId::Snapping => self.prefs.view_aids.snapping = !self.prefs.view_aids.snapping,
            CommandId::ClearGuides => self.with_workspace(|ws| {
                ws.edit("cmd-clear-guides", now, false, |project, _| {
                    project.clear_guides()
                });
            }),
            CommandId::Duplicate | CommandId::DuplicateLayer => {
                self.with_workspace(|ws| ws.duplicate_selection(now));
            }
            CommandId::Group => self.with_workspace(|ws| ws.group_selection(now)),
            CommandId::Ungroup => self.with_workspace(|ws| ws.ungroup_selection(now)),
            CommandId::ConvertToSymbol => self.with_workspace(|ws| {
                ws.convert_to_symbol(now);
            }),
            CommandId::ImportFromLibrary => self.open_import_from_library(),
            CommandId::EditSymbol => self.with_workspace(|ws| {
                if let Some(id) = ws.selected_instance_symbol() {
                    ws.edit_symbol(id, now);
                }
            }),
            CommandId::DetachInstance => {
                self.with_workspace(|ws| ws.detach_selected_instances(now));
            }
            CommandId::FinishSymbol => self.with_workspace(|ws| ws.finish_symbol_edit(now)),
            CommandId::NewLayer => self.with_workspace(|ws| ws.new_layer(now)),
            CommandId::SwapColorTarget => self.with_workspace(|ws| {
                ws.panels.color_target = match ws.panels.color_target {
                    ColorTarget::Fill => ColorTarget::Stroke,
                    ColorTarget::Stroke => ColorTarget::Fill,
                };
            }),
            CommandId::SwapFillStroke => self.with_workspace(|ws| ws.swap_fill_stroke(now)),
            CommandId::DefaultColors => self.with_workspace(|ws| ws.default_colors(now)),
            CommandId::BringForward => self.with_workspace(|ws| ws.bring_forward(now)),
            CommandId::SendBackward => self.with_workspace(|ws| ws.send_backward(now)),
            CommandId::Copy | CommandId::Cut => {
                if let Some(ws) = self.workspace() {
                    self.clipboard = Clipboard {
                        objects: ws.selected_objects(),
                        source: Some((ws.session, Arc::new(ws.project.clone()))),
                    };
                    self.paste_count = 0;
                    // The system clipboard gets the objects' names: the
                    // native backend only reports Cmd/Ctrl+V when it holds
                    // text, so Paste would not fire after copying into an
                    // empty clipboard.
                    let names: Vec<&str> = self
                        .clipboard
                        .objects
                        .iter()
                        .map(|o| o.name.as_str())
                        .collect();
                    let text = names.join("\n");
                    ctx.copy_text(if text.trim().is_empty() {
                        "TruckPaint objects".to_owned()
                    } else {
                        text
                    });
                }
                if id == CommandId::Cut
                    && let Some(ws) = self.workspace_mut()
                {
                    ws.edit("cmd-cut", now, false, |project, selection| {
                        project.surface_mut().remove(selection);
                        selection.clear();
                    });
                }
            }
            CommandId::Paste => {
                let Clipboard { objects, source } = self.clipboard.clone();
                let offset = COPY_OFFSET * f64::from(self.paste_count);
                self.paste_count += 1;
                self.with_workspace(|ws| match source {
                    // From another project: what the objects use comes too.
                    Some((session, project)) if session != ws.session => {
                        ws.paste_from(&project, &objects, offset, now);
                    }
                    _ => ws.paste(&objects, offset, now),
                });
            }
            CommandId::Nudge(direction, big) => {
                let (dx, dy) = direction.delta();
                let step = if big { 10.0 } else { 1.0 };
                self.with_workspace(|ws| {
                    if ws.points.is_empty() {
                        ws.nudge(dx * step, dy * step, now);
                    } else {
                        ws.nudge_points(dx * step, dy * step, now);
                    }
                });
            }
            CommandId::ZoomIn
            | CommandId::ZoomOut
            | CommandId::FitToScreen
            | CommandId::ActualSize => {
                self.with_workspace(|ws| {
                    let (Some(canvas), Some(view)) = (ws.canvas_rect, ws.viewport.as_mut()) else {
                        return;
                    };
                    match id {
                        CommandId::ZoomIn => {
                            let zoom = Viewport::step_zoom(view.zoom, 1);
                            view.zoom_at(canvas, ppp, canvas.center(), zoom);
                        }
                        CommandId::ZoomOut => {
                            let zoom = Viewport::step_zoom(view.zoom, -1);
                            view.zoom_at(canvas, ppp, canvas.center(), zoom);
                        }
                        CommandId::ActualSize => view.set_zoom_centered(1.0),
                        _ => *view = Viewport::fit(ws.project.surface().size, canvas, ppp),
                    }
                });
            }
        }
    }

    /// Adds the last applied color to recent colors once the user is no
    /// longer dragging or typing (so a picker drag adds one color).
    fn flush_recent_color(&mut self, ctx: &egui::Context) {
        let busy = ctx.input(|i| i.pointer.any_down()) || ctx.text_edit_focused();
        if busy {
            return;
        }
        let Some(ws) = self.workspace_mut() else {
            return;
        };
        if let Some(c) = ws.recent_candidate.take() {
            self.prefs.push_recent_color([c.r, c.g, c.b, c.a]);
        }
    }

    /// Per-frame workspace upkeep after the UI: ends a text session whose
    /// text lost the selection, picks up fonts, places requested or dropped
    /// files.
    fn after_frame(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let mut dropped = ctx.input(|i| i.raw.dropped_files.clone());
        let hover = ctx.input(|i| i.pointer.latest_pos());
        let ppp = ctx.pixels_per_point();
        // Dropped vehicle packages are installed (home screen or workspace).
        let is_package = |f: &std::sync::Arc<dyn egui::DroppedFile + Send + Sync>| {
            f.path()
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(tp_vehicles::EXTENSION))
        };
        let named = |f: &std::sync::Arc<dyn egui::DroppedFile + Send + Sync>| {
            let path = f.path();
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            (
                name,
                f.bytes().map(|b| b.to_vec()).map_err(|e| e.to_string()),
            )
        };
        let packages: Vec<(String, Result<Vec<u8>, String>)> = dropped
            .iter()
            .filter(|f| is_package(f))
            .map(named)
            .collect();
        dropped.retain(|f| !is_package(f));
        // Other files dropped on the Custom Vehicle dialog are templates.
        if let Some(Modal::CustomVehicle(dialog)) = &mut self.modal {
            if !dropped.is_empty() {
                dialog.add_files(dropped.iter().map(named).collect(), hover);
            }
            dropped.clear();
        }
        if !packages.is_empty() {
            let results = crate::ui::vehicle_dialogs::install_named(self, packages);
            let failed: Vec<String> = results.iter().filter_map(|r| r.clone().err()).collect();
            if let Some(Modal::CustomVehicle(dialog)) = &mut self.modal {
                dialog.messages = results;
            } else if !failed.is_empty() {
                self.modal = Some(Modal::Message {
                    title: tr("cmd-vehicle-library"),
                    text: failed.join("\n"),
                });
            } else if let Some(ws) = self.workspace_mut() {
                let done: Vec<String> = results.into_iter().filter_map(Result::ok).collect();
                ws.show_hint(done.join(", "), now);
            } else if let Some(Modal::VehicleLibrary(dialog)) = &mut self.modal {
                dialog.messages = results;
            } else {
                self.modal = Some(Modal::Message {
                    title: tr("cmd-vehicle-library"),
                    text: results
                        .into_iter()
                        .filter_map(Result::ok)
                        .collect::<Vec<_>>()
                        .join("\n"),
                });
            }
        }
        self.poll_saves(ctx);
        self.report_library_issue();
        self.write_library(ctx);
        self.write_recovery(ctx, now);
        let Screen::Workspace(ws) = &mut self.screen else {
            return;
        };
        ws.check_text_session(now);
        ws.prune_points();
        if std::mem::take(&mut ws.request_show_guides) {
            self.prefs.view_aids.guides = true;
        }
        if self.system_fonts && !ws.text.fonts.system_requested() {
            let ctx = ctx.clone();
            ws.text
                .fonts
                .load_system_fonts(move || ctx.request_repaint());
        }
        if ws.text.poll_fonts() {
            ws.relayout_all_texts();
        }
        if let Some(which) = ws.picture_request.take()
            && let Some(path) = self.dialogs.pick_mod_image()
        {
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            let bytes = std::fs::read(&path).map_err(|e| e.to_string());
            ws.use_picture_file(which, &name, bytes, now);
        }
        // A file dropped on a mod picture's preview becomes that picture.
        if let Some(i) = hover.and_then(|p| {
            ws.picture_drop_zones
                .iter()
                .position(|z| z.is_some_and(|z| z.contains(p)))
        }) && let Some(file) = dropped.first()
        {
            let which = [ModPicture::Icon, ModPicture::Image][i];
            let path = file.path();
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            ws.use_picture_file(which, &name, file.bytes(), now);
            dropped.clear();
        }
        if let Some(at) = ws.place_request.take() {
            let paths = self.dialogs.pick_images();
            if !paths.is_empty() {
                let files = crate::import::read_files(&paths);
                ws.place_files(files, at, now);
            }
        }
        if !dropped.is_empty() {
            let at = match (hover, ws.canvas_rect, ws.screen_map(ppp)) {
                (Some(p), Some(rect), Some(map)) if rect.contains(p) => Some(map.to_doc(p)),
                _ => None,
            };
            let files = dropped
                .iter()
                .map(|file| {
                    let path = file.path();
                    let name = path.file_name().map_or_else(
                        || path.display().to_string(),
                        |n| n.to_string_lossy().into_owned(),
                    );
                    match file.bytes() {
                        Ok(bytes) => crate::import::read_bytes(&name, bytes),
                        Err(reason) => Err(crate::import::ImportError::unreadable(name, reason)),
                    }
                })
                .collect();
            ws.place_files(files, at, now);
        }
    }

    fn with_workspace(&mut self, f: impl FnOnce(&mut Workspace)) {
        if let Some(ws) = self.workspace_mut() {
            f(ws);
        }
    }

    fn sync_title(&mut self, ctx: &egui::Context) {
        let title = match self.workspace() {
            Some(ws) => format!("{} — {APP_NAME}", ws.project.name),
            None => APP_NAME.to_owned(),
        };
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    /// Current window title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Debounced preference saving; `force` writes immediately (on exit).
    pub fn persist_prefs(&mut self, now: f64, force: bool) {
        if self.prefs == self.saved_prefs {
            self.prefs_changed_at = None;
            return;
        }
        let changed_at = *self.prefs_changed_at.get_or_insert(now);
        if !force && now - changed_at < PREFS_SAVE_DELAY {
            return;
        }
        if let Some(store) = &self.store {
            match store.save(&self.prefs) {
                Ok(()) => tracing::debug!(path = %store.path().display(), "preferences saved"),
                Err(err) => tracing::error!(%err, "failed to save preferences"),
            }
        }
        self.saved_prefs = self.prefs.clone();
        self.prefs_changed_at = None;
    }
}

/// Commands that leave a text editing session running (view changes).
fn keeps_text_session(id: CommandId) -> bool {
    matches!(
        id,
        CommandId::ZoomIn
            | CommandId::ZoomOut
            | CommandId::FitToScreen
            | CommandId::ActualSize
            | CommandId::ShowLeftTab(_)
            | CommandId::TogglePanels
            | CommandId::ResetWorkspace
            | CommandId::DesignGallery
            | CommandId::KeyboardShortcuts
            | CommandId::About
            | CommandId::Preferences
            | CommandId::Save
            | CommandId::SaveAs
            | CommandId::CommandPalette
    )
}

/// Whether a command can run in the current state.
pub fn is_enabled(id: CommandId, edit: &EditContext) -> bool {
    if edit.editing_symbol && id.needs_texture() {
        return false;
    }
    match id.meta().availability {
        Availability::Always => true,
        Availability::NeedsProject => edit.has_project,
        Availability::When(check, _) => check(edit),
        Availability::NotYet(_) => false,
    }
}

/// Tooltip explaining why a command is disabled in the current state
/// (combine commands name what blocks them).
pub fn disabled_reason_for(id: CommandId, edit: &EditContext) -> Option<&'static str> {
    if edit.editing_symbol && id.needs_texture() {
        return Some("reason-editing-symbol");
    }
    if edit.only_instances && id.changes_shapes() {
        return Some("reason-instance-look");
    }
    match (id, edit.combine_block) {
        (CommandId::Combine(_), Some(reason)) if edit.has_selection => Some(reason),
        _ => disabled_reason(id),
    }
}

/// Tooltip explaining why a command is disabled.
pub fn disabled_reason(id: CommandId) -> Option<&'static str> {
    match id.meta().availability {
        Availability::NotYet(reason) => Some(reason),
        Availability::When(_, reason) if !reason.is_empty() => Some(reason),
        Availability::NeedsProject => Some("reason-no-project"),
        Availability::Always | Availability::When(..) => None,
    }
}
