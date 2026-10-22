//! The Side View: a strip under the canvas that draws the vehicle of the open texture with the
//! livery being edited, from the direction chosen in its header.

use egui::{Align, Frame, Layout, Margin, Panel, RichText, Sense, Shape, Ui};
use tp_i18n::tr;
use tp_ui::tokens::{color, size, space, typography};
use tp_ui::widgets::{Popover, SegmentedControl, dropdown_icon, secondary_button};
use tp_vehicles::{CHASSIS_SLOT, PreviewMesh};

use super::panels::{self, PanelEnv};
use crate::layout::WorkspaceLayout;
use crate::side_view::{self, Direction, SideView, Target};
use crate::workspace::Workspace;

/// Width of the Accessories list.
const LIST_WIDTH: f32 = 340.0;

fn list_popover() -> egui::Id {
    Popover::id("side_view_accessories")
}

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
            let direction = layout.side_direction;
            let target = env
                .ws
                .side_view
                .target(&env.ws.project, env.vehicles, direction);
            header(ui, layout, &target, env.ws);
            ui.add_space(space::SM);
            body(ui, &target, env);
        });
    let height = panel.response.rect.height().round();
    if (height - layout.side_height).abs() >= 1.0 {
        layout.side_height = height.clamp(*range.start(), *range.end());
    }
}

/// The directions at the right, the Accessories button beside them and the title at the left, when
/// there is room for it (it gives way before the controls do).
fn header(ui: &mut Ui, layout: &mut WorkspaceLayout, target: &Target, ws: &mut Workspace) {
    let drawn = match target {
        Target::Draw { package, mesh, .. } => Some((package, mesh)),
        Target::Message(_) => None,
    };
    let title = tr("side-view-title");
    ui.horizontal(|ui| {
        ui.set_min_height(size::HIT_MIN);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let labels = Direction::ALL.map(|d| tr(d.label()));
            let mut control = SegmentedControl::new();
            for (direction, label) in Direction::ALL.into_iter().zip(&labels) {
                control = control.segment(direction, "", label, None);
            }
            if let Some(direction) = control.show(ui, layout.side_direction) {
                layout.side_direction = direction;
            }
            let button = ui.add_enabled(
                drawn.is_some(),
                secondary_button(&tr("side-view-accessories"))
                    .min_size(egui::vec2(0.0, size::HIT_MIN)),
            );
            if button.clicked() {
                Popover::toggle(ui.ctx(), list_popover());
            }
            if let Some((package, mesh)) = drawn {
                Popover::new(list_popover(), button.rect)
                    .width(LIST_WIDTH)
                    .show(ui, |ui| accessories(ui, ws, package, mesh));
            }
            // What is left of the row, at its left.
            if ui.available_width() >= heading_width(ui, &title) + space::MD {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    panels::section_heading(ui, &title);
                });
            }
        });
    });
}

/// The width of a section heading (see [`panels::section_heading`]).
fn heading_width(ui: &Ui, text: &str) -> f32 {
    let text = text.to_uppercase();
    let font = egui::FontId::new(typography::CAPTION, tp_ui::fonts::semibold_family());
    let galley = ui
        .painter()
        .layout_no_wrap(text.clone(), font, color::TEXT_MUTED);
    galley.size().x + 0.6 * text.chars().count() as f32
}

/// The name of a slot: in the interface language when TruckPaint knows it, else the game's.
fn slot_name(id: &str) -> String {
    let key = format!("side-view-slot-{id}");
    if tp_i18n::exists(&key) {
        tr(&key)
    } else {
        id.to_owned()
    }
}

/// One row per slot with its options; picking one redraws the view and marks the project changed
/// (the choice is not an undo step).
fn accessories(ui: &mut Ui, ws: &mut Workspace, package: &str, mesh: &PreviewMesh) {
    if mesh.slots.is_empty() {
        ui.add(
            egui::Label::new(
                RichText::new(tr("side-view-accessories-empty"))
                    .size(typography::CONTROL)
                    .color(color::TEXT_MUTED),
            )
            .wrap(),
        );
        return;
    }
    let config = side_view::configuration(mesh, ws.project.side_view.get(package));
    let none = tr("side-view-accessory-none");
    let mut picked = None;
    egui::Grid::new("side_view_slots")
        .num_columns(2)
        .spacing([space::MD, space::SM])
        .show(ui, |ui| {
            for (slot, current) in mesh.slots.iter().zip(&config) {
                let name = slot_name(&slot.id);
                ui.label(RichText::new(&name).color(color::TEXT_SECONDARY));
                // Options by name; two sharing one are told apart by their game id.
                let mut order: Vec<usize> = (0..slot.options.len()).collect();
                order.sort_by(|a, b| slot.options[*a].name.cmp(&slot.options[*b].name));
                let shown = |k: usize| {
                    let option = &slot.options[k];
                    let twin = slot
                        .options
                        .iter()
                        .any(|o| o.id != option.id && o.name == option.name);
                    if twin {
                        format!("{} ({})", option.name, option.id)
                    } else {
                        option.name.clone()
                    }
                };
                let selected = current.map_or(none.clone(), shown);
                let combo = egui::ComboBox::from_id_salt(("side_view_slot", &slot.id))
                    .icon(dropdown_icon)
                    .width(ui.available_width())
                    .selected_text(selected)
                    .show_ui(ui, |ui| {
                        if slot.id != CHASSIS_SLOT
                            && ui.selectable_label(current.is_none(), &none).clicked()
                        {
                            picked = Some((slot.id.clone(), None));
                        }
                        for k in order {
                            if ui.selectable_label(*current == Some(k), shown(k)).clicked() {
                                picked = Some((slot.id.clone(), Some(slot.options[k].id.clone())));
                            }
                        }
                    });
                combo.response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, &name)
                });
                ui.end_row();
            }
        });
    if let Some((slot, option)) = picked {
        side_view::choose(&mut ws.project, package, mesh, &slot, option.as_deref());
        ws.settings_changed = true;
    }
}

/// The drawing, or why there is none.
fn body(ui: &mut Ui, target: &Target, env: &mut PanelEnv<'_>) {
    let ws = &mut *env.ws;
    let area = ui.available_rect_before_wrap();
    match target {
        Target::Draw {
            package, drawing, ..
        } if !drawing.is_empty() => {
            // Each texture of the vehicle that the project has is rendered; a piece painted by one it
            // does not have, or not rendered yet, shows in a neutral color.
            let surfaces: Vec<Option<usize>> = drawing
                .textures
                .iter()
                .map(|t| SideView::surface_of(&ws.project, package, t))
                .collect();
            let wanted: Vec<usize> = surfaces.iter().flatten().copied().collect();
            ws.side_view
                .update(ui.ctx(), &ws.project, &ws.text.fonts, &wanted);
            ui.allocate_rect(area, Sense::hover());
            let texture = |id: &str| {
                let at = drawing.textures.iter().position(|t| t == id)?;
                Some(ws.side_view.texture(surfaces[at]?)?.id())
            };
            for mesh in drawing.meshes(area.shrink(space::SM), texture) {
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
