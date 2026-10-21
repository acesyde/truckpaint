//! The Side View: a strip under the canvas that draws the vehicle of the open texture with the
//! livery being edited, from the direction chosen in its header.

use egui::{Align, Frame, Layout, Margin, Panel, Sense, Shape, Ui};
use tp_i18n::tr;
use tp_ui::tokens::{color, size, space, typography};
use tp_ui::widgets::SegmentedControl;

use super::panels::{self, PanelEnv};
use crate::layout::WorkspaceLayout;
use crate::side_view::{Direction, Target};

/// The panel, resizable by its top edge within bounds; its height is written back to `layout`.
pub fn show(ui: &mut Ui, layout: &mut WorkspaceLayout, env: &mut PanelEnv<'_>) {
    let max = (ui.available_height() * size::SIDE_VIEW_MAX_SHARE).max(size::SIDE_VIEW_MIN);
    let range = size::SIDE_VIEW_MIN..=max;
    let panel = Panel::bottom(egui::Id::new(("side_view", layout.generation)))
        .resizable(true)
        .default_size(layout.side_height.clamp(*range.start(), *range.end()))
        .size_range(range.clone())
        .frame(
            Frame::new()
                .fill(color::SURFACE_1)
                .inner_margin(Margin::same(space::SM as i8 + 2)),
        )
        .show(ui, |ui| {
            header(ui, layout);
            ui.add_space(space::SM);
            body(ui, layout.side_direction, env);
        });
    let height = panel.response.rect.height().round();
    if (height - layout.side_height).abs() >= 1.0 {
        layout.side_height = height.clamp(*range.start(), *range.end());
    }
}

/// The title and the four directions.
fn header(ui: &mut Ui, layout: &mut WorkspaceLayout) {
    ui.horizontal(|ui| {
        ui.set_min_height(size::HIT_MIN);
        panels::section_heading(ui, &tr("side-view-title"));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let labels = Direction::ALL.map(|d| tr(d.label()));
            let mut control = SegmentedControl::new();
            for (direction, label) in Direction::ALL.into_iter().zip(&labels) {
                control = control.segment(direction, "", label, None);
            }
            if let Some(direction) = control.show(ui, layout.side_direction) {
                layout.side_direction = direction;
            }
        });
    });
}

/// The drawing, or why there is none.
fn body(ui: &mut Ui, direction: Direction, env: &mut PanelEnv<'_>) {
    let ws = &mut *env.ws;
    let area = ui.available_rect_before_wrap();
    match ws.side_view.target(&ws.project, env.vehicles, direction) {
        Target::Draw { surface, drawing } if !drawing.is_empty() => {
            ws.side_view
                .update(ui.ctx(), &ws.project, &ws.text.fonts, surface);
            ui.allocate_rect(area, Sense::hover());
            if let Some(texture) = ws.side_view.texture(surface) {
                let mesh = drawing.mesh(area.shrink(space::SM), texture.id());
                ui.painter().add(Shape::mesh(mesh));
            }
        }
        Target::Draw { .. } => message(ui, "side-view-none"),
        Target::Message(id) => message(ui, id),
    }
}

fn message(ui: &mut Ui, id: &str) {
    ui.centered_and_justified(|ui| {
        ui.add(
            egui::Label::new(
                egui::RichText::new(tr(id))
                    .size(typography::CONTROL)
                    .color(color::TEXT_MUTED),
            )
            .wrap(),
        );
    });
}
