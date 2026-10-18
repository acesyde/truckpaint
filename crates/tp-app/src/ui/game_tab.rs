//! The My Game tab of the Vehicle Library: the official vehicles of the
//! player's own game, with a checkbox each, and one Generate button.

use egui::{Align, Layout, RichText, ScrollArea, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{
    EmptyState, SegmentedControl, SquareCheckbox, primary_button, secondary_button,
};
use tp_vehicles::Game;

use super::vehicle_dialogs::{VehicleFilter, game_label, game_name, kind_label};
use crate::game::{FolderStatus, GameState, Generation, RowState, ScanState, generation};
use crate::vehicles::VehicleLibrary;

/// What the tab asks for this frame.
#[derive(Default)]
pub struct Clicks {
    /// Preferences… was clicked.
    pub preferences: bool,
}

/// The tab's body, `height` tall.
#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    height: f32,
    game: &mut GameState,
    library: &VehicleLibrary,
    prefs: &mut crate::prefs::Prefs,
    filter: &mut VehicleFilter,
) -> Clicks {
    let mut clicks = Clicks::default();
    let selected = game.game;
    let folder = prefs.game_folders.get(selected.code()).cloned();
    if !game.folders.contains_key(&selected) {
        game.refresh_folders(&prefs.game_folders);
    }
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), height),
        Layout::top_down(Align::Min),
        |ui| {
            ui.set_min_height(height);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::MD;
                let (ets2, ats) = (game_label(Game::Ets2), game_label(Game::Ats));
                if let Some(next) = SegmentedControl::new()
                    .segment(Game::Ets2, "", ets2, None)
                    .segment(Game::Ats, "", ats, None)
                    .show(ui, selected)
                {
                    game.game = next;
                }
                if let Some(FolderStatus::Valid(info)) = game.folders.get(&selected) {
                    let text = tr!(
                        "game-header",
                        game = game_name(selected),
                        version = info.version.as_str()
                    );
                    ui.label(RichText::new(text).color(color::TEXT_SECONDARY));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = space::SM;
                    clicks.preferences |=
                        ui.add(secondary_button(&tr("game-preferences"))).clicked();
                    let can_refresh = folder.is_some() && game.run.is_none();
                    if ui
                        .add_enabled(can_refresh, secondary_button(&tr("game-refresh")))
                        .clicked()
                    {
                        game.refresh_folders(&prefs.game_folders);
                        game.refresh_scan();
                    }
                });
            });
            if let Some(folder) = &folder {
                ui.label(
                    RichText::new(folder.display().to_string())
                        .small()
                        .color(color::TEXT_MUTED),
                );
            }
            ui.add_space(space::SM);
            let status = game.folders.get(&selected).cloned();
            match (&folder, status) {
                (None, _) => no_folder(ui, game, prefs, selected),
                (Some(_), Some(FolderStatus::Problem(problem))) => message(
                    ui,
                    &tr("game-folder-problem"),
                    &crate::game::problem_text(&problem),
                ),
                (Some(_), Some(FolderStatus::Valid(_))) if selected == Game::Ats => {
                    message(ui, &tr("game-ats-title"), &tr("game-ats-unsupported"));
                }
                (Some(folder), _) => scanned(ui, game, library, filter, folder),
            }
        },
    );
    clicks
}

fn message(ui: &mut Ui, title: &str, text: &str) {
    ui.vertical_centered(|ui| EmptyState::new(icons::VEHICLE, title, text).show(ui));
}

/// No folder is set: the detected one is proposed, or Preferences explains.
fn no_folder(ui: &mut Ui, game: &mut GameState, prefs: &mut crate::prefs::Prefs, selected: Game) {
    let found = game
        .detected
        .entry(selected)
        .or_insert_with(|| crate::game::detect(selected).map(|(folder, _)| folder))
        .clone();
    ui.vertical_centered(|ui| {
        let name = game_name(selected);
        EmptyState::new(
            icons::VEHICLE,
            &tr("game-no-folder"),
            &tr!("game-no-folder-hint", game = name.as_str()),
        )
        .show(ui);
        if let Some(folder) = found {
            ui.label(
                RichText::new(tr!("game-detected", folder = folder.display().to_string()))
                    .color(color::TEXT_SECONDARY),
            );
            ui.add_space(space::SM);
            if ui.add(primary_button(&tr("game-use-detected"))).clicked() {
                game.set_folder(&mut prefs.game_folders, selected, &folder);
            }
        }
    });
}

/// The scan of an ETS2 folder: reading, failed, or the vehicles.
fn scanned(
    ui: &mut Ui,
    game: &mut GameState,
    library: &VehicleLibrary,
    filter: &mut VehicleFilter,
    folder: &std::path::Path,
) {
    game.ensure_scan(folder, repaint(ui.ctx()));
    match &game.scan {
        ScanState::Idle | ScanState::Scanning { .. } => {
            ui.add_space(space::LG);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(tr("game-scanning")).color(color::TEXT_SECONDARY));
            });
        }
        ScanState::Failed { message: why, .. } => {
            let text = tr!("game-scan-failed", reason = why.as_str());
            message(ui, &tr("game-folder-problem"), &text);
        }
        ScanState::Ready { .. } => list(ui, game, library, filter),
    }
}

/// Wakes the interface when a worker has something to tell.
fn repaint(ctx: &egui::Context) -> impl Fn() + Send + 'static {
    let ctx = ctx.clone();
    move || ctx.request_repaint()
}

/// Icon, text and color of a vehicle's state: its row in the run if there is one,
/// else its package against the game.
pub(super) fn state_look(
    id: &str,
    run: Option<&crate::game::Run>,
    generation: &Generation,
) -> (&'static str, String, egui::Color32) {
    if let Some(state) = run.and_then(|r| r.state_of(id)) {
        return match state {
            RowState::Waiting => (icons::RECENT, tr("game-row-waiting"), color::TEXT_MUTED),
            RowState::InProgress => (icons::ROTATE, tr("game-row-working"), color::LINK),
            RowState::Installed(v) => (
                icons::CHECK,
                tr!("game-row-installed", version = v.as_str()),
                color::SUCCESS,
            ),
            RowState::UpToDate => (
                icons::CHECK,
                tr("game-row-up-to-date"),
                color::TEXT_SECONDARY,
            ),
            RowState::Failed(_) => (icons::WARNING, tr("game-row-failed"), color::ERROR),
        };
    }
    match generation {
        Generation::NotGenerated => (
            icons::FILE_MISSING,
            tr("game-status-not-generated"),
            color::TEXT_MUTED,
        ),
        Generation::Generated(v) => (
            icons::CHECK,
            tr!("game-status-generated", version = v.to_string()),
            color::TEXT_SECONDARY,
        ),
        Generation::GameUpdated(_) => (icons::UPDATE, tr("game-status-updated"), color::SIGNAL),
    }
}

/// Room the buttons, the progress and the notice take below the list.
const BOTTOM: f32 = 150.0;

fn list(ui: &mut Ui, game: &mut GameState, library: &VehicleLibrary, filter: &mut VehicleFilter) {
    let Some(scan) = game.scanned().cloned() else {
        return;
    };
    let running = game.run.is_some();
    let generations = scan
        .vehicles
        .iter()
        .map(|v| generation(library, scan.info.game, &v.id, &scan.info))
        .collect::<Vec<_>>();
    if scan.vehicles.is_empty() {
        message(ui, &tr("game-empty"), &tr("game-empty-hint"));
        return;
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.add_enabled_ui(!running, |ui| {
            if ui.add(secondary_button(&tr("game-select-all"))).clicked() {
                game.checked = scan.vehicles.iter().map(|v| v.id.clone()).collect();
            }
            if ui.add(secondary_button(&tr("game-select-none"))).clicked() {
                game.checked.clear();
            }
            if ui
                .add(secondary_button(&tr("game-select-updates")))
                .clicked()
            {
                game.checked = scan
                    .vehicles
                    .iter()
                    .zip(&generations)
                    .filter(|(_, g)| matches!(g, Generation::GameUpdated(_)))
                    .map(|(v, _)| v.id.clone())
                    .collect();
            }
        });
    });
    ui.add_space(space::XS);
    filter.game_locked = true;
    filter.show(ui, "game");
    ui.add_space(space::SM);
    let height = (ui.available_height() - BOTTOM).max(80.0);
    let (run, checked) = (&game.run, &mut game.checked);
    ScrollArea::vertical()
        .id_salt("game_vehicles")
        .max_height(height)
        .min_scrolled_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let shown =
                scan.vehicles.iter().zip(&generations).filter(|(v, _)| {
                    filter.matches_vehicle(&v.name, &v.brand, scan.info.game, v.kind)
                });
            for (v, generation) in shown {
                ui.horizontal(|ui| {
                    ui.set_min_height(32.0);
                    let mut on = checked.contains(&v.id);
                    if ui
                        .add_enabled(!running, SquareCheckbox::new(&mut on, &v.name))
                        .changed()
                    {
                        if on {
                            checked.insert(v.id.clone());
                        } else {
                            checked.remove(&v.id);
                        }
                    }
                    ui.label(
                        RichText::new(format!("{} · {}", v.brand, kind_label(v.kind)))
                            .color(color::TEXT_MUTED),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (icon, text, tint) = state_look(&v.id, run.as_ref(), generation);
                        let label = ui.label(RichText::new(text.as_str()).color(tint));
                        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
                        ui.label(icons::rich(icon).color(tint));
                    });
                });
            }
        });
    ui.add_space(space::SM);
    progress_and_actions(ui, game, library);
}

/// The progress of the run, or how the last one ended; Generate and Cancel; the notice.
fn progress_and_actions(ui: &mut Ui, game: &mut GameState, library: &VehicleLibrary) {
    if let Some(run) = &game.run {
        let total = run.rows.len();
        let done = run.done();
        ui.add(
            egui::ProgressBar::new(done as f32 / total.max(1) as f32).text(tr!(
                "game-progress",
                done = done.to_string(),
                total = total.to_string()
            )),
        );
        let text = match (run.current(), run.cancelling) {
            (Some(name), true) => tr!("game-stopping", name = name),
            (Some(name), false) => tr!("game-making", name = name),
            (None, _) => String::new(),
        };
        ui.label(RichText::new(text).color(color::TEXT_SECONDARY));
    } else if let Some(last) = &game.last {
        let text = crate::game::summary_text(last);
        super::dialogs::messages(ui, &[Ok(text)]);
        for (name, reason) in &last.failed {
            super::dialogs::problem(
                ui,
                &tr!(
                    "game-failed-line",
                    name = name.as_str(),
                    reason = reason.as_str()
                ),
            );
        }
    }
    let count = game.checked.len();
    let running = game.run.is_some();
    let (mut generate, mut cancel) = (false, false);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        generate = ui
            .add_enabled(
                count > 0 && !running,
                primary_button(&tr!("game-generate", count = count.to_string())),
            )
            .clicked();
        if running {
            cancel = ui
                .add_enabled(
                    game.run.as_ref().is_some_and(|r| !r.cancelling),
                    secondary_button(&tr("button-cancel")),
                )
                .clicked();
        }
    });
    ui.add(
        egui::Label::new(
            RichText::new(tr("game-notice"))
                .small()
                .color(color::TEXT_SECONDARY),
        )
        .wrap(),
    );
    if generate {
        game.generate(library, repaint(ui.ctx()));
    }
    if cancel {
        game.cancel();
    }
}
