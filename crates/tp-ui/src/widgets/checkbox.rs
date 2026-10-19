//! The app's checkbox: a square box, never mistaken for a radio button.

use egui::{
    Align2, Painter, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo,
    WidgetType,
};

use super::paint_focus_ring;
use crate::icons;
use crate::tokens::{color, radius, space};

/// Side of the checkbox's square.
pub const CHECK_BOX_SIZE: f32 = 16.0;

/// Paints a checkbox's square in `rect` (16 px): filled with a check when
/// `checked`, outlined otherwise (the check tells them apart, not only the
/// color). `hovered` brightens the outline of an empty box.
pub fn paint_check_box(painter: &Painter, rect: Rect, checked: bool, hovered: bool) {
    if checked {
        painter.rect_filled(rect, radius::SM, color::ACCENT_PRIMARY);
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            icons::CHECK,
            icons::font(11.0),
            color::TEXT_ON_PRIMARY,
        );
    } else {
        let outline = if hovered {
            color::TEXT_SECONDARY
        } else {
            color::BORDER_STRONG
        };
        painter.rect_stroke(
            rect.shrink(0.5),
            radius::SM,
            Stroke::new(1.0, outline),
            StrokeKind::Inside,
        );
    }
}

/// A checkbox with its label, in place of egui's `Checkbox` (which the
/// theme's corner radius turns into a circle). Clicking it or pressing
/// Space toggles `checked` and marks the response changed.
pub struct SquareCheckbox<'a> {
    checked: &'a mut bool,
    label: String,
    indeterminate: bool,
}

impl<'a> SquareCheckbox<'a> {
    pub fn new(checked: &'a mut bool, label: impl Into<String>) -> Self {
        Self {
            checked,
            label: label.into(),
            indeterminate: false,
        }
    }

    /// Shown mixed (a filled box with a dash), as for a selection whose
    /// objects differ; a click still toggles `checked`.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
}

impl Widget for SquareCheckbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let font = egui::TextStyle::Body.resolve(ui.style());
        let gap = if self.label.is_empty() {
            0.0
        } else {
            space::SM
        };
        // A long label ends with "…" rather than widening its container.
        let mut job =
            egui::text::LayoutJob::simple_singleline(self.label.clone(), font, color::TEXT_PRIMARY);
        job.wrap = egui::text::TextWrapping::truncate_at_width(
            (ui.available_width() - CHECK_BOX_SIZE - gap).max(0.0),
        );
        let galley = ui.fonts_mut(|f| f.layout_job(job));
        let size = Vec2::new(
            CHECK_BOX_SIZE + gap + galley.size().x,
            ui.spacing()
                .interact_size
                .y
                .max(galley.size().y)
                .max(CHECK_BOX_SIZE),
        );
        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        let checked = *self.checked;
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Checkbox, ui.is_enabled(), checked, &self.label)
        });
        if ui.is_rect_visible(rect) {
            let mark = Rect::from_min_size(
                egui::pos2(rect.left(), rect.center().y - CHECK_BOX_SIZE / 2.0),
                Vec2::splat(CHECK_BOX_SIZE),
            );
            let painter = ui.painter();
            if self.indeterminate {
                painter.rect_filled(mark, radius::SM, color::ACCENT_PRIMARY);
                painter.hline(
                    mark.center().x - 4.0..=mark.center().x + 4.0,
                    mark.center().y,
                    Stroke::new(2.0, color::TEXT_ON_PRIMARY),
                );
            } else {
                paint_check_box(painter, mark, checked, response.hovered());
            }
            painter.galley(
                egui::pos2(mark.right() + gap, rect.center().y - galley.size().y / 2.0),
                galley,
                color::TEXT_PRIMARY,
            );
            paint_focus_ring(ui, mark, &response, radius::SM);
        }
        response
    }
}
