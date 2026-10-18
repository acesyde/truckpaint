//! Vehicle Library, Update Template, Add Vehicle, Textures and Remove from
//! Project dialogs, and the vehicle list and texture checkboxes shared with
//! the New Project wizard.

use egui::{Align, Layout, RichText, ScrollArea, TextEdit, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{
    EmptyState, SegmentedControl, SquareCheckbox, primary_button, secondary_button,
};
use tp_vehicles::{Game, Kind, Manifest, Package, Part};

use crate::state::AppState;
use crate::vehicle_project::{TextureChange, UpdatePlan, default_textures, is_always_painted};
use crate::vehicles::{InstalledVehicle, VehicleLibrary};

/// Search and filters of a vehicle list.
#[derive(Clone, Debug, Default)]
pub struct VehicleFilter {
    pub query: String,
    pub game: Option<Game>,
    pub kind: Option<Kind>,
    /// The game is set by the project: no game filter is shown.
    pub game_locked: bool,
}

impl VehicleFilter {
    pub fn matches(&self, m: &Manifest) -> bool {
        self.matches_vehicle(&m.name, &m.brand, m.game.id, m.kind)
    }

    pub(crate) fn matches_vehicle(&self, name: &str, brand: &str, game: Game, kind: Kind) -> bool {
        let q = self.query.trim().to_lowercase();
        (q.is_empty() || name.to_lowercase().contains(&q) || brand.to_lowercase().contains(&q))
            && self.game.is_none_or(|g| g == game)
            && self.kind.is_none_or(|k| k == kind)
    }

    /// Search field and game / kind filters.
    pub fn show(&mut self, ui: &mut Ui, id: &str) {
        ui.horizontal(|ui| {
            // As tall as the combos beside it.
            let height = (ui.text_style_height(&egui::TextStyle::Button)
                + 2.0 * ui.spacing().button_padding.y)
                .max(ui.spacing().interact_size.y)
                .ceil();
            let search = ui.add(
                TextEdit::singleline(&mut self.query)
                    .hint_text(format!("{} {}", icons::SEARCH, tr("vehicles-search")))
                    .desired_width(200.0)
                    .vertical_align(Align::Center)
                    .min_size(egui::vec2(0.0, height)),
            );
            search.widget_info(|| {
                WidgetInfo::labeled(WidgetType::TextEdit, true, tr("vehicles-search"))
            });
            let game_name = |g: Option<Game>| g.map_or_else(|| tr("vehicles-all-games"), game_name);
            if !self.game_locked {
                let combo = egui::ComboBox::from_id_salt((id, "game"))
                    .icon(tp_ui::widgets::dropdown_icon)
                    .selected_text(game_name(self.game))
                    .show_ui(ui, |ui| {
                        for g in [None, Some(Game::Ets2), Some(Game::Ats)] {
                            ui.selectable_value(&mut self.game, g, game_name(g));
                        }
                    });
                combo.response.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::ComboBox, true, tr("vehicles-game-filter"))
                });
            }
            let kind_name = |k: Option<Kind>| match k {
                None => tr("vehicles-all-kinds"),
                Some(k) => kind_label(k),
            };
            let combo = egui::ComboBox::from_id_salt((id, "kind"))
                .icon(tp_ui::widgets::dropdown_icon)
                .selected_text(kind_name(self.kind))
                .show_ui(ui, |ui| {
                    for k in [None, Some(Kind::Truck), Some(Kind::Trailer)] {
                        ui.selectable_value(&mut self.kind, k, kind_name(k));
                    }
                });
            combo.response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::ComboBox, true, tr("vehicles-kind-filter"))
            });
        });
    }
}

pub fn kind_label(kind: Kind) -> String {
    tr(match kind {
        Kind::Truck => "vehicles-truck",
        Kind::Trailer => "vehicles-trailer",
    })
}

/// The game's full name.
pub fn game_name(game: Game) -> String {
    match game {
        Game::Ets2 => "Euro Truck Simulator 2".to_owned(),
        Game::Ats => "American Truck Simulator".to_owned(),
    }
}

pub fn game_label(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "ETS2",
        Game::Ats => "ATS",
    }
}

/// Supported game versions in words: "any version", "1.56 or later"…
pub fn versions_text(versions: &semver::VersionReq) -> String {
    crate::game_versions::req_in_words(versions)
}

/// The detail pane's summary of `m`: "Any game version · Main texture:
/// Cabin · 2 accessories".
pub fn details_text(m: &Manifest) -> String {
    let versions = if m.game.versions == semver::VersionReq::STAR {
        tr("vehicles-details-any-version")
    } else {
        tr!(
            "vehicles-details-versions",
            versions = versions_text(&m.game.versions)
        )
    };
    let main: Vec<&str> = m.paint_job.main.iter().map(|p| p.name.as_str()).collect();
    let main = tr!(
        "vehicles-details-main",
        count = main.len(),
        names = main.join(", ")
    );
    let accessories = tr!(
        "vehicles-details-accessories",
        count = m.paint_job.accessories.len()
    );
    format!("{versions} · {main} · {accessories}")
}

/// "Volvo · Truck · ETS2 · 1.3.0".
pub fn summary(m: &Manifest) -> String {
    format!(
        "{} · {} · {} · {}",
        m.brand,
        kind_label(m.kind),
        game_label(m.game.id),
        m.version
    )
}

/// The installed vehicles passing `filter`.
pub fn filtered<'a>(
    library: &'a VehicleLibrary,
    filter: &VehicleFilter,
) -> Vec<&'a InstalledVehicle> {
    library
        .vehicles()
        .iter()
        .filter(|v| filter.matches(&v.newest().manifest))
        .collect()
}

/// Installed vehicles passing the filters, excluding vehicles already selected.
pub fn vehicle_rows<'a>(
    library: &'a VehicleLibrary,
    filter: &VehicleFilter,
    exclude: &[String],
) -> Vec<&'a InstalledVehicle> {
    filtered(library, filter)
        .into_iter()
        .filter(|v| !exclude.contains(&v.id))
        .collect()
}

/// A vehicle and the textures checked for it.
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleChoice {
    pub id: String,
    pub version: semver::Version,
    /// Checked main textures and accessories (part ids).
    pub textures: Vec<String>,
}

impl VehicleChoice {
    /// The newest version of `vehicle` with its default textures checked.
    pub fn of(vehicle: &InstalledVehicle) -> Self {
        let m = &vehicle.newest().manifest;
        Self {
            id: vehicle.id.clone(),
            version: m.version.clone(),
            textures: default_textures(m),
        }
    }

    /// The chosen version's manifest.
    pub fn manifest<'a>(&self, library: &'a VehicleLibrary) -> Option<&'a Manifest> {
        library
            .get(&self.id)?
            .versions
            .iter()
            .find(|v| v.manifest.version == self.version)
            .map(|v| &v.manifest)
    }

    /// Ready to create or add: a main texture painted.
    pub fn is_complete(&self, library: &VehicleLibrary) -> bool {
        self.manifest(library).is_some_and(|m| {
            m.paint_job
                .main
                .iter()
                .any(|p| self.textures.contains(&p.id) || is_always_painted(m, &p.id))
        })
    }

    /// The textures that will be painted, in package order, with their
    /// sizes.
    pub fn painted<'a>(&self, m: &'a Manifest) -> Vec<&'a Part> {
        m.paint_job
            .parts()
            .map(|(_, p)| p)
            .filter(|p| self.textures.contains(&p.id) || is_always_painted(m, &p.id))
            .collect()
    }
}

/// Checks `choice` at the end of the checked vehicles `choices` (the fleet
/// order is the order they were checked). A vehicle already checked keeps
/// its place and textures.
pub fn push_choice(choices: &mut Vec<VehicleChoice>, choice: VehicleChoice) {
    if !choices.iter().any(|c| c.id == choice.id) {
        choices.push(choice);
    }
}

/// Unchecks vehicle `id`; returns whether it was checked.
pub fn remove_choice(choices: &mut Vec<VehicleChoice>, id: &str) -> bool {
    let before = choices.len();
    choices.retain(|c| c.id != id);
    choices.len() != before
}

/// Checks `vehicle` (its newest version, default textures) or unchecks it;
/// returns whether it is checked now.
pub fn toggle_choice(choices: &mut Vec<VehicleChoice>, vehicle: &InstalledVehicle) -> bool {
    if remove_choice(choices, &vehicle.id) {
        false
    } else {
        push_choice(choices, VehicleChoice::of(vehicle));
        true
    }
}

/// Keeps the checked vehicles of `game` only.
pub fn retain_game(choices: &mut Vec<VehicleChoice>, library: &VehicleLibrary, game: Game) {
    choices.retain(|c| c.manifest(library).is_some_and(|m| m.game.id == game));
}

/// Whether every checked vehicle is ready (see
/// [`VehicleChoice::is_complete`]), and at least one is checked.
pub fn all_complete(choices: &[VehicleChoice], library: &VehicleLibrary) -> bool {
    !choices.is_empty() && choices.iter().all(|c| c.is_complete(library))
}

/// Checkboxes for the main textures and accessories of `m`, keeping the
/// package's order in `chosen`. A single main texture is always painted,
/// and the last checked main texture can't be unchecked.
pub fn texture_checkboxes(ui: &mut Ui, m: &Manifest, chosen: &mut Vec<String>) {
    textures_to_paint(ui, m, chosen, &tr("vehicles-main-textures"), None);
}

/// The same checkboxes under `main_heading`; with `mode` (New Project), the
/// main textures as chips (the cabins each covers in its tooltip), then the
/// main texture mode, read only, before the accessories.
pub fn textures_to_paint(
    ui: &mut Ui,
    m: &Manifest,
    chosen: &mut Vec<String>,
    main_heading: &str,
    mode: Option<&str>,
) {
    let job = &m.paint_job;
    let checked_main = job.main.iter().filter(|p| chosen.contains(&p.id)).count();
    let order: Vec<&str> = job.parts().map(|(_, p)| p.id.as_str()).collect();
    let toggle = |ui: &mut Ui, chosen: &mut Vec<String>, part: &Part, locked: bool| {
        let always = is_always_painted(m, &part.id);
        let mut on = always || chosen.contains(&part.id);
        let enabled = !always && !(locked && on);
        let response = ui.add_enabled(enabled, SquareCheckbox::new(&mut on, &part.name));
        let size = part.texture.size;
        let response = response.on_hover_text(format!("{} × {}", size.width, size.height));
        if response.changed() {
            set_painted(chosen, &order, &part.id, on);
        }
    };
    // New Project: the main textures as chips, their cabins' internal names
    // in the tooltip.
    let chip = |ui: &mut Ui, chosen: &mut Vec<String>, part: &Part, locked: bool| {
        let always = is_always_painted(m, &part.id);
        let on = always || chosen.contains(&part.id);
        let enabled = !always && !(locked && on);
        let size = part.texture.size;
        let mut tip = format!("{} × {}", size.width, size.height);
        if !part.game_ids.is_empty() {
            let ids = tr!("project-cabin-ids", ids = part.game_ids.join(", "));
            tip = format!("{tip}\n{ids}");
        }
        let response = ui
            .add_enabled_ui(enabled, |ui| {
                tp_ui::widgets::chip_toggle(ui, on, &part.name)
            })
            .inner
            .on_hover_text(&tip)
            .on_disabled_hover_text(&tip);
        if response.changed() {
            set_painted(chosen, &order, &part.id, !on);
        }
    };
    let heading = |ui: &mut Ui, text: &str| {
        ui.label(RichText::new(text).small().color(color::TEXT_SECONDARY));
    };
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XXS;
        heading(ui, main_heading);
        if mode.is_some() {
            ui.add_space(space::XXS);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(space::XS + 2.0, space::XS + 2.0);
                for part in &job.main {
                    chip(ui, chosen, part, checked_main <= 1);
                }
            });
        } else {
            for part in &job.main {
                toggle(ui, chosen, part, checked_main <= 1);
            }
        }
    });
    if let Some(mode) = mode {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::XXS;
            heading(ui, &tr("project-main-texture"));
            let label = ui.label(RichText::new(mode).color(color::TEXT_PRIMARY));
            label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, mode));
        });
    }
    if !job.accessories.is_empty() {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::XXS;
            heading(ui, &tr("vehicles-accessories"));
            for part in &job.accessories {
                toggle(ui, chosen, part, false);
            }
        });
    }
}

/// Adds texture `id` to the `chosen` ones, keeping the package's `order`,
/// or removes it.
fn set_painted(chosen: &mut Vec<String>, order: &[&str], id: &str, on: bool) {
    if on {
        if !chosen.iter().any(|c| c == id) {
            chosen.push(id.to_owned());
        }
        chosen.sort_by_key(|c| order.iter().position(|o| o == c));
    } else {
        chosen.retain(|c| c != id);
    }
}

/// The vehicles passing `filter` (minus `exclude`), one checkbox row
/// each; each checked vehicle shows its textures as checkboxes under its row.
pub fn vehicle_list(
    ui: &mut Ui,
    library: &VehicleLibrary,
    filter: &VehicleFilter,
    exclude: &[String],
    choices: &mut Vec<VehicleChoice>,
) {
    use super::new_project::{Row, vehicle_row};
    let rows = vehicle_rows(library, filter, exclude);
    ui.spacing_mut().item_spacing.y = 0.0;
    let textures = |ui: &mut Ui, choices: &mut Vec<VehicleChoice>, id: &str| {
        if let Some(c) = choices.iter_mut().find(|c| c.id == id)
            && let Some(m) = c.manifest(library)
        {
            ui.add_space(space::XS);
            ui.indent(("textures", id), |ui| {
                ui.spacing_mut().item_spacing.y = space::XS;
                texture_checkboxes(ui, m, &mut c.textures);
            });
            ui.add_space(space::SM);
        }
    };
    for vehicle in rows {
        let m = &vehicle.newest().manifest;
        let checked = choices.iter().any(|c| c.id == vehicle.id);
        let cells = Row {
            name: &m.name,
            kind: m.kind,
            package: format!("{} {}", m.brand, m.version),
            tip: summary(m),
        };
        if vehicle_row(ui, &cells, checked) {
            toggle_choice(choices, vehicle);
        }
        textures(ui, choices, &vehicle.id);
    }
}

/// Installs `paths`; returns one line per file (success or why it failed).
pub fn install(state: &mut AppState, paths: &[std::path::PathBuf]) -> Vec<Result<String, String>> {
    let files = paths
        .iter()
        .map(|path| {
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            (name, std::fs::read(path).map_err(|e| e.to_string()))
        })
        .collect();
    install_named(state, files)
}

/// Installs files given by name and content.
pub fn install_named(
    state: &mut AppState,
    files: Vec<(String, Result<Vec<u8>, String>)>,
) -> Vec<Result<String, String>> {
    files
        .into_iter()
        .map(|(file, bytes)| {
            let result = match bytes {
                Ok(bytes) => state.vehicles.install_bytes(&bytes),
                Err(e) => Err(crate::vehicles::InstallError::Io(std::io::Error::other(e))),
            };
            match result {
                Ok(m) => Ok(tr!(
                    "vehicles-installed",
                    name = m.name.as_str(),
                    version = m.version.to_string()
                )),
                Err(err) => Err(crate::vehicles::install_error_message(&err, &file)),
            }
        })
        .collect()
}

/// Installs the built-in sample vehicles; returns their result lines.
pub fn install_samples(state: &mut AppState) -> Vec<Result<String, String>> {
    install_named(
        state,
        crate::vehicles::SAMPLES
            .iter()
            .map(|s| (s.file.to_owned(), Ok(s.bytes.to_vec())))
            .collect(),
    )
}

/// The tabs of the Vehicle Library.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LibraryTab {
    /// Every installed vehicle.
    #[default]
    Installed,
    /// The custom vehicles.
    Mine,
    /// The official vehicles of the player's own game.
    Game,
}

/// State of the Vehicle Library dialog.
#[derive(Clone, Debug, Default)]
pub struct LibraryDialog {
    pub filter: VehicleFilter,
    /// Search and kind filter of the My Game tab.
    pub game_filter: VehicleFilter,
    pub tab: LibraryTab,
    /// The vehicle shown in the detail pane (the first listed when unset
    /// or no longer listed).
    pub selected: Option<String>,
    /// Version waiting for removal confirmation: id, version, name.
    pub confirm_remove: Option<(String, semver::Version, String)>,
    /// Results of the last installation.
    pub messages: Vec<Result<String, String>>,
}

/// A vehicle's status in the Vehicle Library.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleStatus {
    /// A custom vehicle.
    Custom,
    /// The open project holds it at an older version than the newest
    /// installed.
    Update,
    UpToDate,
}

impl VehicleStatus {
    /// The status of `vehicle` for the open project's vehicles.
    pub fn of(
        vehicle: &InstalledVehicle,
        library: &VehicleLibrary,
        project: &[tp_core::ProjectVehicle],
    ) -> Self {
        if tp_pack::custom::is_custom(&vehicle.id) {
            Self::Custom
        } else if project
            .iter()
            .any(|v| v.package_id == vehicle.id && library.update_for(v).is_some())
        {
            Self::Update
        } else {
            Self::UpToDate
        }
    }

    /// Icon, text and color: the icon and the text say it, not only the
    /// color.
    fn look(self) -> (&'static str, String, egui::Color32) {
        match self {
            Self::Custom => (icons::LINKED, tr("vehicles-status-custom"), color::LINK),
            Self::Update => (icons::UPDATE, tr("vehicles-status-update"), color::SIGNAL),
            Self::UpToDate => (
                icons::CHECK,
                tr("vehicles-status-current"),
                color::TEXT_SECONDARY,
            ),
        }
    }

    /// Text, text color and outline of the status pill of a list row: the
    /// text says it; the update and custom statuses in their accent, up to
    /// date muted.
    fn pill(self) -> (String, egui::Color32, egui::Color32) {
        match self {
            Self::Custom => (tr("vehicles-status-custom"), color::LINK, color::LINK),
            Self::Update => (tr("vehicles-status-update"), color::SIGNAL, color::SIGNAL),
            Self::UpToDate => (
                tr("vehicles-status-current"),
                color::TEXT_MUTED,
                color::BORDER_STRONG,
            ),
        }
    }
}

/// Width of the Vehicle Library's detail pane.
const DETAIL_WIDTH: f32 = 380.0;
/// Height of a row of the Vehicle Library's list, and its thumbnail.
const LIBRARY_ROW: f32 = 69.0;
const LIBRARY_THUMB: egui::Vec2 = egui::vec2(64.0, 44.0);

/// What the newest version of the shown vehicle changes, and for whom.
#[derive(Default)]
struct UpdateView {
    /// The newest version, its changelog and what it changes compared with
    /// the version before it (only with several versions installed).
    summary: Option<(String, Option<String>, tp_vehicles::ManifestDiff)>,
    /// One line per known project using an older version.
    projects: Vec<String>,
    /// Recent projects are still being read.
    reading: bool,
    /// The open project uses the vehicle at an older version.
    can_update: bool,
}

/// "Transports Ardent: 1 texture, 3 objects to reposition".
fn impact_line(name: &str, impact: &crate::vehicle_project::ProjectImpact) -> String {
    let mut parts = Vec::new();
    if impact.textures > 0 {
        parts.push(tr!("vehicles-impact-textures", count = impact.textures));
        parts.push(tr!("update-objects", count = impact.objects));
    }
    if impact.resized > 0 {
        parts.push(tr!("vehicles-impact-resized", count = impact.resized));
    }
    if impact.removed > 0 {
        parts.push(tr!("vehicles-impact-removed", count = impact.removed));
    }
    if parts.is_empty() {
        parts.push(tr("vehicles-impact-nothing"));
    }
    tr!(
        "vehicles-impact-line",
        name = name,
        impact = parts.join(", ")
    )
}

/// The recent project files that still exist, each once, without the open
/// project's.
fn recent_paths(state: &AppState) -> Vec<std::path::PathBuf> {
    let canonical = |p: &std::path::Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let open = state
        .workspace()
        .and_then(|ws| ws.path.as_deref())
        .map(canonical);
    let mut seen = std::collections::HashSet::new();
    state
        .prefs
        .recent
        .iter()
        .filter(|r| crate::prefs::recent_exists(&r.path))
        .map(|r| canonical(&r.path))
        .filter(|p| Some(p) != open.as_ref() && seen.insert(p.clone()))
        .collect()
}

/// The update view of `vehicle`: the open project counted from memory,
/// then the recent projects as they are read.
fn update_view(
    state: &AppState,
    vehicle: &InstalledVehicle,
    paths: &[std::path::PathBuf],
) -> UpdateView {
    use crate::project_summaries::Known;
    let newest = &vehicle.newest().manifest;
    let mut view = UpdateView {
        summary: vehicle.versions.get(1).map(|previous| {
            (
                newest.version.to_string(),
                newest.changelog.clone(),
                tp_vehicles::diff(&previous.manifest, newest),
            )
        }),
        ..UpdateView::default()
    };
    if let Some(ws) = state.workspace() {
        let summary = tp_file::Summary::of(&ws.project);
        if let Some(impact) = crate::vehicle_project::impact(&summary, newest) {
            view.can_update = true;
            view.projects.push(impact_line(&summary.name, &impact));
        }
    }
    for path in paths {
        match state.project_summaries.get(path) {
            None => view.reading = true,
            Some(Known::Read(summary)) => {
                if let Some(impact) = crate::vehicle_project::impact(summary, newest) {
                    view.projects.push(impact_line(&summary.name, &impact));
                }
            }
            Some(Known::Unreadable) => {
                let name = path
                    .file_stem()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                view.projects
                    .push(tr!("vehicles-impact-unreadable", name = name.as_str()));
            }
        }
    }
    view
}

/// What the Vehicle Library asks for this frame.
#[derive(Default)]
struct LibraryClicks {
    install: bool,
    /// The My Game tab was asked for (the empty state's button).
    game: bool,
    /// Preferences… in the My Game tab.
    preferences: bool,
    sample: bool,
    custom: bool,
    close: bool,
    /// The removal asked for is confirmed.
    confirm: bool,
    new_version: Option<String>,
    update: Option<String>,
    /// Update in the update summary: the shown vehicle of the open project.
    update_open: bool,
    export: Option<(std::path::PathBuf, String)>,
    remove: Option<(String, semver::Version, String)>,
}

/// Shows the Vehicle Library dialog; returns false once it is closed.
pub fn library(ctx: &egui::Context, state: &mut AppState, dialog: &mut LibraryDialog) -> bool {
    let mut keep = true;
    let mut clicks = LibraryClicks::default();
    let screen = ctx.content_rect();
    let width = (screen.width() - 96.0).clamp(760.0, 1100.0);
    let body_height = (screen.height() - 280.0).clamp(300.0, 600.0);
    let project: Vec<tp_core::ProjectVehicle> = state
        .workspace()
        .map(|ws| ws.project.vehicles.clone())
        .unwrap_or_default();
    let library = &state.vehicles;
    let installed = library.vehicles().len();
    let mine = library
        .vehicles()
        .iter()
        .filter(|v| tp_pack::custom::is_custom(&v.id))
        .count();
    let listed: Vec<&InstalledVehicle> = filtered(library, &dialog.filter)
        .into_iter()
        .filter(|v| dialog.tab == LibraryTab::Installed || tp_pack::custom::is_custom(&v.id))
        .collect();
    let shown = listed
        .iter()
        .copied()
        .find(|v| dialog.selected.as_ref() == Some(&v.id))
        .or(listed.first().copied());
    // The shown vehicle's preview first, then the listed ones' (for their
    // thumbnails), one package read at a time.
    for v in shown.into_iter().chain(listed.iter().copied()) {
        let newest = v.newest();
        state
            .package_previews
            .update(ctx, &newest.manifest, &newest.path);
    }
    let shown_id = shown.map(|v| v.id.clone());
    let paths = recent_paths(state);
    state.project_summaries.update(ctx, &paths);
    let view = shown
        .map(|v| update_view(state, v, &paths))
        .unwrap_or_default();
    let previews = &state.package_previews;

    super::dialogs::modal("vehicle_library_modal").show(ctx, |ui| {
        ui.set_width(width);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::XL;
            super::dialogs::title(ui, tr("cmd-vehicle-library").trim_end_matches('…'), None);
            let (all, custom, my_game) = (
                tr!("vehicles-tab-installed", count = installed),
                tr!("vehicles-tab-mine", count = mine),
                tr("vehicles-tab-game"),
            );
            if let Some(tab) = SegmentedControl::new()
                .segment(LibraryTab::Installed, "", &all, None)
                .segment(LibraryTab::Mine, "", &custom, None)
                .segment(LibraryTab::Game, "", &my_game, None)
                .show(ui, dialog.tab)
            {
                dialog.tab = tab;
                dialog.selected = None;
                // The folder is checked again when the tab opens.
                state.game.folders.clear();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = space::SM;
                clicks.custom |= ui
                    .add(secondary_button(&tr("custom-open-library")))
                    .clicked();
                clicks.install |= ui.add(secondary_button(&tr("vehicles-install"))).clicked();
            });
        });
        ui.add_space(space::SM);
        super::dialogs::messages(ui, &dialog.messages);
        ui.add_space(space::SM);
        if dialog.tab == LibraryTab::Game {
            clicks.preferences |= super::game_tab::show(
                ui,
                body_height,
                &mut state.game,
                library,
                &mut state.prefs,
                &mut dialog.game_filter,
            )
            .preferences;
        } else if installed == 0 {
            ui.allocate_ui(egui::vec2(ui.available_width(), body_height), |ui| {
                ui.vertical_centered(|ui| {
                    EmptyState::new(
                        icons::VEHICLE,
                        &tr("vehicles-empty"),
                        &tr("vehicles-empty-hint"),
                    )
                    .show(ui);
                    clicks.sample |= ui
                        .add(primary_button(&tr("vehicles-install-sample")))
                        .clicked();
                    clicks.game |= ui.add(secondary_button(&tr("vehicles-tab-game"))).clicked();
                });
            });
        } else {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = space::XL;
                let list_width = ui.available_width() - DETAIL_WIDTH - space::XL;
                ui.allocate_ui_with_layout(
                    egui::vec2(list_width, body_height),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(list_width);
                        ui.set_min_height(body_height);
                        dialog.filter.show(ui, "library");
                        ui.add_space(space::SM);
                        if dialog.tab == LibraryTab::Mine && mine == 0 {
                            ui.add_space(space::LG);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(tr("vehicles-mine-empty"))
                                        .color(color::TEXT_SECONDARY),
                                )
                                .wrap(),
                            );
                        } else if listed.is_empty() {
                            ui.add_space(space::LG);
                            ui.label(
                                RichText::new(tr("new-project-no-match"))
                                    .color(color::TEXT_SECONDARY),
                            );
                        } else {
                            ScrollArea::vertical()
                                .id_salt("vehicle_library_list")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    for v in &listed {
                                        let selected = shown.is_some_and(|s| s.id == v.id);
                                        let status = VehicleStatus::of(v, library, &project);
                                        if library_row(
                                            ui,
                                            &v.newest().manifest,
                                            status,
                                            selected,
                                            previews,
                                        )
                                        .clicked()
                                        {
                                            dialog.selected = Some(v.id.clone());
                                        }
                                    }
                                });
                        }
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(DETAIL_WIDTH, body_height),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(DETAIL_WIDTH);
                        ui.set_min_height(body_height);
                        if let Some(v) = shown {
                            let status = VehicleStatus::of(v, library, &project);
                            detail(ui, v, status, previews, &view, &mut clicks);
                        }
                    },
                );
            });
        }
        if let Some((_, _, name)) = &dialog.confirm_remove {
            ui.add_space(space::SM);
            ui.label(
                RichText::new(tr!("vehicles-remove-confirm", name = name.as_str()))
                    .color(color::WARNING),
            );
        }
        super::dialogs::footer(ui, |ui| match &dialog.confirm_remove {
            Some(_) => {
                clicks.confirm |= ui.add(primary_button(&tr("vehicles-remove"))).clicked();
                if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                    dialog.confirm_remove = None;
                }
            }
            None => {
                clicks.close |= ui.add(primary_button(&tr("button-close"))).clicked();
            }
        });
    });
    keep &= !clicks.close;
    if clicks.game {
        dialog.tab = LibraryTab::Game;
        state.game.folders.clear();
    }
    if clicks.preferences {
        state.open_preferences();
        return true;
    }
    if clicks.confirm
        && let Some((id, version, _)) = dialog.confirm_remove.take()
        && let Err(err) = state.vehicles.remove(&id, &version)
    {
        dialog.messages = vec![Err(err.to_string())];
    }
    if let Some(r) = clicks.remove {
        dialog.confirm_remove = Some(r);
    }
    if clicks.sample {
        dialog.messages = install_samples(state);
    }
    if let Some((from, suggested)) = clicks.export
        && let Some(to) = state.dialogs.save_package(&suggested)
    {
        let written = std::fs::read(&from)
            .map_err(|e| e.to_string())
            .and_then(|bytes| tp_file::write_atomic(&to, &bytes).map_err(|e| e.to_string()));
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
    let update = clicks
        .update
        .or_else(|| clicks.update_open.then(|| shown_id.clone()).flatten());
    if let Some(id) = update {
        state.open_update_dialog(Some(&id));
        return false;
    }
    if clicks.custom {
        super::custom_vehicle::open(
            state,
            super::custom_vehicle::Origin::Library(std::mem::take(dialog)),
        );
        return true;
    }
    if let Some(id) = clicks.new_version {
        super::custom_vehicle::open_new_version(state, &id, std::mem::take(dialog));
        return true;
    }
    if clicks.install {
        let paths = state.dialogs.pick_packages();
        if !paths.is_empty() {
            dialog.messages = install(state, &paths);
        }
    }
    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    if escape {
        if dialog.confirm_remove.is_some() {
            dialog.confirm_remove = None;
        } else {
            keep = false;
        }
    }
    keep
}

/// A vehicle of the library's list: its thumbnail (the package's preview),
/// its name, brand, kind and game, its newest version and its status. The
/// selected row is a full-width band on the control surface, its name in
/// ink.
fn library_row(
    ui: &mut Ui,
    m: &Manifest,
    status: VehicleStatus,
    selected: bool,
    previews: &crate::package_previews::PackagePreviews,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), LIBRARY_ROW),
        egui::Sense::click(),
    );
    response.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, selected, &m.name));
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 0, color::CONTROL);
    } else if response.hovered() {
        painter.rect_filled(rect, 0, color::SURFACE_2);
    }
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        egui::Stroke::new(1.0, color::BORDER),
    );

    // The thumbnail: the middle of the preview, or a placeholder.
    let thumb = egui::Rect::from_min_size(
        egui::pos2(
            rect.left() + space::MD,
            rect.center().y - LIBRARY_THUMB.y / 2.0,
        ),
        LIBRARY_THUMB,
    );
    match previews.get(&m.id, &m.version) {
        crate::package_previews::Preview::Ready(texture) => {
            let size = texture.size_vec2();
            let uv = cover_uv(size, thumb.size());
            painter.rect_filled(thumb, radius::MD, tp_ui::tokens::canvas::ARTBOARD);
            egui::Image::new((texture.id(), thumb.size()))
                .uv(uv)
                .corner_radius(radius::MD)
                .paint_at(ui, thumb);
        }
        _ => {
            painter.rect_filled(thumb, radius::MD, color::CONTROL);
        }
    }
    let painter = ui.painter();
    painter.rect_stroke(
        thumb,
        radius::MD,
        egui::Stroke::new(1.0, color::OUTLINE),
        egui::StrokeKind::Inside,
    );

    // The pill sits at the right of a slot as wide as the widest status,
    // so the versions line up whatever the status.
    let right = rect.right() - space::LG;
    status_pill(ui, egui::pos2(right, rect.center().y), status);
    let slot = [
        VehicleStatus::Custom,
        VehicleStatus::Update,
        VehicleStatus::UpToDate,
    ]
    .into_iter()
    .map(|s| pill_size(ui, s).x)
    .fold(0.0, f32::max);
    let painter = ui.painter();
    let version = painter.text(
        egui::pos2(right - slot - space::XL, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        m.version.to_string(),
        egui::FontId::monospace(tp_ui::tokens::typography::MONO),
        color::TEXT_SECONDARY,
    );
    let left = thumb.right() + space::LG;
    let clip = egui::Rect::from_min_max(
        egui::pos2(left, rect.top()),
        egui::pos2(version.left() - space::MD, rect.bottom()),
    );
    let painter = painter.with_clip_rect(clip.intersect(ui.clip_rect()));
    painter.text(
        egui::pos2(left, rect.center().y - 9.0),
        egui::Align2::LEFT_CENTER,
        &m.name,
        egui::FontId::proportional(tp_ui::tokens::typography::BODY + 1.0),
        if selected {
            color::TEXT_PRIMARY
        } else {
            color::TEXT_SECONDARY
        },
    );
    painter.text(
        egui::pos2(left, rect.center().y + 10.0),
        egui::Align2::LEFT_CENTER,
        format!(
            "{} · {} · {}",
            m.brand,
            kind_label(m.kind),
            game_label(m.game.id)
        ),
        egui::FontId::proportional(tp_ui::tokens::typography::CONTROL),
        color::TEXT_MUTED,
    );
    tp_ui::widgets::paint_focus_ring(ui, rect.shrink(1.0), &response, 0);
    response
}

/// The part of an image of `size` that covers a box of `target` (centered).
fn cover_uv(size: egui::Vec2, target: egui::Vec2) -> egui::Rect {
    let image = size.x / size.y.max(1.0);
    let wanted = target.x / target.y.max(1.0);
    let (w, h) = if image > wanted {
        (wanted / image, 1.0)
    } else {
        (1.0, image / wanted)
    };
    egui::Rect::from_center_size(egui::pos2(0.5, 0.5), egui::vec2(w, h))
}

/// The laid out text of a status pill, and its padding.
fn pill_text(ui: &Ui, status: VehicleStatus) -> (std::sync::Arc<egui::Galley>, egui::Vec2) {
    let (text, tint, _) = status.pill();
    let label = ui.painter().layout_no_wrap(
        text,
        egui::FontId::proportional(tp_ui::tokens::typography::CONTROL),
        tint,
    );
    (label, egui::vec2(space::SM, space::XXS + 1.0))
}

/// The size of a status pill.
fn pill_size(ui: &Ui, status: VehicleStatus) -> egui::Vec2 {
    let (label, pad) = pill_text(ui, status);
    label.size() + 2.0 * pad
}

/// A status as an outlined pill ending at `right`: its text in its color
/// (see [`VehicleStatus::pill`]). Returns its rectangle.
fn status_pill(ui: &Ui, right: egui::Pos2, status: VehicleStatus) -> egui::Rect {
    let (label, pad) = pill_text(ui, status);
    let (_, tint, outline) = status.pill();
    let painter = ui.painter();
    let size = label.size() + 2.0 * pad;
    let rect =
        egui::Rect::from_min_size(egui::pos2(right.x - size.x, right.y - size.y / 2.0), size);
    painter.rect_stroke(
        rect,
        rect.height() / 2.0,
        egui::Stroke::new(1.0, outline),
        egui::StrokeKind::Inside,
    );
    painter.galley(rect.min + pad, label, tint);
    rect
}

/// The detail pane: the template preview, the vehicle's name, kind and
/// game, its status, versions and paint job, and its actions.
fn detail(
    ui: &mut Ui,
    vehicle: &InstalledVehicle,
    status: VehicleStatus,
    previews: &crate::package_previews::PackagePreviews,
    view: &UpdateView,
    clicks: &mut LibraryClicks,
) {
    use crate::package_previews::Preview;
    let m = &vehicle.newest().manifest;
    let rect = ui.max_rect();
    ui.painter().rect(
        rect,
        radius::LG,
        color::SURFACE_1,
        egui::Stroke::new(1.0, color::BORDER),
        egui::StrokeKind::Inside,
    );
    egui::Frame::new()
        .inner_margin(egui::Margin::same(space::LG as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ScrollArea::vertical()
                .id_salt("vehicle_library_detail")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = space::SM;
                    // The template preview, whole, on the canvas color.
                    let (area, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 180.0),
                        egui::Sense::hover(),
                    );
                    let preview_name = tr!("vehicles-preview", name = m.name.as_str());
                    response.widget_info(|| {
                        WidgetInfo::labeled(WidgetType::Image, true, &preview_name)
                    });
                    ui.painter().rect_filled(area, radius::MD, color::SURFACE_0);
                    match previews.get(&m.id, &m.version) {
                        Preview::Ready(texture) => {
                            let size = texture.size_vec2();
                            let scale = (area.width() / size.x).min(area.height() / size.y);
                            let shown = egui::Rect::from_center_size(area.center(), size * scale);
                            ui.painter().image(
                                texture.id(),
                                shown,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                egui::Color32::WHITE,
                            );
                        }
                        Preview::Loading | Preview::None => {
                            ui.painter().text(
                                area.center(),
                                egui::Align2::CENTER_CENTER,
                                icons::VEHICLE,
                                icons::font(32.0),
                                color::TEXT_DISABLED,
                            );
                        }
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = space::XXS;
                        let title = ui.label(
                            RichText::new(&m.name)
                                .size(18.0)
                                .family(tp_ui::fonts::semibold_family())
                                .color(color::TEXT_PRIMARY),
                        );
                        // The list's row already says the name.
                        title.widget_info(|| {
                            WidgetInfo::labeled(
                                WidgetType::Label,
                                true,
                                tr!("vehicles-details-of", name = m.name.as_str()),
                            )
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} · {} · {}",
                                m.brand,
                                kind_label(m.kind),
                                game_name(m.game.id)
                            ))
                            .color(color::TEXT_SECONDARY),
                        );
                    });
                    let (icon, text, tint) = status.look();
                    let status_text = format!("{icon} {text}");
                    let label = ui.label(RichText::new(&status_text).color(tint));
                    label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
                    let details = details_text(m);
                    ui.add(
                        egui::Label::new(RichText::new(details).color(color::TEXT_SECONDARY))
                            .wrap(),
                    );
                    ui.horizontal_wrapped(|ui| {
                        if status == VehicleStatus::Update
                            && ui.add(primary_button(&tr("cmd-update-template"))).clicked()
                        {
                            clicks.update = Some(vehicle.id.clone());
                        }
                        if tp_pack::custom::is_custom(&vehicle.id) {
                            let name = tr!("vehicles-new-version-of", name = m.name.as_str());
                            let button = ui.add(secondary_button(&tr("vehicles-new-version")));
                            button.widget_info(|| {
                                WidgetInfo::labeled(WidgetType::Button, true, &name)
                            });
                            if button.clicked() {
                                clicks.new_version = Some(vehicle.id.clone());
                            }
                        }
                    });
                    update_section(ui, &m.name, view, clicks);
                    ui.add_space(space::XS);
                    super::dialogs::field_label(ui, &tr("vehicles-installed-versions"));
                    for v in &vehicle.versions {
                        ui.horizontal(|ui| {
                            let version = v.manifest.version.to_string();
                            ui.label(
                                RichText::new(tr!("vehicles-version", version = version.as_str()))
                                    .color(color::TEXT_PRIMARY),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.spacing_mut().item_spacing.x = space::XS;
                                let name = tr!(
                                    "vehicles-remove-version",
                                    name = m.name.as_str(),
                                    version = version.as_str()
                                );
                                let remove = ui.add(
                                    secondary_button(&tr("vehicles-remove"))
                                        .min_size(egui::vec2(0.0, 26.0)),
                                );
                                remove.widget_info(|| {
                                    WidgetInfo::labeled(WidgetType::Button, true, &name)
                                });
                                if remove.on_hover_text(&name).clicked() {
                                    clicks.remove = Some((
                                        vehicle.id.clone(),
                                        v.manifest.version.clone(),
                                        format!("{} {version}", m.name),
                                    ));
                                }
                                let name = tr!(
                                    "vehicles-export-version",
                                    name = m.name.as_str(),
                                    version = version.as_str()
                                );
                                let export = ui.add(
                                    secondary_button(&tr("vehicles-export"))
                                        .min_size(egui::vec2(0.0, 26.0)),
                                );
                                export.widget_info(|| {
                                    WidgetInfo::labeled(WidgetType::Button, true, &name)
                                });
                                if export.on_hover_text(&name).clicked() {
                                    clicks.export = Some((
                                        v.path.clone(),
                                        format!(
                                            "{}-{version}.{}",
                                            vehicle.id,
                                            tp_vehicles::EXTENSION
                                        ),
                                    ));
                                }
                            });
                        });
                    }
                    ui.add(
                        egui::Label::new(
                            RichText::new(tr("custom-scs-reminder"))
                                .small()
                                .color(color::TEXT_SECONDARY),
                        )
                        .wrap(),
                    );
                });
        });
}

/// The newest version's update summary, with Later and Update, and the
/// known projects it touches.
fn update_section(ui: &mut Ui, name: &str, view: &UpdateView, clicks: &mut LibraryClicks) {
    let small = |ui: &mut Ui, text: String| {
        ui.add(egui::Label::new(RichText::new(text).color(color::TEXT_SECONDARY)).wrap());
    };
    if let Some((version, changelog, diff)) = &view.summary {
        ui.add_space(space::XS);
        super::dialogs::field_label(
            ui,
            &tr!("vehicles-update-summary", version = version.as_str()),
        );
        if let Some(changelog) = changelog {
            ui.add(egui::Label::new(changelog.as_str()).wrap());
        }
        for (key, names) in [
            ("vehicles-changes-layout", &diff.layout_changed),
            ("vehicles-changes-resized", &diff.resized),
            ("vehicles-changes-added", &diff.added),
            ("vehicles-changes-removed", &diff.removed),
        ] {
            if !names.is_empty() {
                small(ui, tr!(key, names = names.join(", ")));
            }
        }
        if diff.is_empty() && changelog.is_none() {
            small(ui, tr("vehicles-no-changes"));
        }
    }
    if !view.projects.is_empty() || view.reading {
        ui.add_space(space::XS);
        super::dialogs::field_label(ui, &tr("vehicles-projects"));
        for line in &view.projects {
            ui.add(egui::Label::new(line.as_str()).wrap());
        }
        if view.reading {
            small(ui, tr("vehicles-projects-reading"));
        }
        ui.label(
            RichText::new(tr("vehicles-recent-only"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
    }
    if view.summary.is_some() {
        ui.horizontal_wrapped(|ui| {
            clicks.close |= ui.add(secondary_button(&tr("vehicles-later"))).clicked();
            let update = ui.add_enabled(view.can_update, primary_button(&tr("update-apply")));
            let spoken = tr!("vehicles-update-vehicle", name = name);
            update
                .widget_info(|| WidgetInfo::labeled(WidgetType::Button, view.can_update, &spoken));
            if update
                .on_disabled_hover_text(tr("vehicles-update-hint"))
                .clicked()
            {
                clicks.update_open = true;
            }
        });
        if !view.can_update {
            small(ui, tr("vehicles-update-hint"));
        }
    }
}

/// State of the Update Template dialog.
#[derive(Clone, Debug)]
pub struct UpdateDialog {
    pub package: Box<Package>,
    pub plan: UpdatePlan,
    /// New textures checked to be added.
    pub added: Vec<String>,
}

impl UpdateDialog {
    /// The dialog for `plan`, with the new textures checked by default.
    pub fn new(package: Package, plan: UpdatePlan) -> Self {
        let added = plan
            .new
            .iter()
            .filter(|n| n.checked_by_default())
            .map(|n| n.id.clone())
            .collect();
        Self {
            package: Box::new(package),
            plan,
            added,
        }
    }
}

/// One line describing a change.
pub fn change_text(change: &TextureChange) -> String {
    match change {
        TextureChange::Replaced {
            name,
            layout_changed,
            resized,
            objects,
        } => {
            let objects = tr!("update-objects", count = *objects);
            match (layout_changed, resized) {
                (layout_changed, Some((old, new))) => {
                    let resized = tr!(
                        "update-resized",
                        name = name.as_str(),
                        old = tp_core::size_label(*old),
                        new = tp_core::size_label(*new)
                    );
                    if *layout_changed {
                        format!("{resized}, {objects}")
                    } else {
                        resized
                    }
                }
                (true, None) => tr!(
                    "update-layout-changed",
                    name = name.as_str(),
                    objects = objects.as_str()
                ),
                (false, None) => tr!("update-replaced", name = name.as_str()),
            }
        }
        TextureChange::Removed { name } => tr!("update-removed", name = name.as_str()),
    }
}

/// Shows the Update Template dialog; returns false once it is closed.
pub fn update(ctx: &egui::Context, state: &mut AppState, dialog: &mut UpdateDialog) -> bool {
    let mut keep = true;
    let mut apply = false;
    super::dialogs::modal("update_template_modal").show(ctx, |ui| {
        ui.set_width(480.0);
        super::dialogs::title(ui, &tr("undo-update-template"), None);
        ui.add_space(space::SM);
        ui.label(tr!(
            "update-versions",
            name = dialog.package.manifest.name.as_str(),
            from = dialog.plan.from.as_str(),
            to = dialog.plan.to.as_str()
        ));
        if let Some(changelog) = &dialog.plan.changelog {
            ui.add_space(space::SM);
            ui.label(
                RichText::new(tr!("update-changelog", version = dialog.plan.to.as_str()))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
            ui.add(egui::Label::new(changelog.as_str()).wrap());
        }
        ui.add_space(space::SM);
        for change in &dialog.plan.changes {
            ui.label(RichText::new(format!("• {}", change_text(change))).small());
        }
        if !dialog.plan.new.is_empty() {
            ui.add_space(space::SM);
            ui.label(RichText::new(tr("update-new-textures")).small());
            for new in &dialog.plan.new {
                let mut on = new.always || dialog.added.contains(&new.id);
                let response = ui.add_enabled(!new.always, SquareCheckbox::new(&mut on, &new.name));
                if response.changed() {
                    if on {
                        dialog.added.push(new.id.clone());
                    } else {
                        dialog.added.retain(|id| *id != new.id);
                    }
                }
            }
        }
        ui.add_space(space::SM);
        ui.label(
            RichText::new(tr("update-artwork-kept"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        super::dialogs::footer(ui, |ui| {
            apply |= ui.add(primary_button(&tr("update-apply"))).clicked();
            keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
    });
    if apply {
        let now = ctx.input(|i| i.time);
        if let Some(ws) = state.workspace_mut() {
            ws.apply_update(&dialog.package, &dialog.added, now);
        }
        keep = false;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        keep = false;
    }
    keep
}

/// State of the Add Vehicle dialog.
#[derive(Clone, Debug, Default)]
pub struct AddVehicleDialog {
    pub filter: VehicleFilter,
    /// The checked vehicles, in the order they were checked.
    pub choices: Vec<VehicleChoice>,
    pub messages: Vec<Result<String, String>>,
}

impl AddVehicleDialog {
    /// The dialog for a project of `game`.
    pub fn for_game(game: Option<&str>) -> Self {
        let game = match game {
            Some("ats") => Some(Game::Ats),
            Some(_) => Some(Game::Ets2),
            None => None,
        };
        Self {
            filter: VehicleFilter {
                game,
                game_locked: game.is_some(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Shows the Add Vehicle dialog; returns false once it is closed.
pub fn add_vehicle(
    ctx: &egui::Context,
    state: &mut AppState,
    dialog: &mut AddVehicleDialog,
) -> bool {
    let mut keep = true;
    let mut add = false;
    let mut install_clicked = false;
    let mut custom_clicked = false;
    let exclude: Vec<String> = state
        .workspace()
        .map(|ws| {
            ws.project
                .vehicles
                .iter()
                .map(|v| v.package_id.clone())
                .collect()
        })
        .unwrap_or_default();
    super::dialogs::modal("add_vehicle_modal").show(ctx, |ui| {
        ui.set_width(560.0);
        super::dialogs::title(ui, tr("cmd-add-vehicle").trim_end_matches('…'), None);
        ui.add_space(space::SM);
        super::dialogs::messages(ui, &dialog.messages);
        dialog.filter.show(ui, "add_vehicle");
        ui.add_space(space::SM);
        if vehicle_rows(&state.vehicles, &dialog.filter, &exclude).is_empty() {
            ui.label(
                RichText::new(tr("add-vehicle-none"))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
        } else {
            // A fixed height, as in the New Project wizard; taller on a
            // tall window, for the textures of the checked vehicles.
            ScrollArea::vertical()
                .max_height((ctx.content_rect().height() - 380.0).clamp(320.0, 480.0))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    vehicle_list(
                        ui,
                        &state.vehicles,
                        &dialog.filter,
                        &exclude,
                        &mut dialog.choices,
                    );
                });
        }
        super::dialogs::footer(ui, |ui| {
            let ready = all_complete(&dialog.choices, &state.vehicles);
            add |= ui
                .add_enabled(ready, primary_button(&tr("add-vehicle-add")))
                .on_disabled_hover_text(tr("reason-check-vehicle-to-add"))
                .clicked();
            keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
            install_clicked |= ui.add(secondary_button(&tr("vehicles-install"))).clicked();
            custom_clicked |= ui.add(secondary_button(&tr("custom-open"))).clicked();
        });
    });
    if custom_clicked {
        super::custom_vehicle::open(
            state,
            super::custom_vehicle::Origin::AddVehicle(std::mem::take(dialog)),
        );
        return true;
    }
    if install_clicked {
        let paths = state.dialogs.pick_packages();
        if !paths.is_empty() {
            dialog.messages = install(state, &paths);
        }
    }
    if add {
        let now = ctx.input(|i| i.time);
        match load_choices(&state.vehicles, &dialog.choices) {
            Ok(packages) => {
                let fleet: Vec<(&Package, &[String])> =
                    packages.iter().map(|(p, t)| (p, t.as_slice())).collect();
                if let Some(ws) = state.workspace_mut() {
                    match ws.add_vehicles(&fleet, now) {
                        Ok(()) => keep = false,
                        Err(err) => dialog.messages = vec![Err(fleet_error_message(err))],
                    }
                }
            }
            Err(message) => dialog.messages = vec![Err(message)],
        }
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        keep = false;
    }
    keep
}

/// The packages of the checked vehicles with their checked textures, in
/// order; or why one can't be read.
pub fn load_choices(
    library: &VehicleLibrary,
    choices: &[VehicleChoice],
) -> Result<Vec<(Package, Vec<String>)>, String> {
    choices
        .iter()
        .map(|c| {
            library
                .load(&c.id, &c.version)
                .map(|p| (p, c.textures.clone()))
                .map_err(|err| {
                    crate::vehicles::install_error_message(&err, &format!("{} {}", c.id, c.version))
                })
        })
        .collect()
}

/// Why a fleet change was refused, in the current language.
pub fn fleet_error_message(err: crate::vehicle_project::FleetError) -> String {
    use crate::vehicle_project::FleetError;
    tr(match err {
        FleetError::OtherGame => "fleet-error-other-game",
        FleetError::AlreadyThere => "fleet-error-already-there",
        FleetError::BadTextures => "fleet-error-textures",
        FleetError::WrongVersion => "fleet-error-version",
        FleetError::LastVehicle => "reason-last-vehicle",
        FleetError::Unknown => "fleet-error-unknown",
    })
}

/// State of the Textures dialog of a project's vehicle.
#[derive(Clone, Debug)]
pub struct TexturesDialog {
    pub package_id: String,
    pub name: String,
    pub version: String,
    /// The textures the vehicle paints (part ids).
    pub checked: Vec<String>,
    /// Textures with artwork about to be removed: asks for confirmation.
    pub confirm: Option<Vec<String>>,
}

impl TexturesDialog {
    pub fn new(project: &tp_core::Project, vehicle: &tp_core::ProjectVehicle) -> Self {
        let checked = project.surfaces[project.vehicle_range(&vehicle.package_id)]
            .iter()
            .filter_map(|s| s.template.as_ref().map(|t| t.texture_id.clone()))
            .collect();
        Self {
            package_id: vehicle.package_id.clone(),
            name: vehicle.name.clone(),
            version: vehicle.version.clone(),
            checked,
            confirm: None,
        }
    }
}

/// Shows the Textures dialog; returns false once it is closed.
pub fn textures(ctx: &egui::Context, state: &mut AppState, dialog: &mut TexturesDialog) -> bool {
    let mut keep = true;
    let mut apply = false;
    let mut update = false;
    let installed = state
        .workspace()
        .and_then(|ws| ws.project.vehicle(&dialog.package_id))
        .and_then(|v| state.vehicles.recorded(v))
        .map(|i| i.manifest.clone());
    super::dialogs::modal("textures_modal").show(ctx, |ui| {
        ui.set_width(440.0);
        super::dialogs::title(
            ui,
            &tr!("textures-title", name = dialog.name.as_str()),
            None,
        );
        ui.add_space(space::SM);
        match &installed {
            Some(m) => {
                if dialog.confirm.is_none() {
                    texture_checkboxes(ui, m, &mut dialog.checked);
                }
            }
            None => {
                ui.label(
                    RichText::new(tr!(
                        "textures-missing-version",
                        version = dialog.version.as_str()
                    ))
                    .color(color::WARNING),
                );
            }
        }
        if let Some(names) = &dialog.confirm {
            ui.label(
                RichText::new(tr!("textures-remove-confirm", textures = names.join(", ")))
                    .color(color::WARNING),
            );
        }
        super::dialogs::footer(ui, |ui| match &installed {
            Some(_) => {
                let label = if dialog.confirm.is_some() {
                    tr("textures-remove")
                } else {
                    tr("textures-apply")
                };
                apply |= ui.add(primary_button(&label)).clicked();
                keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
            }
            None => {
                update |= ui.add(primary_button(&tr("cmd-update-template"))).clicked();
                keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
            }
        });
    });
    if apply && let Some(m) = &installed {
        // Textures with artwork being removed ask first.
        let removed_with_art: Vec<String> = state
            .workspace()
            .map(|ws| {
                let p = &ws.project;
                p.surfaces[p.vehicle_range(&dialog.package_id)]
                    .iter()
                    .filter(|s| {
                        s.template.as_ref().is_some_and(|t| {
                            m.paint_job.part(&t.texture_id).is_some()
                                && !dialog.checked.contains(&t.texture_id)
                                && !is_always_painted(m, &t.texture_id)
                        }) && !s.objects.is_empty()
                    })
                    .map(|s| s.name.clone())
                    .collect()
            })
            .unwrap_or_default();
        if dialog.confirm.is_none() && !removed_with_art.is_empty() {
            dialog.confirm = Some(removed_with_art);
        } else {
            let now = ctx.input(|i| i.time);
            let loaded = state.vehicles.load(&m.id, &m.version);
            if let (Ok(package), Some(ws)) = (loaded, state.workspace_mut()) {
                let _ = ws.set_textures(&package, &dialog.checked, now);
            }
            keep = false;
        }
    }
    if update {
        state.modal = None;
        state.open_update_dialog(Some(&dialog.package_id));
        return false;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        if dialog.confirm.is_some() {
            dialog.confirm = None;
        } else {
            keep = false;
        }
    }
    keep
}

/// Asks before removing a vehicle with artwork; returns false once closed.
pub fn remove_vehicle(
    ctx: &egui::Context,
    state: &mut AppState,
    package_id: &str,
    name: &str,
) -> bool {
    let mut keep = true;
    let mut remove = false;
    super::dialogs::modal("remove_vehicle_modal").show(ctx, |ui| {
        ui.set_width(420.0);
        super::dialogs::title(ui, &tr("vehicles-remove-from-project"), None);
        ui.add_space(space::SM);
        ui.label(tr!("remove-vehicle-confirm", name = name));
        super::dialogs::footer(ui, |ui| {
            remove |= ui.add(primary_button(&tr("vehicles-remove"))).clicked();
            keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
    });
    if remove {
        let now = ctx.input(|i| i.time);
        if let Some(ws) = state.workspace_mut() {
            let _ = ws.remove_vehicle(package_id, now);
        }
        keep = false;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        keep = false;
    }
    keep
}

/// State of the Copy from Cabin dialog: the other main textures of the
/// active texture's vehicle, with their object counts, and the chosen one.
#[derive(Clone, Debug)]
pub struct CopyFromCabinDialog {
    /// (surface index, name, object count).
    pub sources: Vec<(usize, String, usize)>,
    pub chosen: usize,
}

impl CopyFromCabinDialog {
    /// The dialog for the active texture; the first source holding objects
    /// is chosen.
    pub fn new(project: &tp_core::Project) -> Self {
        let sources: Vec<(usize, String, usize)> =
            crate::vehicle_project::cabin_sources(project, project.active_surface)
                .into_iter()
                .map(|i| {
                    let s = &project.surfaces[i];
                    (i, s.name.clone(), s.objects.len())
                })
                .collect();
        let chosen = sources
            .iter()
            .find(|(_, _, n)| *n > 0)
            .or(sources.first())
            .map_or(0, |(i, _, _)| *i);
        Self { sources, chosen }
    }
}

/// Shows the Copy from Cabin dialog; returns false once it is closed.
pub fn copy_from_cabin(
    ctx: &egui::Context,
    state: &mut AppState,
    dialog: &mut CopyFromCabinDialog,
) -> bool {
    let mut keep = true;
    let mut copy = false;
    let target = state
        .workspace()
        .map(|ws| ws.project.surface().name.clone())
        .unwrap_or_default();
    super::dialogs::modal("copy_from_cabin_modal").show(ctx, |ui| {
        ui.set_width(420.0);
        super::dialogs::title(ui, tr("cmd-copy-from-cabin").trim_end_matches('…'), None);
        ui.add_space(space::XS);
        ui.label(
            RichText::new(tr!("copy-cabin-hint", texture = target.as_str()))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::SM);
        for (index, name, count) in &dialog.sources {
            let text = format!("{name}   {}", tr!("copy-cabin-objects", count = *count));
            let selected = dialog.chosen == *index;
            let row = ui.selectable_label(selected, text);
            row.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, selected, name));
            if row.clicked() {
                dialog.chosen = *index;
            }
        }
        let empty = dialog
            .sources
            .iter()
            .find(|(i, _, _)| *i == dialog.chosen)
            .is_none_or(|(_, _, n)| *n == 0);
        if empty {
            ui.label(
                RichText::new(tr("copy-cabin-empty"))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
        }
        super::dialogs::footer(ui, |ui| {
            copy |= ui
                .add_enabled(!empty, primary_button(&tr("copy-cabin-copy")))
                .clicked();
            keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
    });
    if copy {
        let now = ctx.input(|i| i.time);
        let source = dialog.chosen;
        if let Some(ws) = state.workspace_mut() {
            ws.copy_from_texture(source, now);
        }
        keep = false;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        keep = false;
    }
    keep
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::NewProjectDraft;
    use crate::vehicles::{SAMPLE_ID, SAMPLE_TRAILER_ID};

    /// A library holding the sample truck and trailer (ETS2).
    fn library(dir: &std::path::Path) -> VehicleLibrary {
        let mut library = VehicleLibrary::open(dir);
        library.install_samples().unwrap();
        library
    }

    fn ids(choices: &[VehicleChoice]) -> Vec<&str> {
        choices.iter().map(|c| c.id.as_str()).collect()
    }

    #[test]
    fn checking_keeps_the_order_and_each_vehicle_once() {
        let dir = tempfile::tempdir().unwrap();
        let library = library(dir.path());
        let truck = library.get(SAMPLE_ID).unwrap();
        let trailer = library.get(SAMPLE_TRAILER_ID).unwrap();
        let mut choices = Vec::new();
        assert!(toggle_choice(&mut choices, trailer));
        assert!(toggle_choice(&mut choices, truck));
        assert_eq!(ids(&choices), [SAMPLE_TRAILER_ID, SAMPLE_ID]);
        // Already checked: stays where it is, with its textures.
        choices[0].textures.clear();
        push_choice(&mut choices, VehicleChoice::of(trailer));
        assert_eq!(ids(&choices), [SAMPLE_TRAILER_ID, SAMPLE_ID]);
        assert!(choices[0].textures.is_empty());
        assert!(
            all_complete(&choices, &library),
            "a trailer's Base is always painted"
        );
        // Unchecking.
        assert!(!toggle_choice(&mut choices, trailer));
        assert_eq!(ids(&choices), [SAMPLE_ID]);
        assert!(!remove_choice(&mut choices, SAMPLE_TRAILER_ID));
        assert!(remove_choice(&mut choices, SAMPLE_ID));
        assert!(!all_complete(&choices, &library), "nothing checked");
        // The other game.
        toggle_choice(&mut choices, truck);
        retain_game(&mut choices, &library, Game::Ets2);
        assert_eq!(ids(&choices), [SAMPLE_ID]);
        retain_game(&mut choices, &library, Game::Ats);
        assert!(choices.is_empty());
    }

    #[test]
    fn new_project_opens_the_card_checked_last() {
        let dir = tempfile::tempdir().unwrap();
        let library = library(dir.path());
        let truck = library.get(SAMPLE_ID).unwrap();
        let trailer = library.get(SAMPLE_TRAILER_ID).unwrap();
        let mut draft = NewProjectDraft::new(&library, &[]);
        assert_eq!(draft.game(), Game::Ets2);
        // The first played game with installed vehicles; ATS has none here.
        let played = ["ats".to_owned(), "ets2".to_owned()];
        assert_eq!(NewProjectDraft::new(&library, &played).game(), Game::Ets2);
        draft.toggle(truck);
        draft.toggle(trailer);
        assert_eq!(ids(&draft.vehicles), [SAMPLE_ID, SAMPLE_TRAILER_ID]);
        assert_eq!(draft.open.as_deref(), Some(SAMPLE_TRAILER_ID));
        // Checking a vehicle already checked opens its card in place.
        draft.check(VehicleChoice::of(truck));
        assert_eq!(ids(&draft.vehicles), [SAMPLE_ID, SAMPLE_TRAILER_ID]);
        assert_eq!(draft.open.as_deref(), Some(SAMPLE_ID));
        // Removing the open one opens the last one left.
        draft.uncheck(SAMPLE_ID);
        assert_eq!(draft.open.as_deref(), Some(SAMPLE_TRAILER_ID));
        // Removing a folded one keeps the open card.
        draft.toggle(truck);
        draft.open = Some(SAMPLE_TRAILER_ID.to_owned());
        draft.toggle(truck);
        assert_eq!(draft.open.as_deref(), Some(SAMPLE_TRAILER_ID));
        // Unchecking the last one: no card.
        draft.toggle(trailer);
        assert!(draft.vehicles.is_empty());
        assert_eq!(draft.open_choice(), None);
        // Changing the game unchecks the other game's vehicles.
        draft.toggle(truck);
        draft.toggle(trailer);
        draft.set_game(Game::Ats, &library);
        assert!(draft.vehicles.is_empty());
        assert_eq!(draft.open, None);
        assert_eq!(draft.game(), Game::Ats);
    }
}
