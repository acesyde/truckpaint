//! Workspace layout model (spaces, left panel tab, panel widths), independent
//! of rendering so it can be persisted and unit-tested.

use serde::{Deserialize, Serialize};
use tp_ui::tokens::size;

use crate::side_view::Direction;

/// The three spaces of an open project. Not persisted: opening a project
/// chooses the space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Space {
    /// The fleet and the mod information.
    Project,
    /// Paints one texture.
    Workshop,
    /// The project's palette, styles, symbols and images.
    Brand,
}

impl Space {
    pub const ALL: [Self; 3] = [Self::Project, Self::Workshop, Self::Brand];

    /// Label of the command that shows the space.
    pub fn label(self) -> &'static str {
        match self {
            Self::Project => "cmd-space-project",
            Self::Workshop => "cmd-space-workshop",
            Self::Brand => "cmd-space-brand",
        }
    }
}

/// The tabs of the Workshop's left panel, in display order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeftTab {
    #[default]
    Textures,
    Layers,
    Resources,
}

impl LeftTab {
    pub const ALL: [Self; 3] = [Self::Textures, Self::Layers, Self::Resources];

    /// Label of the tab and of the command that shows it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Textures => "cmd-tab-textures",
            Self::Layers => "cmd-tab-layers",
            Self::Resources => "cmd-tab-resources",
        }
    }
}

/// Everything about the workspace arrangement that survives restarts.
/// Fields of earlier builds (the panel stack, its column width, the
/// Vehicles sidebar) are ignored when read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceLayout {
    /// The tab shown by the Workshop's left panel.
    pub left_tab: LeftTab,
    pub left_width: f32,
    pub inspector_width: f32,
    /// Hide Panels (focus mode): the left panel and the inspector are
    /// hidden.
    pub panels_hidden: bool,
    /// The Side View panel under the canvas is shown.
    pub side_view: bool,
    /// The direction the Side View looks from.
    pub side_direction: Direction,
    pub side_height: f32,
    /// Bumped on reset so egui forgets remembered panel sizes.
    #[serde(skip)]
    pub generation: u32,
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        Self {
            left_tab: LeftTab::default(),
            left_width: size::LEFT_PANEL_DEFAULT,
            inspector_width: size::INSPECTOR_DEFAULT,
            panels_hidden: false,
            side_view: false,
            side_direction: Direction::default(),
            side_height: size::SIDE_VIEW_DEFAULT,
            generation: 0,
        }
    }
}

impl WorkspaceLayout {
    /// Repairs a deserialized layout: widths in range.
    pub fn sanitized(mut self) -> Self {
        self.left_width = clamp_width(
            self.left_width,
            size::LEFT_PANEL_MIN,
            size::LEFT_PANEL_MAX,
            size::LEFT_PANEL_DEFAULT,
        );
        self.inspector_width = clamp_width(
            self.inspector_width,
            size::INSPECTOR_MIN,
            size::INSPECTOR_MAX,
            size::INSPECTOR_DEFAULT,
        );
        self.side_height = if self.side_height.is_finite() {
            self.side_height.max(size::SIDE_VIEW_MIN)
        } else {
            size::SIDE_VIEW_DEFAULT
        };
        self
    }

    /// Shows `tab`, and the panels when they are hidden.
    pub fn show_tab(&mut self, tab: LeftTab) {
        self.left_tab = tab;
        self.panels_hidden = false;
    }

    /// Restores defaults: both panels shown at their default widths, with
    /// the Textures tab and the Side View hidden.
    pub fn reset(&mut self) {
        let generation = self.generation.wrapping_add(1);
        *self = Self {
            generation,
            ..Self::default()
        };
    }
}

fn clamp_width(width: f32, min: f32, max: f32, default: f32) -> f32 {
    if width.is_finite() {
        width.clamp(min, max)
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let layout = WorkspaceLayout::default();
        assert_eq!(layout.left_tab, LeftTab::Textures);
        assert_eq!(layout.left_width, size::LEFT_PANEL_DEFAULT);
        assert_eq!(layout.inspector_width, size::INSPECTOR_DEFAULT);
        assert!(!layout.panels_hidden);
        assert_eq!(layout.clone().sanitized(), layout);
    }

    #[test]
    fn widths_are_clamped() {
        let layout = WorkspaceLayout {
            left_width: 5.0,
            inspector_width: 10_000.0,
            ..WorkspaceLayout::default()
        }
        .sanitized();
        assert_eq!(layout.left_width, size::LEFT_PANEL_MIN);
        assert_eq!(layout.inspector_width, size::INSPECTOR_MAX);
        let layout = WorkspaceLayout {
            left_width: f32::NAN,
            inspector_width: f32::INFINITY,
            ..WorkspaceLayout::default()
        }
        .sanitized();
        assert_eq!(layout.left_width, size::LEFT_PANEL_DEFAULT);
        assert_eq!(layout.inspector_width, size::INSPECTOR_DEFAULT);
    }

    #[test]
    fn the_side_view_defaults_and_height_are_repaired() {
        let layout = WorkspaceLayout::default();
        assert!(!layout.side_view);
        assert_eq!(layout.side_direction, Direction::Left);
        assert_eq!(layout.side_height, size::SIDE_VIEW_DEFAULT);
        let layout = WorkspaceLayout {
            side_height: 3.0,
            ..WorkspaceLayout::default()
        }
        .sanitized();
        assert_eq!(layout.side_height, size::SIDE_VIEW_MIN);
        let layout = WorkspaceLayout {
            side_height: f32::NAN,
            ..WorkspaceLayout::default()
        }
        .sanitized();
        assert_eq!(layout.side_height, size::SIDE_VIEW_DEFAULT);
    }

    #[test]
    fn showing_a_tab_shows_the_panels() {
        let mut layout = WorkspaceLayout {
            panels_hidden: true,
            ..WorkspaceLayout::default()
        };
        layout.show_tab(LeftTab::Resources);
        assert_eq!(layout.left_tab, LeftTab::Resources);
        assert!(!layout.panels_hidden);
    }

    #[test]
    fn reset_workspace_restores_defaults_and_bumps_generation() {
        let mut layout = WorkspaceLayout {
            left_tab: LeftTab::Resources,
            left_width: 400.0,
            inspector_width: 420.0,
            panels_hidden: true,
            side_view: true,
            side_direction: Direction::Back,
            side_height: 400.0,
            generation: 0,
        };
        layout.reset();
        assert_eq!(
            layout,
            WorkspaceLayout {
                generation: 1,
                ..WorkspaceLayout::default()
            }
        );
    }
}
