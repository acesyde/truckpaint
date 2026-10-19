//! Export Mod dialog: it edits nothing (the mod settings are edited in the
//! Project space). It shows a summary of what the mod holds, the
//! destination with Change…, the problems that block the export just above
//! the buttons (each with Show, which leads to where it is fixed), then the
//! warnings about the textures (which don't block it), and the export's
//! progress.

use std::path::{Path, PathBuf};

use egui::{Key, RichText, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{primary_button, secondary_button};

use crate::dialog_flow::Purpose;
use crate::export::ExportOutcome;
use crate::file_dialogs::DialogRequest;
use crate::layout::Space;
use crate::mod_export::{ModJob, Problem, ProblemPlace, VehicleSummary};
use crate::state::{AppState, Modal};

/// State of the open Export Mod dialog.
pub struct ModExportDialog {
    summary: Vec<VehicleSummary>,
    /// The file the mod will be written to.
    pub destination: PathBuf,
    /// A destination chosen with Change… whose replacement the save dialog
    /// already confirmed.
    confirmed: Option<PathBuf>,
    /// Export was clicked on an existing file: the dialog asks whether to
    /// replace it.
    pub confirm_replace: bool,
    pub job: Option<ModJob>,
}

impl ModExportDialog {
    /// The dialog summarizing the vehicles in `summary`, proposing to write
    /// the mod to `destination`.
    pub fn new(summary: Vec<VehicleSummary>, destination: PathBuf) -> Self {
        Self {
            summary,
            destination,
            confirmed: None,
            confirm_replace: false,
            job: None,
        }
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// The summary of what the mod holds, in a box on the control surface.
fn summary_ui(ui: &mut Ui, summary: &[VehicleSummary]) {
    egui::Frame::new()
        .fill(color::CONTROL)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(radius::CARD)
        .inner_margin(egui::Margin::same(space::MD as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::XS;
            ui.label(RichText::new(tr("mod-summary")).text_style(label_strong_style()));
            for vehicle in summary {
                ui.label(RichText::new(&vehicle.name).color(color::TEXT_PRIMARY));
                let line = |ui: &mut Ui, text: &str| {
                    let label = ui.add(
                        egui::Label::new(RichText::new(text).color(color::TEXT_SECONDARY)).wrap(),
                    );
                    label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
                };
                ui.indent(("mod_summary", &vehicle.name), |ui| {
                    for main in &vehicle.mains {
                        // A trailer's main texture is named alone: it has
                        // no cabins.
                        let text = if !main.painted {
                            tr!("mod-summary-not-painted", texture = main.name.as_str())
                        } else if vehicle.trailer {
                            main.name.clone()
                        } else if main.cabins.is_empty() {
                            tr!("mod-summary-every-cabin", texture = main.name.as_str())
                        } else {
                            tr!(
                                "mod-summary-cabins",
                                texture = main.name.as_str(),
                                cabins = main.cabins.join(", "),
                                count = main.cabins.len()
                            )
                        };
                        line(ui, &text);
                    }
                    if !vehicle.accessories.is_empty() {
                        line(
                            ui,
                            &tr!(
                                "mod-summary-accessories",
                                accessories = vehicle.accessories.join(", ")
                            ),
                        );
                    }
                });
            }
        });
}

/// `text` shortened from its start with "…" so that it fits `width` in
/// `font` (the end, the file name, stays visible).
fn elide_start(ui: &Ui, text: &str, font: &egui::FontId, width: f32) -> String {
    let fits = |t: &str| {
        ui.painter()
            .layout_no_wrap(t.to_owned(), font.clone(), color::TEXT_PRIMARY)
            .size()
            .x
            <= width
    };
    if fits(text) {
        return text.to_owned();
    }
    let chars: Vec<char> = text.chars().collect();
    for start in 1..chars.len() {
        let shown = format!("…{}", chars[start..].iter().collect::<String>());
        if fits(&shown) {
            return shown;
        }
    }
    "…".to_owned()
}

/// The destination under its label: the path in the monospace face,
/// shortened from its start to the room Change… leaves (in full on hover),
/// then Change…. Returns whether Change… was clicked.
fn destination_ui(ui: &mut Ui, destination: &Path) -> bool {
    let full = destination.display().to_string();
    super::dialogs::labelled(ui, &tr("mod-destination"), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::MD;
            let change = tr("mod-change-destination");
            let button_width = button_width(ui, &change);
            let font = egui::TextStyle::Monospace.resolve(ui.style());
            let room =
                (ui.available_width() - button_width - ui.spacing().item_spacing.x).max(40.0);
            let shown = elide_start(ui, &full, &font, room);
            let galley = ui
                .painter()
                .layout_no_wrap(shown, font, color::TEXT_PRIMARY);
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(room, galley.size().y.max(ui.spacing().interact_size.y)),
                Sense::hover(),
            );
            ui.painter().galley(
                egui::pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
                galley,
                color::TEXT_PRIMARY,
            );
            response
                .on_hover_text(&full)
                .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &full));
            ui.add(secondary_button(&change)).clicked()
        })
        .inner
    })
}

/// The width of a [`secondary_button`] reading `text`.
fn button_width(ui: &Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let text = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, color::TEXT_PRIMARY)
        .size()
        .x;
    (text + 2.0 * ui.spacing().button_padding.x).max(88.0)
}

/// A problem that blocks the export with its Show button, named after what
/// it shows (`name`); whether Show was clicked. Show is as tall as a line
/// of the dialog, its text's first line centered on it.
fn problem_with_show(ui: &mut Ui, text: &str, name: &str) -> bool {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        let height = ui.spacing().interact_size.y;
        let row = ui.text_style_height(&egui::TextStyle::Body);
        let lead = ((height - row) / 2.0).max(0.0);
        let show = tr("project-show");
        let button_width = button_width(ui, &show);
        // The icon and the text lowered to the button's middle.
        let lowered = egui::Layout::top_down(egui::Align::Min);
        ui.allocate_ui_with_layout(Vec2::ZERO, lowered, |ui| {
            ui.add_space(lead);
            ui.label(icons::rich(icons::WARNING).color(color::ERROR));
        });
        let width = ui.available_width() - button_width - space::SM;
        let label = ui
            .allocate_ui_with_layout(Vec2::new(width, 0.0), lowered, |ui| {
                ui.set_width(width);
                ui.add_space(lead);
                ui.add(egui::Label::new(RichText::new(text).color(color::ERROR)).wrap())
            })
            .inner;
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
        // No slack in the button: its text stays centered on it.
        ui.spacing_mut().button_padding.y = lead;
        let button = ui.add(secondary_button(&show).min_size(Vec2::new(button_width, height)));
        button.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name));
        button.clicked()
    })
    .inner
}

/// What Show on `problem` is named after: the field's label or the
/// vehicle's name ("Show Price", "Show TruckPaint Sample Truck").
pub fn show_name(project: &tp_core::Project, problem: &Problem) -> String {
    let target = match problem.place() {
        ProblemPlace::Field(field) => field.label(),
        ProblemPlace::Vehicle(id) => project
            .vehicle(&id)
            .map_or_else(String::new, |v| v.name.clone()),
    };
    tr!("project-show-named", name = target)
}

/// What the footer does this frame.
#[derive(Default)]
struct Clicked {
    export: bool,
    replace: bool,
    cancel_replace: bool,
    close: bool,
    change: bool,
    show: Option<ProblemPlace>,
}

/// Shows the dialog; returns false when it should close.
pub fn show(ctx: &egui::Context, state: &mut AppState, dialog: &mut ModExportDialog) -> bool {
    let now = ctx.input(|i| i.time);
    let Some(ws) = state.workspace_mut() else {
        return false;
    };

    // A running export.
    if let Some(job) = &dialog.job
        && let Some(outcome) = job.poll()
    {
        dialog.job = None;
        match outcome {
            ExportOutcome::Written(path) => {
                ws.show_hint(tr!("mod-export-done", file = file_name(&path)), now);
                return false;
            }
            ExportOutcome::Cancelled => {}
            ExportOutcome::Failed { path, reason } => {
                state.modal = Some(Modal::Message {
                    title: tr("export-failed-title"),
                    text: tr!("export-failed", file = file_name(&path), reason = reason),
                });
                return false;
            }
        }
    }

    let problems = crate::mod_export::problems(&ws.project);
    let problem_lines: Vec<(String, String)> = problems
        .iter()
        .map(|p| (p.message(), show_name(&ws.project, p)))
        .collect();
    // Each warning, and whether it is a To check one (in the signal color,
    // as everywhere else).
    let warnings: Vec<(String, bool)> = crate::mod_export::warnings(&ws.project)
        .iter()
        .map(|w| {
            (
                w.message(&ws.project),
                matches!(w, crate::mod_export::Warning::ToCheck { .. }),
            )
        })
        .collect();
    let exporting = dialog.job.as_ref().map(|j| (j.progress(), j.path.clone()));
    let mut clicked = Clicked::default();
    let game = match ws.project.game() {
        Some("ats") => tp_vehicles::Game::Ats,
        _ => tp_vehicles::Game::Ets2,
    };
    let subtitle = format!(
        "{} · {}",
        ws.project.name,
        super::vehicle_dialogs::game_name(game)
    );
    // Sized to its content; the summary only scrolls when the window is
    // too short for it, the title, the destination, the problems, the
    // warnings and the buttons. The chrome: the dialog's margins, title,
    // destination, footer and a little room around it.
    let chrome = 2.0 * space::XL + 70.0 + 64.0 + 64.0 + 2.0 * space::LG;
    let lines = problem_lines.len() + warnings.len() + usize::from(dialog.confirm_replace);
    let body_height = (ctx.content_rect().height() - chrome - 30.0 * lines as f32).max(120.0);
    super::dialogs::modal("mod_export_modal").show(ctx, |ui| {
        // 640 points wide with the dialog's margins.
        ui.set_width(640.0 - 2.0 * space::XL);
        super::dialogs::title(ui, &tr("dialog-export-mod"), Some(&subtitle));
        ui.add_space(space::LG);
        egui::ScrollArea::vertical()
            .id_salt("mod_export_summary")
            .max_height(body_height)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                summary_ui(ui, &dialog.summary);
            });
        ui.add_space(space::MD);
        ui.add_enabled_ui(exporting.is_none() && !dialog.confirm_replace, |ui| {
            clicked.change = destination_ui(ui, &dialog.destination);
        });
        // The problems that block the export, just above the buttons.
        if !problem_lines.is_empty() {
            ui.add_space(space::MD);
            for (problem, (text, name)) in problems.iter().zip(&problem_lines) {
                if problem_with_show(ui, text, name) {
                    clicked.show = Some(problem.place());
                }
            }
        }
        // The warnings about the textures, which don't block it.
        if !warnings.is_empty() {
            ui.add_space(if problem_lines.is_empty() {
                space::MD
            } else {
                space::XS
            });
            for (warning, to_check) in &warnings {
                if *to_check {
                    super::dialogs::warning_in(ui, warning, color::SIGNAL);
                } else {
                    super::dialogs::warning(ui, warning);
                }
            }
        }
        match &exporting {
            Some((progress, path)) => {
                ui.add_space(space::LG);
                ui.add(
                    egui::ProgressBar::new(*progress)
                        .text(tr!("export-progress", file = file_name(path))),
                );
                super::dialogs::footer(ui, |ui| {
                    if ui.add(secondary_button(&tr("button-cancel"))).clicked()
                        && let Some(job) = &dialog.job
                    {
                        job.cancel();
                    }
                });
            }
            None if dialog.confirm_replace => {
                ui.add_space(space::MD);
                super::dialogs::warning(
                    ui,
                    &tr!(
                        "mod-export-replace-question",
                        file = file_name(&dialog.destination)
                    ),
                );
                super::dialogs::footer(ui, |ui| {
                    clicked.replace |= ui.add(primary_button(&tr("mod-export-replace"))).clicked();
                    clicked.cancel_replace |=
                        ui.add(secondary_button(&tr("button-cancel"))).clicked();
                });
            }
            None => {
                super::dialogs::footer(ui, |ui| {
                    clicked.export |= ui
                        .add_enabled(problems.is_empty(), primary_button(&tr("mod-export-start")))
                        .clicked();
                    clicked.close |= ui.add(secondary_button(&tr("button-cancel"))).clicked();
                });
            }
        }
    });
    if exporting.is_none() && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
        // Escape first leaves the replace question, then the dialog.
        if dialog.confirm_replace {
            clicked.cancel_replace = true;
        } else {
            clicked.close = true;
        }
    }

    if let Some(place) = clicked.show {
        ws.reveal = Some(place);
        ws.space = Space::Project;
        return false;
    }
    if clicked.cancel_replace {
        dialog.confirm_replace = false;
    }
    if clicked.change {
        change_destination(state, dialog);
    }
    if clicked.export && problems.is_empty() {
        let replacing = dialog.destination.exists()
            && dialog.confirmed.as_deref() != Some(dialog.destination.as_path());
        if replacing {
            dialog.confirm_replace = true;
        } else {
            start(ctx, state, dialog);
        }
    }
    if clicked.replace && problems.is_empty() {
        dialog.confirm_replace = false;
        start(ctx, state, dialog);
    }
    !clicked.close
}

/// Change…: the native save dialog in the destination's folder, proposing
/// its file name; the chosen file becomes the destination (`.scs` added
/// when left out, see `AppState::apply_answer`). Cancelling keeps it.
fn change_destination(state: &mut AppState, dialog: &ModExportDialog) {
    let folder = dialog.destination.parent().map(Path::to_path_buf);
    let name = file_name(&dialog.destination);
    state.want_dialog(
        DialogRequest::save_mod(&name, folder.as_deref()),
        Purpose::ExportMod,
    );
}

impl ModExportDialog {
    /// Makes `path` the destination.
    pub fn choose_destination(&mut self, path: PathBuf) {
        // The save dialog asked before choosing an existing file.
        self.confirmed = path.exists().then(|| path.clone());
        self.destination = path;
    }
}

/// Starts writing the project's mod to the destination. The project is
/// left as it is: exporting records nothing.
fn start(ctx: &egui::Context, state: &mut AppState, dialog: &mut ModExportDialog) {
    let Some(ws) = state.workspace_mut() else {
        return;
    };
    let Ok(plan) = crate::mod_export::plan(&ws.project) else {
        return;
    };
    let project = ws.project.clone();
    let fonts = ws.text.fonts.fork();
    let path = dialog.destination.clone();
    state.last_mod_folder = path.parent().map(Path::to_path_buf);
    let ctx = ctx.clone();
    dialog.job = Some(ModJob::start(project, plan, fonts, path, move || {
        ctx.request_repaint();
    }));
}
