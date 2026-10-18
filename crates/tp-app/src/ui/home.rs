//! Home screen: the application's name, New Project and Open…, the
//! recovered projects, and the recent projects as cards with their
//! thumbnail.

use egui::{
    Align, Align2, CentralPanel, CornerRadius, Frame, Layout, Margin, Panel, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::{display_style, label_strong_style, title_style};
use tp_ui::tokens::{color, radius, size, space};
use tp_ui::widgets::{EmptyState, IconButton, paint_focus_ring, primary_button, secondary_button};

use super::workspace::top_bar;
use super::{CommandUi, menu_bar};
use crate::commands::{CommandId, EditContext};
use crate::prefs::{RecentProject, now_unix};
use crate::recent_thumbnails::RecentThumbnails;
use crate::recovery::Recovered;
use crate::state::{AppState, PendingAction, disabled_reason, is_enabled};
use crate::title_bar::TitleBarMode;

/// Width of the home screen's content, at most.
const CONTENT_WIDTH: f32 = 1040.0;
/// Smallest width of a recent project's card, its picture's height and its
/// text's height.
const CARD_MIN_WIDTH: f32 = 228.0;
const CARD_PICTURE: f32 = 128.0;
const CARD_TEXT: f32 = 66.0;
/// Gap between cards.
const CARD_GAP: f32 = space::LG;

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let ctx = ui.ctx().clone();
    let mode = state.title_bar_mode();
    let menu_row = state.draws_menu_row();
    let double_click = state.double_click;
    let mut cmds = CommandUi::new(&ctx, &mut state.queue, EditContext::default());

    match mode {
        // An empty title strip that moves the window.
        TitleBarMode::MacNative => {
            Panel::top("home_title_bar")
                .exact_size(size::TOP_BAR_HEIGHT)
                .frame(bar_frame())
                .show(ui, |ui| top_bar::drag_area(ui, double_click));
        }
        // The mark, the menus and the window controls.
        TitleBarMode::Drawn => {
            Panel::top("home_title_bar")
                .exact_size(size::TOP_BAR_HEIGHT)
                .frame(bar_frame().inner_margin(Margin::ZERO))
                .show(ui, |ui| {
                    super::title_bar::show(ui, &mut cmds, None, None, Default::default())
                });
        }
        TitleBarMode::System => {}
    }
    if menu_row {
        Panel::top("home_menu_bar")
            .frame(bar_frame())
            .show(ui, |ui| {
                menu_bar::show(ui, &mut cmds, None, Default::default())
            });
    }

    if state.first_run.is_some() {
        return super::first_run::show(ui, state);
    }

    // The thumbnails of the files that are there.
    let paths: Vec<&std::path::Path> = state
        .prefs
        .recent
        .iter()
        .zip(&state.recent_available)
        .filter(|(_, exists)| **exists)
        .map(|(r, _)| r.path.as_path())
        .collect();
    state.recent_thumbnails.update(&ctx, &paths);

    let mut action = None;
    let mut recovery_action = None;
    CentralPanel::no_frame()
        .frame(Frame::new().fill(color::SURFACE_0))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("home")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let content = (ui.available_width() - 96.0).min(CONTENT_WIDTH);
                    let side = ((ui.available_width() - content) / 2.0).max(0.0);
                    ui.add_space(40.0);
                    ui.horizontal_top(|ui| {
                        let spacing = ui.spacing().item_spacing.x;
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.add_space(side);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.x = spacing;
                            ui.set_width(content);
                            header(ui, &mut cmds);
                            ui.add_space(space::XXL + space::SM);
                            if !state.recovered.is_empty() {
                                recovery_action = recovered_list(ui, &state.recovered);
                                ui.add_space(space::XXL);
                            }
                            action = recent_list(
                                ui,
                                &mut cmds,
                                &state.prefs.recent,
                                &state.recent_available,
                                &state.recent_thumbnails,
                            );
                        });
                    });
                    ui.add_space(40.0);
                });
        });
    match action {
        Some((index, RowAction::Remove)) => {
            state.prefs.recent.remove(index);
            state.recent_available.remove(index);
        }
        Some((index, RowAction::Open)) => {
            let path = state.prefs.recent[index].path.clone();
            state.guard(&ctx, PendingAction::Open(path));
        }
        None => {}
    }
    match recovery_action {
        Some((session, true)) => state.guard(&ctx, PendingAction::Restore(session)),
        Some((session, false)) => state.discard_recovered(&session),
        None => {}
    }
}

/// What a click on a recent entry asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RowAction {
    Open,
    Remove,
}

/// The application's name and tagline, New Project and Open…, and
/// Preferences on the right. Buttons are as wide as their labels.
fn header(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    icons::rich(icons::VEHICLE)
                        .size(30.0)
                        .color(color::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new(crate::paths::APP_NAME)
                        .text_style(display_style())
                        .color(color::TEXT_PRIMARY),
                );
            });
            ui.label(RichText::new(tr("app-tagline")).color(color::TEXT_SECONDARY));
            ui.add_space(space::XL);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = space::SM + 2.0;
                let tall = Vec2::new(0.0, 34.0);
                let new_tip = cmds.shortcuts.command(CommandId::NewProject);
                let response = ui
                    .add(
                        primary_button(&format!(
                            "{}  {}",
                            icons::NEW_PROJECT,
                            tr("home-new-project")
                        ))
                        .min_size(tall),
                    )
                    .on_hover_text(tr!(
                        "home-new-project-tip",
                        shortcut = new_tip.unwrap_or_default()
                    ));
                response.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Button, true, tr("home-new-project"))
                });
                if response.clicked() {
                    cmds.push(CommandId::NewProject);
                }
                let open_enabled = cmds.enabled(CommandId::OpenProject);
                let open_tip = cmds.shortcuts.command(CommandId::OpenProject);
                let response = ui
                    .add_enabled(
                        open_enabled,
                        secondary_button(&format!(
                            "{}  {}",
                            icons::OPEN_PROJECT,
                            tr("cmd-open-project")
                        ))
                        .min_size(tall),
                    )
                    .on_hover_text(tr!(
                        "home-open-tip",
                        shortcut = open_tip.unwrap_or_default()
                    ))
                    .on_disabled_hover_text(
                        disabled_reason(CommandId::OpenProject)
                            .map(tr)
                            .unwrap_or_default(),
                    );
                response.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Button, open_enabled, tr("cmd-open-project"))
                });
                if response.clicked() {
                    cmds.push(CommandId::OpenProject);
                }
            });
        });
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let label = tr("home-preferences");
            let response = ui.add(secondary_button(&format!("{}  {label}", icons::SETTINGS)));
            response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &label));
            if response.clicked() {
                cmds.push(CommandId::Preferences);
            }
        });
    });
}

/// Recovered projects; returns a session to restore (true) or discard.
fn recovered_list(ui: &mut Ui, recovered: &[Recovered]) -> Option<(String, bool)> {
    ui.horizontal(|ui| {
        ui.label(icons::rich(icons::WARNING).size(20.0).color(color::WARNING));
        ui.label(
            RichText::new(tr("home-recovered"))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
    });
    ui.label(RichText::new(tr("home-recovered-hint")).color(color::TEXT_SECONDARY));
    ui.add_space(space::SM);
    let now = now_unix();
    let mut chosen = None;
    for r in recovered {
        Frame::new()
            .fill(color::SURFACE_1)
            .stroke(Stroke::new(1.0, color::BORDER))
            .corner_radius(CornerRadius::same(radius::LG))
            .inner_margin(Margin::symmetric(space::LG as i8, space::MD as i8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = space::XXS;
                        // The detail: the path (in the mono face, as in
                        // Recent projects) and when it was saved.
                        let (title, path, detail) = match &r.meta {
                            Some(meta) => (
                                meta.name.clone(),
                                meta.original.as_ref().map(|p| p.display().to_string()),
                                match meta.original {
                                    Some(_) => format!(" · {}", relative_time(now, meta.saved_at)),
                                    None => format!(
                                        "{} · {}",
                                        tr("home-never-saved"),
                                        relative_time(now, meta.saved_at)
                                    ),
                                },
                            ),
                            None => (
                                tr("home-recovery-damaged"),
                                None,
                                tr("home-recovery-damaged-hint"),
                            ),
                        };
                        let label = ui.label(
                            RichText::new(&title)
                                .text_style(label_strong_style())
                                .color(color::TEXT_PRIMARY),
                        );
                        label.widget_info(|| {
                            WidgetInfo::labeled(
                                WidgetType::Label,
                                true,
                                tr!("home-recovered-item", name = title.as_str()),
                            )
                        });
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            if let Some(path) = &path {
                                let size = egui::TextStyle::Small.resolve(ui.style()).size;
                                ui.label(
                                    RichText::new(path)
                                        .font(egui::FontId::monospace(size))
                                        .color(color::TEXT_SECONDARY),
                                );
                            }
                            ui.label(RichText::new(&detail).small().color(color::TEXT_SECONDARY));
                        });
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if r.meta.is_some() && ui.add(primary_button(&tr("home-restore"))).clicked()
                        {
                            chosen = Some((r.session.clone(), true));
                        }
                        if ui.add(secondary_button(&tr("home-discard"))).clicked() {
                            chosen = Some((r.session.clone(), false));
                        }
                    });
                });
            });
        ui.add_space(space::SM);
    }
    chosen
}

/// The empty Recent projects area: the empty state centered on the whole
/// list's width, in a frame where the cards will be, with a button that
/// creates a project.
fn empty_recent(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    Frame::new()
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(CornerRadius::same(radius::LG))
        .inner_margin(Margin::same(space::XL as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                EmptyState::new(
                    icons::RECENT,
                    &tr("home-recent-empty"),
                    &tr("home-recent-empty-hint"),
                )
                .show(ui);
                let response = ui.add(secondary_button(&format!(
                    "{}  {}",
                    icons::NEW_PROJECT,
                    tr("home-recent-empty-create")
                )));
                response.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Button, true, tr("home-recent-empty-create"))
                });
                if response.clicked() {
                    cmds.push(CommandId::NewProject);
                }
            });
        });
}

pub fn bar_frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_1)
        .inner_margin(Margin::symmetric(space::SM as i8, 0))
}

/// Draws the recent projects as cards; returns the entry clicked and what
/// for.
fn recent_list(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    recent: &[RecentProject],
    available: &[bool],
    thumbnails: &RecentThumbnails,
) -> Option<(usize, RowAction)> {
    ui.label(
        RichText::new(tr("home-recent"))
            .text_style(title_style())
            .color(color::TEXT_PRIMARY),
    );
    ui.add_space(space::LG);

    if recent.is_empty() {
        empty_recent(ui, cmds);
        return None;
    }

    let now = now_unix();
    let mut action = None;
    let available_width = ui.available_width();
    let columns =
        (((available_width + CARD_GAP) / (CARD_MIN_WIDTH + CARD_GAP)).floor() as usize).max(1);
    let width = ((available_width - CARD_GAP * (columns - 1) as f32) / columns as f32).floor();
    for start in (0..recent.len()).step_by(columns) {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = CARD_GAP;
            let end = (start + columns).min(recent.len());
            for (index, project) in recent.iter().enumerate().take(end).skip(start) {
                let exists = available.get(index).copied().unwrap_or(false);
                let thumbnail = exists.then(|| thumbnails.texture(&project.path)).flatten();
                if let Some(a) = recent_card(ui, project, exists, thumbnail, width, now) {
                    action = Some((index, a));
                }
            }
        });
        ui.add_space(CARD_GAP - ui.spacing().item_spacing.y);
    }
    action
}

/// One recent project: its thumbnail, name, file and last-opened date. It
/// opens on click; when its file is missing it shows as unavailable (an
/// icon and "File not found", not only a color) and offers "Remove from
/// list".
fn recent_card(
    ui: &mut Ui,
    project: &RecentProject,
    exists: bool,
    thumbnail: Option<&egui::TextureHandle>,
    width: f32,
    now: u64,
) -> Option<RowAction> {
    let sense = if exists {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, CARD_PICTURE + CARD_TEXT), sense);
    let open_enabled = is_enabled(CommandId::OpenProject, &EditContext::default());
    let label = if exists {
        project.name.clone()
    } else {
        tr!("home-recent-missing-item", name = project.name.as_str())
    };
    response
        .widget_info(|| WidgetInfo::labeled(WidgetType::Button, open_enabled && exists, &label));

    let hovered = exists && response.hovered();
    let painter = ui.painter();
    let corner = CornerRadius::same(radius::LG);
    painter.rect_filled(
        rect,
        corner,
        if hovered {
            color::SURFACE_2
        } else {
            color::SURFACE_1
        },
    );

    // The picture: the thumbnail, or a placeholder icon while it is read
    // and when the file can't be read.
    let picture = Rect::from_min_size(rect.min, Vec2::new(width, CARD_PICTURE));
    let top = CornerRadius {
        nw: radius::LG,
        ne: radius::LG,
        sw: 0,
        se: 0,
    };
    match thumbnail {
        Some(texture) => {
            // The texture's shape: a square's middle, a wide one whole on
            // the picture's background.
            let (rect, uv) = tp_ui::widgets::thumbnail_place(picture, texture.size_vec2());
            if rect != picture {
                painter.rect_filled(picture, top, color::SURFACE_2);
            }
            let corners = if rect == picture {
                top
            } else {
                CornerRadius::ZERO
            };
            egui::Image::new((texture.id(), rect.size()))
                .uv(uv)
                .corner_radius(corners)
                .paint_at(ui, rect);
        }
        None => {
            painter.rect_filled(picture, top, color::SURFACE_2);
            let (icon, tint) = if exists {
                (icons::FILE, color::TEXT_DISABLED)
            } else {
                (icons::FILE_MISSING, color::TEXT_SECONDARY)
            };
            painter.text(
                picture.center(),
                Align2::CENTER_CENTER,
                icon,
                icons::font(32.0),
                tint,
            );
        }
    }
    painter.hline(
        picture.x_range(),
        picture.bottom(),
        Stroke::new(1.0, color::BORDER),
    );
    painter.rect_stroke(
        rect,
        corner,
        Stroke::new(
            1.0,
            if hovered {
                color::BORDER_STRONG
            } else {
                color::BORDER
            },
        ),
        StrokeKind::Inside,
    );

    // The text: name, file, then the date (or why it is unavailable).
    let text = Rect::from_min_max(
        egui::pos2(rect.left() + space::MD, picture.bottom() + space::SM),
        egui::pos2(rect.right() - space::MD, rect.bottom() - space::SM),
    );
    let mut removed = false;
    let mut text_right = text.right();
    if !exists {
        let button = Rect::from_center_size(
            egui::pos2(text.right() - size::HIT_MIN / 2.0 + 4.0, text.center().y),
            Vec2::splat(size::HIT_MIN),
        );
        removed = ui
            .put(
                button,
                IconButton::new(icons::REMOVE, &tr("home-remove-recent")),
            )
            .clicked();
        text_right = button.left() - space::XS;
    }
    let clip = Rect::from_min_max(text.min, egui::pos2(text_right, text.bottom()));
    let painter = ui.painter().with_clip_rect(clip.intersect(ui.clip_rect()));
    let name_color = if exists {
        color::TEXT_PRIMARY
    } else {
        color::TEXT_SECONDARY
    };
    // Long names and paths end with "…" rather than being cut mid-glyph.
    let max_width = clip.width();
    for (line, y, font, tint) in [
        (
            project.name.clone(),
            8.0,
            label_strong_style().resolve(ui.style()),
            name_color,
        ),
        // Paths are values: in the mono face.
        (
            project.path.display().to_string(),
            26.0,
            egui::FontId::monospace(egui::TextStyle::Small.resolve(ui.style()).size),
            color::TEXT_SECONDARY,
        ),
    ] {
        let mut job = egui::text::LayoutJob::simple_singleline(line, font, tint);
        job.wrap = egui::text::TextWrapping::truncate_at_width(max_width);
        let galley = ui.fonts_mut(|f| f.layout_job(job));
        let at = egui::pos2(text.left(), text.top() + y - galley.size().y / 2.0);
        painter.galley(at, galley, tint);
    }
    if exists {
        painter.text(
            egui::pos2(text.left(), text.top() + 42.0),
            Align2::LEFT_CENTER,
            relative_time(now, project.last_opened),
            egui::TextStyle::Small.resolve(ui.style()),
            color::TEXT_SECONDARY,
        );
    } else {
        let icon = painter.text(
            egui::pos2(text.left(), text.top() + 42.0),
            Align2::LEFT_CENTER,
            icons::WARNING,
            icons::font(size::ICON - 2.0),
            color::WARNING,
        );
        painter.text(
            egui::pos2(icon.right() + space::XS, text.top() + 42.0),
            Align2::LEFT_CENTER,
            tr("home-file-not-found"),
            egui::TextStyle::Small.resolve(ui.style()),
            color::WARNING,
        );
    }
    paint_focus_ring(ui, rect, &response, radius::LG);

    if removed {
        return Some(RowAction::Remove);
    }
    if exists && open_enabled {
        let response = response.on_hover_text(tr!(
            "home-open-recent",
            path = project.path.display().to_string()
        ));
        if response.clicked() {
            return Some(RowAction::Open);
        }
    } else if !exists {
        response.on_hover_text(tr("home-file-moved"));
    }
    None
}

/// Human-friendly "time ago" text.
pub fn relative_time(now: u64, then: u64) -> String {
    let secs = now.saturating_sub(then);
    let ago = |id: &str, n: u64| tr!(id, count = n);
    match secs {
        0..60 => tr("time-just-now"),
        60..3_600 => ago("time-ago-minutes", secs / 60),
        3_600..86_400 => ago("time-ago-hours", secs / 3_600),
        86_400..172_800 => tr("time-yesterday"),
        172_800..2_592_000 => ago("time-ago-days", secs / 86_400),
        2_592_000..31_536_000 => ago("time-ago-months", secs / 2_592_000),
        _ => ago("time-ago-years", secs / 31_536_000),
    }
}

#[cfg(test)]
mod tests {
    use super::relative_time;

    #[test]
    fn relative_times() {
        assert_eq!(relative_time(100, 90), "Just now");
        assert_eq!(relative_time(10_000, 10_000 - 120), "2 minutes ago");
        assert_eq!(relative_time(100_000, 100_000 - 3_600), "1 hour ago");
        assert_eq!(relative_time(1_000_000, 1_000_000 - 90_000), "Yesterday");
        assert_eq!(
            relative_time(10_000_000, 10_000_000 - 5 * 86_400),
            "5 days ago"
        );
        assert_eq!(relative_time(5, 10), "Just now");
    }
}
