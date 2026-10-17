//! An open project and its editor state: selection, view, history, gesture.

use std::collections::HashSet;

use egui::Rect;
use tp_core::document::tree::{self, Placement};
use tp_core::document::{
    CharStyle, DEFAULT_FILL, Frame, Gradient, GradientKind, History, Object, ObjectId, Paint,
    PaintKind, Rgba, ShapeKind, StrokeStyle, translate,
};
use tp_core::kurbo::Vec2;
use tp_core::{Project, Snapshot};
use tp_i18n::tr;

use crate::geometry_cache::GeometryCache;
use crate::gesture::Gesture;
use crate::image_cache::ImageCache;
use crate::text_engine::TextEngine;
use crate::text_session::TextSession;
use crate::tool::Tool;
use crate::viewport::Viewport;

/// Offset applied by Duplicate and repeated Paste, in texture pixels.
pub const COPY_OFFSET: f64 = 20.0;

/// Document save state shown in the status bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveState {
    Saved,
    Unsaved,
    /// A save to the project file is running.
    Saving,
}

/// A save to the project file in progress.
#[derive(Clone, Debug)]
pub struct PendingSave {
    pub id: u64,
    /// The document being written: it becomes the saved state on success.
    pub snapshot: Snapshot,
    pub path: std::path::PathBuf,
}

/// A short message shown over the canvas until `until` (seconds).
#[derive(Clone, Debug)]
pub struct Hint {
    pub text: String,
    pub until: f64,
}

/// Default width of a newly added stroke, in texture pixels.
pub const DEFAULT_STROKE_WIDTH: f64 = 4.0;

/// Which color the color popover edits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorTarget {
    #[default]
    Fill,
    Stroke,
}

/// Color model shown by the color popover's sliders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorModel {
    #[default]
    Rgb,
    Hsv,
    Hsl,
}

/// Fill and stroke used for new shapes (not part of the document history).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub fill: Paint,
    pub stroke: StrokeStyle,
    pub stroke_enabled: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: Paint::Solid(DEFAULT_FILL),
            stroke: StrokeStyle {
                paint: Paint::Solid(Rgba::rgb(0, 0, 0)),
                width: DEFAULT_STROKE_WIDTH,
                ..Default::default()
            },
            stroke_enabled: false,
        }
    }
}

impl Style {
    pub fn stroke(&self) -> Option<StrokeStyle> {
        self.stroke_enabled.then_some(self.stroke)
    }
}

/// UI state of the editing panels (not part of the document history).
#[derive(Debug, Default)]
pub struct PanelState {
    pub color_target: ColorTarget,
    /// Selected stop of the gradient being edited (clamped on use).
    pub gradient_stop: usize,
    pub color_model: ColorModel,
    /// Last color shown in the picker with its HSV, so hue survives at zero
    /// saturation or value.
    pub picker: Option<(Rgba, tp_ui::widgets::Hsv)>,
    pub lock_proportions: bool,
    /// Groups expanded in the Layers tab.
    pub expanded: HashSet<ObjectId>,
    /// Anchor row for Shift+click range selection in the Layers tab.
    pub layers_anchor: Option<ObjectId>,
    /// Row being renamed, with its edit buffer.
    pub renaming: Option<(ObjectId, String)>,
    /// Rows being dragged in the Layers tab.
    pub layers_drag: Option<Vec<ObjectId>>,
    /// Open font picker.
    pub font_picker: Option<FontPicker>,
    /// Asset being renamed in the Images section, with its edit buffer.
    pub renaming_asset: Option<(tp_core::document::AssetId, String)>,
    /// What the align commands align to (session only).
    pub align_to: crate::arrange::AlignTo,
    /// Open before/after editor of a swatch or a graphic style.
    pub brand_edit: Option<crate::brand_ops::BrandEdit>,
    /// Style being renamed in the Styles panel, with its edit buffer.
    pub renaming_style: Option<(tp_core::document::StyleId, String)>,
    /// Symbol being renamed in the Symbols panel, with its edit buffer.
    pub renaming_symbol: Option<(tp_core::document::SymbolId, String)>,
    /// Used symbol waiting for the deletion confirmation.
    pub confirm_delete_symbol: Option<tp_core::document::SymbolId>,
}

/// State of the font family picker popup.
#[derive(Debug, Default)]
pub struct FontPicker {
    pub query: String,
    /// Highlighted row among the filtered families.
    pub highlighted: usize,
    pub focus_requested: bool,
}

pub struct Workspace {
    pub project: Project,
    /// Identifies this opening of the project in the session (a project
    /// closed and opened again gets a new one); not saved.
    pub session: u64,
    /// The space shown (Project, Workshop or Brand); not saved.
    pub space: crate::layout::Space,
    /// Which textures the Project space lists (All when the project
    /// opens); not saved.
    pub texture_filter: tp_core::TextureFilter,
    pub tool: Tool,
    /// Tool to restore when the temporary Hand tool (Space) is released.
    pub tool_before_space: Option<Tool>,
    /// The project's file, once saved or opened.
    pub path: Option<std::path::PathBuf>,
    /// The document as last saved or opened (`None`: never saved).
    pub saved: Option<Snapshot>,
    /// Save in progress, and a path to save again once it finishes.
    pub saving: Option<PendingSave>,
    pub save_queued: Option<std::path::PathBuf>,
    /// Last recovery copy: when, and of which document.
    pub recovery_at: f64,
    pub recovery_written: Option<Snapshot>,
    /// Selected objects of the active surface, in selection order.
    pub selection: Vec<ObjectId>,
    /// `None` until the canvas size is known (first frame).
    pub viewport: Option<Viewport>,
    /// Views of the other surfaces, by index (`None`: fit when shown).
    pub viewports: Vec<Option<Viewport>>,
    /// Views of the symbols, when not edited.
    pub symbol_viewports: std::collections::HashMap<tp_core::document::SymbolId, Viewport>,
    /// Template opacity or visibility changed since the last save (not in
    /// the undo history).
    pub settings_changed: bool,
    pub history: History<Snapshot>,
    pub gesture: Gesture,
    /// Canvas area on screen during the last frame.
    pub canvas_rect: Option<Rect>,
    pub hint: Option<Hint>,
    pub geometry: GeometryCache,
    /// Fill and stroke for new shapes.
    pub style: Style,
    pub panels: PanelState,
    /// Live panel edit not yet recorded: label and document before it.
    pending: Option<(&'static str, Snapshot)>,
    /// Last color applied, waiting to be added to recent colors once the
    /// interaction ends.
    pub recent_candidate: Option<Rgba>,
    /// Fonts, text layouts and glyph meshes.
    pub text: TextEngine,
    /// Character style for new texts.
    pub text_style: CharStyle,
    /// Text being edited on the canvas.
    pub text_session: Option<TextSession>,
    /// Display textures of image assets.
    pub images: ImageCache,
    /// Thumbnails of the surfaces' artwork and of their templates (the
    /// Textures tab, the Project space).
    pub thumbnails: crate::surface_thumbnails::SurfaceThumbnails,
    /// Thumbnails of the textures affected by the open before/after
    /// editor, with its new value (reset when an edit starts).
    pub preview_thumbs: crate::surface_thumbnails::SurfaceThumbnails,
    /// Usage of the swatches, styles and symbols (see [`Workspace::usage`]).
    pub usage_cache: crate::brand_ops::UsageCache,
    /// The mod's pictures shown by the Project space.
    pub mod_previews: crate::mod_previews::ModPreviews,
    /// Ramp textures of the gradients drawn on the canvas.
    pub gradients: crate::gradient_textures::GradientTextures,
    /// Shadow layers drawn on the canvas.
    pub shadows: crate::shadow_cache::ShadowCache,
    /// The file dialog should open to place images, at a point or the view
    /// center (Place command, Image tool).
    pub place_request: Option<Option<tp_core::kurbo::Point>>,
    /// Where the Project space should take the player once shown: a
    /// field of the Mod information column or a vehicle's card (Show on a
    /// problem); taken by the space on its next frame.
    pub reveal: Option<crate::mod_export::ProblemPlace>,
    /// The text of the mod setting field being typed in, mirrored each
    /// frame so that a command can commit it before the field sees that it
    /// lost the focus (see [`Workspace::commit_mod_draft`]).
    pub mod_draft: Option<(crate::mod_export::ModField, String)>,
    /// The file dialog should open to choose this mod picture.
    pub picture_request: Option<crate::mod_export::ModPicture>,
    /// Where the mod pictures' previews are on screen this frame (shop
    /// icon, Mod Manager image), to take files dropped on them; `None`
    /// when not shown.
    pub picture_drop_zones: [Option<Rect>; 2],
    /// Why the last file chosen or dropped for a picture couldn't be used,
    /// until the next picture change.
    pub picture_error: Option<(crate::mod_export::ModPicture, String)>,
    /// Path being drawn with the Pen tool.
    pub pen: Option<crate::path_edit::PenSession>,
    /// Sides and star settings for new polygons.
    pub polygon_style: crate::path_edit::PolygonStyle,
    /// Line width for new open paths (lines).
    pub line_width: f64,
    /// Dash pattern, caps and joins for new lines.
    pub line_style: tp_core::document::LineStyle,
    /// Dash lengths last used, restored when Dashed is turned back on.
    pub last_dash: tp_core::document::Dash,
    /// Grid, guides and snapping settings (copied from preferences).
    pub aids: crate::prefs::ViewAids,
    /// Guides should be shown (a guide was created while hidden).
    pub request_show_guides: bool,
    /// Snap targets of the gesture in progress.
    pub snapper: Option<crate::snap::Snapper>,
    /// What the gesture in progress snapped to (alignment lines).
    pub snap_hits: Vec<crate::snap::SnapHit>,
    /// Selected path points (Direct Selection tool).
    pub points: std::collections::BTreeSet<tp_core::document::PointRef>,
}

impl Workspace {
    pub fn new(project: Project) -> Self {
        Self::with_text_engine(project, TextEngine::default())
    }

    pub fn with_text_engine(project: Project, text: TextEngine) -> Self {
        Self {
            project,
            session: 0,
            space: crate::layout::Space::Project,
            texture_filter: tp_core::TextureFilter::All,
            tool: Tool::default(),
            tool_before_space: None,
            path: None,
            saved: None,
            saving: None,
            save_queued: None,
            recovery_at: 0.0,
            recovery_written: None,
            selection: Vec::new(),
            viewport: None,
            viewports: Vec::new(),
            symbol_viewports: std::collections::HashMap::new(),
            settings_changed: false,
            history: History::default(),
            gesture: Gesture::Idle,
            canvas_rect: None,
            hint: None,
            geometry: GeometryCache::default(),
            style: Style::default(),
            panels: PanelState::default(),
            pending: None,
            recent_candidate: None,
            text,
            text_style: CharStyle::default(),
            text_session: None,
            images: ImageCache::default(),
            thumbnails: Default::default(),
            preview_thumbs: crate::surface_thumbnails::SurfaceThumbnails::named(
                "preview_thumbnail",
            ),
            usage_cache: Default::default(),
            mod_previews: Default::default(),
            gradients: Default::default(),
            shadows: Default::default(),
            place_request: None,
            reveal: None,
            mod_draft: None,
            picture_request: None,
            picture_drop_zones: [None; 2],
            picture_error: None,
            pen: None,
            polygon_style: Default::default(),
            line_width: tp_core::document::DEFAULT_LINE_WIDTH,
            line_style: Default::default(),
            last_dash: crate::ui::workspace::panels::line_style::DASHED,
            aids: Default::default(),
            request_show_guides: false,
            snapper: None,
            snap_hits: Vec::new(),
            points: Default::default(),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.project
            .snapshot(&self.selection)
            .with_points(self.points.iter().copied())
    }

    /// Which vehicle texture each surface paints, in order.
    fn surface_keys(&self) -> Vec<Option<tp_core::TextureKey>> {
        self.project
            .surfaces
            .iter()
            .map(|s| s.template.as_ref().map(tp_core::SurfaceTemplate::key))
            .collect()
    }

    /// After a change of the surface list (vehicles or variants added or
    /// removed), views stored by index no longer match: every surface is
    /// fitted again when shown.
    fn reset_views_if_reshaped(&mut self, before: &[Option<tp_core::TextureKey>]) -> bool {
        if self.surface_keys() == before {
            return false;
        }
        self.viewports.clear();
        self.viewport = None;
        true
    }

    /// Restores a snapshot with its object and point selection.
    pub fn restore(&mut self, state: &Snapshot) {
        let before = self.project.active_surface;
        let before_symbol = self.project.editing_symbol;
        let keys = self.surface_keys();
        self.selection = self.project.restore(state);
        self.points = state.points().iter().copied().collect();
        if self.reset_views_if_reshaped(&keys) {
            return;
        }
        let after = self.project.active_surface;
        let after_symbol = self.project.editing_symbol;
        if after != before || after_symbol != before_symbol {
            // The restored state is on another surface or symbol: show it
            // with its view.
            self.project.active_surface = before;
            self.project.editing_symbol = before_symbol;
            self.show_view(after, after_symbol);
        }
    }

    /// Stores the current view, then shows texture `index` or, when set,
    /// symbol `symbol`, each with its own view (`None`: fit when shown).
    pub(crate) fn show_view(&mut self, index: usize, symbol: Option<tp_core::document::SymbolId>) {
        let active = self.project.active_surface;
        match self.project.editing_symbol {
            Some(current) => {
                if let Some(v) = self.viewport {
                    self.symbol_viewports.insert(current, v);
                }
            }
            None => {
                self.viewports
                    .resize(self.viewports.len().max(active + 1), None);
                self.viewports[active] = self.viewport;
            }
        }
        self.project.active_surface = index;
        self.project.editing_symbol = symbol;
        let stored = match symbol {
            Some(id) => self.symbol_viewports.remove(&id),
            None => self.viewports.get_mut(index).and_then(Option::take),
        };
        // Never shown: keep a view marked to fit, so the canvas (which may
        // be in the middle of a frame) fits the new surface on its next.
        self.viewport = stored.or_else(|| {
            self.viewport.map(|mut v| {
                v.fitted = true;
                v
            })
        });
    }

    /// Makes surface `index` active, with its own view; the selection and
    /// point selection are cleared and text editing ends.
    pub fn set_active_surface(&mut self, index: usize) {
        if index >= self.project.surfaces.len() {
            return;
        }
        if self.project.editing_symbol.is_some() {
            // Choosing a texture ends the symbol's edit.
            if self.is_editing_text() {
                self.end_text_session(0.0);
            }
            self.selection.clear();
            self.points.clear();
            self.show_view(index, None);
            return;
        }
        if index == self.project.active_surface {
            return;
        }
        if self.is_editing_text() {
            self.end_text_session(0.0);
        }
        self.selection.clear();
        self.points.clear();
        self.swap_view(index);
    }

    /// Stores the current view and shows surface `index` with its own.
    fn swap_view(&mut self, index: usize) {
        let current = self.project.active_surface;
        let needed = self.project.surfaces.len().max(current + 1).max(index + 1);
        self.viewports.resize(needed, None);
        self.viewports[current] = self.viewport;
        self.viewport = self.viewports[index].take();
        self.project.active_surface = index;
    }

    /// Records the change from `before` to the current state, unless nothing
    /// changed in the document.
    pub fn record(&mut self, label: &'static str, before: Snapshot, now: f64, coalesce: bool) {
        debug_assert!(
            tp_i18n::exists(label),
            "undo label {label:?} is not a message id"
        );
        // Links to swatches and styles hold while the values still match:
        // whatever changed a look detached it.
        self.project.relink();
        // An update scales and re-flags objects itself.
        if label != "undo-update-template" {
            self.project.unflag_moved(&before);
        }
        let after = self.snapshot();
        if before.same_document(&after) {
            return;
        }
        self.history.record(label, before, after, now, coalesce);
    }

    /// Runs a change of text styles as one undoable step, then lays the
    /// texts out again: their character settings may have changed on
    /// every texture.
    pub fn text_style_edit(
        &mut self,
        label: &'static str,
        now: f64,
        edit: impl FnOnce(&mut Project, &mut Vec<ObjectId>),
    ) {
        let before = self.snapshot();
        edit(&mut self.project, &mut self.selection);
        self.relayout_all_texts();
        self.record(label, before, now, false);
    }

    /// Runs `edit` on the project and selection as one undoable step.
    pub fn edit(
        &mut self,
        label: &'static str,
        now: f64,
        coalesce: bool,
        edit: impl FnOnce(&mut Project, &mut Vec<ObjectId>),
    ) {
        let before = self.snapshot();
        let keys = self.surface_keys();
        edit(&mut self.project, &mut self.selection);
        self.reset_views_if_reshaped(&keys);
        self.record(label, before, now, coalesce);
    }

    /// Applies a live change (a field being typed, a slider being dragged…).
    /// The first call of an interaction remembers the document; the whole
    /// interaction becomes one undo step on [`Self::commit_pending`].
    pub fn live_edit(
        &mut self,
        label: &'static str,
        edit: impl FnOnce(&mut Project, &mut Vec<ObjectId>),
    ) {
        debug_assert!(
            tp_i18n::exists(label),
            "undo label {label:?} is not a message id"
        );
        if self.pending.is_none() {
            self.pending = Some((label, self.snapshot()));
        }
        edit(&mut self.project, &mut self.selection);
    }

    /// Records the pending live edit, if any, as one undo step.
    pub fn commit_pending(&mut self, now: f64) {
        if let Some((label, before)) = self.pending.take() {
            self.record(label, before, now, false);
        }
    }

    /// Reverts the pending live edit, if any.
    pub fn cancel_pending(&mut self) {
        if let Some((_, before)) = self.pending.take() {
            self.selection = self.project.restore(&before);
        }
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.history.undo() {
            self.restore(&state);
            self.gesture = Gesture::Idle;
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.history.redo() {
            self.restore(&state);
            self.gesture = Gesture::Idle;
        }
    }

    /// Whether the document differs from its saved state (always true for a
    /// project never saved).
    pub fn has_unsaved_changes(&self) -> bool {
        self.settings_changed
            || self
                .saved
                .as_ref()
                .is_none_or(|saved| !self.snapshot().same_saved(saved))
    }

    /// Save state shown in the status bar.
    pub fn save_state(&self) -> SaveState {
        if self.saving.is_some() {
            SaveState::Saving
        } else if self.has_unsaved_changes() {
            SaveState::Unsaved
        } else {
            SaveState::Saved
        }
    }

    /// Marks the current document as saved to `path`.
    pub fn mark_saved(&mut self, path: std::path::PathBuf) {
        self.saved = Some(self.snapshot());
        self.path = Some(path);
        self.settings_changed = false;
    }

    /// Selected objects in stacking order (bottom to top).
    pub fn selected_objects(&self) -> Vec<Object> {
        self.project.selected_objects(&self.selection)
    }

    /// Removes ids that are gone or have a selected ancestor.
    pub fn normalize_selection(&mut self) {
        self.selection =
            tree::normalize_selection(&self.project.surface().objects, &self.selection);
    }

    /// Selects every visible, unlocked top-level object.
    pub fn select_all(&mut self) {
        self.selection = tree::selectable_top_level(&self.project.surface().objects);
    }

    /// Where new objects go: the selected group, or the common parent of the
    /// selection, or the top level.
    pub fn active_layer(&self) -> Option<ObjectId> {
        let objects = &self.project.surface().objects;
        if let [single] = self.selection.as_slice()
            && tree::get(objects, *single).is_some_and(|o| o.is_group())
        {
            return Some(*single);
        }
        let mut parents = self
            .selection
            .iter()
            .map(|id| tree::parent_of(objects, *id).flatten());
        let first = parents.next()??;
        parents.all(|p| p == Some(first)).then_some(first)
    }

    /// Escape: clears the selected path points if any, else the selection.
    pub fn deselect(&mut self) {
        self.clear_points_or_selection();
    }

    /// A shape with the current style.
    pub fn styled_shape(&self, kind: ShapeKind, frame: Frame) -> Object {
        let mut object = Object::new(ObjectId(0), kind, frame);
        object.name = object_name(kind);
        object.fill = self.style.fill;
        object.stroke = self.style.stroke();
        object
    }

    /// Adds a shape with the current style to the active layer, selects it,
    /// records it.
    pub fn create_shape(&mut self, kind: ShapeKind, frame: Frame, now: f64) -> ObjectId {
        let object = self.styled_shape(kind, frame);
        self.create_object(object, now)
    }

    /// Adds `object` to the active layer, selects it, records it.
    pub fn create_object(&mut self, object: Object, now: f64) -> ObjectId {
        let label = match object.kind {
            ShapeKind::Rectangle { .. } => "undo-create-rectangle",
            ShapeKind::Ellipse => "undo-create-ellipse",
            ShapeKind::Polygon { .. } => "undo-create-polygon",
            ShapeKind::Path => "undo-create-path",
            ShapeKind::Group => "undo-create-group",
            ShapeKind::Text => "undo-create-text",
            ShapeKind::Image { .. } | ShapeKind::Instance { .. } => "undo-place",
        };
        self.create_object_as(object, label, now)
    }

    /// [`Self::create_object`] with an explicit undo label.
    pub fn create_object_as(&mut self, object: Object, label: &'static str, now: f64) -> ObjectId {
        let layer = self.active_layer();
        if let Some(layer) = layer {
            self.panels.expanded.insert(layer);
        }
        let mut id = None;
        self.edit(label, now, false, |project, selection| {
            let new = project.add_to(layer, object);
            *selection = vec![new];
            id = Some(new);
        });
        id.expect("shape created")
    }

    pub fn delete_selection(&mut self, now: f64) {
        self.edit("cmd-delete", now, false, |project, selection| {
            project.surface_mut().remove(selection);
            selection.clear();
        });
    }

    pub fn duplicate_selection(&mut self, now: f64) {
        self.edit("cmd-duplicate", now, false, |project, selection| {
            *selection = project.duplicate(selection, Vec2::new(COPY_OFFSET, COPY_OFFSET));
        });
    }

    /// Copies the artwork of surface `source`, another main texture of the
    /// active texture's vehicle, onto the active texture: same positions
    /// (scaled when the sizes differ), on top, selected. One undo step.
    pub fn copy_from_texture(&mut self, source: usize, now: f64) {
        let target = self.project.active_surface;
        let Some(from) = self.project.surfaces.get(source) else {
            return;
        };
        if source == target || from.objects.is_empty() {
            return;
        }
        let objects: Vec<Object> = from.objects.iter().map(|o| (**o).clone()).collect();
        let copies =
            crate::vehicle_project::scale_objects(&objects, from.size, self.project.surface().size);
        self.edit("undo-copy-from-cabin", now, false, |project, selection| {
            *selection = project.add_copies(&copies, Vec2::ZERO, None);
        });
    }

    /// Inserts copies of `objects` with `offset`, selecting them.
    pub fn paste(&mut self, objects: &[Object], offset: f64, now: f64) {
        let layer = self.active_layer();
        self.edit("cmd-paste", now, false, |project, selection| {
            *selection = project.add_copies(objects, Vec2::new(offset, offset), layer);
        });
    }

    pub fn bring_forward(&mut self, now: f64) {
        self.edit("cmd-bring-forward", now, false, |project, selection| {
            project.surface_mut().bring_forward(selection);
        });
    }

    pub fn send_backward(&mut self, now: f64) {
        self.edit("cmd-send-backward", now, false, |project, selection| {
            project.surface_mut().send_backward(selection);
        });
    }

    /// Moves the selection by `(dx, dy)` texture pixels; consecutive nudges
    /// within a second are one undo step.
    pub fn nudge(&mut self, dx: f64, dy: f64, now: f64) {
        let moved = translate(&self.selected_objects(), Vec2::new(dx, dy), false);
        self.edit("undo-nudge", now, true, |project, _| {
            project.surface_mut().replace(&moved);
        });
    }

    /// Groups the selection (Cmd/Ctrl+G).
    pub fn group_selection(&mut self, now: f64) {
        self.edit("cmd-group", now, false, |project, selection| {
            if let Some(group) = project.group(selection) {
                if let Some(o) = project.surface().get(group) {
                    let mut named = (**o).clone();
                    named.name = object_name(ShapeKind::Group);
                    project.surface_mut().replace(&[named]);
                }
                *selection = vec![group];
            }
        });
    }

    /// Ungroups the selected groups (Cmd/Ctrl+Shift+G).
    pub fn ungroup_selection(&mut self, now: f64) {
        self.edit("cmd-ungroup", now, false, |project, selection| {
            let released = project.ungroup(selection);
            if !released.is_empty() {
                *selection = released;
            }
        });
    }

    /// Adds an empty top-level group "Layer N" at the top and selects it.
    pub fn new_layer(&mut self, now: f64) {
        let next = self
            .project
            .surface()
            .objects
            .iter()
            .filter_map(|o| layer_number(&o.name))
            .max()
            .unwrap_or(0)
            + 1;
        self.edit("cmd-new-layer", now, false, |project, selection| {
            let mut layer = Object::group(ObjectId(0), Vec::new());
            layer.name = tr!("object-layer-n", n = next);
            *selection = vec![project.add(layer)];
        });
    }

    /// Moves objects in the tree (Layers drag and drop).
    pub fn move_objects(&mut self, ids: &[ObjectId], placement: Placement, now: f64) {
        let label = if matches!(placement, Placement::IntoTop(_)) {
            "undo-move-to-group"
        } else {
            "undo-reorder"
        };
        let ids = ids.to_vec();
        self.edit(label, now, false, |project, _| {
            project.move_objects(&ids, placement);
        });
    }

    /// Edits one object (rename, visibility, lock…) as one undo step.
    pub fn edit_object(
        &mut self,
        id: ObjectId,
        label: &'static str,
        now: f64,
        f: impl FnOnce(&mut Object),
    ) {
        let Some(mut object) = self.project.surface().get(id).map(|o| (**o).clone()) else {
            return;
        };
        f(&mut object);
        self.edit(label, now, false, |project, _| {
            project.surface_mut().replace(&[object])
        });
    }

    pub fn rename(&mut self, id: ObjectId, name: &str, now: f64) {
        let trimmed = name.trim().to_owned();
        self.edit_object(id, "undo-rename", now, |o| {
            o.name = if trimmed.is_empty() {
                object_name(o.kind)
            } else {
                trimmed
            };
        });
    }

    /// Shows or hides an object; hidden objects leave the selection.
    pub fn set_visible(&mut self, id: ObjectId, visible: bool, now: f64) {
        let label = if visible { "undo-show" } else { "undo-hide" };
        self.edit_object(id, label, now, |o| o.visible = visible);
        if !visible {
            self.drop_from_selection(id);
        }
    }

    /// Locks or unlocks an object; locked objects leave the selection.
    pub fn set_locked(&mut self, id: ObjectId, locked: bool, now: f64) {
        let label = if locked { "undo-lock" } else { "undo-unlock" };
        self.edit_object(id, label, now, |o| o.locked = locked);
        if locked {
            self.drop_from_selection(id);
        }
    }

    /// Removes `id` and its descendants from the selection.
    fn drop_from_selection(&mut self, id: ObjectId) {
        let objects = &self.project.surface().objects;
        self.selection
            .retain(|s| *s != id && !tree::is_ancestor(objects, id, *s));
    }

    /// The paint being edited for `target`: the current style's with no
    /// selection, else the first selected shape's (the first stroked one
    /// for strokes). `None` when no selected shape has a stroke.
    pub fn edited_paint(&self, target: ColorTarget) -> Option<Paint> {
        if self.selection.is_empty() {
            return Some(match target {
                ColorTarget::Fill => self.style.fill,
                ColorTarget::Stroke => self.style.stroke.paint,
            });
        }
        self.selected_shapes()
            .iter()
            .find_map(|o| paint_of(o, target))
    }

    /// Applies a color (live): to the selected stop when the edited paint
    /// is a gradient (the whole gradient then goes to every selected shape),
    /// else as a solid color. With nothing selected, to the current style.
    pub fn apply_color(&mut self, target: ColorTarget, color: Rgba) {
        self.recent_candidate = Some(color);
        match self.edited_paint(target) {
            Some(Paint::Gradient(mut g)) => {
                let i = self.panels.gradient_stop.min(g.stops().len() - 1);
                let mut stops = g.stops().to_vec();
                stops[i].color = color;
                g.set_stops(&stops);
                let label = match target {
                    ColorTarget::Fill => "undo-change-fill-gradient",
                    ColorTarget::Stroke => "undo-change-stroke-gradient",
                };
                self.apply_paint(target, Paint::Gradient(g), label);
            }
            _ => {
                let label = match target {
                    ColorTarget::Fill => "undo-change-fill",
                    ColorTarget::Stroke => "undo-change-stroke",
                };
                self.apply_paint(target, Paint::Solid(color), label);
            }
        }
    }

    /// Sets the paint of `target` (live) on every selected shape, or on the
    /// current style. Shapes without a stroke get one (current width).
    pub fn apply_paint(&mut self, target: ColorTarget, paint: Paint, label: &'static str) {
        self.map_paints(target, label, |_, _| paint);
    }

    /// Replaces the paint of `target` (live) with `f(current, frame)` on
    /// every selected shape (`current` is `None` for a missing stroke, which
    /// then gets one at the current width), or on the current style (with
    /// a unit frame).
    pub fn map_paints(
        &mut self,
        target: ColorTarget,
        label: &'static str,
        mut f: impl FnMut(Option<Paint>, &Frame) -> Paint,
    ) {
        if self.selection.is_empty() {
            let unit = Frame::from_rect(tp_core::kurbo::Rect::new(0.0, 0.0, 1.0, 1.0));
            match target {
                ColorTarget::Fill => self.style.fill = f(Some(self.style.fill), &unit),
                ColorTarget::Stroke => {
                    self.style.stroke.paint = f(Some(self.style.stroke.paint), &unit);
                    self.style.stroke_enabled = true;
                }
            }
            return;
        }
        let width = self.style.stroke.width;
        self.map_selected_shapes(label, |o| match target {
            ColorTarget::Fill => o.fill = f(Some(o.fill), &o.frame),
            ColorTarget::Stroke => {
                let paint = f(o.stroke.map(|s| s.paint), &o.frame);
                let mut stroke = o.stroke.unwrap_or(StrokeStyle {
                    width,
                    ..Default::default()
                });
                stroke.paint = paint;
                o.stroke = Some(stroke);
            }
        });
    }

    /// Changes the kind of paint of `target` (live): a solid color becomes
    /// a gradient from that color to transparent, a gradient keeps its stops
    /// when switching between linear and radial and becomes its first
    /// stop's color when made solid.
    pub fn set_paint_kind(&mut self, target: ColorTarget, kind: PaintKind) {
        let label = match target {
            ColorTarget::Fill => "undo-change-fill-type",
            ColorTarget::Stroke => "undo-change-stroke-type",
        };
        self.panels.gradient_stop = 0;
        let fallback = self.style.stroke.paint;
        self.map_paints(target, label, |paint, _| {
            convert_paint(paint.unwrap_or(fallback), kind)
        });
    }

    /// Swaps fill and stroke paints (Shift+X). Shapes without a stroke get
    /// one in their fill paint.
    pub fn swap_fill_stroke(&mut self, now: f64) {
        if self.selection.is_empty() {
            let style = &mut self.style;
            std::mem::swap(&mut style.fill, &mut style.stroke.paint);
            return;
        }
        let width = self.style.stroke.width;
        self.map_selected_shapes("cmd-swap-fill-and-stroke", |o| match o.stroke {
            Some(mut stroke) => {
                std::mem::swap(&mut o.fill, &mut stroke.paint);
                o.stroke = Some(stroke);
            }
            None => {
                o.stroke = Some(StrokeStyle {
                    paint: o.fill,
                    width,
                    ..Default::default()
                })
            }
        });
        self.commit_pending(now);
    }

    /// Default fill and no stroke (D), for the selection or the current style.
    pub fn default_colors(&mut self, now: f64) {
        if self.selection.is_empty() {
            self.style = Style::default();
            return;
        }
        self.map_selected_shapes("cmd-default-colors", |o| {
            o.fill = Paint::Solid(DEFAULT_FILL);
            o.stroke = None;
        });
        self.commit_pending(now);
    }

    /// Applies `f` to every selected shape (descending into groups) as a live
    /// edit labelled `label`.
    pub fn map_selected_shapes(&mut self, label: &'static str, mut f: impl FnMut(&mut Object)) {
        let mut objects = self.selected_objects();
        for o in &mut objects {
            o.for_each_shape(&mut f);
        }
        self.live_edit(label, |project, _| project.surface_mut().replace(&objects));
    }

    /// Selected shapes (descending into groups).
    pub fn selected_shapes(&self) -> Vec<Object> {
        self.selected_objects()
            .iter()
            .flat_map(|o| o.shapes().into_iter().cloned().collect::<Vec<_>>())
            .collect()
    }

    /// Document-to-screen map of the last frame, if the canvas was shown.
    pub fn screen_map(&self, pixels_per_point: f32) -> Option<crate::viewport::ScreenMap> {
        Some(self.viewport?.map(self.canvas_rect?, pixels_per_point))
    }

    /// The artboard on screen during the last frame.
    pub fn artboard_rect(&self, pixels_per_point: f32) -> Option<Rect> {
        let map = self.screen_map(pixels_per_point)?;
        let size = self.project.surface().size;
        Some(Rect::from_two_pos(
            map.to_screen(tp_core::kurbo::Point::ORIGIN),
            map.to_screen(tp_core::kurbo::Point::new(size.width, size.height)),
        ))
    }

    /// Shows a transient message over the canvas.
    pub fn show_hint(&mut self, text: impl Into<String>, now: f64) {
        self.hint = Some(Hint {
            text: text.into(),
            until: now + 2.5,
        });
    }
}

/// The paint of `target` on `o` (`None`: no stroke).
pub fn paint_of(o: &Object, target: ColorTarget) -> Option<Paint> {
    match target {
        ColorTarget::Fill => Some(o.fill),
        ColorTarget::Stroke => o.stroke.map(|s| s.paint),
    }
}

/// `paint` turned into `kind` (see [`Workspace::set_paint_kind`]).
pub fn convert_paint(paint: Paint, kind: PaintKind) -> Paint {
    let gradient_kind = match kind {
        PaintKind::Solid => return Paint::Solid(paint.first_color()),
        PaintKind::Linear => GradientKind::Linear,
        PaintKind::Radial => GradientKind::Radial,
    };
    match paint {
        Paint::Solid(c) => Paint::Gradient(Gradient::from_color(gradient_kind, c)),
        Paint::Gradient(mut g) => {
            g.set_kind(gradient_kind);
            Paint::Gradient(g)
        }
    }
}

/// The message naming objects of `kind`.
fn object_key(kind: ShapeKind) -> &'static str {
    match kind {
        ShapeKind::Rectangle { .. } => "object-rectangle",
        ShapeKind::Ellipse => "object-ellipse",
        ShapeKind::Polygon { .. } => "object-polygon",
        ShapeKind::Path => "object-path",
        ShapeKind::Group => "object-group",
        ShapeKind::Text => "object-text",
        ShapeKind::Image { .. } => "object-image",
        ShapeKind::Instance { .. } => "object-instance",
    }
}

/// Default name of a new object of `kind`, in the current language.
pub fn object_name(kind: ShapeKind) -> String {
    tr(object_key(kind))
}

/// Whether `name` is a default name of `kind` in any supported language
/// (or the core's English name), so a document made in one language reads
/// the same in another. Stored names are never changed for this.
pub fn is_default_name(kind: ShapeKind, name: &str) -> bool {
    name == kind.name()
        || name == object_name(kind)
        || tp_i18n::tr_all(object_key(kind)).iter().any(|n| n == name)
}

/// The number of a layer named like "Layer 3" in the current language.
pub fn layer_number(name: &str) -> Option<u32> {
    let template = tr!("object-layer-n", n = "\u{1}");
    let (prefix, suffix) = template.split_once('\u{1}')?;
    name.strip_prefix(prefix)?
        .strip_suffix(suffix)?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use tp_core::TextureResolution;
    use tp_core::kurbo::{Point, Size};

    use super::*;

    fn ws() -> Workspace {
        Workspace::new(Project::new("t", TextureResolution::R2048))
    }

    fn frame(x: f64) -> Frame {
        Frame::new(Point::new(x, 100.0), Size::new(100.0, 100.0), 0.0)
    }

    #[test]
    fn undo_creation_and_redo_restores_selection() {
        let mut ws = ws();
        let id = ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        assert_eq!(ws.history.undo_label(), Some("undo-create-rectangle"));
        ws.undo();
        assert!(ws.project.surface().objects.is_empty());
        assert!(ws.selection.is_empty());
        ws.redo();
        assert_eq!(ws.selection, vec![id]);
    }

    #[test]
    fn duplicate_selects_copies_with_offset() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.duplicate_selection(1.0);
        let copy = &ws.selected_objects()[0];
        assert_eq!(copy.frame.center, Point::new(120.0, 120.0));
        assert_eq!(ws.project.surface().objects.len(), 2);
    }

    #[test]
    fn nudges_coalesce() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.nudge(10.0, 0.0, 5.0);
        ws.nudge(10.0, 0.0, 5.5);
        assert_eq!(ws.history.len(), 2);
        ws.undo();
        assert_eq!(ws.selected_objects()[0].frame.center.x, 100.0);
    }

    #[test]
    fn selection_only_changes_are_not_recorded() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        let before = ws.snapshot();
        ws.deselect();
        ws.record("undo-reorder", before, 1.0, false);
        assert_eq!(ws.history.len(), 1);
    }

    #[test]
    fn live_edit_is_one_step_and_cancel_restores() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        for value in [10, 50, 200] {
            ws.apply_color(ColorTarget::Fill, Rgba::rgb(value, 0, 0));
        }
        ws.commit_pending(1.0);
        assert_eq!(ws.history.len(), 2);
        assert_eq!(ws.history.undo_label(), Some("undo-change-fill"));
        assert_eq!(
            ws.selected_objects()[0].fill,
            Paint::Solid(Rgba::rgb(200, 0, 0))
        );

        ws.apply_color(ColorTarget::Fill, Rgba::rgb(0, 255, 0));
        ws.cancel_pending();
        assert_eq!(
            ws.selected_objects()[0].fill,
            Paint::Solid(Rgba::rgb(200, 0, 0))
        );
        assert_eq!(ws.history.len(), 2);
    }

    #[test]
    fn color_with_empty_selection_sets_style_for_new_shapes() {
        let mut ws = ws();
        ws.apply_color(ColorTarget::Fill, Rgba::rgb(255, 255, 255));
        ws.apply_color(ColorTarget::Stroke, Rgba::rgb(0, 0, 0));
        ws.style.stroke.width = 8.0;
        assert!(!ws.has_pending());
        ws.create_shape(ShapeKind::Ellipse, frame(100.0), 0.0);
        let o = &ws.selected_objects()[0];
        assert_eq!(o.fill, Paint::Solid(Rgba::rgb(255, 255, 255)));
        assert_eq!(o.stroke.unwrap().width, 8.0);
    }

    fn red_blue() -> Paint {
        Paint::Gradient(Gradient::new(
            GradientKind::Linear,
            &[
                tp_core::document::ColorStop::new(0.0, Rgba::rgb(255, 0, 0)),
                tp_core::document::ColorStop::new(1.0, Rgba::rgb(0, 0, 255)),
            ],
        ))
    }

    #[test]
    fn color_goes_to_the_selected_stop_of_a_gradient() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.apply_paint(ColorTarget::Fill, red_blue(), "undo-change-fill-type");
        ws.commit_pending(1.0);
        ws.panels.gradient_stop = 1;
        ws.apply_color(ColorTarget::Fill, Rgba::rgb(0, 255, 0));
        ws.commit_pending(2.0);
        assert_eq!(ws.history.undo_label(), Some("undo-change-fill-gradient"));
        let g = *ws.selected_objects()[0].fill.gradient().unwrap();
        assert_eq!(g.stops()[0].color, Rgba::rgb(255, 0, 0));
        assert_eq!(g.stops()[1].color, Rgba::rgb(0, 255, 0));
        assert_eq!(ws.recent_candidate, Some(Rgba::rgb(0, 255, 0)));
    }

    #[test]
    fn paint_kind_changes() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.apply_color(ColorTarget::Fill, Rgba::rgb(255, 0, 0));
        ws.commit_pending(1.0);
        ws.set_paint_kind(ColorTarget::Fill, PaintKind::Linear);
        ws.commit_pending(2.0);
        let g = *ws.selected_objects()[0].fill.gradient().unwrap();
        assert_eq!(g.kind, GradientKind::Linear);
        assert_eq!(g.stops()[0].color, Rgba::rgb(255, 0, 0));
        assert_eq!(g.stops()[1].color, Rgba::with_alpha(255, 0, 0, 0));
        // Linear → radial keeps the stops.
        ws.set_paint_kind(ColorTarget::Fill, PaintKind::Radial);
        ws.commit_pending(3.0);
        let r = *ws.selected_objects()[0].fill.gradient().unwrap();
        assert_eq!((r.kind, r.stops()), (GradientKind::Radial, g.stops()));
        // Gradient → solid: the first stop.
        ws.set_paint_kind(ColorTarget::Fill, PaintKind::Solid);
        ws.commit_pending(4.0);
        assert_eq!(
            ws.selected_objects()[0].fill,
            Paint::Solid(Rgba::rgb(255, 0, 0))
        );
        ws.undo();
        ws.undo();
        ws.undo();
        assert_eq!(
            ws.selected_objects()[0].fill,
            Paint::Solid(Rgba::rgb(255, 0, 0))
        );
    }

    #[test]
    fn swap_and_default_with_gradients() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.apply_paint(ColorTarget::Fill, red_blue(), "undo-change-fill");
        ws.apply_color(ColorTarget::Stroke, Rgba::rgb(0, 0, 0));
        ws.commit_pending(1.0);
        ws.swap_fill_stroke(2.0);
        let o = &ws.selected_objects()[0];
        assert_eq!(o.fill, Paint::Solid(Rgba::rgb(0, 0, 0)));
        assert_eq!(o.stroke.unwrap().paint, red_blue());
        ws.default_colors(3.0);
        let o = &ws.selected_objects()[0];
        assert_eq!(o.fill, Paint::Solid(DEFAULT_FILL));
        assert!(o.stroke.is_none());
    }

    #[test]
    fn recoloring_a_stroke_keeps_its_style() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.apply_color(ColorTarget::Stroke, Rgba::rgb(0, 0, 0));
        ws.map_selected_shapes("undo-change-stroke", |o| {
            o.stroke.as_mut().unwrap().align = tp_core::document::StrokeAlign::Inside;
        });
        ws.apply_color(ColorTarget::Stroke, Rgba::rgb(9, 9, 9));
        let s = ws.selected_objects()[0].stroke.unwrap();
        assert_eq!(s.align, tp_core::document::StrokeAlign::Inside);
        assert_eq!(s.paint, Paint::Solid(Rgba::rgb(9, 9, 9)));
    }

    #[test]
    fn default_names_in_any_language() {
        tp_i18n::set_language(tp_i18n::Language::French);
        for name in ["Text", "Texte", "Texto"] {
            assert!(is_default_name(ShapeKind::Text, name), "{name}");
        }
        assert!(!is_default_name(ShapeKind::Text, "ACE"));
        // Another kind's default name is not this kind's.
        assert!(!is_default_name(ShapeKind::Text, "Rectangle"));
        assert!(is_default_name(ShapeKind::rectangle(), "Rechteck"));
        // The name shown in the long-text mode.
        tp_i18n::set_language(tp_i18n::Language::English);
        tp_i18n::set_lengthening(40);
        let long = object_name(ShapeKind::Text);
        let default = is_default_name(ShapeKind::Text, &long);
        tp_i18n::set_lengthening(0);
        assert!(long.len() > "Text".len() && default, "{long}");
    }

    #[test]
    fn default_names_and_layer_numbers() {
        tp_i18n::set_language(tp_i18n::Language::English);
        let names: Vec<String> = [
            ShapeKind::rectangle(),
            ShapeKind::Ellipse,
            ShapeKind::Polygon {
                sides: 5,
                star: Some(0.5),
            },
            ShapeKind::Path,
            ShapeKind::Group,
            ShapeKind::Text,
            ShapeKind::Image {
                asset: tp_core::document::AssetId(1),
            },
        ]
        .into_iter()
        .map(object_name)
        .collect();
        assert_eq!(
            names,
            [
                "Rectangle",
                "Ellipse",
                "Polygon",
                "Path",
                "Group",
                "Text",
                "Image"
            ]
        );
        assert_eq!(layer_number("Layer 12"), Some(12));
        assert_eq!(layer_number("Layer x"), None);
        assert_eq!(layer_number("Stripes"), None);
        let mut ws = ws();
        ws.new_layer(0.0);
        ws.new_layer(1.0);
        let top = ws.project.surface().objects.last().unwrap().name.clone();
        assert_eq!(top, "Layer 2");
    }

    #[test]
    fn new_shapes_go_to_the_active_layer() {
        let mut ws = ws();
        ws.new_layer(0.0);
        let layer = ws.selection[0];
        assert_eq!(ws.project.surface().get(layer).unwrap().name, "Layer 1");
        let id = ws.create_shape(ShapeKind::Ellipse, frame(100.0), 1.0);
        let group = ws.project.surface().get(layer).unwrap();
        assert_eq!(group.children.last().unwrap().id, id);
        ws.new_layer(2.0);
        assert_eq!(
            ws.project.surface().get(ws.selection[0]).unwrap().name,
            "Layer 2"
        );
    }

    #[test]
    fn hiding_removes_from_selection() {
        let mut ws = ws();
        let id = ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.set_visible(id, false, 1.0);
        assert!(ws.selection.is_empty());
        assert_eq!(ws.history.undo_label(), Some("undo-hide"));
        ws.rename(id, "  ", 2.0);
        assert_eq!(ws.project.surface().get(id).unwrap().name, "Rectangle");
    }

    #[test]
    fn save_state_follows_the_saved_document() {
        let mut ws = ws();
        assert_eq!(ws.save_state(), SaveState::Unsaved, "never saved");
        ws.mark_saved("/tmp/a.truckpaint".into());
        assert_eq!(ws.save_state(), SaveState::Saved);
        ws.create_shape(ShapeKind::Ellipse, frame(100.0), 0.0);
        assert_eq!(ws.save_state(), SaveState::Unsaved);
        ws.undo();
        assert_eq!(ws.save_state(), SaveState::Saved);
        ws.redo();
        assert_eq!(ws.save_state(), SaveState::Unsaved);
        // Selection changes are not document changes.
        ws.undo();
        ws.select_all();
        assert_eq!(ws.save_state(), SaveState::Saved);
    }

    #[test]
    fn delete_and_undo() {
        let mut ws = ws();
        let id = ws.create_shape(ShapeKind::Ellipse, frame(100.0), 0.0);
        ws.delete_selection(1.0);
        assert!(ws.project.surface().objects.is_empty());
        ws.undo();
        assert_eq!(ws.selection, vec![id]);
    }
}
