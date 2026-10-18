//! The first launch: language and game, the vehicles to paint, then a first
//! project. It takes the place of the home screen until it is finished or
//! skipped. Folders, scan and generation are the ones of Preferences and the
//! My Game tab (`game.rs`); this file only shows them in three steps.

use egui::{
    Align, CentralPanel, Frame, Key, Layout, Modifiers, RichText, ScrollArea, Ui, WidgetInfo,
    WidgetType,
};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::{display_style, label_strong_style};
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{SegmentedControl, SquareCheckbox, primary_button, secondary_button};
use tp_vehicles::{Game, Kind};

use super::dialogs::{messages, problem};
use super::game_tab::state_look;
use super::vehicle_dialogs::{VehicleFilter, game_label, game_name, kind_label};
use crate::game::{FolderStatus, ScanState, generation};
use crate::state::{AppState, Modal, NewProjectDraft};

/// Width of the first launch's content, at most.
const CONTENT_WIDTH: f32 = 720.0;
/// Height of the vehicle list.
const LIST_HEIGHT: f32 = 300.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Language,
    Vehicles,
}

/// The first launch as it is going.
#[derive(Debug)]
pub struct FirstRun {
    pub step: Step,
    /// Search and kind filter of the vehicle list.
    pub filter: VehicleFilter,
    /// Shown again from Help: it ends on the home screen, not in New Project.
    pub again: bool,
    /// Generate and continue was pressed and its run is not over yet.
    pub generating: bool,
    /// Reading the game was cancelled for the current folder.
    scan_cancelled: bool,
    /// Results of installing the sample vehicles.
    samples: Vec<Result<String, String>>,
}

impl FirstRun {
    pub fn new(again: bool) -> Self {
        Self {
            step: Step::Language,
            filter: VehicleFilter::default(),
            again,
            generating: false,
            scan_cancelled: false,
            samples: Vec::new(),
        }
    }
}

impl AppState {
    /// Shows the first launch from its first step.
    pub fn start_first_run(&mut self, again: bool) {
        self.game.refresh_folders(&self.prefs.game_folders);
        self.game.last = None;
        self.first_run = Some(FirstRun::new(again));
    }

    /// Starts the first launch when it is due; someone with recent projects
    /// or installed vehicles is counted as having done it.
    pub fn begin_first_run_if_needed(&mut self) {
        if self
            .prefs
            .first_run_due(!self.vehicles.vehicles().is_empty())
        {
            self.start_first_run(false);
        } else {
            self.prefs.first_run_done = true;
        }
    }

    /// Ends the first launch (finished or skipped); with `hand_off`, opens New
    /// Project on the game of the folders.
    pub fn end_first_run(&mut self, hand_off: bool) {
        self.first_run = None;
        self.prefs.first_run_done = true;
        if hand_off && self.modal.is_none() {
            let draft = NewProjectDraft::new(&self.vehicles, &self.prefs.played());
            self.modal = Some(Modal::NewProject(draft));
        }
    }
}

/// What the player asked for this frame.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    Nothing,
    Skip,
    Back,
    Next,
}

/// The first launch, in the home screen's place.
pub fn show(ui: &mut Ui, state: &mut AppState) {
    let Some(mut run) = state.first_run.take() else {
        return;
    };
    let mut ask = Ask::Nothing;
    let mut finish = false;
    CentralPanel::no_frame()
        .frame(Frame::new().fill(color::SURFACE_0))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("first_run")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let content = (ui.available_width() - 48.0).min(CONTENT_WIDTH);
                    let side = ((ui.available_width() - content) / 2.0).max(0.0);
                    ui.add_space(space::XXL);
                    ui.horizontal_top(|ui| {
                        ui.add_space(side);
                        ui.vertical(|ui| {
                            ui.set_width(content);
                            indicator(ui, run.step);
                            ui.add_space(space::LG);
                            match run.step {
                                Step::Language => {
                                    ask = language_and_game(ui, state);
                                }
                                Step::Vehicles => {
                                    (ask, finish) = vehicles(ui, state, &mut run);
                                }
                            }
                        });
                    });
                    ui.add_space(space::XXL);
                });
        });
    let ctx = ui.ctx().clone();
    if state.modal.is_none() && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        ask = Ask::Skip;
    }
    match ask {
        Ask::Skip => return state.end_first_run(false),
        Ask::Back => run.step = Step::Language,
        Ask::Next => run.step = Step::Vehicles,
        Ask::Nothing => {}
    }
    if finish {
        return state.end_first_run(!run.again);
    }
    state.first_run = Some(run);
}

/// The three steps, the current one in strong text.
fn indicator(ui: &mut Ui, step: Step) {
    let current = match step {
        Step::Language => 1,
        Step::Vehicles => 2,
    };
    let names = [
        tr("first-run-step-language"),
        tr("first-run-step-vehicles"),
        tr("first-run-step-project"),
    ];
    ui.horizontal_wrapped(|ui| {
        for (i, name) in names.iter().enumerate() {
            let n = i + 1;
            let text = format!("{n}  {name}");
            let rich = if n == current {
                RichText::new(&text)
                    .text_style(label_strong_style())
                    .color(color::TEXT_PRIMARY)
            } else {
                RichText::new(&text).color(color::TEXT_MUTED)
            };
            let label = ui.label(rich);
            label.widget_info(|| {
                WidgetInfo::labeled(
                    WidgetType::Label,
                    true,
                    tr!("first-run-step-of", n = n.to_string(), name = name.as_str()),
                )
            });
            if n < names.len() {
                ui.label(RichText::new(icons::COLLAPSED).color(color::TEXT_MUTED));
            }
        }
    });
}

/// Back, Skip and the primary button of a step.
fn buttons(ui: &mut Ui, back: bool, primary: Option<(&str, bool, &str)>) -> Ask {
    let mut ask = Ask::Nothing;
    ui.add_space(space::LG);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        if let Some((label, enabled, why)) = primary {
            let response = ui
                .add_enabled(enabled, primary_button(label))
                .on_disabled_hover_text(why);
            if response.clicked() {
                ask = Ask::Next;
            }
        }
        if back && ui.add(secondary_button(&tr("first-run-back"))).clicked() {
            ask = Ask::Back;
        }
        if ui.add(secondary_button(&tr("first-run-skip"))).clicked() {
            ask = Ask::Skip;
        }
    });
    ask
}

fn heading(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .text_style(label_strong_style())
            .color(color::TEXT_PRIMARY),
    );
}

/// Step 1: the language, the games played, and their folders.
fn language_and_game(ui: &mut Ui, state: &mut AppState) -> Ask {
    let AppState {
        prefs,
        game,
        dialogs,
        mod_folders,
        system_language,
        ..
    } = state;
    ui.label(
        RichText::new(tr("first-run-welcome"))
            .text_style(display_style())
            .color(color::TEXT_PRIMARY),
    );
    ui.label(RichText::new(tr("first-run-intro")).color(color::TEXT_SECONDARY));
    ui.add_space(space::LG);

    // The language is the system's until the player picks another one.
    heading(ui, &tr("first-run-language"));
    let current = prefs.language().unwrap_or(*system_language);
    let mut picked = current;
    let combo = egui::ComboBox::from_id_salt("first_run_language")
        .icon(tp_ui::widgets::dropdown_icon)
        .width(220.0)
        .selected_text(current.native_name())
        .show_ui(ui, |ui| {
            for l in tp_i18n::Language::ALL {
                ui.selectable_value(&mut picked, l, l.native_name());
            }
        });
    combo
        .response
        .widget_info(|| WidgetInfo::labeled(WidgetType::ComboBox, true, tr("first-run-language")));
    if picked != current {
        prefs.set_language(Some(picked));
    }
    ui.add_space(space::LG);

    heading(ui, &tr("first-run-games"));
    if !game.folders.contains_key(&Game::Ets2) {
        game.refresh_folders(&prefs.game_folders);
    }
    for g in Game::ALL {
        ui.add_space(space::MD);
        heading(ui, game_label(g));
        // A folder not set yet is looked for in Steam, once.
        if !prefs.game_folders.contains_key(g.code()) {
            let found = game
                .detected
                .entry(g)
                .or_insert_with(|| crate::game::detect(g).map(|(folder, _)| folder))
                .clone();
            if let Some(folder) = found {
                game.set_folder(&mut prefs.game_folders, g, &folder);
            }
        }
        let status = game.folders.get(&g).cloned();
        let folder = prefs.game_folders.get(g.code()).cloned();
        let name = game_name(g);
        let mut set = None;
        match (&status, &folder) {
            (Some(FolderStatus::Valid(_)), Some(folder)) => {
                found_row(ui, &tr("first-run-game-found"), Some(folder), None);
            }
            _ => {
                let why = match &status {
                    Some(FolderStatus::Problem(p)) => Some(crate::game::problem_text(p)),
                    _ => None,
                };
                if found_row(ui, &tr("first-run-game-not-found"), None, why.as_deref()) {
                    set =
                        dialogs.pick_folder(&tr!("prefs-game-choose-title", game = name.as_str()));
                }
            }
        }
        if let Some(path) = set {
            game.set_folder(&mut prefs.game_folders, g, &path);
        }
        if let Some(text) = game.refused.get(&g) {
            problem(ui, text);
        }
        // The mod folder: the chosen one, else where Export Mod looks.
        let detected = crate::mod_export::existing_mod_folder(
            g.code(),
            mod_folders.documents.as_deref(),
            mod_folders.data.as_deref(),
        );
        let mod_folder = prefs.mod_folders.get(g.code()).cloned().or(detected);
        let again = match &mod_folder {
            Some(folder) => found_row(ui, &tr("first-run-mod-found"), Some(folder), None),
            None => found_row(ui, &tr("first-run-mod-not-found"), None, None),
        };
        if again
            && let Some(path) =
                dialogs.pick_folder(&tr!("prefs-mod-choose-title", game = name.as_str()))
        {
            prefs.mod_folders.insert(g.code().to_owned(), path);
        }
    }
    buttons(ui, false, Some((&tr("first-run-next"), true, "")))
}

/// A row: an icon and what was found with its path, and **Choose Folder…**
/// (returns whether it was clicked) beside it.
fn found_row(
    ui: &mut Ui,
    text: &str,
    path: Option<&std::path::PathBuf>,
    why: Option<&str>,
) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        let (icon, tint) = if path.is_some() {
            (icons::CHECK, color::SUCCESS)
        } else {
            (icons::FILE_MISSING, color::TEXT_MUTED)
        };
        ui.label(icons::rich(icon).color(tint));
        let line = match path {
            Some(path) => format!("{text}: {}", path.display()),
            None => text.to_owned(),
        };
        ui.add(egui::Label::new(RichText::new(line).color(color::TEXT_PRIMARY)).wrap());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            clicked = ui
                .add(secondary_button(&tr("first-run-choose-folder")))
                .clicked();
        });
    });
    if let Some(why) = why {
        problem(ui, why);
    }
    clicked
}

/// Step 2: the vehicles of the game, generated on Generate and Continue.
/// Returns what was asked and whether the first launch is over.
fn vehicles(ui: &mut Ui, state: &mut AppState, run: &mut FirstRun) -> (Ask, bool) {
    let ctx = ui.ctx().clone();
    let notify = {
        let ctx = ctx.clone();
        move || ctx.request_repaint()
    };
    let ets2 = Game::Ets2;
    let plays = |g: Game| state.prefs.plays(g.code());
    let (plays_ets2, plays_ats) = (plays(ets2), plays(Game::Ats));
    let folder = state.prefs.game_folders.get(ets2.code()).cloned();
    let valid = matches!(state.game.folders.get(&ets2), Some(FolderStatus::Valid(_)));
    let scannable = plays_ets2 && valid;
    ui.label(
        RichText::new(tr("first-run-step-vehicles"))
            .text_style(display_style())
            .color(color::TEXT_PRIMARY),
    );
    ui.label(RichText::new(tr("first-run-vehicles-intro")).color(color::TEXT_SECONDARY));
    ui.add_space(space::MD);

    // A run that ended: on to New Project unless something failed or it was cancelled.
    let mut finish = false;
    if run.generating && state.game.run.is_none() {
        run.generating = false;
        finish = state
            .game
            .last
            .as_ref()
            .is_some_and(|s| s.failed.is_empty() && !s.cancelled);
    }

    let running = state.game.run.is_some();
    let mut unscannable = !plays_ets2 && !plays_ats;
    if plays_ets2 && !scannable {
        unscannable = true;
        problem(ui, &tr("first-run-ets2-missing"));
    }
    if plays_ats {
        unscannable = true;
        ui.label(RichText::new(tr("first-run-ats-unsupported")).color(color::TEXT_SECONDARY));
    }
    if scannable && let Some(folder) = &folder {
        scan_and_list(ui, state, run, folder, &notify);
    }
    if unscannable {
        ui.add_space(space::SM);
        ui.label(RichText::new(tr("first-run-samples-hint")).color(color::TEXT_SECONDARY));
        if state.vehicles.vehicles().is_empty()
            && ui
                .add(secondary_button(&tr("vehicles-install-sample")))
                .clicked()
        {
            run.samples = super::vehicle_dialogs::install_samples(state);
        }
        messages(ui, &run.samples);
    }
    progress(ui, state);

    let count = state.game.checked.len();
    let label = if count > 0 {
        tr("first-run-generate")
    } else {
        tr("first-run-continue")
    };
    let mut ask = buttons(ui, !running, Some((&label, !running, "")));
    if running {
        let cancel = state.game.run.as_ref().is_some_and(|r| !r.cancelling);
        if ui
            .add_enabled(cancel, secondary_button(&tr("button-cancel")))
            .clicked()
        {
            state.game.cancel();
        }
    }
    if ask == Ask::Next {
        if count > 0 {
            state.game.generate(&state.vehicles, notify);
            run.generating = state.game.run.is_some();
            ask = Ask::Nothing;
        } else {
            finish = true;
            ask = Ask::Nothing;
        }
    }
    (ask, finish)
}

/// Reads the game in the background and lists what the player owns.
fn scan_and_list(
    ui: &mut Ui,
    state: &mut AppState,
    run: &mut FirstRun,
    folder: &std::path::Path,
    notify: &(impl Fn() + Clone + Send + 'static),
) {
    if run.scan_cancelled {
        ui.label(RichText::new(tr("first-run-scan-cancelled")).color(color::TEXT_SECONDARY));
        if ui
            .add(secondary_button(&tr("first-run-try-again")))
            .clicked()
        {
            run.scan_cancelled = false;
            state.game.refresh_scan();
        }
        return;
    }
    state.game.game = Game::Ets2;
    state.game.ensure_scan(folder, notify.clone());
    match &state.game.scan {
        ScanState::Idle | ScanState::Scanning { .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(tr("game-scanning")).color(color::TEXT_SECONDARY));
                if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                    run.scan_cancelled = true;
                    state.game.scan = ScanState::Idle;
                }
            });
        }
        ScanState::Failed { message, .. } => {
            problem(ui, &tr!("game-scan-failed", reason = message.as_str()));
        }
        ScanState::Ready { .. } => list(ui, state, run),
    }
}

fn list(ui: &mut Ui, state: &mut AppState, run: &mut FirstRun) {
    let Some(scan) = state.game.scanned().cloned() else {
        return;
    };
    if scan.vehicles.is_empty() {
        ui.label(RichText::new(tr("game-empty")).color(color::TEXT_SECONDARY));
        return;
    }
    let running = state.game.run.is_some();
    let count_of = |kind: Option<Kind>| {
        scan.vehicles
            .iter()
            .filter(|v| kind.is_none_or(|k| k == v.kind))
            .count()
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::MD;
        let search = ui.add(
            egui::TextEdit::singleline(&mut run.filter.query)
                .hint_text(format!("{} {}", icons::SEARCH, tr("vehicles-search")))
                .desired_width(200.0),
        );
        search
            .widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, tr("vehicles-search")));
        let label = |name: &str, kind: Option<Kind>| {
            tr!(
                "first-run-filter-count",
                name = name,
                count = count_of(kind).to_string()
            )
        };
        let (all, trucks, trailers) = (
            label(&tr("vehicles-kind-all"), None),
            label(&tr("vehicles-kind-trucks"), Some(Kind::Truck)),
            label(&tr("vehicles-kind-trailers"), Some(Kind::Trailer)),
        );
        if let Some(kind) = SegmentedControl::new()
            .segment(None, "", &all, None)
            .segment(Some(Kind::Truck), "", &trucks, None)
            .segment(Some(Kind::Trailer), "", &trailers, None)
            .show(ui, run.filter.kind)
        {
            run.filter.kind = kind;
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.add_enabled_ui(!running, |ui| {
            if ui.add(secondary_button(&tr("game-select-all"))).clicked() {
                state.game.checked = scan.vehicles.iter().map(|v| v.id.clone()).collect();
            }
        });
        ui.label(
            RichText::new(tr!(
                "first-run-checked",
                count = state.game.checked.len().to_string()
            ))
            .color(color::TEXT_SECONDARY),
        );
    });
    let generations = scan
        .vehicles
        .iter()
        .map(|v| generation(&state.vehicles, scan.info.game, &v.id, &scan.info))
        .collect::<Vec<_>>();
    let (game_run, checked) = (&state.game.run, &mut state.game.checked);
    ScrollArea::vertical()
        .id_salt("first_run_vehicles")
        .max_height(LIST_HEIGHT)
        .min_scrolled_height(LIST_HEIGHT)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let shown = scan.vehicles.iter().zip(&generations).filter(|(v, _)| {
                run.filter
                    .matches_vehicle(&v.name, &v.brand, scan.info.game, v.kind)
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
                        let (icon, text, tint) = state_look(&v.id, game_run.as_ref(), generation);
                        let label = ui.label(RichText::new(text.as_str()).color(tint));
                        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
                        ui.label(icons::rich(icon).color(tint));
                    });
                });
            }
        });
}

/// The progress of the run, or how the last one ended with the failed vehicles.
fn progress(ui: &mut Ui, state: &AppState) {
    if let Some(run) = &state.game.run {
        let total = run.rows.len();
        let done = run.done();
        ui.add(
            egui::ProgressBar::new(done as f32 / total.max(1) as f32).text(tr!(
                "game-progress",
                done = done.to_string(),
                total = total.to_string()
            )),
        );
        if let Some(name) = run.current() {
            ui.label(RichText::new(tr!("game-making", name = name)).color(color::TEXT_SECONDARY));
        }
    } else if let Some(last) = &state.game.last {
        messages(ui, &[Ok(crate::game::summary_text(last))]);
        for (name, reason) in &last.failed {
            problem(
                ui,
                &tr!(
                    "game-failed-line",
                    name = name.as_str(),
                    reason = reason.as_str()
                ),
            );
        }
    }
}
