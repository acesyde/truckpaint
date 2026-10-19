use egui::{
    Align2, Button, CornerRadius, Response, RichText, Sense, Stroke, Ui, Vec2, Widget, WidgetInfo,
    WidgetType,
};

use super::{name_and_shortcut_tooltip, paint_focus_ring};
use crate::tokens::{color, radius, size};

/// Small icon-only button. A tooltip name is mandatory (spec: icon-only
/// controls always have a tooltip).
pub struct IconButton<'a> {
    icon: &'a str,
    name: &'a str,
    shortcut: Option<&'a str>,
    selected: bool,
    disabled_reason: Option<&'a str>,
    size: Vec2,
    framed: bool,
}

impl<'a> IconButton<'a> {
    pub fn new(icon: &'a str, name: &'a str) -> Self {
        Self {
            icon,
            name,
            shortcut: None,
            selected: false,
            disabled_reason: None,
            size: Vec2::splat(size::HIT_MIN),
            framed: false,
        }
    }

    /// A button of `size` (at least the minimum hit area) on the control
    /// surface, as the inspector's align buttons.
    pub fn framed(mut self, size: Vec2) -> Self {
        self.size = size.max(Vec2::splat(size::HIT_MIN));
        self.framed = true;
        self
    }

    pub fn shortcut(mut self, shortcut: Option<&'a str>) -> Self {
        self.shortcut = shortcut;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Explanation shown in the tooltip when the button is disabled.
    pub fn disabled_reason(mut self, reason: &'a str) -> Self {
        self.disabled_reason = Some(reason);
        self
    }
}

impl Widget for IconButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Button, enabled, self.selected, self.name)
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let corner = CornerRadius::same(if self.framed { radius::MD } else { radius::SM });
            if self.framed {
                painter.rect_filled(rect, corner, color::CONTROL);
            }
            let fg = if !enabled {
                color::TEXT_DISABLED
            } else if response.hovered() || self.selected {
                color::TEXT_PRIMARY
            } else {
                color::TEXT_SECONDARY
            };
            if enabled && response.is_pointer_button_down_on() {
                painter.rect_filled(rect, corner, color::SURFACE_4);
            } else if enabled && response.hovered() {
                painter.rect_filled(rect, corner, color::SURFACE_3);
            } else if self.selected {
                painter.rect_filled(rect, corner, color::SELECTED);
            }
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                self.icon,
                crate::icons::font(size::ICON),
                fg,
            );
            paint_focus_ring(ui, rect, &response, radius::SM);
        }

        name_and_shortcut_tooltip(response, self.name, self.shortcut, self.disabled_reason)
    }
}

/// Icon button with two states (e.g. eye open/closed, lock open/closed): the
/// state is shown by the glyph shape, and the name says what a click does.
/// Returns true when clicked.
pub fn toggle_icon_button(
    ui: &mut Ui,
    on: bool,
    on_icon: &str,
    off_icon: &str,
    name_when_on: &str,
    name_when_off: &str,
) -> bool {
    let (icon, name) = if on {
        (on_icon, name_when_on)
    } else {
        (off_icon, name_when_off)
    };
    ui.add(IconButton::new(icon, name).selected(!on)).clicked()
}

/// Call-to-action button: the primary accent (white) with dark semibold
/// text.
pub fn primary_button(text: &str) -> Button<'static> {
    Button::new(
        RichText::new(text.to_owned())
            .color(color::TEXT_ON_PRIMARY)
            .family(crate::fonts::semibold_family()),
    )
    .fill(color::ACCENT_PRIMARY)
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(radius::MD))
    .min_size(Vec2::new(88.0, 30.0))
}

/// Neutral button.
pub fn secondary_button(text: &str) -> Button<'static> {
    Button::new(text.to_owned())
        .corner_radius(CornerRadius::same(radius::MD))
        .min_size(Vec2::new(88.0, 30.0))
}

/// A [`secondary_button`] with `icon` before its text.
pub fn secondary_icon_button(icon: &str, text: &str) -> Button<'static> {
    Button::new((
        crate::icons::rich(icon).color(color::TEXT_PRIMARY),
        text.to_owned(),
    ))
    .corner_radius(CornerRadius::same(radius::MD))
    .min_size(Vec2::new(88.0, 30.0))
}

/// The ⋯ button of a row or card: a plain icon (a fill only on hover)
/// opening a menu of `content`. `name` is its accessible name and tooltip.
pub fn more_menu<R>(ui: &mut Ui, name: &str, content: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let button = Button::new(crate::icons::rich(crate::icons::MORE).color(color::TEXT_SECONDARY))
        .frame(true)
        .frame_when_inactive(false)
        .min_size(Vec2::splat(size::HIT_MIN));
    let (response, inner) = egui::containers::menu::MenuButton::from_button(button).ui(ui, content);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name));
    response.on_hover_text(name);
    inner.map(|i| i.inner)
}

/// Width and height of a [`paint_switch`], in points.
pub const SWITCH_SIZE: Vec2 = Vec2::new(24.0, 14.0);

/// Paints an on/off switch in `rect` ([`SWITCH_SIZE`]): on, a white track
/// with a dark knob on the right; off, a dark track with a light knob on
/// the left. The knob's side tells the state without color.
pub fn paint_switch(ui: &Ui, rect: egui::Rect, on: bool, enabled: bool) {
    let painter = ui.painter();
    let (track, knob) = match (on, enabled) {
        (true, true) => (color::ACCENT_PRIMARY, color::TEXT_ON_PRIMARY),
        (true, false) => (color::TEXT_DISABLED, color::SURFACE_1),
        (false, true) => (color::BORDER_STRONG, color::TEXT_SECONDARY),
        (false, false) => (color::BORDER, color::TEXT_DISABLED),
    };
    let r = rect.height() / 2.0;
    painter.rect_filled(rect, CornerRadius::same(r as u8), track);
    let x = if on {
        rect.right() - r
    } else {
        rect.left() + r
    };
    painter.circle_filled(egui::pos2(x, rect.center().y), r - 2.0, knob);
}

/// Paints the small ▾ of a dropdown centered on `center`.
pub fn paint_dropdown_arrow(painter: &egui::Painter, center: egui::Pos2, ink: egui::Color32) {
    let (w, h) = (7.0, 4.0);
    painter.add(egui::Shape::convex_polygon(
        vec![
            center + egui::vec2(-w / 2.0, -h / 2.0),
            center + egui::vec2(w / 2.0, -h / 2.0),
            center + egui::vec2(0.0, h / 2.0),
        ],
        ink,
        Stroke::NONE,
    ));
}

/// The icon of every combo box (`ComboBox::icon`): the small muted ▾.
pub fn dropdown_icon(
    ui: &Ui,
    rect: egui::Rect,
    _visuals: &egui::style::WidgetVisuals,
    _is_open: bool,
) {
    let ink = if ui.is_enabled() {
        color::TEXT_MUTED
    } else {
        color::TEXT_DISABLED
    };
    paint_dropdown_arrow(ui.painter(), rect.center(), ink);
}

/// A chip that toggles a choice (the cabins of New Project): a pill filled
/// white with dark text when on, outlined when off, so that the state shows
/// in grayscale too. Reported as a checkbox named `label`; its response is
/// marked changed when clicked. A disabled chip keeps its state, dimmed;
/// `enabled` rather than a disabled scope, which would stop a wrapped row
/// from wrapping.
pub fn chip_toggle(ui: &mut Ui, on: bool, enabled: bool, label: &str) -> Response {
    let enabled = enabled && ui.is_enabled();
    let font = egui::FontId::proportional(crate::tokens::typography::CONTROL);
    let ink = if on {
        color::TEXT_ON_PRIMARY
    } else {
        color::TEXT_PRIMARY
    };
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font, ink);
    let pad = Vec2::new(10.0, 5.0);
    let size = Vec2::new(
        galley.size().x + 2.0 * pad.x,
        size::HIT_MIN.max(galley.size().y + 2.0 * pad.y),
    );
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, mut response) = ui.allocate_exact_size(size, sense);
    if response.clicked() {
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, on, label));
    if ui.is_rect_visible(rect) {
        // A chip that can't be turned off stays readable: its fill fades,
        // its dark text doesn't (so the disabled Ui's own fading is undone).
        let mut painter = ui.painter().clone();
        painter.set_opacity(1.0);
        let painter = &painter;
        let fade = if enabled { 1.0 } else { 0.5 };
        let corner = CornerRadius::same((rect.height() / 2.0) as u8);
        if on {
            let fill = if enabled { 1.0 } else { 0.75 };
            painter.rect_filled(rect, corner, color::ACCENT_PRIMARY.gamma_multiply(fill));
        } else {
            let outline = if enabled && response.hovered() {
                color::TEXT_SECONDARY
            } else {
                color::BORDER_STRONG
            };
            painter.rect_stroke(
                rect,
                corner,
                Stroke::new(1.0, outline),
                egui::StrokeKind::Inside,
            );
        }
        let ink = if on { ink } else { ink.gamma_multiply(fade) };
        painter.galley(rect.center() - galley.size() / 2.0, galley, ink);
        paint_focus_ring(ui, rect, &response, (rect.height() / 2.0) as u8);
    }
    response
}
