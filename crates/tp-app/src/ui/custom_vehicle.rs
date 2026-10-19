//! The Custom Vehicle dialog: a vehicle described from the game's template
//! files, packed and installed like any package. Opened from New Project,
//! Add Vehicle… and the Vehicle Library, it returns to the dialog it came
//! from.

use std::path::PathBuf;
use std::sync::Arc;

use egui::{
    Align, Frame, Layout, Margin, Rect, RichText, ScrollArea, TextEdit, Ui, WidgetInfo, WidgetType,
};
use tp_i18n::tr;
use tp_pack::custom::{CustomPart, CustomVehicle, Problem, TemplateFile, probe};
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{IconButton, SquareCheckbox, primary_button, secondary_button};
use tp_vehicles::{Game, Kind, Package, Role, TextureSize};

pub use super::origin::Origin;
use super::vehicle_dialogs::{LibraryDialog, game_name};
use crate::custom_vehicles::{PackJob, PackOutcome};
use crate::state::{AppState, Modal};
use crate::vehicles::{VehicleLibrary, pack_error_message, template_error_message};

/// State of the Custom Vehicle dialog.
pub struct CustomVehicleDialog {
    pub form: CustomVehicle,
    pub origin: Origin,
    /// The game is the project's (Add Vehicle): it can't be changed.
    pub game_locked: bool,
    /// Files that couldn't be added, and why.
    pub file_errors: Vec<String>,
    /// Results of installing packages dropped on the dialog.
    pub messages: Vec<Result<String, String>>,
    /// Why the last build failed.
    pub error: Option<String>,
    /// Fields the player has left (their problems are shown).
    touched: Vec<&'static str>,
    job: Option<PackJob>,
    /// Screen rectangles of the rows in the last frame (a file dropped on
    /// a row replaces its template).
    row_rects: Vec<Rect>,
}

impl CustomVehicleDialog {
    fn with_form(form: CustomVehicle, origin: Origin, game_locked: bool) -> Self {
        Self {
            form,
            origin,
            game_locked,
            file_errors: Vec::new(),
            messages: Vec::new(),
            error: None,
            touched: Vec::new(),
            job: None,
            row_rects: Vec::new(),
        }
    }

    /// An empty form, for the origin's game (locked in Add Vehicle).
    pub fn new(origin: Origin) -> Self {
        let locked = matches!(&origin, Origin::AddVehicle(d) if d.filter.game_locked);
        let form = CustomVehicle::new(origin.game().unwrap_or(Game::Ets2));
        Self::with_form(form, origin, locked)
    }

    /// A new version of an installed custom vehicle, filled in from it.
    pub fn new_version(package: &Package, origin: Origin) -> Self {
        Self::with_form(CustomVehicle::from_package(package), origin, true)
    }

    pub fn is_new_version(&self) -> bool {
        self.form.base.is_some()
    }

    pub fn is_building(&self) -> bool {
        self.job.is_some()
    }

    /// Adds files given by name and content; a single file dropped at
    /// `at` over a row replaces that row's template.
    pub fn add_files(
        &mut self,
        files: Vec<(String, Result<Vec<u8>, String>)>,
        at: Option<egui::Pos2>,
    ) {
        if self.is_building() {
            return;
        }
        let target = match (files.len(), at) {
            (1, Some(p)) => self.row_rects.iter().position(|r| r.contains(p)),
            _ => None,
        };
        self.file_errors.clear();
        for (name, bytes) in files {
            match read_template(&name, bytes) {
                Ok(file) => match target.and_then(|i| self.form.rows.get_mut(i)) {
                    Some(row) => row.replace(file),
                    None => self.form.add(file),
                },
                Err(message) => self.file_errors.push(message),
            }
        }
    }

    /// Adds files from disk (Add Templates…); `replace` replaces that
    /// row's template with the first file.
    pub(crate) fn add_paths(&mut self, paths: &[PathBuf], replace: Option<usize>) {
        let files: Vec<_> = paths
            .iter()
            .take(if replace.is_some() { 1 } else { usize::MAX })
            .map(|path| {
                let name = path.file_name().map_or_else(
                    || path.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );
                (name, std::fs::read(path).map_err(|e| e.to_string()))
            })
            .collect();
        if let Some(i) = replace {
            self.file_errors.clear();
            for (name, bytes) in files {
                match (read_template(&name, bytes), self.form.rows.get_mut(i)) {
                    (Ok(file), Some(row)) => row.replace(file),
                    (Err(message), _) => self.file_errors.push(message),
                    (Ok(_), None) => {}
                }
            }
        } else {
            self.add_files(files, None);
        }
    }

    fn touch(&mut self, field: &'static str) {
        if !self.touched.contains(&field) {
            self.touched.push(field);
        }
    }
}

/// A template from a file's content, or why it can't be one.
fn read_template(name: &str, bytes: Result<Vec<u8>, String>) -> Result<TemplateFile, String> {
    let bytes =
        bytes.map_err(|reason| tr!("custom-file-io", file = name, reason = reason.as_str()))?;
    probe(name, Arc::from(bytes)).map_err(|e| template_error_message(&e, name))
}

/// Every installed vehicle version (id, version).
fn installed(library: &VehicleLibrary) -> Vec<(String, semver::Version)> {
    library
        .vehicles()
        .iter()
        .flat_map(|v| {
            v.versions
                .iter()
                .map(|x| (v.id.clone(), x.manifest.version.clone()))
        })
        .collect()
}

/// A problem in the current language.
pub fn problem_message(problem: &Problem) -> String {
    match problem {
        Problem::NameMissing => tr("custom-problem-name"),
        Problem::BrandMissing => tr("custom-problem-brand"),
        Problem::BadGamePath => tr("custom-problem-game-path"),
        Problem::BadGameVersions => tr("custom-problem-game-versions"),
        Problem::BadVersion => tr("custom-problem-version"),
        Problem::VersionNotHigher(v) => {
            tr!("custom-problem-version-not-higher", version = v.to_string())
        }
        Problem::AlreadyInstalled(id) => tr!("custom-problem-installed", id = id.as_str()),
        Problem::NoMainTexture => tr("custom-problem-no-main"),
        Problem::TrailerMainTextures => tr("custom-problem-trailer-main"),
        Problem::TextureNameMissing(_) => tr("custom-problem-texture-name"),
        Problem::MissingCabins(_) => tr("custom-problem-cabins"),
        Problem::MissingAccessoryIds(_) => tr("custom-problem-accessory-ids"),
        Problem::BadGameId(_, id) => tr!("custom-problem-game-id", id = id.as_str()),
        Problem::DuplicateGameId(_, id) => {
            tr!("custom-problem-duplicate-game-id", id = id.as_str())
        }
        Problem::ChangelogTooLong => {
            tr!("custom-problem-changelog", max = tp_vehicles::MAX_CHANGELOG)
        }
    }
}

fn problem_label(ui: &mut Ui, problem: &Problem) {
    super::dialogs::problem(ui, &problem_message(problem));
}

/// A single-line text field under its label; returns whether it lost
/// focus.
fn text_field(ui: &mut Ui, label: &str, text: &mut String, hint: &str, enabled: bool) -> bool {
    super::dialogs::labelled(ui, label, |ui| {
        let response = ui.add_enabled(
            enabled,
            TextEdit::singleline(text)
                .hint_text(hint)
                .desired_width(f32::INFINITY)
                .margin(egui::Margin::symmetric(8, 5)),
        );
        response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, enabled, label));
        response.lost_focus()
    })
}

/// A labelled combo box over `choices`, as wide as the space left.
fn combo<T: Copy + PartialEq>(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    value: &mut T,
    choices: &[(T, String)],
    enabled: bool,
) {
    super::dialogs::field_label(ui, label);
    let width =
        ui.available_width() - ui.spacing().icon_width - 2.0 * ui.spacing().button_padding.x;
    combo_box(ui, id, label, value, choices, enabled, width);
}

/// A combo box over `choices`, `width` wide, named `name` for assistive
/// technologies.
fn combo_box<T: Copy + PartialEq>(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    name: &str,
    value: &mut T,
    choices: &[(T, String)],
    enabled: bool,
    width: f32,
) {
    let selected = choices
        .iter()
        .find(|(c, _)| c == value)
        .map(|(_, t)| t.clone())
        .unwrap_or_default();
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            egui::ComboBox::from_id_salt(id)
                .icon(tp_ui::widgets::dropdown_icon)
                .width(width)
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for (choice, text) in choices {
                        ui.selectable_value(value, *choice, text);
                    }
                })
                .response
        })
        .inner;
    response.widget_info(|| WidgetInfo::labeled(WidgetType::ComboBox, enabled, name));
}

/// Width of a texture row's role combo.
const ROLE_WIDTH: f32 = 150.0;
/// Width of a texture row's width and height combos.
const SIDE_WIDTH: f32 = 84.0;

/// A texture row's field: its short `label` above it (the long name stays
/// the field's accessible name), `width` wide.
fn row_field<R>(ui: &mut Ui, label: &str, width: f32, body: impl FnOnce(&mut Ui) -> R) -> R {
    ui.allocate_ui_with_layout(egui::vec2(width, 0.0), Layout::top_down(Align::Min), |ui| {
        ui.set_width(width);
        ui.spacing_mut().item_spacing.y = space::XS;
        tp_ui::widgets::truncated_label(
            ui,
            RichText::new(label).small().color(color::TEXT_SECONDARY),
            width,
        );
        body(ui)
    })
    .inner
}

/// The fields of texture row `i` as a fixed grid: name, role, width and
/// height (short labels above them), then Replace… and remove, so that a
/// long choice never widens the dialog.
fn texture_row_fields(
    ui: &mut Ui,
    i: usize,
    row: &mut CustomPart,
    replace: &mut Option<usize>,
    remove: &mut Option<usize>,
) {
    let n = i + 1;
    let gap = ui.spacing().item_spacing.x;
    let replace_text = tr("custom-row-replace");
    let replace_width = {
        let font = egui::TextStyle::Button.resolve(ui.style());
        let text = ui
            .painter()
            .layout_no_wrap(replace_text.clone(), font, color::TEXT_PRIMARY)
            .size()
            .x;
        (text + 2.0 * ui.spacing().button_padding.x).max(88.0)
    };
    let fixed = ROLE_WIDTH + 2.0 * SIDE_WIDTH + replace_width + tp_ui::tokens::size::HIT_MIN;
    let name_width = (ui.available_width() - fixed - 5.0 * gap).max(80.0);
    ui.horizontal_top(|ui| {
        let label = tr!("custom-row-name", n = n);
        row_field(ui, &tr("custom-name"), name_width, |ui| {
            ui.add(TextEdit::singleline(&mut row.name).desired_width(name_width))
                .widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, &label));
        });
        row_field(ui, &tr("custom-row-role-short"), ROLE_WIDTH, |ui| {
            combo_box(
                ui,
                ("custom_role", i),
                &tr!("custom-row-role", n = n),
                &mut row.role,
                &[
                    (Role::Main, tr("custom-role-main")),
                    (Role::Accessory, tr("custom-role-accessory")),
                ],
                true,
                ROLE_WIDTH,
            );
        });
        let sides: Vec<(u32, String)> = TextureSize::allowed_sides()
            .map(|s| (s, s.to_string()))
            .collect();
        row_field(ui, &tr("custom-row-width-short"), SIDE_WIDTH, |ui| {
            combo_box(
                ui,
                ("custom_width", i),
                &tr!("custom-row-width", n = n),
                &mut row.size.width,
                &sides,
                true,
                SIDE_WIDTH,
            );
        });
        row_field(ui, &tr("custom-row-height-short"), SIDE_WIDTH, |ui| {
            combo_box(
                ui,
                ("custom_height", i),
                &tr!("custom-row-height", n = n),
                &mut row.size.height,
                &sides,
                true,
                SIDE_WIDTH,
            );
        });
        // The buttons line up with the fields, under the labels' height.
        ui.vertical(|ui| {
            ui.add_space(ui.text_style_height(&egui::TextStyle::Small) + space::XS);
            ui.horizontal(|ui| {
                let name = tr!("custom-row-replace-name", n = n);
                let button = ui.add(secondary_button(&replace_text));
                button.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &name));
                if button.clicked() {
                    *replace = Some(i);
                }
                if ui
                    .add(IconButton::new(
                        icons::REMOVE,
                        &tr!("custom-row-remove", n = n),
                    ))
                    .clicked()
                {
                    *remove = Some(i);
                }
            });
        });
    });
}

/// Shows the dialog; returns false once it is closed (the origin is then
/// back in `state.modal`).
pub fn show(ctx: &egui::Context, state: &mut AppState, dialog: &mut CustomVehicleDialog) -> bool {
    let problems = dialog.form.problems(&installed(&state.vehicles));
    let building = dialog.is_building();
    let new_version = dialog.is_new_version();
    let mut cancel = false;
    let mut create = false;
    let mut add_templates = false;
    let mut replace = None;
    let mut remove = None;
    let mut lost_focus: Vec<&'static str> = Vec::new();
    // The problems of each field: shown once the player has left it, or
    // at once for the version of a new version.
    let field_problems = |field: &'static str| -> Vec<Problem> {
        let shown = dialog.touched.contains(&field);
        problems
            .iter()
            .filter(|p| match (field, p) {
                ("name", Problem::NameMissing)
                | ("brand", Problem::BrandMissing)
                | ("path", Problem::BadGamePath)
                | ("versions", Problem::BadGameVersions) => shown,
                ("version", Problem::BadVersion | Problem::VersionNotHigher(_))
                | ("changelog", Problem::ChangelogTooLong) => true,
                _ => false,
            })
            .cloned()
            .collect()
    };
    let fields: Vec<(&'static str, Vec<Problem>)> =
        ["version", "changelog", "name", "brand", "path", "versions"]
            .into_iter()
            .map(|f| (f, field_problems(f)))
            .collect();
    // The problems of a field, under it.
    let under = |ui: &mut Ui, field: &str| {
        for (_, list) in fields.iter().filter(|(f, _)| *f == field) {
            for problem in list {
                problem_label(ui, problem);
            }
        }
    };

    super::dialogs::modal("custom_vehicle_modal").show(ctx, |ui| {
        ui.set_width(680.0);
        let title = if new_version {
            tr!("custom-new-version-title", name = dialog.form.name.as_str())
        } else {
            tr("custom-title")
        };
        let id = dialog.form.id();
        super::dialogs::title(ui, &title, Some(&id));
        ui.add_space(space::SM);
        super::dialogs::messages(ui, &dialog.messages);
        ui.add_space(space::SM);

        ui.add_enabled_ui(!building, |ui| {
            ui.spacing_mut().item_spacing.y = space::SM;
            let form = &mut dialog.form;
            if new_version {
                super::dialogs::columns(ui, &[1.0, 1.0], |ui, column| {
                    if column == 0 {
                        if text_field(ui, &tr("custom-version"), &mut form.version, "", true) {
                            lost_focus.push("version");
                        }
                        under(ui, "version");
                    }
                });
                let label = tr("custom-what-changed");
                super::dialogs::labelled(ui, &label, |ui| {
                    ui.add(
                        TextEdit::multiline(&mut form.changelog)
                            .hint_text(tr("custom-what-changed-hint"))
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .margin(egui::Margin::symmetric(8, 5)),
                    )
                    .widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, &label));
                });
                ui.label(
                    RichText::new(tr!(
                        "custom-what-changed-count",
                        count = form.changelog.trim().chars().count(),
                        max = tp_vehicles::MAX_CHANGELOG
                    ))
                    .small()
                    .color(ui.visuals().weak_text_color()),
                );
                under(ui, "changelog");
            }
            super::dialogs::columns(ui, &[1.0, 1.0], |ui, column| match column {
                0 => {
                    if text_field(ui, &tr("custom-name"), &mut form.name, "", true) {
                        lost_focus.push("name");
                    }
                    under(ui, "name");
                }
                _ => {
                    if text_field(ui, &tr("custom-brand"), &mut form.brand, "", true) {
                        lost_focus.push("brand");
                    }
                    under(ui, "brand");
                }
            });
            super::dialogs::columns(ui, &[1.0, 1.0], |ui, column| match column {
                0 => combo(
                    ui,
                    "custom_kind",
                    &tr("custom-kind"),
                    &mut form.kind,
                    &[
                        (Kind::Truck, tr("vehicles-truck")),
                        (Kind::Trailer, tr("vehicles-trailer")),
                    ],
                    !new_version,
                ),
                _ => combo(
                    ui,
                    "custom_game",
                    &tr("custom-game"),
                    &mut form.game,
                    &[
                        (Game::Ets2, game_name(Game::Ets2)),
                        (Game::Ats, game_name(Game::Ats)),
                    ],
                    !dialog.game_locked,
                ),
            });
            super::dialogs::columns(ui, &[1.0, 1.0], |ui, column| match column {
                0 => {
                    if text_field(
                        ui,
                        &tr("custom-game-path"),
                        &mut form.path,
                        &tr("custom-game-path-hint"),
                        true,
                    ) {
                        lost_focus.push("path");
                    }
                    under(ui, "path");
                }
                _ => {
                    if text_field(
                        ui,
                        &tr("custom-game-versions"),
                        &mut form.versions,
                        &tr("custom-game-versions-hint"),
                        true,
                    ) {
                        lost_focus.push("versions");
                    }
                    under(ui, "versions");
                }
            });
            ui.horizontal(|ui| {
                for (value, key) in [
                    (&mut form.alt_uv, "custom-alt-uv"),
                    (&mut form.colour_picker, "custom-colour-picker"),
                ] {
                    ui.add(SquareCheckbox::new(value, tr(key)));
                }
            });

            ui.add_space(space::MD);
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr("custom-textures")).text_style(label_strong_style()));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    add_templates |= ui
                        .add(secondary_button(&tr("custom-add-templates")))
                        .clicked();
                });
            });
            ui.label(
                RichText::new(tr("custom-drop-hint"))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
            for error in &dialog.file_errors {
                super::dialogs::problem(ui, error);
            }
            dialog.row_rects.clear();
            let trailer = dialog.form.kind == Kind::Trailer;
            let main_count = dialog
                .form
                .rows
                .iter()
                .filter(|r| r.role == Role::Main)
                .count();
            ScrollArea::vertical()
                .max_height(300.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (i, row) in dialog.form.rows.iter_mut().enumerate() {
                        let n = i + 1;
                        let frame = Frame::new()
                            .fill(color::SURFACE_1)
                            .corner_radius(radius::MD)
                            .inner_margin(Margin::same(space::SM as i8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                texture_row_fields(ui, i, row, &mut replace, &mut remove);
                                let t = &row.template;
                                ui.horizontal(|ui| {
                                    let file = tr!(
                                        "custom-file",
                                        file = t.file_name.as_str(),
                                        width = t.width.to_string(),
                                        height = t.height.to_string()
                                    );
                                    ui.label(
                                        RichText::new(&file).small().color(color::TEXT_SECONDARY),
                                    );
                                    if row.is_scaled() {
                                        let text =
                                            tr!("custom-template-scaled", size = row.size.label());
                                        ui.label(
                                            RichText::new(format!("{} {text}", icons::WARNING))
                                                .small()
                                                .color(color::WARNING),
                                        )
                                        .widget_info(
                                            || WidgetInfo::labeled(WidgetType::Label, true, &text),
                                        );
                                    }
                                });
                                let hint = match row.role {
                                    Role::Main if main_count > 1 => tr("custom-hint-cabins"),
                                    Role::Main => tr("custom-hint-cabins-optional"),
                                    Role::Accessory => tr("custom-hint-accessories"),
                                };
                                let label = tr!("custom-row-game-ids", n = n);
                                ui.add(
                                    TextEdit::singleline(&mut row.game_ids)
                                        .desired_width(f32::INFINITY),
                                )
                                .widget_info(|| {
                                    WidgetInfo::labeled(WidgetType::TextEdit, true, &label)
                                });
                                ui.label(RichText::new(&hint).small().color(color::TEXT_SECONDARY));
                                for problem in problems.iter().filter(|p| p.row() == Some(i)) {
                                    problem_label(ui, problem);
                                }
                            });
                        dialog.row_rects.push(frame.response.rect);
                        ui.add_space(space::XS);
                    }
                });
            if !dialog.form.rows.is_empty() {
                for problem in &problems {
                    if matches!(problem, Problem::NoMainTexture)
                        || (trailer && matches!(problem, Problem::TrailerMainTextures))
                    {
                        problem_label(ui, problem);
                    }
                }
            }
            for problem in &problems {
                if matches!(problem, Problem::AlreadyInstalled(_)) {
                    problem_label(ui, problem);
                }
            }
        });

        ui.add_space(space::SM);
        ui.label(
            RichText::new(tr("custom-scs-reminder"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        if let Some(error) = &dialog.error {
            super::dialogs::problem(ui, error);
        }
        if let Some(job) = &dialog.job {
            let (done, total) = job.progress();
            let text = if total > 0 {
                tr!(
                    "custom-building",
                    done = done.to_string(),
                    total = total.to_string()
                )
            } else {
                tr("custom-building-plain")
            };
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(&text).color(color::TEXT_SECONDARY))
                    .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
            });
        }
        super::dialogs::footer(ui, |ui| {
            let ready = problems.is_empty() && !building;
            let button = ui
                .add_enabled(ready, primary_button(&tr("custom-create")))
                .on_disabled_hover_text(
                    problems
                        .first()
                        .map(problem_message)
                        .unwrap_or_else(|| tr("custom-building-plain")),
                );
            create |= button.clicked();
            cancel |= ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
    });

    for field in lost_focus {
        dialog.touch(field);
    }
    if let Some(i) = remove
        && i < dialog.form.rows.len()
    {
        dialog.form.rows.remove(i);
    }
    if add_templates || replace.is_some() {
        state.want_dialog(
            crate::file_dialogs::DialogRequest::pick_templates(),
            crate::dialog_flow::Purpose::PickTemplates { replace },
        );
    }
    if create && problems.is_empty() && dialog.job.is_none() {
        let ctx = ctx.clone();
        dialog.error = None;
        dialog.job = Some(PackJob::start(dialog.form.clone(), move || {
            ctx.request_repaint();
        }));
    }
    if let Some(outcome) = dialog.job.as_ref().and_then(PackJob::poll) {
        dialog.job = None;
        let name = dialog.form.name.trim().to_owned();
        match outcome {
            PackOutcome::Built(packed) => match state.vehicles.install_bytes(&packed.bytes) {
                Ok(m) => {
                    let message = tr!(
                        "vehicles-installed",
                        name = m.name.as_str(),
                        version = m.version.to_string()
                    );
                    dialog.origin.created(&state.vehicles, &m, message);
                    return false;
                }
                Err(err) => {
                    dialog.error = Some(crate::vehicles::install_error_message(&err, &name));
                }
            },
            PackOutcome::Failed(err) => dialog.error = Some(pack_error_message(&err, &name)),
            PackOutcome::Stopped => dialog.error = Some(tr!("custom-build-stopped", name = name)),
        }
    }
    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    !(cancel || escape)
}

/// Shows the Custom Vehicle modal held in `state.modal`, and puts the
/// dialog it came from back once it closes.
pub fn show_modal(ctx: &egui::Context, state: &mut AppState) {
    let Some(Modal::CustomVehicle(mut dialog)) = state.modal.take() else {
        return;
    };
    if show(ctx, state, &mut dialog) {
        if state.modal.is_none() {
            state.modal = Some(Modal::CustomVehicle(dialog));
        }
    } else {
        state.modal = Some(dialog.origin.into_modal());
    }
}

/// Opens the dialog over `origin`.
pub fn open(state: &mut AppState, origin: Origin) {
    state.modal = Some(Modal::CustomVehicle(Box::new(CustomVehicleDialog::new(
        origin,
    ))));
}

/// Opens New Version… of the installed vehicle `id` over the Vehicle
/// Library.
pub fn open_new_version(state: &mut AppState, id: &str, library: LibraryDialog) {
    let newest = state
        .vehicles
        .get(id)
        .map(|v| v.newest().manifest.version.clone());
    let loaded = newest.map(|version| state.vehicles.load(id, &version));
    match loaded {
        Some(Ok(package)) => {
            state.modal = Some(Modal::CustomVehicle(Box::new(
                CustomVehicleDialog::new_version(&package, Origin::Library(library)),
            )));
        }
        Some(Err(err)) => {
            let mut library = library;
            library.messages = vec![Err(crate::vehicles::install_error_message(&err, id))];
            state.modal = Some(Modal::VehicleLibrary(library));
        }
        None => state.modal = Some(Modal::VehicleLibrary(library)),
    }
}
