//! The New Project dialog, on one page: the project's name and game on
//! top; the installed vehicles of that game on the left, with
//! Custom Vehicle… and Install…, each vehicle with a checkbox; "Your fleet"
//! on the right, one card per checked vehicle (the open one with its
//! textures to tick), and the textures that will be created.

use egui::{
    Align, Align2, CornerRadius, Frame, Key, Layout, Margin, Rect, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType,
};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space, typography};
use tp_ui::widgets::{
    SegmentedControl, paint_focus_ring, primary_button, secondary_button, truncated_label,
};
use tp_vehicles::{Game, Kind, Manifest};

use super::dialogs::{field_label, footer, labelled, messages, modal, title};
use super::vehicle_dialogs::{
    VehicleChoice, all_complete, game_name, kind_label, load_choices, summary, textures_to_paint,
    vehicle_rows,
};
use crate::state::{AppState, Modal, NewProjectDraft};
use crate::vehicles::{SAMPLE_ID, VehicleLibrary};

/// Width of the "Your fleet" column.
const FLEET_WIDTH: f32 = 340.0;
/// Height of a row of the vehicle list (here and in Add Vehicle…), and of
/// its column headers.
const ROW_HEIGHT: f32 = 32.0;
const HEADER_HEIGHT: f32 = 30.0;

/// The x positions of the vehicle list's columns in a row `rect`: the
/// checkbox's center, then the left edges of Vehicle, Type and Package.
struct Columns {
    mark: f32,
    name: f32,
    kind: f32,
    package: f32,
}

impl Columns {
    fn of(rect: Rect) -> Self {
        let pad = space::MD + 2.0;
        let gap = space::MD;
        let package = rect.right() - pad - 140.0;
        let kind = package - gap - 100.0;
        Self {
            mark: rect.left() + pad + 8.0,
            name: rect.left() + pad + 28.0 + gap,
            kind,
            package,
        }
    }
}

/// What the dialog asks for this frame.
enum Outcome {
    Pending,
    Cancel,
    /// Create a project from the checked vehicles and their checked
    /// textures, in fleet order.
    Create {
        name: String,
        choices: Vec<VehicleChoice>,
    },
    /// Install packages, then come back.
    Install,
    /// Install the built-in sample vehicles and choose the truck.
    InstallSample,
    /// Open the Custom Vehicle dialog, then come back.
    CustomVehicle,
}

/// Shows the New Project dialog held in `state.modal` and runs what it
/// asks for.
pub fn show_modal(ctx: &egui::Context, state: &mut AppState) {
    let Some(Modal::NewProject(draft)) = state.modal.as_mut() else {
        return;
    };
    match show(ctx, draft, &state.vehicles) {
        Outcome::Pending => {}
        Outcome::Cancel => state.modal = None,
        Outcome::Install => {
            let paths = state.dialogs.pick_packages();
            let results = super::vehicle_dialogs::install(state, &paths);
            if let Some(Modal::NewProject(draft)) = &mut state.modal {
                draft.messages = results;
            }
        }
        Outcome::CustomVehicle => {
            if let Some(Modal::NewProject(draft)) = state.modal.take() {
                super::custom_vehicle::open(
                    state,
                    super::custom_vehicle::Origin::NewProject(draft),
                );
            }
        }
        Outcome::InstallSample => {
            let results = super::vehicle_dialogs::install_samples(state);
            let sample = results.first().is_some_and(Result::is_ok);
            let pick = state
                .vehicles
                .vehicles()
                .iter()
                .find(|v| sample && v.newest().manifest.id == SAMPLE_ID)
                .map(|v| (VehicleChoice::of(v), v.newest().manifest.game.id));
            if let Some(Modal::NewProject(draft)) = &mut state.modal {
                draft.messages = results;
                if let Some((choice, game)) = pick {
                    draft.set_game(game, &state.vehicles);
                    draft.check(choice);
                }
            }
        }
        Outcome::Create { name, choices } => {
            let loaded = load_choices(&state.vehicles, &choices);
            match loaded.map(|fleet| crate::vehicle_project::fleet_project(&name, &fleet)) {
                Ok(Ok(project)) => {
                    state.modal = None;
                    state.open_project(project);
                }
                Ok(Err(_)) => state.modal = None,
                Err(text) => {
                    state.modal = Some(Modal::Message {
                        title: tr("home-new-project"),
                        text,
                    });
                }
            }
        }
    }
}

/// The first checked vehicle, which names the project when the name is left
/// empty.
fn first<'a>(draft: &NewProjectDraft, library: &'a VehicleLibrary) -> Option<&'a Manifest> {
    draft.vehicles.first()?.manifest(library)
}

fn show(ctx: &egui::Context, draft: &mut NewProjectDraft, library: &VehicleLibrary) -> Outcome {
    // The game is always one of the two (a draft made by hand has none).
    let game = draft.game();
    draft.filter.game = Some(game);
    let screen = ctx.content_rect();
    let width = (screen.width() - 96.0).clamp(720.0, 1080.0);
    let body_height = (screen.height() - 300.0).clamp(300.0, 560.0);

    let response = modal("new_project_modal").show(ctx, |ui| {
        ui.set_width(width);
        title(ui, &tr("home-new-project"), None);
        ui.add_space(space::LG);
        header(ui, draft, library);
        ui.add_space(space::LG);

        let mut wanted = Wanted::default();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::XL;
            let list_width = ui.available_width() - FLEET_WIDTH - space::XL;
            ui.allocate_ui_with_layout(
                Vec2::new(list_width, body_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(list_width);
                    vehicle_column(ui, draft, library, body_height, &mut wanted);
                },
            );
            ui.allocate_ui_with_layout(
                Vec2::new(FLEET_WIDTH, body_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(FLEET_WIDTH);
                    ui.set_min_height(body_height);
                    fleet(ui, draft, library);
                },
            );
        });

        let ready = all_complete(&draft.vehicles, library);
        let mut cancel = false;
        let mut create = false;
        footer(ui, |ui| {
            create |= ui
                .add_enabled(ready, primary_button(&tr("new-project-create")))
                .on_disabled_hover_text(tr("reason-check-vehicle"))
                .clicked();
            cancel |= ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
        // Enter creates the project (a focused button handles Enter as its
        // own click above).
        if ready && !cancel && !wanted.any() && ui.input(|i| i.key_pressed(Key::Enter)) {
            create = true;
        }
        (cancel, create, wanted)
    });

    let (cancel, create, wanted) = response.inner;
    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
    if cancel || escape {
        Outcome::Cancel
    } else if wanted.install {
        Outcome::Install
    } else if wanted.sample {
        Outcome::InstallSample
    } else if wanted.custom {
        Outcome::CustomVehicle
    } else if create
        && all_complete(&draft.vehicles, library)
        && let Some(m) = first(draft, library)
    {
        let typed = draft.name.trim().to_owned();
        Outcome::Create {
            name: if typed.is_empty() {
                m.name.clone()
            } else {
                typed
            },
            choices: draft.vehicles.clone(),
        }
    } else {
        Outcome::Pending
    }
}

/// Buttons of the vehicle column clicked this frame.
#[derive(Default)]
struct Wanted {
    install: bool,
    sample: bool,
    custom: bool,
}

impl Wanted {
    fn any(&self) -> bool {
        self.install || self.sample || self.custom
    }
}

/// The project's name and game, and the note that vehicles can be added
/// later.
fn header(ui: &mut Ui, draft: &mut NewProjectDraft, library: &VehicleLibrary) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::XL;
        ui.allocate_ui_with_layout(Vec2::new(320.0, 0.0), Layout::top_down(Align::Min), |ui| {
            ui.set_width(320.0);
            labelled(ui, &tr("new-project-name"), |ui| {
                let hint =
                    first(draft, library).map_or_else(|| tr("object-untitled"), |m| m.name.clone());
                let name = ui.add(
                    TextEdit::singleline(&mut draft.name)
                        .hint_text(hint)
                        .desired_width(f32::INFINITY)
                        .margin(Margin::symmetric(8, 6)),
                );
                name.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::TextEdit, true, tr("new-project-name"))
                });
                if !draft.focus_requested {
                    name.request_focus();
                    draft.focus_requested = true;
                }
            });
        });
        labelled(ui, &tr("custom-game"), |ui| {
            let (ets2, ats) = (game_name(Game::Ets2), game_name(Game::Ats));
            let picked = SegmentedControl::new()
                .named_segment(Game::Ets2, "", "ETS2", &ets2)
                .named_segment(Game::Ats, "", "ATS", &ats)
                .show(ui, draft.game());
            if let Some(game) = picked {
                // The vehicles of the other game are no longer listed.
                draft.set_game(game, library);
            }
        });
        // Level with the fields, under their labels.
        ui.vertical(|ui| {
            ui.add_space(ui.text_style_height(&egui::TextStyle::Small) + space::XS + 6.0);
            ui.add(
                egui::Label::new(
                    RichText::new(tr("new-project-add-later")).color(color::TEXT_SECONDARY),
                )
                .wrap(),
            );
        });
    });
}

/// Search, kind filter and Install…, the vehicle list, then the Custom
/// vehicle… entry.
fn vehicle_column(
    ui: &mut Ui,
    draft: &mut NewProjectDraft,
    library: &VehicleLibrary,
    height: f32,
    wanted: &mut Wanted,
) {
    let top = ui.cursor().top();
    let enabled = true;
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            let search = ui.add(
                TextEdit::singleline(&mut draft.filter.query)
                    .hint_text(format!("{} {}", icons::SEARCH, tr("vehicles-search")))
                    .desired_width(220.0)
                    .margin(Margin::symmetric(8, 5)),
            );
            search.widget_info(|| {
                WidgetInfo::labeled(WidgetType::TextEdit, true, tr("vehicles-search"))
            });
            let (all, trucks, trailers) = (
                tr("vehicles-kind-all"),
                tr("vehicles-kind-trucks"),
                tr("vehicles-kind-trailers"),
            );
            if let Some(kind) = SegmentedControl::new()
                .segment(None, "", &all, None)
                .segment(Some(Kind::Truck), "", &trucks, None)
                .segment(Some(Kind::Trailer), "", &trailers, None)
                .show(ui, draft.filter.kind)
            {
                draft.filter.kind = kind;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                wanted.install |= ui.add(secondary_button(&tr("vehicles-install"))).clicked();
            });
        });
    });
    ui.add_space(space::SM);
    messages(ui, &draft.messages);
    // The entry under the list keeps its place: the list takes the rest.
    let entry_height = 78.0;
    let list_height = (height - (ui.cursor().top() - top) - entry_height - space::MD).max(120.0);
    ui.add_enabled_ui(enabled, |ui| {
        Frame::new()
            .fill(color::SURFACE_1)
            .stroke(Stroke::new(1.0, color::BORDER))
            .corner_radius(radius::CARD)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(list_height);
                if library
                    .vehicles()
                    .iter()
                    .all(|v| v.newest().manifest.game.id != draft.game())
                {
                    no_vehicle(ui, wanted);
                } else {
                    list(ui, draft, library, list_height);
                }
            });
    });
    ui.add_space(space::MD);
    custom_entry(ui, wanted);
}

/// With no vehicle installed: what packages are, and the sample vehicles.
fn no_vehicle(ui: &mut Ui, wanted: &mut Wanted) {
    ui.vertical_centered(|ui| {
        ui.add_space(space::XXL);
        ui.label(
            icons::rich(icons::VEHICLE)
                .size(28.0)
                .color(color::TEXT_SECONDARY),
        );
        ui.label(RichText::new(tr("vehicles-empty")).text_style(label_strong_style()));
        ui.add(
            egui::Label::new(
                RichText::new(tr("new-project-no-vehicles-hint")).color(color::TEXT_SECONDARY),
            )
            .wrap(),
        );
        ui.add_space(space::SM);
        wanted.sample |= ui
            .add(primary_button(&tr("vehicles-install-sample")))
            .clicked();
    });
}

/// The vehicles of the chosen game passing the search and kind filter, one
/// row per installed vehicle.
fn list(ui: &mut Ui, draft: &mut NewProjectDraft, library: &VehicleLibrary, height: f32) {
    let rows = vehicle_rows(library, &draft.filter, &[]);
    if rows.is_empty() {
        ui.add_space(space::LG);
        ui.vertical_centered(|ui| {
            let text = if draft.filter.query.trim().is_empty() && draft.filter.kind.is_none() {
                tr!(
                    "new-project-no-game-vehicles",
                    game = game_name(draft.game())
                )
            } else {
                tr("new-project-no-match")
            };
            ui.label(RichText::new(text).color(color::TEXT_SECONDARY));
        });
        return;
    }
    column_headers(ui);
    let checked = |draft: &NewProjectDraft, id: &str| draft.vehicles.iter().any(|c| c.id == id);
    ScrollArea::vertical()
        .id_salt("new_project_vehicles")
        .max_height(height - HEADER_HEIGHT)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for vehicle in rows {
                let m = &vehicle.newest().manifest;
                let cells = Row {
                    name: &m.name,
                    kind: m.kind,
                    package: format!("{} {}", m.brand, m.version),
                    tip: summary(m),
                };
                if vehicle_row(ui, &cells, checked(draft, &vehicle.id)) {
                    draft.toggle(vehicle);
                }
            }
        });
}

/// The headers of the list's columns: Vehicle, Type and Package, in small
/// muted capitals over a hairline.
fn column_headers(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), HEADER_HEIGHT),
        Sense::hover(),
    );
    let columns = Columns::of(rect);
    let painter = ui.painter();
    let font = egui::FontId::new(typography::CAPTION, tp_ui::fonts::semibold_family());
    for (x, key) in [
        (columns.name, "new-project-column-vehicle"),
        (columns.kind, "new-project-column-type"),
        (columns.package, "new-project-column-package"),
    ] {
        painter.text(
            egui::pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            tr(key).to_uppercase(),
            font.clone(),
            color::TEXT_MUTED,
        );
    }
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, color::BORDER),
    );
}

/// The cells of a row of the vehicle list.
pub(super) struct Row<'a> {
    pub name: &'a str,
    pub kind: Kind,
    /// The Package column: the brand with the newest version, or
    /// "Official".
    pub package: String,
    /// The row's tooltip.
    pub tip: String,
}

/// One vehicle: a checkbox, its name, its kind, and its package. Reported
/// as a checkbox named after the vehicle. Returns whether it was clicked
/// (or toggled with Space).
pub(super) fn vehicle_row(ui: &mut Ui, m: &Row<'_>, selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, true, selected, m.name));
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, CornerRadius::ZERO, color::CONTROL);
    } else if response.hovered() {
        painter.rect_filled(rect, CornerRadius::ZERO, color::SURFACE_2);
    }
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, color::BORDER),
    );
    let columns = Columns::of(rect);
    // The checkbox: the app's square box (the row's fill shows the hover).
    let mark = Rect::from_center_size(
        egui::pos2(columns.mark, rect.center().y),
        Vec2::splat(tp_ui::widgets::CHECK_BOX_SIZE),
    );
    tp_ui::widgets::paint_check_box(painter, mark, selected, false);
    let y = rect.center().y;
    let cell = |from: f32, to: f32| {
        painter.with_clip_rect(
            Rect::from_x_y_ranges(from..=to - space::SM, rect.y_range()).intersect(ui.clip_rect()),
        )
    };
    cell(columns.name, columns.kind).text(
        egui::pos2(columns.name, y),
        Align2::LEFT_CENTER,
        m.name,
        egui::TextStyle::Body.resolve(ui.style()),
        if selected {
            color::TEXT_PRIMARY
        } else {
            color::TEXT_SECONDARY
        },
    );
    cell(columns.kind, columns.package).text(
        egui::pos2(columns.kind, y),
        Align2::LEFT_CENTER,
        kind_label(m.kind),
        egui::TextStyle::Body.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );
    cell(columns.package, rect.right()).text(
        egui::pos2(columns.package, y),
        Align2::LEFT_CENTER,
        &m.package,
        egui::FontId::monospace(typography::CAPTION),
        color::TEXT_MUTED,
    );
    paint_focus_ring(ui, rect.shrink(1.0), &response, 0);
    response.on_hover_text(&m.tip).clicked()
}

/// "Vehicle without a package": what Custom Vehicle… does, and the button.
fn custom_entry(ui: &mut Ui, wanted: &mut Wanted) {
    let frame = Frame::new()
        .inner_margin(Margin::symmetric(space::LG as i8, space::MD as i8))
        .corner_radius(radius::LG)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let button = tr("custom-open");
                let font = egui::TextStyle::Button.resolve(ui.style());
                let button_width = ui
                    .painter()
                    .layout_no_wrap(button.clone(), font, color::TEXT_PRIMARY)
                    .size()
                    .x
                    + 2.0 * ui.spacing().button_padding.x;
                let text_width =
                    (ui.available_width() - button_width.max(88.0) - space::LG).max(120.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(text_width, 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(text_width);
                        ui.spacing_mut().item_spacing.y = space::XXS;
                        ui.label(
                            RichText::new(tr("new-project-custom-title"))
                                .color(color::TEXT_PRIMARY),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(tr("new-project-custom-hint"))
                                    .small()
                                    .color(color::TEXT_SECONDARY),
                            )
                            .wrap(),
                        );
                    },
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    wanted.custom |= ui.add(secondary_button(&button)).clicked();
                });
            });
        });
    dashed_outline(ui, frame.response.rect);
}

/// A dashed hairline around `rect` (an entry that leads elsewhere).
fn dashed_outline(ui: &Ui, rect: Rect) {
    let rect = rect.shrink(0.5);
    let stroke = Stroke::new(1.0, color::BORDER_STRONG);
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    for pair in corners.windows(2) {
        ui.painter()
            .extend(egui::Shape::dashed_line(pair, stroke, 5.0, 4.0));
    }
}

/// What a card of "Your fleet" asks for this frame.
enum CardAction {
    None,
    Open,
    Remove,
}

/// "Your fleet": the count of checked vehicles, one card per checked
/// vehicle in fleet order (the open one with its textures to tick, the
/// others folded to one line), and the textures that will be created.
fn fleet(ui: &mut Ui, draft: &mut NewProjectDraft, library: &VehicleLibrary) {
    let rect = ui.max_rect();
    ui.painter().rect(
        rect,
        CornerRadius::same(radius::LG),
        color::SURFACE_1,
        Stroke::new(1.0, color::BORDER),
        StrokeKind::Inside,
    );
    Frame::new()
        .inner_margin(Margin::same(space::LG as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::SM;
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr("new-project-fleet"))
                        .text_style(egui::TextStyle::Heading)
                        .color(color::TEXT_PRIMARY),
                );
                if !draft.vehicles.is_empty() {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let count = tr!("new-project-vehicle-count", count = draft.vehicles.len());
                        let label = ui.label(RichText::new(&count).color(color::TEXT_SECONDARY));
                        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &count));
                    });
                }
            });
            if draft.vehicles.is_empty() {
                ui.add(
                    egui::Label::new(
                        RichText::new(tr("new-project-check-vehicles"))
                            .color(color::TEXT_SECONDARY),
                    )
                    .wrap(),
                );
                return;
            }
            // The cards, then the textures created for every vehicle,
            // scroll; their total stays visible under them.
            let total_height = ui.text_style_height(&label_strong_style()) + 2.0 * space::SM;
            let mut action = None;
            let mut count = 0;
            ScrollArea::vertical()
                .id_salt("new_project_fleet")
                .max_height((ui.available_height() - total_height).max(80.0))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = space::SM;
                    let open = draft.open.clone();
                    for choice in &mut draft.vehicles {
                        let Some(m) = choice.manifest(library) else {
                            continue;
                        };
                        let is_open = open.as_deref() == Some(choice.id.as_str());
                        match card(ui, m, choice, is_open) {
                            CardAction::None => {}
                            CardAction::Open => action = Some((true, choice.id.clone())),
                            CardAction::Remove => action = Some((false, choice.id.clone())),
                        }
                    }
                    let fleet: Vec<(&Manifest, &VehicleChoice)> = draft
                        .vehicles
                        .iter()
                        .filter_map(|c| Some((c.manifest(library)?, c)))
                        .collect();
                    count = created(ui, &fleet);
                });
            let rect = ui.available_rect_before_wrap();
            ui.painter()
                .hline(rect.x_range(), rect.top(), Stroke::new(1.0, color::BORDER));
            ui.add_space(space::SM);
            let total = tr!("new-project-total", count = count);
            let label = ui.label(RichText::new(&total).text_style(label_strong_style()));
            label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &total));
            match action {
                Some((true, id)) => draft.open = Some(id),
                Some((false, id)) => draft.uncheck(&id),
                None => {}
            }
        });
}

/// A card of "Your fleet": the vehicle's name, its brand and kind, and
/// Remove; open, its textures to tick and their count; folded, its checked
/// main textures and the count on one line, the whole card opening it.
fn card(ui: &mut Ui, m: &Manifest, choice: &mut VehicleChoice, open: bool) -> CardAction {
    let mut action = CardAction::None;
    let scope = ui.scope_builder(
        UiBuilder::new()
            .id_salt(("fleet_card", &choice.id))
            .sense(if open { Sense::hover() } else { Sense::click() }),
        |ui| {
            let hovered = !open && ui.response().hovered();
            Frame::new()
                .fill(if hovered {
                    color::SURFACE_2
                } else {
                    color::CONTROL
                })
                .stroke(Stroke::new(1.0, color::BORDER))
                .corner_radius(radius::CARD)
                .inner_margin(Margin::same(space::MD as i8 + 2))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = space::SM;
                    ui.horizontal_top(|ui| {
                        let remove = tr("new-project-remove");
                        let font = egui::TextStyle::Small.resolve(ui.style());
                        let remove_width = ui
                            .painter()
                            .layout_no_wrap(remove.clone(), font, color::TEXT_SECONDARY)
                            .size()
                            .x
                            + 2.0 * ui.spacing().button_padding.x;
                        let text_width =
                            (ui.available_width() - remove_width - space::SM).max(80.0);
                        ui.allocate_ui_with_layout(
                            Vec2::new(text_width, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                ui.set_width(text_width);
                                ui.spacing_mut().item_spacing.y = space::XXS;
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&m.name)
                                            .text_style(label_strong_style())
                                            .color(color::TEXT_PRIMARY),
                                    )
                                    .truncate(),
                                );
                                ui.label(
                                    RichText::new(format!("{} · {}", m.brand, kind_label(m.kind)))
                                        .small()
                                        .color(color::TEXT_SECONDARY),
                                );
                            },
                        );
                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            let button = egui::Button::new(
                                RichText::new(&remove).small().color(color::TEXT_SECONDARY),
                            )
                            .frame(false);
                            if ui.add(button).clicked() {
                                action = CardAction::Remove;
                            }
                        });
                    });
                    let painted = choice.painted(m);
                    let main = painted
                        .iter()
                        .filter(|p| m.paint_job.main.iter().any(|q| q.id == p.id))
                        .count();
                    let counts = tr!(
                        "new-project-counts",
                        main = main,
                        accessories = painted.len() - main
                    );
                    if open {
                        let main_heading = match m.kind {
                            Kind::Truck => tr("new-project-cabins"),
                            Kind::Trailer => tr("vehicles-main-textures"),
                        };
                        textures_to_paint(
                            ui,
                            m,
                            &mut choice.textures,
                            &main_heading,
                            Some(&mode_text(m)),
                        );
                        // In the face of the folded card's summary.
                        let label = ui.label(
                            RichText::new(format!("→ {counts}"))
                                .small()
                                .color(color::TEXT_SECONDARY),
                        );
                        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &counts));
                    } else {
                        let mains: Vec<&str> = painted
                            .iter()
                            .filter(|p| m.paint_job.main.iter().any(|q| q.id == p.id))
                            .map(|p| p.name.as_str())
                            .collect();
                        let line = format!("{} · {counts}", mains.join(", "));
                        let label = ui.add(
                            egui::Label::new(
                                RichText::new(&line).small().color(color::TEXT_SECONDARY),
                            )
                            .wrap()
                            .sense(Sense::hover()),
                        );
                        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &line));
                    }
                });
        },
    );
    let response = scope.response;
    if !open {
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &m.name));
        paint_focus_ring(ui, response.rect, &response, radius::CARD);
        if response.clicked() && matches!(action, CardAction::None) {
            action = CardAction::Open;
        }
    }
    action
}

/// The main texture mode, given by the package: "One per cabin layout" with
/// several main textures, "One for every cabin" for a truck with a single
/// one, "Single main texture" for a trailer.
fn mode_text(m: &Manifest) -> String {
    tr(match m.kind {
        Kind::Trailer => "project-mode-single",
        Kind::Truck if m.paint_job.main.len() > 1 => "project-mode-per-layout",
        Kind::Truck => "project-mode-every-cabin",
    })
}

/// "Textures created": each texture that will be created for every
/// vehicle, in fleet order, with its size (under each vehicle's name when
/// there are several). Returns their count.
fn created(ui: &mut Ui, fleet: &[(&Manifest, &VehicleChoice)]) -> usize {
    ui.add_space(space::XS);
    field_label(ui, &tr("new-project-created"));
    ui.spacing_mut().item_spacing.y = space::XS;
    let mut count = 0;
    for (m, choice) in fleet {
        if fleet.len() > 1 {
            ui.label(RichText::new(&m.name).small().color(color::TEXT_MUTED));
        }
        for part in choice.painted(m) {
            count += 1;
            ui.horizontal(|ui| {
                let size = part.texture.size.label();
                let text = tr!(
                    "new-project-texture",
                    texture = part.name.as_str(),
                    size = size.as_str()
                );
                let size_width = ui
                    .painter()
                    .layout_no_wrap(
                        size.clone(),
                        egui::TextStyle::Monospace.resolve(ui.style()),
                        color::TEXT_SECONDARY,
                    )
                    .size()
                    .x;
                let name_width = ui.available_width() - size_width - space::SM;
                let row = truncated_label(
                    ui,
                    RichText::new(&part.name).color(color::TEXT_SECONDARY),
                    name_width,
                );
                row.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(size).monospace().color(color::TEXT_SECONDARY));
                });
            });
        }
    }
    count
}
