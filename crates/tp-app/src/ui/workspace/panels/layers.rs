//! Layers tab: the object tree with selection, rename, visibility, lock,
//! drag and drop, and grouping commands.

use std::collections::HashSet;
use std::sync::Arc;
use tp_i18n::tr;

use egui::{
    Align2, CornerRadius, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
    WidgetInfo, WidgetType,
};
use tp_core::document::tree::{self, Placement};
use tp_core::document::{Object, ObjectId, ShapeKind};
use tp_ui::icons;
use tp_ui::tokens::{color, radius, size, space, stroke};
use tp_ui::widgets::{EmptyState, MenuRow, toggle_icon_button};

use super::PanelEnv;
use super::properties::kind_icon;
use crate::commands::CommandId;
use crate::ui::CommandUi;

/// Side of a row's tile (the kind of object on the artboard's color).
const TILE: f32 = 26.0;
/// Height of a row, as the Textures tab's.
const ROW_HEIGHT: f32 = TILE + 10.0;
const INDENT: f32 = 14.0;
/// Room of a group's disclosure caret.
const CARET: f32 = 14.0;

/// One visible row of the tree.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: ObjectId,
    pub depth: usize,
    pub is_group: bool,
    pub expanded: bool,
}

/// Visible rows, topmost first, descending into expanded groups.
pub fn visible_rows(list: &[Arc<Object>], expanded: &HashSet<ObjectId>) -> Vec<Row> {
    fn walk(list: &[Arc<Object>], expanded: &HashSet<ObjectId>, depth: usize, out: &mut Vec<Row>) {
        for o in list.iter().rev() {
            let open = o.is_group() && expanded.contains(&o.id);
            out.push(Row {
                id: o.id,
                depth,
                is_group: o.is_group(),
                expanded: open,
            });
            if open {
                walk(&o.children, expanded, depth + 1, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(list, expanded, 0, &mut out);
    out
}

/// Where a drop at `fraction` (0 = top, 1 = bottom of the row) lands. The
/// middle of a group row drops into the group.
pub fn drop_placement(row: &Row, fraction: f32) -> Placement {
    if row.is_group && (0.3..=0.7).contains(&fraction) {
        Placement::IntoTop(row.id)
    } else if fraction < 0.5 {
        Placement::Above(row.id)
    } else {
        Placement::Below(row.id)
    }
}

/// Ids of the rows between `a` and `b` (inclusive), in row order.
pub fn row_range(rows: &[Row], a: ObjectId, b: ObjectId) -> Vec<ObjectId> {
    let ia = rows.iter().position(|r| r.id == a);
    let ib = rows.iter().position(|r| r.id == b);
    match (ia, ib) {
        (Some(ia), Some(ib)) => {
            let (lo, hi) = if ia <= ib { (ia, ib) } else { (ib, ia) };
            rows[lo..=hi].iter().map(|r| r.id).collect()
        }
        _ => vec![b],
    }
}

/// Expands every ancestor of the selection so selected rows are visible.
fn reveal_selection(env: &mut PanelEnv<'_>) {
    let objects = &env.ws.project.surface().objects;
    for id in env.ws.selection.clone() {
        let mut current = id;
        while let Some(Some(parent)) = tree::parent_of(objects, current) {
            env.ws.panels.expanded.insert(parent);
            current = parent;
        }
    }
}

/// The Layers tab: a heading naming the texture (or the symbol being
/// edited) and its number of top-level objects, with New Layer (+); then
/// the tree, and the footer's commands.
pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let project = &env.ws.project;
    let name = project
        .edited_symbol()
        .map_or_else(|| project.surface().name.clone(), |s| s.name.clone());
    let heading = tr!(
        "layers-heading",
        texture = name.as_str(),
        count = project.surface().objects.len()
    );
    super::list_heading(ui, &heading, |ui| {
        cmds.add_button(ui, CommandId::NewLayer);
    });
    if env.ws.project.surface().objects.is_empty() {
        EmptyState::new(icons::LAYERS, &tr("empty-layers"), &tr("layers-empty-hint")).show(ui);
        footer(ui, cmds);
        return;
    }
    reveal_selection(env);
    let rows = visible_rows(&env.ws.project.surface().objects, &env.ws.panels.expanded);
    let mut row_rects: Vec<(Row, Rect)> = Vec::with_capacity(rows.len());
    ui.spacing_mut().item_spacing.y = 1.0;
    for row in &rows {
        let rect = row_ui(ui, cmds, env, row, &rows);
        row_rects.push((row.clone(), rect));
    }
    drag_and_drop(ui, env, &row_rects);
    ui.add_space(space::XS);
    footer(ui, cmds);
}

fn footer(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    ui.horizontal(|ui| {
        cmds.icon_button(ui, CommandId::NewLayer, false);
        cmds.icon_button(ui, CommandId::Group, false);
        cmds.icon_button(ui, CommandId::Ungroup, false);
        cmds.icon_button(ui, CommandId::Delete, false);
    });
}

/// Whether `object` follows a graphic or text style.
fn follows_style(object: &Object) -> bool {
    object.style.is_some() || object.text.as_ref().is_some_and(|t| t.style_id.is_some())
}

/// Colors of a row's kind icon and name. Instances and objects following a
/// style are linked to the brand: both in the link color, never alone (an
/// instance shows the symbol icon, a styled object a link icon after its
/// name). Other names are in the secondary color, in ink when `selected`
/// (as the Textures tab's rows). Hidden and locked rows are dimmed.
pub fn row_colors(object: &Object, dim: bool, selected: bool) -> (egui::Color32, egui::Color32) {
    if dim {
        (color::TEXT_DISABLED, color::TEXT_DISABLED)
    } else if object.is_instance() || follows_style(object) {
        (color::LINK, color::LINK)
    } else if selected {
        (color::TEXT_SECONDARY, color::TEXT_PRIMARY)
    } else {
        (color::TEXT_SECONDARY, color::TEXT_SECONDARY)
    }
}

/// Color of the kind glyph on a row's tile (the artboard's color): the
/// object's solid fill when it reads on the tile, a dark ink otherwise.
fn tile_glyph(object: &Object) -> egui::Color32 {
    let ink = color::SURFACE_4;
    let tp_core::document::Paint::Solid(c) = &object.fill else {
        return ink;
    };
    if c.a < 128 || object.is_instance() || object.is_group() {
        return ink;
    }
    let fill = egui::Color32::from_rgb(c.r, c.g, c.b);
    if tp_ui::contrast::contrast_ratio(fill, tp_ui::tokens::canvas::ARTBOARD) >= 2.0 {
        fill
    } else {
        ink
    }
}

/// Draws one row; returns its rectangle.
/// Whether `object`, or for a group an object inside it, is in `flagged`.
fn has_flag(object: &Object, flagged: &std::collections::BTreeSet<ObjectId>) -> bool {
    flagged.contains(&object.id)
        || (object.kind == ShapeKind::Group && object.children.iter().any(|c| has_flag(c, flagged)))
}

/// The objects an update asked to reposition on the edited texture (none
/// while a symbol is edited, or once the texture is checked).
fn to_reposition(project: &tp_core::Project) -> Option<&std::collections::BTreeSet<ObjectId>> {
    if project.edited_symbol().is_some() {
        return None;
    }
    project
        .surface()
        .template
        .as_ref()
        .filter(|t| t.status == tp_core::TemplateStatus::LayoutChanged && !t.flagged.is_empty())
        .map(|t| &t.flagged)
}

fn row_ui(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    env: &mut PanelEnv<'_>,
    row: &Row,
    rows: &[Row],
) -> Rect {
    let object = env
        .ws
        .project
        .surface()
        .get(row.id)
        .cloned()
        .expect("row object exists");
    let (hidden, locked) =
        tree::effective_flags(&env.ws.project.surface().objects, row.id).unwrap_or((false, false));
    let width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click_and_drag());
    let selected = env.ws.selection.contains(&row.id);
    // The name as shown (a text named after its kind reads its first line).
    let (shown, kind_name) = crate::ui::workspace::inspector::name_and_kind(&object);
    let flagged = to_reposition(&env.ws.project).is_some_and(|f| has_flag(&object, f));
    let spoken = if flagged {
        tr!("layers-name-to-reposition", name = shown.as_str())
    } else {
        shown.clone()
    };
    response
        .widget_info(|| WidgetInfo::selected(WidgetType::SelectableLabel, true, selected, &spoken));

    // Background: selected rows are filled (their name in ink), as on the
    // Textures tab.
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, radius::MD, color::SURFACE_3);
    } else if response.hovered() {
        painter.rect_filled(rect, radius::MD, color::SURFACE_2);
    }

    // Rows start at the panel's edge; a caret's room is kept in front of
    // the tiles only when the list holds a group, so that tiles line up.
    let carets = rows.iter().any(|r| r.is_group);
    let mut x = rect.left() + space::XS + 2.0 + row.depth as f32 * INDENT;
    // Disclosure caret for groups.
    if row.is_group {
        let caret_rect = Rect::from_center_size(
            Pos2::new(x + CARET / 2.0, rect.center().y),
            Vec2::splat(18.0),
        );
        let caret = ui.interact(caret_rect, ui.id().with(("caret", row.id)), Sense::click());
        let name = if row.expanded {
            tr!("layers-collapse", name = object.name.as_str())
        } else {
            tr!("layers-expand", name = object.name.as_str())
        };
        caret.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &name));
        ui.painter().text(
            caret_rect.center(),
            Align2::CENTER_CENTER,
            if row.expanded {
                icons::EXPANDED
            } else {
                icons::COLLAPSED
            },
            icons::font(size::ICON - 4.0),
            color::TEXT_SECONDARY,
        );
        if caret.clicked() {
            if row.expanded {
                env.ws.panels.expanded.remove(&row.id);
            } else {
                env.ws.panels.expanded.insert(row.id);
            }
        }
    }
    if carets {
        x += CARET;
    }
    let dim = hidden || locked;
    let styled = follows_style(&object);
    let (_, text_color) = row_colors(&object, dim, selected);
    // The tile: the kind's glyph on the artboard's color.
    let tile = Rect::from_min_size(
        Pos2::new(x, rect.center().y - TILE / 2.0),
        Vec2::splat(TILE),
    );
    let fade = if dim { 0.5 } else { 1.0 };
    let painter = ui.painter();
    painter.rect_filled(
        tile,
        radius::SM,
        tp_ui::tokens::canvas::ARTBOARD.gamma_multiply(fade),
    );
    painter.rect_stroke(
        tile,
        radius::SM,
        Stroke::new(1.0, color::OUTLINE),
        StrokeKind::Outside,
    );
    painter.text(
        tile.center(),
        Align2::CENTER_CENTER,
        kind_icon(object.kind),
        icons::font(size::ICON - 2.0),
        tile_glyph(&object).gamma_multiply(fade),
    );
    x = tile.right() + space::SM;

    // The kind, in small type before the toggles (unless the name says it
    // already).
    let toggles_width = 2.0 * size::HIT_MIN + space::XS;
    let kind_right = rect.right() - toggles_width - space::XS;
    let kind = match kind_name {
        None => Rect::from_min_max(
            Pos2::new(kind_right, rect.top()),
            Pos2::new(kind_right, rect.bottom()),
        ),
        Some(kind_name) => ui.painter().text(
            Pos2::new(kind_right, rect.center().y),
            Align2::RIGHT_CENTER,
            kind_name,
            egui::FontId::proportional(tp_ui::tokens::typography::CAPTION),
            color::TEXT_DISABLED,
        ),
    };

    // Name, or the inline rename editor.
    let name_rect = Rect::from_min_max(
        Pos2::new(x, rect.top()),
        Pos2::new(kind.left() - space::SM, rect.bottom()),
    );
    let renaming = matches!(&env.ws.panels.renaming, Some((id, _)) if *id == row.id);
    if renaming {
        rename_editor(ui, env, row.id, name_rect);
    } else {
        // On one line, ending with "…" when it doesn't fit (the style link
        // icon kept after it), the whole name in a tooltip.
        let link = styled.then(|| {
            ui.painter().layout_no_wrap(
                icons::LINKED.to_owned(),
                icons::font(size::ICON - 4.0),
                text_color,
            )
        });
        let mark = flagged.then(|| {
            ui.painter().layout_no_wrap(
                icons::WARNING.to_owned(),
                icons::font(size::ICON - 4.0),
                color::WARNING,
            )
        });
        let link_room = [&link, &mark]
            .iter()
            .filter_map(|g| g.as_ref())
            .map(|g| g.size().x + space::XS)
            .sum::<f32>();
        let mut job = egui::text::LayoutJob::simple_singleline(
            shown.clone(),
            egui::FontId::proportional(tp_ui::tokens::typography::CONTROL),
            text_color,
        );
        job.wrap =
            egui::text::TextWrapping::truncate_at_width((name_rect.width() - link_room).max(0.0));
        let galley = ui.fonts_mut(|f| f.layout_job(job));
        let elided = galley.elided;
        let text = Rect::from_min_size(
            Pos2::new(name_rect.left(), rect.center().y - galley.size().y / 2.0),
            galley.size(),
        );
        let painter = ui.painter();
        painter.galley(text.min, galley, text_color);
        let mut after = text.right() + space::XS;
        if let Some(link) = link {
            let at = Pos2::new(after, rect.center().y - link.size().y / 2.0);
            after += link.size().x + space::XS;
            painter.galley(at, link, text_color);
        }
        // The update mark: an icon (never color alone) explained on hover.
        if let Some(mark) = mark {
            let at = Pos2::new(after, rect.center().y - mark.size().y / 2.0);
            let mark_rect = Rect::from_min_size(at, mark.size());
            painter.galley(at, mark, color::WARNING);
            ui.interact(
                mark_rect,
                ui.id().with(("reposition", row.id)),
                Sense::hover(),
            )
            .on_hover_text(tr("layers-to-reposition-tooltip"));
        }
        if elided {
            response.clone().on_hover_text(&shown);
        }
    }

    // Visibility and lock toggles.
    let eye_rect = Rect::from_min_size(
        Pos2::new(
            rect.right() - toggles_width,
            rect.center().y - size::HIT_MIN / 2.0,
        ),
        Vec2::splat(size::HIT_MIN),
    );
    let lock_rect = eye_rect.translate(Vec2::new(size::HIT_MIN + space::XS, 0.0));
    let visible = object.visible;
    let unlocked = !object.locked;
    // Child uis: they don't move the list's cursor back up.
    let eye_clicked = {
        let ui = &mut ui.new_child(
            egui::UiBuilder::new()
                .id_salt(("eye", row.id))
                .max_rect(eye_rect),
        );
        toggle_icon_button(
            ui,
            visible,
            icons::VISIBLE,
            icons::HIDDEN,
            &tr!("layers-hide", name = object.name.as_str()),
            &tr!("layers-show", name = object.name.as_str()),
        )
    };
    let lock_clicked = {
        let ui = &mut ui.new_child(
            egui::UiBuilder::new()
                .id_salt(("lock", row.id))
                .max_rect(lock_rect),
        );
        toggle_icon_button(
            ui,
            unlocked,
            icons::UNLOCKED,
            icons::LOCKED,
            &tr!("layers-lock", name = object.name.as_str()),
            &tr!("layers-unlock", name = object.name.as_str()),
        )
    };
    if eye_clicked {
        env.ws.set_visible(row.id, !visible, env.now);
    }
    if lock_clicked {
        env.ws.set_locked(row.id, unlocked, env.now);
    }

    // Selection.
    if response.clicked() && !eye_clicked && !lock_clicked {
        let modifiers = ui.input(|i| i.modifiers);
        select_row(env, rows, row.id, locked, modifiers);
    }
    if response.double_clicked() && !locked {
        env.ws.panels.renaming = Some((row.id, object.name.clone()));
    }
    if response.secondary_clicked() && !selected && !locked {
        env.ws.selection = vec![row.id];
    }
    if response.drag_started() && !locked {
        let dragged = if selected {
            env.ws.selection.clone()
        } else {
            vec![row.id]
        };
        env.ws.panels.layers_drag = Some(dragged);
    }
    response.context_menu(|ui| {
        if ui.add(MenuRow::new(&tr("undo-rename"))).clicked() {
            env.ws.panels.renaming = Some((row.id, object.name.clone()));
            ui.close();
        }
        for id in [
            CommandId::Duplicate,
            CommandId::Delete,
            CommandId::Group,
            CommandId::Ungroup,
            CommandId::NewLayer,
        ] {
            cmds.menu_item(ui, id);
        }
    });
    rect
}

fn select_row(
    env: &mut PanelEnv<'_>,
    rows: &[Row],
    id: ObjectId,
    locked: bool,
    modifiers: egui::Modifiers,
) {
    if locked {
        return;
    }
    let ws = &mut env.ws;
    if modifiers.command {
        if let Some(i) = ws.selection.iter().position(|s| *s == id) {
            ws.selection.remove(i);
        } else {
            ws.selection.push(id);
        }
    } else if modifiers.shift
        && let Some(anchor) = ws.panels.layers_anchor
    {
        let objects = &ws.project.surface().objects;
        ws.selection = row_range(rows, anchor, id)
            .into_iter()
            .filter(|r| tree::effective_flags(objects, *r) == Some((false, false)))
            .collect();
    } else {
        ws.selection = vec![id];
    }
    if !modifiers.shift {
        ws.panels.layers_anchor = Some(id);
    }
    ws.normalize_selection();
}

fn rename_editor(ui: &mut Ui, env: &mut PanelEnv<'_>, id: ObjectId, rect: Rect) {
    let Some((_, buffer)) = &mut env.ws.panels.renaming else {
        return;
    };
    let mut text = buffer.clone();
    let edit_id = ui.id().with(("rename", id));
    tp_ui::widgets::remember_escape(ui, edit_id);
    let response = ui.put(
        rect.shrink2(Vec2::new(0.0, 2.0)),
        TextEdit::singleline(&mut text)
            .id(edit_id)
            .margin(Margin::symmetric(4, 1)),
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, tr("layers-name")));
    if !response.has_focus() && !response.lost_focus() {
        response.request_focus();
    }
    *buffer = text.clone();
    if response.lost_focus() {
        let cancelled = tp_ui::widgets::take_escape(ui, edit_id);
        env.ws.panels.renaming = None;
        if !cancelled {
            env.ws.rename(id, &text, env.now);
        }
    }
}

fn drag_and_drop(ui: &mut Ui, env: &mut PanelEnv<'_>, rows: &[(Row, Rect)]) {
    let Some(dragged) = env.ws.panels.layers_drag.clone() else {
        return;
    };
    let pointer = ui.ctx().pointer_latest_pos();
    let target = pointer.and_then(|p| {
        rows.iter()
            .find(|(_, r)| r.y_range().contains(p.y))
            .map(|(row, r)| {
                let fraction = (p.y - r.top()) / r.height();
                (drop_placement(row, fraction), *r)
            })
    });
    if let Some((placement, r)) = target {
        let painter = ui.painter();
        match placement {
            Placement::IntoTop(_) => {
                painter.rect_stroke(
                    r.shrink(1.0),
                    CornerRadius::same(2),
                    Stroke::new(stroke::FOCUS, color::INDICATOR),
                    StrokeKind::Inside,
                );
            }
            Placement::Above(_) => {
                painter.hline(r.x_range(), r.top(), Stroke::new(2.0, color::INDICATOR));
            }
            Placement::Below(_) | Placement::Top => {
                painter.hline(r.x_range(), r.bottom(), Stroke::new(2.0, color::INDICATOR));
            }
        }
    }
    if ui.input(|i| i.pointer.any_released()) {
        env.ws.panels.layers_drag = None;
        if let Some((placement, _)) = target {
            env.ws.move_objects(&dragged, placement, env.now);
        }
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Frame, ShapeKind};
    use tp_core::kurbo::{Point, Size};

    use super::*;

    fn rect(id: u64) -> Arc<Object> {
        Arc::new(Object::new(
            ObjectId(id),
            ShapeKind::rectangle(),
            Frame::new(Point::ORIGIN, Size::new(10.0, 10.0), 0.0),
        ))
    }

    fn tree() -> Vec<Arc<Object>> {
        vec![
            rect(1),
            Arc::new(Object::group(ObjectId(10), vec![rect(2), rect(3)])),
            rect(4),
        ]
    }

    #[test]
    fn rows_are_topmost_first_and_respect_expansion() {
        let collapsed = visible_rows(&tree(), &HashSet::new());
        assert_eq!(
            collapsed.iter().map(|r| r.id.0).collect::<Vec<_>>(),
            vec![4, 10, 1]
        );
        let expanded = visible_rows(&tree(), &HashSet::from([ObjectId(10)]));
        assert_eq!(
            expanded
                .iter()
                .map(|r| (r.id.0, r.depth))
                .collect::<Vec<_>>(),
            vec![(4, 0), (10, 0), (3, 1), (2, 1), (1, 0)]
        );
    }

    #[test]
    fn drop_zones() {
        let group = Row {
            id: ObjectId(10),
            depth: 0,
            is_group: true,
            expanded: false,
        };
        let shape = Row {
            id: ObjectId(1),
            depth: 0,
            is_group: false,
            expanded: false,
        };
        assert_eq!(
            drop_placement(&group, 0.5),
            Placement::IntoTop(ObjectId(10))
        );
        assert_eq!(drop_placement(&group, 0.1), Placement::Above(ObjectId(10)));
        assert_eq!(drop_placement(&group, 0.9), Placement::Below(ObjectId(10)));
        assert_eq!(drop_placement(&shape, 0.45), Placement::Above(ObjectId(1)));
        assert_eq!(drop_placement(&shape, 0.55), Placement::Below(ObjectId(1)));
    }

    #[test]
    fn instances_and_styled_objects_in_the_link_color() {
        let plain = (*rect(1)).clone();
        assert_eq!(
            row_colors(&plain, false, false),
            (color::TEXT_SECONDARY, color::TEXT_SECONDARY)
        );
        assert_eq!(
            row_colors(&plain, false, true),
            (color::TEXT_SECONDARY, color::TEXT_PRIMARY),
            "selected: in ink"
        );
        let mut styled = plain.clone();
        styled.style = Some(tp_core::document::StyleId(1));
        assert_eq!(
            row_colors(&styled, false, false),
            (color::LINK, color::LINK)
        );
        let mut instance = plain.clone();
        instance.kind = ShapeKind::Instance {
            symbol: tp_core::document::SymbolId(1),
            placement: Default::default(),
        };
        assert_eq!(
            row_colors(&instance, false, true),
            (color::LINK, color::LINK)
        );
        assert_eq!(kind_icon(instance.kind), icons::SYMBOL, "and its icon");
        // Hidden or locked rows are dimmed, linked or not.
        assert_eq!(
            row_colors(&instance, true, false),
            (color::TEXT_DISABLED, color::TEXT_DISABLED)
        );
    }

    #[test]
    fn ranges_follow_row_order() {
        let rows = visible_rows(&tree(), &HashSet::from([ObjectId(10)]));
        let ids = |v: Vec<ObjectId>| v.into_iter().map(|i| i.0).collect::<Vec<_>>();
        assert_eq!(
            ids(row_range(&rows, ObjectId(1), ObjectId(3))),
            vec![3, 2, 1]
        );
        assert_eq!(ids(row_range(&rows, ObjectId(4), ObjectId(4))), vec![4]);
    }
}
