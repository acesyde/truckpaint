//! The before/after editor of a swatch or a graphic style: one body, shown
//! in a panel on the right of the Brand space ([`panel`]) and in a dialog
//! over the Workshop ([`modal`], Edit Swatch… and Edit Style… from the color
//! popover and the Resources tab).
//!
//! The body edits `ws.panels.brand_edit` only: the new value is shown on a
//! copy of the project (see `brand_ops`), and the document changes when
//! Apply to Fleet records it as one undo step.

use egui::{
    Align, Color32, CornerRadius, Frame, Key, Layout, Margin, Modifiers, Panel, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_core::Look;
use tp_core::document::{Paint, Rgba, StrokeStyle};
use tp_i18n::tr;
use tp_ui::tokens::{canvas, color, radius, space, typography};
use tp_ui::widgets::{
    FieldEvent, Hsv, NumericField, SegmentedControl, ThinSlider, alpha_slider, chip_toggle,
    hue_slider, paint_checkerboard, primary_button, secondary_button, sv_square,
};

use super::panels::PanelEnv;
use super::panels::colors::{from_color32, to_widget_hsv};
use crate::brand_ops::{BrandEditTarget, EditValue, impact_label, look_color};
use crate::workspace::ColorTarget;

/// Width of the editor, in the Brand space and as a dialog.
pub const WIDTH: f32 = 340.0;
/// Padding of the Brand space's editor panel.
const PANEL_PAD: i8 = 18;
/// Height of the saturation/value square.
const SV_HEIGHT: f32 = 160.0;
/// Height of the Before and After blocks.
const BLOCK_HEIGHT: f32 = 36.0;
/// Texture tiles per row of the impact box, and rows shown before it
/// scrolls.
const TILE_COLUMNS: usize = 6;
const TILE_ROWS: usize = 4;
const TILE_GAP: f32 = 6.0;

/// What the footer asks for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    None,
    Apply,
    Cancel,
}

/// The editor as a panel on the right of the Brand space, its footer at the
/// bottom. Drawn by the Brand space before its sections.
pub fn panel(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    if env.ws.panels.brand_edit.is_none() {
        return;
    }
    Panel::right("brand_edit")
        .exact_size(WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(color::SURFACE_1)
                .inner_margin(Margin::same(PANEL_PAD)),
        )
        .show(ui, |ui| {
            let outcome = Panel::bottom("brand_edit_footer")
                .show_separator_line(false)
                .frame(Frame::new().inner_margin(Margin {
                    top: space::MD as i8,
                    ..Margin::ZERO
                }))
                .show(ui, |ui| footer(ui, env))
                .inner;
            ScrollArea::vertical()
                .id_salt("brand_edit_body")
                .auto_shrink([false, false])
                .show(ui, |ui| body(ui, env));
            finish(ui.ctx(), env, outcome);
        });
}

/// The editor as a dialog over the Workshop, for the edits opened outside
/// the Brand space (whose panel shows them).
pub fn modal(ctx: &egui::Context, env: &mut PanelEnv<'_>) {
    if env.ws.panels.brand_edit.is_none() || env.ws.space == crate::layout::Space::Brand {
        return;
    }
    let outcome = crate::ui::dialogs::modal("brand_edit_modal")
        .show(ctx, |ui| {
            ui.set_width(WIDTH - 2.0 * space::XL);
            body(ui, env);
            ui.add_space(space::LG);
            footer(ui, env)
        })
        .inner;
    finish(ctx, env, outcome);
}

/// Cancel (or Escape) and Apply to Fleet.
fn finish(ctx: &egui::Context, env: &mut PanelEnv<'_>, outcome: Outcome) {
    let escape = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
    if escape || outcome == Outcome::Cancel {
        env.ws.cancel_brand_edit();
    } else if outcome == Outcome::Apply {
        env.ws.apply_brand_edit(env.now);
    }
}

/// The editor's state the body edits (everything but the preview project).
struct Fields {
    target: BrandEditTarget,
    name: String,
    before: EditValue,
    after: EditValue,
    hex: String,
    hsv: Hsv,
    look_target: ColorTarget,
}

/// Caption and name, the value's controls, Before / After, the hex code and
/// the impact.
fn body(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let Some(edit) = &env.ws.panels.brand_edit else {
        return;
    };
    let mut f = Fields {
        target: edit.target,
        name: edit.name.clone(),
        before: edit.before,
        after: edit.after,
        hex: edit.hex.clone(),
        hsv: edit.hsv,
        look_target: edit.look_target,
    };
    ui.spacing_mut().item_spacing.y = space::LG;
    caption(ui, &mut f);
    let mut value = match (f.after, f.before) {
        (EditValue::Color(_), _) => color_controls(ui, &mut f),
        (EditValue::Look(after), EditValue::Look(before)) => {
            look_controls(ui, &mut f, &before, after)
        }
        _ => None,
    };
    if let Some(v) = value {
        f.after = v;
    }
    before_after(ui, &f);
    if let Some(v) = hex_field(ui, &mut f) {
        value = Some(v);
    }
    // The tiles show the current thumbnails until the new value is
    // rendered: they are kept up to date in every space.
    let affected = env
        .ws
        .panels
        .brand_edit
        .as_ref()
        .map(|e| e.affected.clone())
        .unwrap_or_default();
    let ws = &mut *env.ws;
    ws.thumbnails
        .update_only(ui.ctx(), &ws.project, &ws.text.fonts, &affected);
    impact(ui, env);
    let picked = f.look_target;
    if let Some(edit) = &mut env.ws.panels.brand_edit {
        (edit.name, edit.hex, edit.hsv, edit.look_target) = (f.name, f.hex, f.hsv, picked);
    }
    if let Some(v) = value {
        env.ws.set_edit_value(v);
    }
}

/// "Edit color" or "Edit style", then the name (editable).
fn caption(ui: &mut Ui, f: &mut Fields) {
    let (caption, name_label) = match f.target {
        BrandEditTarget::Swatch(_) => ("brand-edit-color", "colors-swatch-name"),
        BrandEditTarget::Style(_) => ("brand-edit-style", "styles-name"),
    };
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.label(
            RichText::new(tr(caption))
                .size(typography::CONTROL)
                .color(color::TEXT_SECONDARY),
        );
        let label = tr(name_label);
        let name = ui.add(
            TextEdit::singleline(&mut f.name)
                .id_salt("brand_edit_name")
                .font(egui::FontId::new(17.0, tp_ui::fonts::semibold_family()))
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(6, 3)),
        );
        name.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, &label));
    });
}

/// The picker's color as the new value of a swatch.
fn color_controls(ui: &mut Ui, f: &mut Fields) -> Option<EditValue> {
    let color = picker(ui, &mut f.hsv, true, None)?;
    f.hex = color.to_hex();
    Some(EditValue::Color(color))
}

/// The saturation/value square, the hue and alpha bars; returns the new
/// color when one changed. Disabled with `reason` as its tooltip when the
/// color can't be picked.
fn picker(ui: &mut Ui, hsv: &mut Hsv, enabled: bool, reason: Option<&str>) -> Option<Rgba> {
    ui.add_enabled_ui(enabled, |ui| {
        ui.spacing_mut().item_spacing.y = space::SM;
        let responses = [
            sv_square(ui, hsv, SV_HEIGHT),
            hue_slider(ui, hsv),
            alpha_slider(ui, hsv),
        ];
        let changed = responses.iter().any(|r| r.changed());
        if let Some(reason) = reason {
            for r in responses {
                r.on_disabled_hover_text(reason);
            }
        }
        changed.then(|| from_color32(hsv.to_color32()))
    })
    .inner
}

/// A graphic style's controls: which color the picker edits (Fill or
/// Stroke), the picker, the stroke width with None, and the opacity.
/// Returns the new look when one changed.
fn look_controls(ui: &mut Ui, f: &mut Fields, before: &Look, after: Look) -> Option<EditValue> {
    let mut look = after;
    let mut changed = false;
    let picked = ui
        .push_id("brand_edit_target", |ui| {
            SegmentedControl::new()
                .named_segment(ColorTarget::Fill, "", &tr("props-fill"), &tr("props-fill"))
                .named_segment(
                    ColorTarget::Stroke,
                    "",
                    &tr("panel-stroke"),
                    &tr("panel-stroke"),
                )
                .fill()
                .track(color::FIELD)
                .show(ui, f.look_target)
        })
        .inner;
    if let Some(target) = picked
        && target != f.look_target
    {
        f.look_target = target;
        if let Some(c) = look_color(&look, target) {
            f.hsv = to_widget_hsv(c);
            f.hex = c.to_hex();
        }
    }

    // A gradient fill keeps its colors: Redefine from Selection changes it.
    let gradient = f.look_target == ColorTarget::Fill && matches!(look.fill, Paint::Gradient(_));
    let hint = tr("brand-gradient-fill-hint");
    if gradient {
        ui.label(RichText::new(tr("brand-gradient-fill")).color(color::TEXT_SECONDARY))
            .on_hover_text(&hint);
    }
    let pickable = look_color(&look, f.look_target).is_some();
    if let Some(c) = picker(ui, &mut f.hsv, pickable, gradient.then_some(hint.as_str())) {
        f.hex = c.to_hex();
        look = with_color(before, look, f.look_target, c);
        changed = true;
    }

    // Width and Opacity, their boxes under each other.
    let (width_label, opacity_label) = (tr("field-width"), tr("props-opacity"));
    let column = tp_ui::widgets::label_column(ui, &[&width_label, &opacity_label]);
    ui.horizontal(|ui| {
        let width = look.stroke.map(|s| s.width);
        let e = ui
            .add_enabled_ui(look.stroke.is_some(), |ui| {
                NumericField::new(&width_label, &tr("stroke-width"), width)
                    .label_width(column)
                    .suffix("px")
                    .decimals(1)
                    .speed(0.5)
                    .range(0.5..=500.0)
                    .show(ui)
            })
            .inner;
        if let FieldEvent::Live(v) | FieldEvent::Commit(v) = e
            && let Some(s) = &mut look.stroke
        {
            s.width = v;
            changed = true;
        }
        if chip_toggle(ui, look.stroke.is_none(), true, &tr("stroke-none")).changed() {
            look.stroke = match look.stroke {
                Some(_) => None,
                None => before.stroke.or(Some(StrokeStyle::default())),
            };
            changed = true;
        }
    });

    ui.horizontal(|ui| {
        let opacity = f64::from(look.opacity) * 100.0;
        let e = NumericField::new(&opacity_label, &tr("props-opacity"), Some(opacity))
            .label_width(column)
            .suffix("%")
            .range(0.0..=100.0)
            .width(36.0)
            .show(ui);
        if let FieldEvent::Live(v) | FieldEvent::Commit(v) = e {
            look.opacity = (v / 100.0) as f32;
            changed = true;
        }
        let mut value = opacity as f32;
        let name = tr("props-opacity-slider");
        let slider = ui.add(
            ThinSlider::new(&mut value, 0.0..=100.0, &name)
                .width(ui.available_width().max(40.0))
                .step(1.0),
        );
        if slider.changed() {
            look.opacity = value / 100.0;
            changed = true;
        }
    });
    changed.then_some(EditValue::Look(look))
}

/// `look` with its `target` color set to `c`, keeping the link the style
/// had (`brand_ops` drops it when `c` isn't the swatch's color).
fn with_color(before: &Look, mut look: Look, target: ColorTarget, c: Rgba) -> Look {
    match target {
        ColorTarget::Fill => {
            look.fill = Paint::Solid(c);
            look.fill_swatch = before.fill_swatch;
        }
        ColorTarget::Stroke => {
            if let Some(s) = &mut look.stroke {
                s.paint = Paint::Solid(c);
                s.swatch = before.stroke.and_then(|s| s.swatch);
            }
        }
    }
    look
}

/// The Before and After blocks, side by side.
fn before_after(ui: &mut Ui, f: &Fields) {
    ui.columns(2, |columns| {
        for (ui, (label, value)) in columns
            .iter_mut()
            .zip([("brand-before", &f.before), ("brand-after", &f.after)])
        {
            ui.spacing_mut().item_spacing.y = space::XS;
            let label = tr(label);
            ui.label(
                RichText::new(&label)
                    .size(typography::CAPTION)
                    .color(color::TEXT_SECONDARY),
            );
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), BLOCK_HEIGHT),
                Sense::hover(),
            );
            let text = match value {
                EditValue::Color(c) => c.to_hex(),
                EditValue::Look(look) => look_color(look, f.look_target)
                    .map(Rgba::to_hex)
                    .unwrap_or_default(),
            };
            response.widget_info(|| {
                let mut info = WidgetInfo::labeled(WidgetType::Other, true, &label);
                info.current_text_value = Some(text.clone());
                info
            });
            match value {
                EditValue::Color(c) => paint_color(ui, rect, *c),
                EditValue::Look(look) => paint_look(ui, rect, look),
            }
        }
    });
}

/// Paints `c` in `rect`, over a checkerboard when it's translucent.
fn paint_color(ui: &Ui, rect: Rect, c: Rgba) {
    let painter = ui.painter();
    if c.a < 255 {
        paint_checkerboard(painter, rect, 6.0);
    }
    painter.rect_filled(rect, radius::MD, to_color32(c));
}

fn to_color32(c: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// Paints a style's look in `rect`: its fill, outlined by its stroke.
pub fn paint_look(ui: &Ui, rect: Rect, look: &Look) {
    let painter = ui.painter();
    match &look.fill {
        Paint::Solid(c) => {
            if c.a < 255 {
                paint_checkerboard(painter, rect, 6.0);
            }
            painter.rect_filled(rect, radius::SM, to_color32(*c));
        }
        Paint::Gradient(g) => super::panels::properties::gradient_preview(g).paint(painter, rect),
    }
    if let Some(s) = &look.stroke {
        painter.rect_stroke(
            rect,
            radius::SM,
            Stroke::new(3.0, to_color32(s.paint.first_color())),
            StrokeKind::Inside,
        );
    }
}

/// The hex code of the picker's color; returns the new value when a valid
/// code was typed.
fn hex_field(ui: &mut Ui, f: &mut Fields) -> Option<EditValue> {
    let (label, enabled) = match (f.target, &f.after) {
        (BrandEditTarget::Swatch(_), _) => (tr("colors-swatch-hex"), true),
        (_, EditValue::Look(look)) => (
            tr("colors-hex-color"),
            look_color(look, f.look_target).is_some(),
        ),
        _ => (tr("colors-hex-color"), false),
    };
    let mut value = None;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr("colors-hex"))
                .size(typography::CAPTION)
                .color(color::TEXT_SECONDARY),
        );
        let hex = ui.add_enabled(
            enabled,
            TextEdit::singleline(&mut f.hex)
                .id_salt("brand_edit_hex")
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(8, 5)),
        );
        hex.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, enabled, &label));
        if hex.changed()
            && let Some(c) = Rgba::from_hex(&f.hex)
        {
            f.hsv = to_widget_hsv(c);
            value = match f.after {
                EditValue::Color(_) => Some(EditValue::Color(c)),
                EditValue::Look(look) => {
                    let EditValue::Look(before) = f.before else {
                        return;
                    };
                    Some(EditValue::Look(with_color(&before, look, f.look_target, c)))
                }
            };
        }
    });
    if let Some(v) = value {
        f.after = v;
    }
    value
}

/// The impact box: the impact line, a tile per affected texture with the
/// new value (the current thumbnail until the preview is rendered), and the
/// note.
fn impact(ui: &mut Ui, env: &PanelEnv<'_>) {
    let Some(edit) = &env.ws.panels.brand_edit else {
        return;
    };
    Frame::new()
        .fill(color::SURFACE_2)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(CornerRadius::same(radius::LG))
        .inner_margin(Margin::same(space::MD as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.add(
                egui::Label::new(
                    RichText::new(impact_label(&edit.impact))
                        .family(tp_ui::fonts::semibold_family())
                        .color(color::TEXT_PRIMARY),
                )
                .wrap(),
            );
            if !edit.affected.is_empty() {
                let width = ui.available_width();
                let tile =
                    ((width - TILE_GAP * (TILE_COLUMNS - 1) as f32) / TILE_COLUMNS as f32).floor();
                let rows = edit.affected.len().div_ceil(TILE_COLUMNS).min(TILE_ROWS);
                ScrollArea::vertical()
                    .id_salt("brand_edit_tiles")
                    .max_height(rows as f32 * (tile + TILE_GAP) - TILE_GAP)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(TILE_GAP);
                        for row in edit.affected.chunks(TILE_COLUMNS) {
                            ui.horizontal(|ui| {
                                for &surface in row {
                                    texture_tile(ui, env, surface, tile);
                                }
                            });
                        }
                    });
            }
            ui.add(
                egui::Label::new(
                    RichText::new(tr("brand-impact-note"))
                        .size(typography::CONTROL)
                        .color(color::TEXT_SECONDARY),
                )
                .wrap(),
            );
        });
}

/// The tile of texture `surface`: its preview with the new value.
fn texture_tile(ui: &mut Ui, env: &PanelEnv<'_>, surface: usize, side: f32) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
    let name = env
        .ws
        .project
        .surfaces
        .get(surface)
        .map_or("", |s| s.name.as_str());
    let label = tr!("brand-impact-tile", name = name);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Image, true, &label));
    let painter = ui.painter();
    // The texture's shape, on the artboard's color.
    let size = env
        .ws
        .project
        .surfaces
        .get(surface)
        .map_or(Vec2::splat(1.0), |s| {
            Vec2::new(s.size.width as f32, s.size.height as f32)
        });
    let (shape, uv) = tp_ui::widgets::thumbnail_place(rect, size);
    painter.rect_filled(shape, radius::SM, canvas::ARTBOARD);
    let texture = env
        .ws
        .preview_thumbs
        .texture(surface)
        .or_else(|| env.ws.thumbnails.texture(surface));
    if let Some(texture) = texture {
        painter.image(texture.id(), shape, uv, Color32::WHITE);
    }
    painter.rect_stroke(
        rect,
        radius::SM,
        Stroke::new(1.0, color::BORDER_STRONG),
        StrokeKind::Inside,
    );
    response.on_hover_text(name);
}

/// Cancel and Apply to Fleet, right-aligned; Apply is disabled, with the
/// reason shown above, while the name can't be used.
fn footer(ui: &mut Ui, env: &PanelEnv<'_>) -> Outcome {
    let Some(edit) = &env.ws.panels.brand_edit else {
        return Outcome::None;
    };
    let problem = edit.problem(&env.ws.project).map(tr);
    let mut outcome = Outcome::None;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::SM;
        if let Some(problem) = &problem {
            crate::ui::dialogs::problem(ui, problem);
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = space::SM;
            let apply = ui.add_enabled(
                problem.is_none(),
                primary_button(&tr("brand-apply-to-fleet")),
            );
            let apply = match &problem {
                Some(p) => apply.on_disabled_hover_text(p),
                None => apply,
            };
            if apply.clicked() {
                outcome = Outcome::Apply;
            }
            if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                outcome = Outcome::Cancel;
            }
        });
    });
    outcome
}
