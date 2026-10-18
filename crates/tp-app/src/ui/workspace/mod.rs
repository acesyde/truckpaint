//! Editor workspace: the title bar or the menu row and the top bar, and the
//! status bar, shared by
//! the three spaces, and each space's content. The Workshop is the tool
//! options bar, the tool rail, the left panel, the canvas area and the
//! inspector.

pub mod brand_editor;
pub mod breadcrumb;
pub mod canvas;
pub mod inspector;
pub mod left_panel;
pub mod panels;
pub mod spaces;
pub mod status_bar;
pub mod tool_options;
pub mod tool_rail;
pub mod top_bar;

use egui::{CentralPanel, Frame, Margin, Panel, Ui};
use tp_ui::tokens::{color, size, space};

use super::home::bar_frame;
use super::{CommandUi, menu_bar, title_bar};
use crate::layout::Space;
use crate::state::{AppState, Screen};
use crate::title_bar::{TRAFFIC_LIGHTS_WIDTH, TitleBarMode};

/// Margins of the top bar: on macOS its left keeps the traffic lights free.
pub fn top_bar_margin(ctx: &egui::Context, mode: TitleBarMode) -> Margin {
    let mut margin = Margin::symmetric(space::MD as i8, 0);
    if mode == TitleBarMode::MacNative {
        margin.left = traffic_lights_inset(ctx) as i8;
    }
    margin
}

/// Width of the traffic lights in points (they keep their native size
/// whatever the UI scale).
pub fn traffic_lights_inset(ctx: &egui::Context) -> f32 {
    (TRAFFIC_LIGHTS_WIDTH / ctx.zoom_factor()).round()
}

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let ctx = ui.ctx().clone();
    let edit = state.edit_context();
    let mode = state.title_bar_mode();
    let menu_row = state.draws_menu_row();
    let double_click = state.double_click;
    let mcp = state.mcp.view();
    let AppState {
        prefs,
        preferences_request,
        queue,
        screen,
        vehicles,
        vehicle_request,
        palette_request,
        library,
        ..
    } = state;
    let Screen::Workspace(ws) = screen else {
        return;
    };
    let crate::prefs::Prefs {
        layout,
        recent_colors,
        view_aids,
        ..
    } = prefs;
    let aids = *view_aids;
    let now = ctx.input(|i| i.time);
    let mut cmds = CommandUi::new(&ctx, queue, edit);
    let space = ws.space;
    // The Project space sets where its pictures take dropped files; a mod
    // setting typed in when another space was shown is committed.
    ws.picture_drop_zones = [None; 2];
    if space != Space::Project {
        ws.commit_mod_draft(now);
    }

    // On macOS the top bar is the window's title strip, then the menu row
    // until the menus are in the menu bar; with the drawn title bar it holds
    // the menus, the switcher and Export…; with the system title bar the
    // menu row comes first.
    let mac = mode == TitleBarMode::MacNative;
    if mode == TitleBarMode::Drawn {
        Panel::top("title_bar")
            .exact_size(size::TOP_BAR_HEIGHT)
            .frame(bar_frame().inner_margin(Margin::ZERO))
            .show(ui, |ui| {
                title_bar::show(ui, &mut cmds, Some(space), Some(layout), aids)
            });
    } else {
        if menu_row && !mac {
            Panel::top("menu_bar")
                .frame(bar_frame())
                .show(ui, |ui| menu_bar::show(ui, &mut cmds, Some(layout), aids));
        }
        Panel::top("top_bar")
            .exact_size(size::TOP_BAR_HEIGHT)
            .frame(bar_frame().inner_margin(top_bar_margin(&ctx, mode)))
            .show(ui, |ui| {
                top_bar::show(ui, &mut cmds, ws, mode, double_click)
            });
        if menu_row && mac {
            Panel::top("menu_bar")
                .frame(bar_frame())
                .show(ui, |ui| menu_bar::show(ui, &mut cmds, Some(layout), aids));
        }
    }

    Panel::bottom("status_bar")
        .exact_size(size::STATUS_BAR_HEIGHT)
        .frame(bar_frame().inner_margin(Margin::symmetric(space::MD as i8, 0)))
        .show(ui, |ui| {
            if status_bar::show(ui, &mut cmds, ws, aids, &mcp) {
                *preferences_request = Some(crate::state::PrefsTab::Ai);
                cmds.push(crate::commands::CommandId::Preferences);
            }
        });

    macro_rules! env {
        () => {
            panels::PanelEnv {
                ws: &mut *ws,
                recent_colors: recent_colors.as_slice(),
                vehicles: &*vehicles,
                vehicle_request: &mut *vehicle_request,
                palette_request: &mut *palette_request,
                library: &mut *library,
                now,
            }
        };
    }

    // Renders finish in the background: pick them up in every space, not
    // only in the one that started them, so that they are never left
    // pending when the space changes.
    ws.thumbnails.poll(&ctx);
    ws.mod_previews.poll(&ctx);
    if space != Space::Workshop {
        // The canvas picks them up itself, once a frame.
        ws.shadows.begin_frame(&ctx, ws.text.fonts.generation());
    }

    // The before/after editor of a swatch or a graphic style: a panel of
    // the Brand space, a dialog over the other spaces.
    ws.update_brand_edit(&ctx);
    brand_editor::modal(&ctx, &mut env!());

    match space {
        Space::Project => spaces::project::show(ui, &mut cmds, &mut env!()),
        Space::Brand => spaces::brand::show(ui, &mut cmds, &mut env!()),
        Space::Workshop => {
            Panel::top("tool_options")
                .exact_size(size::TOOL_OPTIONS_HEIGHT)
                .frame(bar_frame().inner_margin(Margin::symmetric(space::MD as i8, 0)))
                .show(ui, |ui| tool_options::show(ui, &mut env!()));

            Panel::left("tool_rail")
                .exact_size(size::TOOL_RAIL_WIDTH)
                .resizable(false)
                .frame(
                    Frame::new()
                        .fill(color::SURFACE_1)
                        .inner_margin(Margin::symmetric(0, space::SM as i8)),
                )
                .show(ui, |ui| tool_rail::show(ui, &mut cmds, ws.tool));

            // Hide Panels (Tab) hides the left panel and the inspector; the
            // rail and the bars stay.
            if !layout.panels_hidden {
                left_panel::show(ui, &mut cmds, layout, &mut env!());
                inspector::show(ui, &mut cmds, layout, &mut env!());
            }

            CentralPanel::no_frame()
                .frame(Frame::new().fill(color::SURFACE_0))
                .show(ui, |ui| {
                    canvas::show(ui, &mut cmds, ws);
                    breadcrumb::inlaid(ui, ws);
                });
        }
    }
}
