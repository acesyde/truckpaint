//! Modal dialogs: Preferences, Keyboard Shortcuts, About, messages, and
//! the pieces every dialog shares (title, labelled fields, problems,
//! footer).

use egui::{
    Align, Button, CornerRadius, Frame, Key, Layout, Margin, RichText, Sense, Stroke, TextEdit, Ui,
    WidgetInfo, WidgetType,
};

use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::{TEXT_SCALE_RANGE, UI_SCALE_RANGE, label_strong_style, title_style};
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{
    SWITCH_SIZE, SegmentedControl, paint_switch, primary_button, secondary_button,
};

use crate::commands::{CommandId, ShortcutFormatter};
use crate::state::{AppState, Modal, PrefsTab};

fn dialog_frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_1)
        .stroke(Stroke::new(1.0, color::OUTLINE))
        .corner_radius(CornerRadius::same(radius::DIALOG))
        .inner_margin(Margin::same(space::XL as i8))
        .shadow(egui::Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: color::SHADOW,
        })
}

pub(crate) fn modal(id: &str) -> egui::Modal {
    egui::Modal::new(egui::Id::new(id))
        .frame(dialog_frame())
        .backdrop_color(color::BACKDROP)
}

/// A dialog's title, with the line under it (what it applies to), if any.
pub(crate) fn title(ui: &mut Ui, title: &str, subtitle: Option<&str>) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.label(
            RichText::new(title)
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        if let Some(subtitle) = subtitle {
            ui.label(RichText::new(subtitle).color(color::TEXT_SECONDARY));
        }
    });
}

/// The label above a field.
pub(crate) fn field_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(color::TEXT_SECONDARY));
}

/// A field under its label: labels sit above their fields, so a longer
/// translation never squeezes the field.
pub(crate) fn labelled<R>(ui: &mut Ui, label: &str, body: impl FnOnce(&mut Ui) -> R) -> R {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        field_label(ui, label);
        body(ui)
    })
    .inner
}

/// Height of a dialog's single-line fields.
pub(crate) const FIELD_HEIGHT: f32 = 34.0;

/// How a [`committed_text`] field shows its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextKind {
    Single,
    /// Single line, in the monospace face (versions).
    Mono,
    /// Several lines: Enter starts a new line.
    Multiline,
    /// Single line without its own frame, for a field that draws one around it.
    Inline,
}

/// What a [`committed_text`] field did this frame.
pub(crate) struct Committed {
    pub response: egui::Response,
    /// The text being typed, while the field has the focus.
    pub typing: Option<String>,
    /// The text to commit: the field was left (Enter on a single line,
    /// Tab, a click elsewhere), not with Escape.
    pub committed: Option<String>,
    /// The field was left this frame, committed or not.
    pub left: bool,
}

/// A text field committed when it is left: it shows `current` unless it
/// has the focus, when it shows what is typed (kept in egui's temporary
/// data under `id`, so Undo and Redo show at once). Enter on a single line
/// and leaving it commit; Escape restores `current`. As tall as the
/// dialogs' fields (34 pt), named `label` for assistive technologies.
/// `width`: its width, the available width when `None`.
pub(crate) fn committed_text(
    ui: &mut Ui,
    id: egui::Id,
    label: &str,
    current: &str,
    kind: TextKind,
    hint: &str,
    width: Option<f32>,
) -> Committed {
    tp_ui::widgets::remember_escape(ui, id);
    let focused = ui.memory(|m| m.has_focus(id));
    let mut buffer = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| current.to_owned());
    let width = width.unwrap_or(f32::INFINITY);
    let edit = match kind {
        TextKind::Multiline => TextEdit::multiline(&mut buffer)
            .desired_rows(2)
            .margin(Margin::symmetric(10, 8)),
        TextKind::Single | TextKind::Mono => TextEdit::singleline(&mut buffer)
            .margin(Margin::symmetric(10, 0))
            .vertical_align(Align::Center)
            .min_size(egui::vec2(0.0, FIELD_HEIGHT)),
        TextKind::Inline => inline_edit(TextEdit::singleline(&mut buffer)),
    };
    let mut edit = edit.id(id).desired_width(width).hint_text(hint);
    if kind == TextKind::Mono {
        edit = edit.font(egui::TextStyle::Monospace);
    }
    let response = ui.add(edit);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, label));
    let mut out = Committed {
        typing: None,
        committed: None,
        left: false,
        response,
    };
    if out.response.has_focus() {
        ui.data_mut(|d| d.insert_temp(id, buffer.clone()));
        out.typing = Some(buffer);
    } else if out.response.lost_focus() {
        ui.data_mut(|d| d.remove::<String>(id));
        out.left = true;
        if !tp_ui::widgets::take_escape(ui, id) {
            out.committed = Some(buffer);
        }
    } else if !focused {
        // Left without this field seeing it (it wasn't shown): what it
        // held was committed by then.
        ui.data_mut(|d| d.remove::<String>(id));
    }
    out
}

/// A single-line edit that sits in a [`PathField`]'s frame.
fn inline_edit(edit: TextEdit<'_>) -> TextEdit<'_> {
    edit.frame(Frame::NONE)
        .margin(Margin::symmetric(8, 0))
        .vertical_align(Align::Center)
        .min_size(egui::vec2(0.0, FIELD_HEIGHT - 8.0))
}

/// `columns` side by side, each `ratio` of the width (the gaps aside).
pub(crate) fn columns(ui: &mut Ui, ratios: &[f32], mut body: impl FnMut(&mut Ui, usize)) {
    let gap = space::MD;
    let total: f32 = ratios.iter().sum();
    let width = ui.available_width() - gap * (ratios.len().saturating_sub(1)) as f32;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (i, ratio) in ratios.iter().enumerate() {
            let w = (width * ratio / total).floor();
            ui.allocate_ui_with_layout(egui::vec2(w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(w);
                body(ui, i);
            });
        }
    });
}

/// A problem that blocks the dialog's action: an icon and its text.
pub(crate) fn problem(ui: &mut Ui, text: &str) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.label(icons::rich(icons::WARNING).color(color::ERROR));
        let label = ui.add(egui::Label::new(RichText::new(text).color(color::ERROR)).wrap());
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    });
}

/// A warning that doesn't block the action, with its own icon color so it
/// can't be mistaken for a [`problem`].
pub(crate) fn warning(ui: &mut Ui, text: &str) {
    warning_in(ui, text, color::WARNING);
}

/// A [`warning`] whose icon is in `tint` (the signal color for a texture
/// To check).
pub(crate) fn warning_in(ui: &mut Ui, text: &str, tint: egui::Color32) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.label(icons::rich(icons::WARNING).color(tint));
        let label =
            ui.add(egui::Label::new(RichText::new(text).color(color::TEXT_SECONDARY)).wrap());
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    });
}

/// Results of the last action (packages installed, files written): a
/// success or an error each, with an icon so they don't rely on color.
pub(crate) fn messages(ui: &mut Ui, messages: &[Result<String, String>]) {
    for message in messages {
        let (icon, text, tint) = match message {
            Ok(text) => (icons::CHECK, text, color::SUCCESS),
            Err(text) => (icons::WARNING, text, color::ERROR),
        };
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::SM;
            ui.label(icons::rich(icon).color(tint));
            let label = ui.add(egui::Label::new(RichText::new(text).color(tint)).wrap());
            label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
        });
    }
}

/// The dialog's buttons, right-aligned under a hairline, the primary
/// action last (rightmost): `body` adds them from the right.
pub(crate) fn footer<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    footer_rule(ui);
    ui.with_layout(Layout::right_to_left(Align::Center), body)
        .inner
}

/// The hairline above a dialog's buttons, with its spacing: for a footer
/// that also has a button on the left.
pub(crate) fn footer_rule(ui: &mut Ui) {
    ui.add_space(space::LG);
    let rect = ui.available_rect_before_wrap();
    ui.painter()
        .hline(rect.x_range(), rect.top(), Stroke::new(1.0, color::BORDER));
    ui.add_space(space::LG);
}

/// Shows the active modal, if any.
pub fn show_modal(ctx: &egui::Context, state: &mut AppState) {
    if matches!(state.modal, Some(Modal::Export(_))) {
        let Some(Modal::Export(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        let keep = super::export_dialog::show(ctx, state, &mut dialog);
        // The dialog may have opened another modal (an error message).
        if keep && state.modal.is_none() {
            state.modal = Some(Modal::Export(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::ExportMod(_))) {
        let Some(Modal::ExportMod(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        let keep = super::mod_export_dialog::show(ctx, state, &mut dialog);
        if keep && state.modal.is_none() {
            state.modal = Some(Modal::ExportMod(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::CopyFromCabin(_))) {
        let Some(Modal::CopyFromCabin(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::copy_from_cabin(ctx, state, &mut dialog) && state.modal.is_none()
        {
            state.modal = Some(Modal::CopyFromCabin(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::CustomVehicle(_))) {
        super::custom_vehicle::show_modal(ctx, state);
        return;
    }
    if matches!(state.modal, Some(Modal::NewProject(_))) {
        super::new_project::show_modal(ctx, state);
        return;
    }
    if matches!(state.modal, Some(Modal::Preferences)) {
        let mcp = state.mcp.view();
        if preferences(
            ctx,
            &mut state.prefs,
            !state.macos,
            &mcp,
            &mut state.preferences_tab,
            &mut GamesEnv {
                state: &mut state.game,
                dialogs: state.dialogs.as_mut(),
                mod_folders: &state.mod_folders,
            },
        ) {
            state.modal = None;
        }
        return;
    }
    if matches!(state.modal, Some(Modal::VehicleLibrary(_))) {
        let Some(Modal::VehicleLibrary(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::library(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::VehicleLibrary(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::ImportFromLibrary(_))) {
        let Some(Modal::ImportFromLibrary(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::library_dialog::show(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::ImportFromLibrary(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::UpdateTemplate(_))) {
        let Some(Modal::UpdateTemplate(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::update(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::UpdateTemplate(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::AddVehicle(_))) {
        let Some(Modal::AddVehicle(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::add_vehicle(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::AddVehicle(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::Textures(_))) {
        let Some(Modal::Textures(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::textures(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::Textures(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::CommandPalette(_))) {
        let Some(Modal::CommandPalette(mut palette)) = state.modal.take() else {
            unreachable!()
        };
        if super::palette::show(ctx, state, &mut palette) && state.modal.is_none() {
            state.modal = Some(Modal::CommandPalette(palette));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::RemoveVehicle { .. })) {
        let Some(Modal::RemoveVehicle { package_id, name }) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::remove_vehicle(ctx, state, &package_id, &name)
            && state.modal.is_none()
        {
            state.modal = Some(Modal::RemoveVehicle { package_id, name });
        }
        return;
    }
    let Some(modal_kind) = state.modal.as_mut() else {
        return;
    };
    let close = match modal_kind {
        Modal::KeyboardShortcuts => shortcuts(ctx),
        Modal::About => about(ctx),
        Modal::Message { title, text } => message(ctx, title, text),
        Modal::NewProject(_)
        | Modal::Export(_)
        | Modal::ExportMod(_)
        | Modal::VehicleLibrary(_)
        | Modal::ImportFromLibrary(_)
        | Modal::UpdateTemplate(_)
        | Modal::AddVehicle(_)
        | Modal::Textures(_)
        | Modal::CustomVehicle(_)
        | Modal::Preferences
        | Modal::CopyFromCabin(_)
        | Modal::CommandPalette(_)
        | Modal::RemoveVehicle { .. } => {
            unreachable!("handled above")
        }
        Modal::UnsavedChanges(action) => {
            let action = action.clone();
            let name = state
                .workspace()
                .map(|ws| ws.project.name.clone())
                .unwrap_or_default();
            match unsaved_changes(ctx, &name) {
                PromptAnswer::Pending => false,
                answer => {
                    state.modal = None;
                    match answer {
                        PromptAnswer::Save => state.save_then(ctx, action),
                        PromptAnswer::DontSave => state.run_action(ctx, action),
                        PromptAnswer::Cancel | PromptAnswer::Pending => {}
                    }
                    return;
                }
            }
        }
    };
    if close {
        state.modal = None;
    }
}

enum PromptAnswer {
    Pending,
    Save,
    DontSave,
    Cancel,
}

/// "Save changes to “<name>” before closing?" with Save / Don't Save / Cancel.
fn unsaved_changes(ctx: &egui::Context, name: &str) -> PromptAnswer {
    let mut answer = PromptAnswer::Pending;
    modal("unsaved_changes_modal").show(ctx, |ui| {
        ui.set_width(440.0);
        ui.horizontal(|ui| {
            ui.label(icons::rich(icons::WARNING).size(22.0).color(color::WARNING));
            ui.label(
                RichText::new(tr!("unsaved-title", name = name))
                    .text_style(label_strong_style())
                    .color(color::TEXT_PRIMARY),
            );
        });
        ui.add_space(space::XS);
        ui.label(RichText::new(tr("unsaved-hint")).color(color::TEXT_SECONDARY));
        // The message dialog's footer, Don't Save on the left.
        footer_rule(ui);
        ui.horizontal(|ui| {
            if ui.add(secondary_button(&tr("button-dont-save"))).clicked() {
                answer = PromptAnswer::DontSave;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(primary_button(&tr("cmd-save"))).clicked() {
                    answer = PromptAnswer::Save;
                }
                if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                    answer = PromptAnswer::Cancel;
                }
            });
        });
    });
    if matches!(answer, PromptAnswer::Pending) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            answer = PromptAnswer::Cancel;
        } else if ctx.memory(|m| m.focused().is_none())
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter))
        {
            answer = PromptAnswer::Save;
        }
    }
    answer
}

/// An error or information message with an OK button.
fn message(ctx: &egui::Context, title: &str, text: &str) -> bool {
    let mut close = false;
    modal("message_modal").show(ctx, |ui| {
        ui.set_width(420.0);
        ui.horizontal(|ui| {
            ui.label(icons::rich(icons::WARNING).size(22.0).color(color::WARNING));
            ui.label(
                RichText::new(title)
                    .text_style(label_strong_style())
                    .color(color::TEXT_PRIMARY),
            );
        });
        ui.add_space(space::XS);
        let label = ui.label(RichText::new(text).color(color::TEXT_SECONDARY));
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
        footer(ui, |ui| {
            close |= ui.add(primary_button(&tr("button-ok"))).clicked();
        });
    });
    close
        || ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, Key::Escape)
                || i.consume_key(egui::Modifiers::NONE, Key::Enter)
        })
}

/// A scale in percent: its label and value (dragged or typed) on one line,
/// a thin slider by steps of 5 % under them. Returns the slider's response
/// and whether the value was changed through the value.
fn scale_slider(
    ui: &mut Ui,
    pct: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    name: &str,
) -> (egui::Response, bool) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::SM;
        let typed = ui
            .horizontal(|ui| {
                ui.label(name);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(pct)
                            .range(range.clone())
                            .speed(1.0)
                            .suffix("%")
                            .custom_formatter(|v, _| tp_i18n::format_number(v, 0)),
                    )
                    .changed()
                })
                .inner
            })
            .inner;
        let slider = ui.add(
            tp_ui::widgets::ThinSlider::new(pct, range, name)
                .width(ui.available_width())
                .step(5.0),
        );
        (slider, typed)
    })
    .inner
}

/// What the game cards work with (Preferences' Games tab and the first launch).
pub(crate) struct GamesEnv<'a> {
    pub state: &'a mut crate::game::GameState,
    pub dialogs: &'a mut dyn crate::file_dialogs::FileDialogs,
    /// Where Export Mod looks for each game's mod folder.
    pub mod_folders: &'a crate::mod_export::Folders,
}

/// The words of a game card's state.
#[derive(Clone, Copy)]
pub(crate) enum CardWords {
    /// Preferences: the game's version, or Not set.
    Preferences,
    /// The first launch: Game found or Not found, never a version.
    FirstLaunch,
}

/// The height (points) of the settings area of Preferences: the Games tab,
/// the tallest, fits whole at 100% in every language.
const PREFERENCES_PAGE: f32 = 560.0;
/// Room the dialog's title, tabs, footer and margins take around the
/// settings area.
const PREFERENCES_CHROME: f32 = 240.0;

/// Preferences: one tab at a time, in an area that keeps its size and
/// scrolls when the window is too small for it.
fn preferences(
    ctx: &egui::Context,
    prefs: &mut crate::prefs::Prefs,
    title_bar: bool,
    mcp: &crate::mcp::McpView,
    tab: &mut PrefsTab,
    games: &mut GamesEnv,
) -> bool {
    let response = modal("preferences_modal").show(ctx, |ui| {
        ui.set_width(440.0);
        title(ui, &tr("home-preferences"), None);
        ui.add_space(space::SM);
        let (general, games_tab, ai) = (
            tr("prefs-tab-general"),
            tr("prefs-tab-games"),
            tr("prefs-tab-ai"),
        );
        if let Some(picked) = SegmentedControl::new()
            .fill()
            .segment(PrefsTab::General, "", &general, None)
            .segment(PrefsTab::Games, "", &games_tab, None)
            .segment(PrefsTab::Ai, "", &ai, None)
            .show(ui, *tab)
        {
            *tab = picked;
        }
        ui.add_space(space::LG);
        let page =
            PREFERENCES_PAGE.min((ctx.content_rect().height() - PREFERENCES_CHROME).max(160.0));
        // The settings area is `page` tall on every tab. The tab scrolls
        // only once its content, measured the frame before, is taller.
        let measured_id = egui::Id::new(("preferences_content", *tab as u8));
        let measured = ctx.data(|d| d.get_temp::<f32>(measured_id)).unwrap_or(0.0);
        let mut content = |ui: &mut Ui| {
            ui.spacing_mut().item_spacing.y = space::MD;
            match *tab {
                PrefsTab::General => general_tab(ctx, ui, prefs, title_bar),
                PrefsTab::Games => games_section(ui, prefs, games),
                PrefsTab::Ai => mcp_section(ui, prefs, mcp),
            }
        };
        let used = if measured > page {
            egui::ScrollArea::vertical()
                .id_salt(measured_id)
                .max_height(page)
                .auto_shrink([false, false])
                .show(ui, content)
                .content_size
                .y
        } else {
            let top = ui.cursor().top();
            content(ui);
            let used = ui.cursor().top() - top;
            ui.add_space((page - used).max(0.0));
            used
        };
        ctx.data_mut(|d| d.insert_temp(measured_id, used));
        let mut close = false;
        footer(ui, |ui| {
            close |= ui.add(primary_button(&tr("button-done"))).clicked();
            // In the footer, so it stays in reach when the settings scroll;
            // a quiet text button at the far left.
            if *tab == PrefsTab::General {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let reset =
                        Button::new(RichText::new(tr("prefs-reset")).color(color::TEXT_SECONDARY))
                            .frame(false)
                            .min_size(egui::vec2(0.0, 30.0));
                    if ui.add(reset).clicked() {
                        prefs.reset_scaling();
                    }
                });
            }
        });
        close
    });
    response.inner || response.should_close()
}

/// The title of a group of settings: small, bold, in capitals and the
/// secondary color, so it reads above the labels of its settings (regular
/// size, primary color). Assistive technologies get the text as written.
pub(crate) fn section_title(ui: &mut Ui, text: &str) {
    let title = ui.label(
        RichText::new(text.to_uppercase())
            .text_style(label_strong_style())
            .size(11.0)
            .color(color::TEXT_SECONDARY),
    );
    title.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
}

/// A hairline between two groups of settings.
fn rule(ui: &mut Ui) {
    ui.add_space(space::SM);
    let rect = ui.available_rect_before_wrap();
    ui.painter()
        .hline(rect.x_range(), rect.top(), Stroke::new(1.0, color::BORDER));
    ui.add_space(space::MD);
}

/// The General tab: the interface (language first) and the canvas.
fn general_tab(ctx: &egui::Context, ui: &mut Ui, prefs: &mut crate::prefs::Prefs, title_bar: bool) {
    section_title(ui, &tr("prefs-interface"));
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::SM;
        ui.label(tr("prefs-language"));
        let current = prefs.language();
        let name = |l: Option<tp_i18n::Language>| {
            l.map_or_else(|| tr("language-system"), |l| l.native_name().to_owned())
        };
        let mut picked = current;
        let combo = egui::ComboBox::from_id_salt("prefs_language")
            .icon(tp_ui::widgets::dropdown_icon)
            .width(ui.available_width())
            .selected_text(name(current))
            .show_ui(ui, |ui| {
                let choices =
                    std::iter::once(None).chain(tp_i18n::Language::ALL.into_iter().map(Some));
                for choice in choices {
                    ui.selectable_value(&mut picked, choice, name(choice));
                }
            });
        combo
            .response
            .widget_info(|| WidgetInfo::labeled(WidgetType::ComboBox, true, tr("prefs-language")));
        if picked != current {
            prefs.set_language(picked);
            ctx.request_repaint();
        }
    });
    let mut ui_pct = (prefs.ui_scale * 100.0).round();
    let range = (UI_SCALE_RANGE.start() * 100.0)..=(UI_SCALE_RANGE.end() * 100.0);
    let (r, typed) = scale_slider(ui, &mut ui_pct, range, &tr("prefs-ui-scale"));
    // Apply on release while dragging, so the slider does not move under the
    // pointer as the interface rescales.
    if r.changed() && !r.dragged() || r.drag_stopped() || typed {
        prefs.ui_scale = ui_pct / 100.0;
    }
    let mut text_pct = (prefs.text_scale * 100.0).round();
    let range = (TEXT_SCALE_RANGE.start() * 100.0)..=(TEXT_SCALE_RANGE.end() * 100.0);
    let (r, typed) = scale_slider(ui, &mut text_pct, range, &tr("prefs-text-size"));
    if r.changed() || typed {
        prefs.text_scale = text_pct / 100.0;
    }
    // A hint is smaller and dimmer than a label.
    ui.label(
        RichText::new(tr("prefs-scale-hint"))
            .small()
            .color(color::TEXT_MUTED),
    );
    if title_bar {
        // Its hint on hover keeps the dialog within the smallest window
        // at large UI scales.
        ui.add(tp_ui::widgets::SquareCheckbox::new(
            &mut prefs.system_title_bar,
            tr("prefs-system-title-bar"),
        ))
        .on_hover_text(tr("prefs-system-title-bar-hint"));
    }
    rule(ui);
    section_title(ui, &tr("prefs-canvas"));
    ui.horizontal(|ui| {
        ui.label(tr("prefs-grid-spacing"));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let range = crate::prefs::GRID_SPACING_RANGE;
            let r = ui.add(
                egui::DragValue::new(&mut prefs.view_aids.grid_spacing)
                    .range(range)
                    .speed(1.0)
                    .max_decimals(0)
                    .suffix(" px"),
            );
            r.widget_info(|| {
                WidgetInfo::labeled(WidgetType::DragValue, true, tr("prefs-grid-spacing"))
            });
        });
    });
}

/// Gives `response` an accessible name made of the action and what it is for
/// ("Choose…, Euro Truck Simulator 2 game folder"); its visible text stays
/// the action. No name is made when `target` is empty.
fn name_for(response: &egui::Response, action: &str, target: &str) {
    if !target.is_empty() {
        let name = tr!("prefs-button-for", action = action, target = target);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name.clone()));
    }
}

/// Draws a button whose outline is stronger than a plain control's and, when
/// `transparent`, whose idle fill is the surface under it. The theme still
/// gives it the hover and pressed fills and outlines.
fn outlined(
    ui: &mut Ui,
    transparent: bool,
    button: impl FnOnce(&mut Ui) -> egui::Response,
) -> egui::Response {
    ui.scope(|ui| {
        let widgets = &mut ui.style_mut().visuals.widgets;
        widgets.inactive.bg_stroke = Stroke::new(1.0, color::BORDER_STRONG);
        widgets.hovered.bg_stroke = Stroke::new(1.0, color::TEXT_SECONDARY);
        if transparent {
            widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        }
        button(ui).on_hover_cursor(egui::CursorIcon::PointingHand)
    })
    .inner
}

/// A button inside a field: an outline and a fill of its own, an icon and
/// the action's words; it reacts to the pointer like the other controls.
fn inline_button(ui: &mut Ui, icon: &str, text: &str, target: &str) -> egui::Response {
    let response = outlined(ui, false, |ui| {
        ui.add(
            Button::new((
                icons::rich(icon).color(color::TEXT_PRIMARY),
                RichText::new(text).color(color::TEXT_PRIMARY),
            ))
            .corner_radius(CornerRadius::same(radius::SM))
            .min_size(egui::vec2(0.0, FIELD_HEIGHT - 12.0)),
        )
    });
    name_for(&response, text, target);
    response
}

/// The width [`inline_button`] takes: its icon and its words, the space
/// between them and the button's padding.
fn inline_button_width(ui: &Ui, icon: &str, text: &str) -> f32 {
    let galley = |text: egui::WidgetText| {
        text.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Button,
        )
        .size()
        .x
    };
    galley(icons::rich(icon).into())
        + galley(RichText::new(text).into())
        + ui.spacing().icon_spacing
        + 2.0 * ui.spacing().button_padding.x
}

/// An outlined button for an action of a card (Detect), with an icon.
fn outline_button(ui: &mut Ui, icon: &str, text: &str, target: &str) -> egui::Response {
    let response = outlined(ui, true, |ui| {
        ui.add(
            Button::new((
                icons::rich(icon).color(color::TEXT_PRIMARY),
                RichText::new(text).color(color::TEXT_PRIMARY),
            ))
            .corner_radius(CornerRadius::same(radius::MD))
            .min_size(egui::vec2(0.0, 32.0)),
        )
    });
    name_for(&response, text, target);
    response
}

/// The cross that clears a field, named "Clear, target".
fn clear_button(ui: &mut Ui, target: &str) -> egui::Response {
    let clear = ui.add(
        Button::new(icons::rich(icons::CLOSE).color(color::TEXT_SECONDARY))
            .frame(false)
            .min_size(egui::Vec2::splat(FIELD_HEIGHT - 6.0)),
    );
    name_for(&clear, &tr("prefs-game-clear"), target);
    clear
}

/// What a [`PathField`] did this frame.
#[derive(Default)]
struct PathOut {
    /// A path typed and committed.
    committed: Option<String>,
    /// The action button was clicked.
    action: bool,
    /// The clear button was clicked.
    clear: bool,
}

/// A path in one field, with its buttons inside it at the right: an action
/// (Choose…, Copy) and, once there is something to clear, a clear button.
/// The frame shows the focus of the text inside it.
struct PathField<'a> {
    id: egui::Id,
    /// What assistive technologies call the field and its buttons' target
    /// (the buttons are named "action, target"); empty keeps their text.
    target: &'a str,
    shown: &'a str,
    hint: &'a str,
    /// Typing a path is allowed (the text is committed on Enter or leaving).
    editable: bool,
    mono: bool,
    tint: egui::Color32,
    /// The action button: its icon and its words.
    icon: &'a str,
    action: &'a str,
    can_clear: bool,
}

impl PathField<'_> {
    fn show(self, ui: &mut Ui) -> PathOut {
        // Its own id scope: two fields in one parent must not share the ids
        // of their buttons.
        let id = self.id;
        ui.push_id(id, |ui| self.show_in(ui)).inner
    }

    fn show_in(self, ui: &mut Ui) -> PathOut {
        let mut out = PathOut::default();
        let mut prepared = Frame::new()
            .fill(color::FIELD)
            .stroke(Stroke::new(1.0, color::BORDER))
            .corner_radius(CornerRadius::same(radius::MD))
            .inner_margin(Margin::symmetric(4, 2))
            .begin(ui);
        let mut focused = false;
        // The widgets are made in the order they are read and tabbed through:
        // the path first, then Clear, then the action; the path gets what the
        // buttons leave.
        let inner = &mut prepared.content_ui;
        let width = inner.available_width();
        // What the buttons need, worked out from their icon and words, so the
        // path gets the rest in this very frame (no frame before to wait for).
        let buttons = inline_button_width(inner, self.icon, self.action)
            + if self.can_clear {
                FIELD_HEIGHT - 6.0 + space::XS
            } else {
                0.0
            };
        let path_width = (width - buttons - space::XS).max(60.0);
        inner.set_min_width(width);
        inner.allocate_ui_with_layout(
            egui::vec2(width, FIELD_HEIGHT - 4.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = space::XS;
                if self.editable {
                    let field = committed_text(
                        ui,
                        self.id,
                        self.target,
                        self.shown,
                        TextKind::Inline,
                        self.hint,
                        Some(path_width),
                    );
                    focused = field.response.has_focus();
                    out.committed = field.committed;
                } else {
                    let mut text = self.shown;
                    let mut edit = inline_edit(TextEdit::singleline(&mut text))
                        .id_salt(self.id)
                        .desired_width(path_width)
                        .text_color(self.tint)
                        .hint_text(self.hint);
                    if self.mono {
                        edit = edit.font(egui::TextStyle::Monospace);
                    }
                    let field = ui.add(edit);
                    field.widget_info(|| {
                        WidgetInfo::labeled(WidgetType::TextEdit, true, self.target)
                    });
                    focused = field.has_focus();
                }
                if self.can_clear {
                    out.clear = clear_button(ui, self.target)
                        .on_hover_text(tr("prefs-game-clear"))
                        .clicked();
                }
                out.action = inline_button(ui, self.icon, self.action, self.target).clicked();
            },
        );
        // The focus ring is drawn over the frame, outside it, so the field and
        // what is around it do not move when it appears.
        let response = prepared.end(ui);
        if focused {
            ui.painter().rect_stroke(
                response.rect,
                CornerRadius::same(radius::MD),
                Stroke::new(2.0, color::FOCUS),
                egui::StrokeKind::Outside,
            );
        }
        out
    }
}

/// How a game's folder stands, as a dot or a mark and words.
enum Standing {
    Ready,
    Unset,
    Problem,
}

/// A state as a mark and words, never by color alone: a filled dot, a
/// hollow dot, or the warning sign. Made for a right-to-left row (the state
/// at the right of a heading): the words first, so the mark reads first.
fn state_chip(ui: &mut Ui, standing: Standing, text: &str) {
    ui.label(RichText::new(text).color(color::TEXT_SECONDARY));
    if matches!(standing, Standing::Problem) {
        ui.label(icons::rich(icons::WARNING).color(color::WARNING));
    } else {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
        if matches!(standing, Standing::Ready) {
            ui.painter()
                .circle_filled(rect.center(), 4.0, color::SUCCESS);
        } else {
            ui.painter()
                .circle_stroke(rect.center(), 3.5, Stroke::new(1.5, color::TEXT_SECONDARY));
        }
    }
}

/// A bordered surface that groups the settings of one thing.
fn card<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(color::RAISED)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(CornerRadius::same(radius::CARD))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::MD;
            body(ui)
        })
        .inner
}

/// Applies a folder typed, chosen, detected or cleared for `game`: an empty
/// path clears it, like Clear.
fn apply_game_folder(
    games: &mut crate::game::GameState,
    prefs: &mut crate::prefs::Prefs,
    game: tp_vehicles::Game,
    set: Option<std::path::PathBuf>,
) {
    match set {
        Some(path) if path.as_os_str().is_empty() => {
            games.clear_folder(&mut prefs.game_folders, game);
        }
        Some(path) => games.set_folder(&mut prefs.game_folders, game, &path),
        None => {}
    }
}

/// The Games tab of Preferences: what is read, then one card per game.
fn games_section(ui: &mut Ui, prefs: &mut crate::prefs::Prefs, env: &mut GamesEnv) {
    ui.add(
        egui::Label::new(
            RichText::new(tr("prefs-games-hint"))
                .small()
                .color(color::TEXT_SECONDARY),
        )
        .wrap(),
    );
    game_cards(ui, prefs, env, CardWords::Preferences);
}

/// One card per game with its state beside its name, its game folder and its
/// mod folder.
pub(crate) fn game_cards(
    ui: &mut Ui,
    prefs: &mut crate::prefs::Prefs,
    env: &mut GamesEnv,
    words: CardWords,
) {
    use crate::game::FolderStatus;
    let GamesEnv {
        state: games,
        dialogs,
        mod_folders,
    } = env;
    for game in tp_vehicles::Game::ALL {
        let name = super::vehicle_dialogs::game_name(game);
        let target = tr!("prefs-game-folder-target", game = name.as_str());
        card(ui, |ui| {
            ui.horizontal(|ui| {
                section(ui, &name);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    match games.folders.get(&game) {
                        Some(FolderStatus::Valid(info)) => state_chip(
                            ui,
                            Standing::Ready,
                            &match words {
                                CardWords::Preferences => {
                                    tr!("prefs-game-version", version = info.version.as_str())
                                }
                                CardWords::FirstLaunch => tr("first-run-game-found"),
                            },
                        ),
                        Some(FolderStatus::Problem(_)) => {
                            state_chip(ui, Standing::Problem, &tr("prefs-game-state-problem"));
                        }
                        _ => state_chip(
                            ui,
                            Standing::Unset,
                            &tr(match words {
                                CardWords::Preferences => "prefs-folder-not-set",
                                CardWords::FirstLaunch => "first-run-game-not-found",
                            }),
                        ),
                    }
                });
            });
            let folder = prefs.game_folders.get(game.code()).cloned();
            let shown = folder
                .as_deref()
                .map_or_else(String::new, |p| p.display().to_string());
            let mut set = None;
            labelled(ui, &tr("prefs-game-folder-label"), |ui| {
                let out = PathField {
                    id: egui::Id::new(("prefs_game_folder", game.code())),
                    target: &target,
                    shown: &shown,
                    hint: &tr("prefs-folder-not-set"),
                    editable: true,
                    mono: false,
                    tint: color::TEXT_PRIMARY,
                    icon: icons::OPEN_PROJECT,
                    action: &tr("prefs-game-choose"),
                    can_clear: folder.is_some(),
                }
                .show(ui);
                if let Some(text) = out.committed.filter(|t| t.trim() != shown) {
                    set = Some(std::path::PathBuf::from(text.trim()));
                }
                if out.action {
                    set =
                        dialogs.pick_folder(&tr!("prefs-game-choose-title", game = name.as_str()));
                }
                if out.clear {
                    games.clear_folder(&mut prefs.game_folders, game);
                }
            });
            mod_folder_row(ui, prefs, game, &name, &mut **dialogs, mod_folders);
            // Detect, at the foot of the card, while the folder is empty.
            if folder.is_none()
                && outline_button(ui, icons::SEARCH, &tr("prefs-game-detect"), &target)
                    .on_hover_text(tr("prefs-game-detect-hint"))
                    .clicked()
            {
                match crate::game::detect(game) {
                    Some((found, _)) => set = Some(found),
                    None => {
                        games.refused.insert(game, tr("prefs-game-not-detected"));
                    }
                }
            }
            apply_game_folder(games, prefs, game, set);
            if let Some(text) = games.refused.get(&game) {
                problem(ui, text);
            } else if let Some(FolderStatus::Problem(p)) = games.folders.get(&game) {
                problem(ui, &crate::game::problem_text(p));
            }
        });
    }
}

/// A game's mod folder row: the chosen folder, else the one Export Mod finds,
/// in a field that shows its path and changes through Choose… and Clear.
fn mod_folder_row(
    ui: &mut Ui,
    prefs: &mut crate::prefs::Prefs,
    game: tp_vehicles::Game,
    name: &str,
    dialogs: &mut dyn crate::file_dialogs::FileDialogs,
    folders: &crate::mod_export::Folders,
) {
    let code = game.code();
    let target = tr!("prefs-mod-folder", game = name);
    let chosen = prefs.mod_folders.get(code).cloned();
    let detected = crate::mod_export::existing_mod_folder(
        code,
        folders.documents.as_deref(),
        folders.data.as_deref(),
    );
    let shown = chosen.clone().or(detected);
    labelled(ui, &tr("prefs-mod-folder-label"), |ui| {
        let text = shown
            .as_ref()
            .map_or_else(String::new, |p| p.display().to_string());
        let out = PathField {
            id: egui::Id::new(("prefs_mod_folder", code)),
            target: &target,
            shown: &text,
            hint: &tr("prefs-folder-not-set"),
            editable: false,
            mono: false,
            // The detected folder is dimmer than a chosen one.
            tint: if chosen.is_some() {
                color::TEXT_PRIMARY
            } else {
                color::TEXT_SECONDARY
            },
            icon: icons::OPEN_PROJECT,
            action: &tr("prefs-game-choose"),
            can_clear: chosen.is_some(),
        }
        .show(ui);
        if out.action
            && let Some(path) = dialogs.pick_folder(&tr!("prefs-mod-choose-title", game = name))
        {
            prefs.mod_folders.insert(code.to_owned(), path);
        }
        if out.clear {
            prefs.mod_folders.remove(code);
        }
    });
}

/// A card with a switch: its title and a line under it, the switch at the
/// right. Announced as a switch with its title; Space or a click toggles it.
fn switch_card(ui: &mut Ui, on: &mut bool, title: &str, subtitle: &str) {
    card(ui, |ui| {
        let width = ui.available_width();
        ui.allocate_ui_with_layout(
            egui::vec2(width, 40.0),
            Layout::right_to_left(Align::Center),
            |ui| {
                let (rect, mut response) =
                    ui.allocate_exact_size(egui::vec2(SWITCH_SIZE.x + 20.0, 32.0), Sense::click());
                if response.clicked() {
                    *on = !*on;
                    response.mark_changed();
                }
                response
                    .widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, true, *on, title));
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_role(egui::accesskit::Role::Switch);
                });
                let knob = egui::Rect::from_center_size(rect.center(), SWITCH_SIZE);
                paint_switch(ui, knob, *on, true);
                if response.has_focus() {
                    ui.painter().rect_stroke(
                        knob.expand(3.0),
                        CornerRadius::same(10),
                        Stroke::new(2.0, color::FOCUS),
                        egui::StrokeKind::Outside,
                    );
                }
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.y = space::XS;
                    section(ui, title);
                    ui.add(
                        egui::Label::new(
                            RichText::new(subtitle).small().color(color::TEXT_SECONDARY),
                        )
                        .wrap(),
                    );
                });
            },
        );
    });
}

/// One line of what the assistant can or cannot do: a check or a cross and
/// the words, which assistive technologies read with "Allowed" or
/// "Not allowed".
fn ability(ui: &mut Ui, allowed: bool, text: &str) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        let (icon, tint) = if allowed {
            (icons::CHECK, color::SUCCESS)
        } else {
            (icons::CLOSE, color::ERROR)
        };
        ui.label(icons::rich(icon).color(tint));
        let label = ui.add(egui::Label::new(text).wrap());
        let prefix = tr(if allowed {
            "prefs-mcp-allowed"
        } else {
            "prefs-mcp-not-allowed"
        });
        label.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Label, true, format!("{prefix}: {text}"))
        });
    });
}

/// How long (seconds) the address's button says "Copied".
const COPIED_FOR: f64 = 1.5;

/// The AI Assistant tab of Preferences: the switch, the server's state, the
/// address with Copy in its field, the port (editable while the server is
/// off) and what the assistant can and cannot do.
fn mcp_section(ui: &mut Ui, prefs: &mut crate::prefs::Prefs, mcp: &crate::mcp::McpView) {
    switch_card(
        ui,
        &mut prefs.mcp_enabled,
        &tr("prefs-mcp-enable"),
        &tr("prefs-mcp-enable-hint"),
    );
    use tp_mcp::ServerState;
    match &mcp.state {
        ServerState::PortInUse => problem(ui, &tr!("prefs-mcp-port-in-use", port = mcp.port)),
        ServerState::Failed(error) => problem(ui, &tr!("prefs-mcp-failed", error = error.as_str())),
        state => {
            let (icon, text, tint) = match state {
                ServerState::Listening if mcp.connected => (
                    icons::MCP_CONNECTED,
                    tr("prefs-mcp-connected"),
                    color::SUCCESS,
                ),
                ServerState::Listening => (icons::MCP, tr("prefs-mcp-on"), color::SUCCESS),
                _ => (icons::MCP, tr("prefs-mcp-off"), color::TEXT_SECONDARY),
            };
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::SM;
                ui.label(icons::rich(icon).color(tint));
                ui.label(RichText::new(text).color(color::TEXT_SECONDARY));
            });
        }
    }
    let url = tp_mcp::url(prefs.mcp_port);
    // After Copy, the button says "Copied" for a moment.
    let copied_id = egui::Id::new("prefs_mcp_copied");
    let now = ui.input(|i| i.time);
    let since = ui.data(|d| d.get_temp::<f64>(copied_id)).map(|t| now - t);
    let copied = since.is_some_and(|elapsed| elapsed < COPIED_FOR);
    if let Some(elapsed) = since.filter(|_| copied) {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(COPIED_FOR - elapsed));
    }
    labelled(ui, &tr("prefs-mcp-url"), |ui| {
        let out = PathField {
            id: egui::Id::new("prefs_mcp_url"),
            target: &tr("prefs-mcp-url"),
            shown: &url,
            hint: "",
            editable: false,
            mono: true,
            tint: color::TEXT_PRIMARY,
            icon: if copied { icons::CHECK } else { icons::COPY },
            action: &tr(if copied {
                "prefs-mcp-copied"
            } else {
                "prefs-mcp-copy"
            }),
            can_clear: false,
        }
        .show(ui);
        if out.action {
            ui.ctx().copy_text(url.clone());
            ui.data_mut(|d| d.insert_temp(copied_id, now));
            ui.ctx().request_repaint();
        }
    });
    ui.horizontal(|ui| {
        ui.label(tr("prefs-mcp-port"));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let r = ui.add_enabled(
                !prefs.mcp_enabled,
                egui::DragValue::new(&mut prefs.mcp_port)
                    .range(crate::prefs::MCP_PORT_RANGE)
                    .speed(1.0),
            );
            r.widget_info(|| {
                WidgetInfo::labeled(WidgetType::DragValue, true, tr("prefs-mcp-port"))
            });
        });
    });
    rule(ui);
    section_title(ui, &tr("prefs-mcp-can-title"));
    ability(ui, true, &tr("prefs-mcp-can-read"));
    ability(ui, true, &tr("prefs-mcp-can-draw"));
    ability(ui, true, &tr("prefs-mcp-can-import"));
    ability(ui, false, &tr("prefs-mcp-cannot-save"));
    ui.add(
        egui::Label::new(
            RichText::new(tr("prefs-mcp-hint"))
                .small()
                .color(color::TEXT_SECONDARY),
        )
        .wrap(),
    );
}

/// A section's heading in a dialog ("Interface", "Canvas").
fn section(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .text_style(label_strong_style())
            .color(color::TEXT_PRIMARY),
    );
}

fn shortcuts(ctx: &egui::Context) -> bool {
    let formatter = ShortcutFormatter::new(ctx);
    let response = modal("shortcuts_modal").show(ctx, |ui| {
        ui.set_width(480.0);
        title(ui, &tr("cmd-keyboard-shortcuts"), None);
        ui.add_space(space::MD);
        egui::ScrollArea::vertical()
            .max_height(420.0)
            .show(ui, |ui| {
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .striped(true)
                    .spacing([space::XL, space::XS + 2.0])
                    .show(ui, |ui| {
                        for id in CommandId::all() {
                            // Every key of the command, alternates included.
                            let keys: Vec<String> = id
                                .meta()
                                .shortcuts
                                .iter()
                                .map(|s| formatter.format(s))
                                .collect();
                            if !keys.is_empty() {
                                ui.label(tr(id.meta().label));
                                ui.label(
                                    RichText::new(keys.join(", "))
                                        .monospace()
                                        .color(color::TEXT_SECONDARY),
                                );
                                ui.end_row();
                            }
                        }
                        ui.label(tr("shortcuts-temporary-hand"));
                        ui.label(
                            RichText::new(tr("shortcuts-hold-space")).color(color::TEXT_SECONDARY),
                        );
                        ui.end_row();
                    });
            });
        footer(ui, |ui| {
            ui.add(primary_button(&tr("button-close"))).clicked()
        })
    });
    response.inner || response.should_close()
}

fn about(ctx: &egui::Context) -> bool {
    let response = modal("about_modal").show(ctx, |ui| {
        ui.set_width(360.0);
        ui.horizontal(|ui| {
            ui.label(
                icons::rich(icons::VEHICLE)
                    .size(32.0)
                    .color(color::TEXT_PRIMARY),
            );
            ui.label(
                RichText::new(crate::paths::APP_NAME)
                    .text_style(title_style())
                    .color(color::TEXT_PRIMARY),
            );
        });
        ui.label(
            RichText::new(tr!("about-version", version = env!("CARGO_PKG_VERSION")))
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::SM);
        ui.label(tr("about-tagline"));
        ui.add_space(space::SM);
        footer(ui, |ui| {
            ui.add(primary_button(&tr("button-close"))).clicked()
        })
    });
    response.inner || response.should_close()
}
