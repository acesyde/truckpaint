//! Command registry: every user action (menu, tool rail, context menu,
//! shortcut) is a [`CommandId`] with metadata and a single dispatch path.

use egui::{Key, KeyboardShortcut, Modifiers};
use tp_i18n::tr;
use tp_ui::icons;

use tp_core::document::{BooleanOp, DistributeAxis, DistributeMode, Edge, FlipAxis};

use crate::layout::{LeftTab, Space};
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandId {
    // File
    NewProject,
    OpenProject,
    Save,
    SaveAs,
    Place,
    CloseProject,
    Preferences,
    Quit,
    // Edit
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Duplicate,
    Delete,
    SelectAll,
    // Object
    EditText,
    Group,
    Ungroup,
    ConvertToSymbol,
    ImportFromLibrary,
    EditSymbol,
    DetachInstance,
    FinishSymbol,
    ConvertToPath,
    CreateOutlines,
    Align(Edge),
    Distribute(DistributeAxis, DistributeMode),
    Combine(BooleanOp),
    BringForward,
    SendBackward,
    Flip(FlipAxis),
    // Layer
    NewLayer,
    DuplicateLayer,
    DeleteLayer,
    // View
    CommandPalette,
    ZoomIn,
    ZoomOut,
    FitToScreen,
    ActualSize,
    ShowSpace(Space),
    ShowLeftTab(LeftTab),
    TogglePanels,
    ResetWorkspace,
    ShowGrid,
    ShowGuides,
    ClearGuides,
    Snapping,
    DesignGallery,
    // Vehicle
    VehicleLibrary,
    AddVehicle,
    NextTexture,
    PreviousTexture,
    CopyFromCabin,
    UpdateTemplate,
    ShowTemplate,
    // Export
    ExportTexture,
    ExportMod,
    // Help
    KeyboardShortcuts,
    RunFirstLaunch,
    About,
    // Colors
    SwapColorTarget,
    SwapFillStroke,
    DefaultColors,
    // Canvas
    Deselect,
    Nudge(Direction, bool),
    // Tools
    SelectTool(Tool),
}

/// Arrow-key nudge direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Up, Self::Down];

    /// Unit step in texture pixels.
    pub fn delta(self) -> (f64, f64) {
        match self {
            Self::Left => (-1.0, 0.0),
            Self::Right => (1.0, 0.0),
            Self::Up => (0.0, -1.0),
            Self::Down => (0.0, 1.0),
        }
    }
}

/// State that decides whether commands can run, built once per frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct EditContext {
    pub has_project: bool,
    /// Textures (surfaces) of the open project.
    pub texture_count: usize,
    pub has_selection: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub has_clipboard: bool,
    /// A canvas drag (move, resize, draw…) is in progress.
    pub gesture_active: bool,
    pub undo_label: Option<&'static str>,
    pub redo_label: Option<&'static str>,
    /// At least one selected object is a group.
    pub selection_has_group: bool,
    /// Exactly one editable (visible, unlocked) text is selected.
    pub single_text: bool,
    /// A text is being edited on the canvas.
    pub editing_text: bool,
    /// The selection holds a rectangle, ellipse or polygon (maybe in a group).
    pub selection_has_convertible: bool,
    /// The active surface has guides.
    pub has_guides: bool,
    /// Number of selected objects.
    pub selection_count: usize,
    /// The selection holds a text (maybe in a group).
    pub selection_has_text: bool,
    /// Why the selection cannot be combined (None: it can).
    pub combine_block: Option<&'static str>,
    /// Align to: Key object (aligning needs two objects).
    pub align_to_key: bool,
    /// The active surface has a template, and whether it is shown.
    pub has_template: bool,
    pub template_visible: bool,
    /// A newer installed version of the project's vehicle can be applied.
    pub update_available: bool,
    /// Other main textures of the active texture's vehicle (Copy From
    /// Cabin).
    pub cabin_sources: usize,
    /// A symbol is edited instead of a texture.
    pub editing_symbol: bool,
    /// The selection holds an instance (maybe in a group).
    pub selection_has_instance: bool,
    /// The selection is instances only.
    pub only_instances: bool,
    /// Exactly one instance is selected.
    pub single_instance: bool,
    /// The space shown (`None` without a project).
    pub space: Option<Space>,
}

impl CommandId {
    /// Commands about the active texture, disabled while a symbol is
    /// edited.
    pub fn needs_texture(self) -> bool {
        matches!(
            self,
            Self::CopyFromCabin
                | Self::NextTexture
                | Self::PreviousTexture
                | Self::UpdateTemplate
                | Self::ShowTemplate
                | Self::ExportTexture
                | Self::ExportMod
                | Self::AddVehicle
                | Self::ConvertToSymbol
                | Self::ImportFromLibrary
                | Self::EditSymbol
                | Self::DetachInstance
        )
    }

    /// Commands the command palette lists: all but the held or repeated
    /// keys (Nudge) and the palette itself.
    pub fn in_palette(self) -> bool {
        !matches!(self, Self::Nudge(..) | Self::CommandPalette)
    }

    /// Commands whose key acts only in the Workshop (`Tab`, `G`).
    pub fn key_needs_workshop(self) -> bool {
        matches!(self, Self::TogglePanels | Self::ShowTemplate)
    }

    /// Commands that change shapes, which an instance's are not: they are
    /// its symbol's.
    pub fn changes_shapes(self) -> bool {
        matches!(
            self,
            Self::ConvertToPath | Self::CreateOutlines | Self::Combine(_)
        )
    }
}

fn can_align(c: &EditContext) -> bool {
    editable_selection(c)
        && !c.editing_text
        && c.selection_count >= if c.align_to_key { 2 } else { 1 }
}

fn can_distribute(c: &EditContext) -> bool {
    editable_selection(c) && !c.editing_text && c.selection_count >= 3
}

fn editable_selection(c: &EditContext) -> bool {
    c.has_project && c.has_selection && !c.gesture_active
}

/// Where a command's shortcut is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Always, even while typing in a text field.
    Global,
    /// On any screen, but not while typing.
    App,
    /// Only with a project open, and not while typing.
    Workspace,
}

/// Whether a command can run.
#[derive(Clone, Copy, Debug)]
pub enum Availability {
    Always,
    NeedsProject,
    /// Depends on the editing state; the string explains when it is disabled.
    When(fn(&EditContext) -> bool, &'static str),
    /// Feature not built yet; the string explains it in tooltips.
    NotYet(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct CommandMeta {
    pub label: &'static str,
    pub icon: Option<&'static str>,
    /// First entry is the one displayed; others are alternatives.
    pub shortcuts: &'static [KeyboardShortcut],
    pub scope: Scope,
    pub availability: Availability,
}

const fn sc(modifiers: Modifiers, key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(modifiers, key)
}

const CMD: Modifiers = Modifiers::COMMAND;
const CMD_SHIFT: Modifiers = Modifiers::COMMAND.plus(Modifiers::SHIFT);
const CMD_ALT: Modifiers = Modifiers::COMMAND.plus(Modifiers::ALT);
const ALT: Modifiers = Modifiers::ALT;
const ALT_SHIFT: Modifiers = Modifiers::ALT.plus(Modifiers::SHIFT);
const NONE: Modifiers = Modifiers::NONE;
const SHIFT: Modifiers = Modifiers::SHIFT;

const NEEDS_SELECTION: &str = "reason-no-selection";

impl CommandId {
    /// Every command, including each parameterized variant.
    pub fn all() -> Vec<Self> {
        use CommandId::*;
        let mut all = vec![
            NewProject,
            OpenProject,
            Save,
            SaveAs,
            Place,
            CloseProject,
            Preferences,
            Quit,
            Undo,
            Redo,
            Cut,
            Copy,
            Paste,
            Duplicate,
            Delete,
            SelectAll,
            EditText,
            Group,
            Ungroup,
            ConvertToSymbol,
            ImportFromLibrary,
            EditSymbol,
            DetachInstance,
            FinishSymbol,
            ConvertToPath,
            CreateOutlines,
            BringForward,
            SendBackward,
            NewLayer,
            DuplicateLayer,
            DeleteLayer,
            CommandPalette,
            ZoomIn,
            ZoomOut,
            FitToScreen,
            ActualSize,
            ResetWorkspace,
            ShowGrid,
            ShowGuides,
            ClearGuides,
            Snapping,
            DesignGallery,
            VehicleLibrary,
            AddVehicle,
            NextTexture,
            PreviousTexture,
            CopyFromCabin,
            UpdateTemplate,
            ShowTemplate,
            ExportTexture,
            ExportMod,
            KeyboardShortcuts,
            RunFirstLaunch,
            About,
        ];
        all.extend(Space::ALL.map(ShowSpace));
        all.extend(LeftTab::ALL.map(ShowLeftTab));
        all.push(TogglePanels);
        all.extend([SwapColorTarget, SwapFillStroke, DefaultColors, Deselect]);
        for direction in Direction::ALL {
            all.push(Nudge(direction, false));
            all.push(Nudge(direction, true));
        }
        all.extend(Tool::ALL.map(SelectTool));
        all.extend(Edge::ALL.map(Align));
        all.extend(FlipAxis::ALL.map(Flip));
        all.extend(BooleanOp::ALL.map(Combine));
        for axis in [DistributeAxis::Horizontal, DistributeAxis::Vertical] {
            for mode in [DistributeMode::Centers, DistributeMode::Spacing] {
                all.push(Distribute(axis, mode));
            }
        }
        all
    }

    pub fn meta(self) -> CommandMeta {
        use Availability::{Always, NeedsProject, When};
        use CommandId::*;
        use Scope::{App, Global, Workspace};

        let m = |label, icon, shortcuts, scope, availability| CommandMeta {
            label,
            icon,
            shortcuts,
            scope,
            availability,
        };
        match self {
            NewProject => m(
                "cmd-new-project",
                Some(icons::NEW_PROJECT),
                const { &[sc(CMD, Key::N)] },
                Global,
                Always,
            ),
            OpenProject => m(
                "cmd-open-project",
                Some(icons::OPEN_PROJECT),
                const { &[sc(CMD, Key::O)] },
                Global,
                When(|c| !c.gesture_active, ""),
            ),
            Save => m(
                "cmd-save",
                Some(icons::SAVE),
                const { &[sc(CMD, Key::S)] },
                Global,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            SaveAs => m(
                "cmd-save-as",
                None,
                const { &[sc(CMD_SHIFT, Key::S)] },
                Global,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            Place => m(
                "cmd-place",
                Some(icons::IMAGE),
                const { &[sc(CMD_SHIFT, Key::P)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),
            CloseProject => m(
                "cmd-close",
                None,
                const { &[sc(CMD, Key::W)] },
                Global,
                NeedsProject,
            ),
            Preferences => m(
                "cmd-preferences",
                Some(icons::SETTINGS),
                const { &[sc(CMD, Key::Comma)] },
                Global,
                Always,
            ),
            Quit => m(
                "cmd-quit",
                None,
                const { &[sc(CMD, Key::Q)] },
                Global,
                Always,
            ),

            Undo => m(
                "cmd-undo",
                None,
                const { &[sc(CMD, Key::Z)] },
                Workspace,
                When(
                    |c| c.has_project && (c.can_undo || c.editing_text) && !c.gesture_active,
                    "reason-nothing-to-undo",
                ),
            ),
            Redo => m(
                "cmd-redo",
                None,
                const { &[sc(CMD_SHIFT, Key::Z), sc(CMD, Key::Y)] },
                Workspace,
                When(
                    |c| c.has_project && (c.can_redo || c.editing_text) && !c.gesture_active,
                    "reason-nothing-to-redo",
                ),
            ),
            Cut => m(
                "cmd-cut",
                None,
                const { &[sc(CMD, Key::X)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Copy => m(
                "cmd-copy",
                None,
                const { &[sc(CMD, Key::C)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Paste => m(
                "cmd-paste",
                None,
                const { &[sc(CMD, Key::V)] },
                Workspace,
                When(
                    |c| c.has_project && c.has_clipboard && !c.gesture_active,
                    "reason-clipboard-empty",
                ),
            ),
            Duplicate => m(
                "cmd-duplicate",
                None,
                const { &[sc(CMD, Key::D)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Delete => m(
                "cmd-delete",
                Some(icons::REMOVE),
                const { &[sc(NONE, Key::Delete), sc(NONE, Key::Backspace)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            SelectAll => m(
                "cmd-select-all",
                None,
                const { &[sc(CMD, Key::A)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),

            EditText => m(
                "cmd-edit-text",
                Some(icons::TEXT),
                const { &[sc(NONE, Key::Enter)] },
                Workspace,
                When(
                    |c| c.single_text && !c.gesture_active,
                    "reason-select-one-text",
                ),
            ),
            Group => m(
                "cmd-group",
                Some(icons::GROUP),
                const { &[sc(CMD, Key::G)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Ungroup => m(
                "cmd-ungroup",
                Some(icons::UNGROUP),
                const { &[sc(CMD_SHIFT, Key::G)] },
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_group,
                    "reason-select-group",
                ),
            ),
            ConvertToSymbol => m(
                "cmd-convert-to-symbol",
                Some(icons::SYMBOL),
                &[],
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            ImportFromLibrary => m(
                "cmd-import-from-library",
                None,
                &[],
                Workspace,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            EditSymbol => m(
                "cmd-edit-symbol",
                None,
                &[],
                Workspace,
                When(
                    |c| editable_selection(c) && c.single_instance,
                    "reason-no-instance",
                ),
            ),
            DetachInstance => m(
                "cmd-detach-instance",
                None,
                &[],
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_instance,
                    "reason-no-instance",
                ),
            ),
            FinishSymbol => m(
                "cmd-finish-symbol",
                None,
                &[],
                Workspace,
                When(
                    |c| c.editing_symbol && !c.gesture_active,
                    "reason-not-editing-symbol",
                ),
            ),
            ConvertToPath => m(
                "cmd-convert-to-path",
                Some(icons::PEN),
                &[],
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_convertible,
                    "reason-select-convertible",
                ),
            ),
            CreateOutlines => m(
                "cmd-create-outlines",
                Some(icons::TEXT),
                const { &[sc(CMD_SHIFT, Key::O)] },
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_text,
                    "reason-select-text",
                ),
            ),
            Combine(op) => m(
                crate::combine::combine_label(op),
                Some(match op {
                    BooleanOp::Unite => icons::UNITE,
                    BooleanOp::MinusFront => icons::MINUS_FRONT,
                    BooleanOp::Intersect => icons::INTERSECT,
                    BooleanOp::Exclude => icons::EXCLUDE,
                }),
                match op {
                    BooleanOp::Unite => const { &[sc(CMD_SHIFT, Key::U)] },
                    BooleanOp::MinusFront => const { &[sc(CMD_SHIFT, Key::Minus)] },
                    _ => &[],
                },
                Workspace,
                When(
                    |c| editable_selection(c) && !c.editing_text && c.combine_block.is_none(),
                    "reason-select-two-shapes",
                ),
            ),
            Align(edge) => m(
                crate::arrange::align_label(edge),
                Some(match edge {
                    Edge::Left => icons::OBJ_ALIGN_LEFT,
                    Edge::HCenter => icons::OBJ_ALIGN_HCENTER,
                    Edge::Right => icons::OBJ_ALIGN_RIGHT,
                    Edge::Top => icons::OBJ_ALIGN_TOP,
                    Edge::VCenter => icons::OBJ_ALIGN_VCENTER,
                    Edge::Bottom => icons::OBJ_ALIGN_BOTTOM,
                }),
                match edge {
                    Edge::Left => const { &[sc(ALT, Key::A)] },
                    Edge::HCenter => const { &[sc(ALT, Key::H)] },
                    Edge::Right => const { &[sc(ALT, Key::D)] },
                    Edge::Top => const { &[sc(ALT, Key::W)] },
                    Edge::VCenter => const { &[sc(ALT, Key::V)] },
                    Edge::Bottom => const { &[sc(ALT, Key::S)] },
                },
                Workspace,
                When(can_align, "reason-select-to-align"),
            ),
            Distribute(axis, mode) => m(
                crate::arrange::distribute_label(axis, mode),
                Some(match (axis, mode) {
                    (DistributeAxis::Horizontal, DistributeMode::Centers) => {
                        icons::DISTRIBUTE_H_CENTERS
                    }
                    (DistributeAxis::Vertical, DistributeMode::Centers) => {
                        icons::DISTRIBUTE_V_CENTERS
                    }
                    (DistributeAxis::Horizontal, DistributeMode::Spacing) => {
                        icons::DISTRIBUTE_H_SPACING
                    }
                    (DistributeAxis::Vertical, DistributeMode::Spacing) => {
                        icons::DISTRIBUTE_V_SPACING
                    }
                }),
                match (axis, mode) {
                    (DistributeAxis::Horizontal, DistributeMode::Spacing) => {
                        const { &[sc(ALT_SHIFT, Key::H)] }
                    }
                    (DistributeAxis::Vertical, DistributeMode::Spacing) => {
                        const { &[sc(ALT_SHIFT, Key::V)] }
                    }
                    _ => &[],
                },
                Workspace,
                When(can_distribute, "reason-select-three"),
            ),
            BringForward => m(
                "cmd-bring-forward",
                None,
                const { &[sc(CMD_ALT, Key::CloseBracket)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            SendBackward => m(
                "cmd-send-backward",
                None,
                const { &[sc(CMD_ALT, Key::OpenBracket)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Flip(axis) => m(
                crate::arrange::flip_label(axis),
                Some(match axis {
                    FlipAxis::Horizontal => icons::FLIP_HORIZONTAL,
                    FlipAxis::Vertical => icons::FLIP_VERTICAL,
                }),
                match axis {
                    FlipAxis::Horizontal => const { &[sc(SHIFT, Key::H)] },
                    FlipAxis::Vertical => const { &[sc(SHIFT, Key::V)] },
                },
                Workspace,
                When(
                    |c| editable_selection(c) && !c.editing_text,
                    NEEDS_SELECTION,
                ),
            ),

            NewLayer => m(
                "cmd-new-layer",
                Some(icons::NEW_LAYER),
                const { &[sc(CMD_SHIFT, Key::N)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),
            DuplicateLayer => m(
                "cmd-duplicate-layer",
                None,
                &[],
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            DeleteLayer => m(
                "cmd-delete-layer",
                None,
                &[],
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),

            // Global: it opens while a text field has the focus.
            CommandPalette => m(
                "cmd-command-palette",
                Some(icons::SEARCH),
                const { &[sc(CMD, Key::K)] },
                Global,
                When(|c| !c.gesture_active, ""),
            ),
            ZoomIn => m(
                "cmd-zoom-in",
                None,
                const { &[sc(CMD, Key::Equals), sc(CMD, Key::Plus)] },
                Workspace,
                NeedsProject,
            ),
            ZoomOut => m(
                "cmd-zoom-out",
                None,
                const { &[sc(CMD, Key::Minus)] },
                Workspace,
                NeedsProject,
            ),
            FitToScreen => m(
                "cmd-fit-to-screen",
                None,
                const { &[sc(CMD_SHIFT, Key::Num0)] },
                Workspace,
                NeedsProject,
            ),
            ActualSize => m(
                "cmd-actual-size-100pct",
                None,
                const { &[sc(CMD, Key::Num0)] },
                Workspace,
                NeedsProject,
            ),
            ShowSpace(space) => m(
                space.label(),
                None,
                match space {
                    Space::Project => const { &[sc(CMD, Key::Num1)] },
                    Space::Workshop => const { &[sc(CMD, Key::Num2)] },
                    Space::Brand => const { &[sc(CMD, Key::Num3)] },
                },
                Workspace,
                NeedsProject,
            ),
            ShowLeftTab(tab) => m(
                tab.label(),
                None,
                match tab {
                    LeftTab::Textures => const { &[sc(NONE, Key::Num1)] },
                    LeftTab::Layers => const { &[sc(NONE, Key::Num2)] },
                    LeftTab::Resources => const { &[sc(NONE, Key::Num3)] },
                },
                Workspace,
                NeedsProject,
            ),
            TogglePanels => m(
                "cmd-hide-panels",
                None,
                const { &[sc(NONE, Key::Tab)] },
                Workspace,
                When(
                    |c| c.space == Some(Space::Workshop),
                    "reason-not-in-workshop",
                ),
            ),
            ResetWorkspace => m(
                "cmd-reset-workspace",
                Some(icons::RESET),
                &[],
                Workspace,
                NeedsProject,
            ),
            ShowGrid => m(
                "cmd-show-grid",
                None,
                const { &[sc(CMD, Key::Quote)] },
                Workspace,
                NeedsProject,
            ),
            ShowGuides => m(
                "cmd-show-guides",
                None,
                const { &[sc(CMD, Key::Semicolon)] },
                Workspace,
                NeedsProject,
            ),
            ClearGuides => m(
                "cmd-clear-guides",
                None,
                &[],
                Workspace,
                When(
                    |c| c.has_project && c.has_guides && !c.gesture_active,
                    "reason-no-guides",
                ),
            ),
            Snapping => m(
                "cmd-snapping",
                None,
                const { &[sc(CMD_SHIFT, Key::Semicolon)] },
                Workspace,
                NeedsProject,
            ),
            DesignGallery => m("cmd-design-system-gallery", None, &[], App, Always),

            VehicleLibrary => m(
                "cmd-vehicle-library",
                Some(icons::VEHICLE),
                &[],
                App,
                Always,
            ),
            AddVehicle => m(
                "cmd-add-vehicle",
                Some(icons::ADD),
                &[],
                Workspace,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            NextTexture => m(
                "cmd-next-texture",
                None,
                const { &[sc(CMD, Key::CloseBracket), sc(CMD, Key::PageDown)] },
                Workspace,
                When(|c| c.texture_count > 1, "reason-one-texture"),
            ),
            PreviousTexture => m(
                "cmd-previous-texture",
                None,
                const { &[sc(CMD, Key::OpenBracket), sc(CMD, Key::PageUp)] },
                Workspace,
                When(|c| c.texture_count > 1, "reason-one-texture"),
            ),
            CopyFromCabin => m(
                "cmd-copy-from-cabin",
                None,
                &[],
                Workspace,
                When(
                    |c| c.cabin_sources > 0 && !c.gesture_active,
                    "reason-no-other-cabin",
                ),
            ),
            UpdateTemplate => m(
                "cmd-update-template",
                None,
                &[],
                Workspace,
                When(
                    |c| c.update_available && !c.gesture_active,
                    "reason-no-update",
                ),
            ),
            ShowTemplate => m(
                "cmd-show-template",
                None,
                const { &[sc(NONE, Key::G)] },
                Workspace,
                When(|c| c.has_template, "reason-no-template"),
            ),

            ExportTexture => m(
                "cmd-export-texture",
                Some(icons::EXPORT),
                const { &[sc(CMD_SHIFT, Key::E)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            // Global: it also commits a mod setting being typed in.
            ExportMod => m(
                "cmd-export-mod",
                None,
                const { &[sc(CMD, Key::E)] },
                Global,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),

            KeyboardShortcuts => m(
                "cmd-keyboard-shortcuts",
                None,
                const { &[sc(CMD, Key::Slash)] },
                App,
                Always,
            ),
            RunFirstLaunch => m("cmd-run-first-launch", None, &[], App, Always),
            About => m("cmd-about-truckpaint", None, &[], App, Always),

            SwapColorTarget => m(
                "cmd-switch-fill-stroke-target",
                None,
                const { &[sc(NONE, Key::X)] },
                Workspace,
                NeedsProject,
            ),
            SwapFillStroke => m(
                "cmd-swap-fill-and-stroke",
                None,
                const { &[sc(SHIFT, Key::X)] },
                Workspace,
                NeedsProject,
            ),
            DefaultColors => m(
                "cmd-default-colors",
                None,
                const { &[sc(NONE, Key::D)] },
                Workspace,
                NeedsProject,
            ),
            Deselect => m(
                "cmd-deselect",
                None,
                const { &[sc(NONE, Key::Escape)] },
                Workspace,
                // With nothing selected, Escape leaves the symbol being
                // edited.
                When(
                    |c| editable_selection(c) || (c.editing_symbol && !c.gesture_active),
                    NEEDS_SELECTION,
                ),
            ),
            Nudge(direction, big) => m(
                match (direction, big) {
                    (Direction::Left, false) => "cmd-nudge-left",
                    (Direction::Right, false) => "cmd-nudge-right",
                    (Direction::Up, false) => "cmd-nudge-up",
                    (Direction::Down, false) => "cmd-nudge-down",
                    (Direction::Left, true) => "cmd-nudge-left-10",
                    (Direction::Right, true) => "cmd-nudge-right-10",
                    (Direction::Up, true) => "cmd-nudge-up-10",
                    (Direction::Down, true) => "cmd-nudge-down-10",
                },
                None,
                nudge_shortcut(direction, big),
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            SelectTool(tool) => m(
                tool.name(),
                Some(tool.icon()),
                tool_shortcut(tool),
                Workspace,
                NeedsProject,
            ),
        }
    }

    /// Primary (displayed) shortcut.
    pub fn shortcut(self) -> Option<KeyboardShortcut> {
        self.meta().shortcuts.first().copied()
    }
}

fn nudge_shortcut(direction: Direction, big: bool) -> &'static [KeyboardShortcut] {
    match (direction, big) {
        (Direction::Left, false) => const { &[sc(NONE, Key::ArrowLeft)] },
        (Direction::Right, false) => const { &[sc(NONE, Key::ArrowRight)] },
        (Direction::Up, false) => const { &[sc(NONE, Key::ArrowUp)] },
        (Direction::Down, false) => const { &[sc(NONE, Key::ArrowDown)] },
        (Direction::Left, true) => const { &[sc(SHIFT, Key::ArrowLeft)] },
        (Direction::Right, true) => const { &[sc(SHIFT, Key::ArrowRight)] },
        (Direction::Up, true) => const { &[sc(SHIFT, Key::ArrowUp)] },
        (Direction::Down, true) => const { &[sc(SHIFT, Key::ArrowDown)] },
    }
}

fn tool_shortcut(tool: Tool) -> &'static [KeyboardShortcut] {
    match tool {
        Tool::Select => const { &[sc(NONE, Key::V)] },
        Tool::DirectSelect => const { &[sc(NONE, Key::A)] },
        Tool::Move => const { &[sc(NONE, Key::M)] },
        Tool::Rectangle => const { &[sc(NONE, Key::R)] },
        Tool::Ellipse => const { &[sc(NONE, Key::E)] },
        Tool::Polygon => const { &[sc(NONE, Key::Y)] },
        Tool::Pen => const { &[sc(NONE, Key::P)] },
        Tool::Line => const { &[sc(NONE, Key::Backslash)] },
        Tool::Text => const { &[sc(NONE, Key::T)] },
        Tool::Image => const { &[sc(SHIFT, Key::I)] },
        Tool::Eyedropper => const { &[sc(NONE, Key::I)] },
        Tool::Gradient => const { &[sc(SHIFT, Key::G)] },
        Tool::Zoom => const { &[sc(NONE, Key::Z)] },
        Tool::Hand => const { &[sc(NONE, Key::H)] },
    }
}

/// Formats a shortcut in the platform's notation.
///
/// macOS uses Apple's symbol order `⌃⌥⇧⌘` followed by the key (e.g. `⇧⌘Z`)
/// when `symbols` is true; other platforms use `Ctrl+Alt+Shift+Z`.
pub fn format_shortcut(shortcut: &KeyboardShortcut, is_mac: bool, symbols: bool) -> String {
    let m = shortcut.modifiers;
    let key = key_label(shortcut.logical_key);
    if is_mac {
        let parts = [
            (m.ctrl, "⌃", "Ctrl"),
            (m.alt, "⌥", "Option"),
            (m.shift, "⇧", "Shift"),
            (m.command || m.mac_cmd, "⌘", "Cmd"),
        ];
        if symbols {
            let mut s: String = parts.iter().filter(|p| p.0).map(|p| p.1).collect();
            s.push_str(&key);
            s
        } else {
            let mut names: Vec<&str> = parts.iter().filter(|p| p.0).map(|p| p.2).collect();
            names.push(&key);
            names.join("+")
        }
    } else {
        // Windows and Linux: modifier names of the current language.
        let mut names = Vec::new();
        if m.ctrl || m.command {
            names.push(tr("key-ctrl"));
        }
        if m.alt {
            names.push(tr("key-alt"));
        }
        if m.shift {
            names.push(tr("key-shift"));
        }
        names.push(key.into_owned());
        names.join("+")
    }
}

fn key_label(key: Key) -> std::borrow::Cow<'static, str> {
    let named = match key {
        Key::Delete => "key-delete",
        Key::Escape => "key-escape",
        Key::Enter => "key-enter",
        Key::Backspace => "key-backspace",
        Key::Space => "key-space",
        Key::Tab => "key-tab",
        _ => "",
    };
    if !named.is_empty() {
        return tr(named).into();
    }
    std::borrow::Cow::Borrowed(match key {
        Key::Comma => ",",
        Key::Slash => "/",
        Key::Backslash => "\\",
        Key::Quote => "'",
        Key::Semicolon => ";",
        Key::OpenBracket => "[",
        Key::CloseBracket => "]",
        Key::Equals => "=",
        Key::Minus => "-",
        Key::Plus => "+",
        other => other.symbol_or_name(),
    })
}

/// Glyphs needed to display macOS modifier symbols.
pub const MAC_SYMBOL_GLYPHS: &str = "⌃⌥⇧⌘";

/// Formats shortcuts for the current OS. Build it once per frame: it checks
/// whether the fonts can display the macOS modifier symbols, and falls back
/// to names otherwise.
#[derive(Clone, Copy, Debug)]
pub struct ShortcutFormatter {
    is_mac: bool,
    symbols: bool,
}

impl ShortcutFormatter {
    pub fn new(ctx: &egui::Context) -> Self {
        let is_mac = ctx.os().is_mac();
        let symbols = is_mac
            && ctx.fonts_mut(|f| {
                let font = egui::FontId::proportional(12.0);
                MAC_SYMBOL_GLYPHS.chars().all(|c| f.has_glyph(&font, c))
            });
        Self { is_mac, symbols }
    }

    pub fn format(&self, shortcut: &KeyboardShortcut) -> String {
        format_shortcut(shortcut, self.is_mac, self.symbols)
    }

    /// The command's displayed shortcut, if any.
    pub fn command(&self, id: CommandId) -> Option<String> {
        id.shortcut().map(|s| self.format(&s))
    }
}

/// Exact modifier match: `COMMAND` stands for Cmd on macOS and Ctrl elsewhere,
/// and extra modifiers are not tolerated (so `R` does not fire on `Cmd+R`).
fn modifiers_match(pressed: Modifiers, pattern: Modifiers) -> bool {
    pressed.command == pattern.command
        && pressed.shift == pattern.shift
        && pressed.alt == pattern.alt
        && (pattern.command || !(pressed.ctrl || pressed.mac_cmd))
}

/// Consumes key presses matching enabled commands' shortcuts and returns the
/// triggered commands. Shortcuts with more modifiers are matched first.
pub fn take_triggered(
    input: &mut egui::InputState,
    typing: bool,
    has_project: bool,
    is_enabled: impl Fn(CommandId) -> bool,
) -> Vec<CommandId> {
    let mut candidates: Vec<(CommandId, KeyboardShortcut)> = CommandId::all()
        .into_iter()
        .filter(|id| {
            let meta = id.meta();
            match meta.scope {
                Scope::Global => true,
                Scope::App => !typing,
                Scope::Workspace => !typing && has_project,
            }
        })
        .flat_map(|id| id.meta().shortcuts.iter().map(move |s| (id, *s)))
        .collect();
    let modifier_count = |m: Modifiers| u8::from(m.command) + u8::from(m.shift) + u8::from(m.alt);
    candidates.sort_by_key(|(_, s)| std::cmp::Reverse(modifier_count(s.modifiers)));

    let mut triggered = Vec::new();
    input.events.retain(|event| {
        let hit = match event {
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => candidates
                .iter()
                .find(|(_, s)| s.logical_key == *key && modifiers_match(*modifiers, s.modifiers))
                .map(|(id, _)| *id),
            // The native backend turns Cmd/Ctrl+C, X and V into clipboard
            // events instead of key presses.
            egui::Event::Copy => clipboard_command(&candidates, CommandId::Copy),
            egui::Event::Cut => clipboard_command(&candidates, CommandId::Cut),
            egui::Event::Paste(_) => clipboard_command(&candidates, CommandId::Paste),
            _ => None,
        };
        match hit {
            Some(id) if is_enabled(id) => {
                triggered.push(id);
                false
            }
            _ => true,
        }
    });
    triggered
}

/// `id` when its shortcut is active (in scope).
fn clipboard_command(
    candidates: &[(CommandId, KeyboardShortcut)],
    id: CommandId,
) -> Option<CommandId> {
    candidates.iter().any(|(c, _)| *c == id).then_some(id)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn every_command_has_a_label() {
        for id in CommandId::all() {
            assert!(!id.meta().label.is_empty(), "{id:?}");
        }
    }

    #[test]
    fn no_duplicate_default_shortcuts() {
        let mut seen: HashMap<(bool, bool, bool, Key), CommandId> = HashMap::new();
        for id in CommandId::all() {
            for s in id.meta().shortcuts {
                let m = s.modifiers;
                let key = (m.command, m.shift, m.alt, s.logical_key);
                if let Some(other) = seen.insert(key, id) {
                    panic!("{id:?} and {other:?} share shortcut {s:?}");
                }
            }
        }
    }

    /// The shortcut table of the workspace (command-system, Default
    /// workspace shortcuts), each key against its command.
    #[test]
    fn default_workspace_shortcuts() {
        use CommandId::*;
        let table: [(CommandId, &[KeyboardShortcut]); 20] = [
            (ShowSpace(Space::Project), &[sc(CMD, Key::Num1)]),
            (ShowSpace(Space::Workshop), &[sc(CMD, Key::Num2)]),
            (ShowSpace(Space::Brand), &[sc(CMD, Key::Num3)]),
            (ShowLeftTab(LeftTab::Textures), &[sc(NONE, Key::Num1)]),
            (ShowLeftTab(LeftTab::Layers), &[sc(NONE, Key::Num2)]),
            (ShowLeftTab(LeftTab::Resources), &[sc(NONE, Key::Num3)]),
            (TogglePanels, &[sc(NONE, Key::Tab)]),
            (ActualSize, &[sc(CMD, Key::Num0)]),
            (FitToScreen, &[sc(CMD_SHIFT, Key::Num0)]),
            (BringForward, &[sc(CMD_ALT, Key::CloseBracket)]),
            (SendBackward, &[sc(CMD_ALT, Key::OpenBracket)]),
            (
                NextTexture,
                &[sc(CMD, Key::CloseBracket), sc(CMD, Key::PageDown)],
            ),
            (
                PreviousTexture,
                &[sc(CMD, Key::OpenBracket), sc(CMD, Key::PageUp)],
            ),
            (ShowTemplate, &[sc(NONE, Key::G)]),
            (SelectTool(Tool::Gradient), &[sc(SHIFT, Key::G)]),
            (ExportMod, &[sc(CMD, Key::E)]),
            (ExportTexture, &[sc(CMD_SHIFT, Key::E)]),
            (KeyboardShortcuts, &[sc(CMD, Key::Slash)]),
            (DefaultColors, &[sc(NONE, Key::D)]),
            (CommandPalette, &[sc(CMD, Key::K)]),
        ];
        for (id, shortcuts) in table {
            assert_eq!(id.meta().shortcuts, shortcuts, "{id:?}");
        }
    }

    #[test]
    fn former_panel_keys_are_unbound() {
        for id in CommandId::all() {
            for s in id.meta().shortcuts {
                assert!(
                    !matches!(s.logical_key, Key::F5 | Key::F6 | Key::F7 | Key::F8),
                    "{id:?} uses {s:?}"
                );
            }
        }
    }

    #[test]
    fn hide_panels_only_in_the_workshop() {
        use crate::state::is_enabled;
        let workshop = EditContext {
            has_project: true,
            space: Some(Space::Workshop),
            ..EditContext::default()
        };
        assert!(is_enabled(CommandId::TogglePanels, &workshop));
        for space in [Space::Project, Space::Brand] {
            let other = EditContext {
                space: Some(space),
                ..workshop
            };
            assert!(!is_enabled(CommandId::TogglePanels, &other), "{space:?}");
            assert!(is_enabled(CommandId::ShowLeftTab(LeftTab::Layers), &other));
            assert!(is_enabled(CommandId::ShowSpace(Space::Workshop), &other));
        }
    }

    #[test]
    fn mac_redo_uses_symbols_in_apple_order() {
        let redo = CommandId::Redo.shortcut().unwrap();
        assert_eq!(format_shortcut(&redo, true, true), "⇧⌘Z");
        assert_eq!(format_shortcut(&redo, true, false), "Shift+Cmd+Z");
    }

    #[test]
    fn other_platforms_use_ctrl_names() {
        let undo = CommandId::Undo.shortcut().unwrap();
        let redo = CommandId::Redo.shortcut().unwrap();
        assert_eq!(format_shortcut(&undo, false, false), "Ctrl+Z");
        assert_eq!(format_shortcut(&redo, false, false), "Ctrl+Shift+Z");
        assert_eq!(format_shortcut(&undo, true, true), "⌘Z");
    }

    #[test]
    fn editing_commands_follow_the_edit_context() {
        use crate::state::is_enabled;
        let idle = EditContext {
            has_project: true,
            ..EditContext::default()
        };
        assert!(!is_enabled(CommandId::Delete, &idle));
        assert!(!is_enabled(CommandId::Paste, &idle));
        assert!(!is_enabled(CommandId::Undo, &idle));
        assert!(is_enabled(CommandId::SelectAll, &idle));
        assert!(is_enabled(CommandId::FitToScreen, &idle));

        let selected = EditContext {
            has_selection: true,
            has_clipboard: true,
            can_undo: true,
            ..idle
        };
        for id in [
            CommandId::Delete,
            CommandId::Duplicate,
            CommandId::Copy,
            CommandId::Paste,
            CommandId::Undo,
        ] {
            assert!(is_enabled(id, &selected), "{id:?}");
        }
        let dragging = EditContext {
            gesture_active: true,
            ..selected
        };
        assert!(!is_enabled(CommandId::Delete, &dragging));
        assert!(!is_enabled(CommandId::Undo, &dragging));
        assert!(!is_enabled(CommandId::Deselect, &dragging));
    }

    #[test]
    fn grouping_commands_follow_the_selection() {
        use crate::state::is_enabled;
        let one = EditContext {
            has_project: true,
            has_selection: true,
            ..EditContext::default()
        };
        assert!(is_enabled(CommandId::Group, &one));
        assert!(!is_enabled(CommandId::Ungroup, &one));
        let group = EditContext {
            selection_has_group: true,
            ..one
        };
        assert!(is_enabled(CommandId::Ungroup, &group));
        assert!(is_enabled(
            CommandId::NewLayer,
            &EditContext {
                has_project: true,
                ..EditContext::default()
            }
        ));
    }

    #[test]
    fn plain_key_does_not_match_with_command() {
        assert!(!modifiers_match(Modifiers::COMMAND, Modifiers::NONE));
        assert!(modifiers_match(Modifiers::NONE, Modifiers::NONE));
        assert!(!modifiers_match(Modifiers::SHIFT, Modifiers::NONE));
    }
}
