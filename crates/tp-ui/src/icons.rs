//! Icon glyphs used by the application, named by meaning rather than shape so
//! a glyph can be swapped without touching call sites.

use egui::{FontFamily, FontId, RichText};
use egui_phosphor::regular as ph;

/// Font family containing only the icon font (plus emoji fallbacks).
///
/// Icons must be drawn with this family: the UI typeface also defines glyphs
/// in the Private Use Area and would otherwise shadow the icon codepoints.
pub const FAMILY: &str = "icons";

pub fn family() -> FontFamily {
    FontFamily::Name(FAMILY.into())
}

/// Icon font at the given size.
pub fn font(size: f32) -> FontId {
    FontId::new(size, family())
}

/// Rich text for a single icon glyph.
pub fn rich(icon: &str) -> RichText {
    RichText::new(icon).family(family())
}

pub const SELECT: &str = ph::NAVIGATION_ARROW;
pub const DIRECT_SELECT: &str = ph::CURSOR;
pub const MOVE: &str = ph::ARROWS_OUT_CARDINAL;
pub const RECTANGLE: &str = ph::SQUARE;
pub const ELLIPSE: &str = ph::CIRCLE;
pub const POLYGON: &str = ph::POLYGON;
pub const PEN: &str = ph::PEN_NIB;
pub const LINE: &str = ph::LINE_SEGMENT;
pub const TEXT: &str = ph::TEXT_T;
pub const IMAGE: &str = ph::IMAGE;
pub const EYEDROPPER: &str = ph::EYEDROPPER;
pub const GRADIENT_TOOL: &str = ph::GRADIENT;
pub const ZOOM: &str = ph::MAGNIFYING_GLASS;
pub const HAND: &str = ph::HAND;

pub const NEW_PROJECT: &str = ph::PLUS;
pub const OPEN_PROJECT: &str = ph::FOLDER_OPEN;
pub const SAVE: &str = ph::FLOPPY_DISK;
pub const EXPORT: &str = ph::EXPORT;
pub const RECENT: &str = ph::CLOCK_COUNTER_CLOCKWISE;
pub const FILE: &str = ph::FILE;
pub const FILE_MISSING: &str = ph::FILE_DASHED;
pub const CLOSE: &str = ph::X;
pub const SETTINGS: &str = ph::GEAR;
pub const RESET: &str = ph::ARROW_COUNTER_CLOCKWISE;
pub const ROTATE: &str = ph::ARROW_CLOCKWISE;
pub const REMOVE: &str = ph::TRASH;
pub const MORE: &str = ph::DOTS_THREE;
pub const COPY: &str = ph::COPY;

pub const EXPANDED: &str = ph::CARET_DOWN;
pub const COLLAPSED: &str = ph::CARET_RIGHT;
/// A newer version is available.
pub const UPDATE: &str = ph::ARROW_CIRCLE_UP;

pub const SAVED: &str = ph::CHECK_CIRCLE;
pub const CHECK: &str = ph::CHECK;
pub const UNSAVED: &str = ph::CIRCLE_DASHED;
pub const WARNING: &str = ph::WARNING;
// Boolean operations.
pub const UNITE: &str = ph::UNITE_SQUARE;
pub const MINUS_FRONT: &str = ph::SUBTRACT_SQUARE;
pub const INTERSECT: &str = ph::INTERSECT_SQUARE;
pub const EXCLUDE: &str = ph::EXCLUDE_SQUARE;
// Object alignment and distribution.
pub const OBJ_ALIGN_LEFT: &str = ph::ALIGN_LEFT;
pub const OBJ_ALIGN_HCENTER: &str = ph::ALIGN_CENTER_HORIZONTAL;
pub const OBJ_ALIGN_RIGHT: &str = ph::ALIGN_RIGHT;
pub const OBJ_ALIGN_TOP: &str = ph::ALIGN_TOP;
pub const OBJ_ALIGN_VCENTER: &str = ph::ALIGN_CENTER_VERTICAL;
pub const OBJ_ALIGN_BOTTOM: &str = ph::ALIGN_BOTTOM;
pub const FLIP_HORIZONTAL: &str = ph::FLIP_HORIZONTAL;
pub const FLIP_VERTICAL: &str = ph::FLIP_VERTICAL;
pub const DISTRIBUTE_H_CENTERS: &str = ph::ARROWS_OUT_LINE_HORIZONTAL;
pub const DISTRIBUTE_V_CENTERS: &str = ph::ARROWS_OUT_LINE_VERTICAL;
pub const DISTRIBUTE_H_SPACING: &str = ph::COLUMNS;
pub const DISTRIBUTE_V_SPACING: &str = ph::ROWS;
pub const ALIGN_LEFT: &str = ph::TEXT_ALIGN_LEFT;
pub const ALIGN_CENTER: &str = ph::TEXT_ALIGN_CENTER;
pub const ALIGN_RIGHT: &str = ph::TEXT_ALIGN_RIGHT;
pub const ITALIC: &str = ph::TEXT_ITALIC;
pub const SEARCH: &str = ph::MAGNIFYING_GLASS;
pub const RENAME: &str = ph::PENCIL_SIMPLE;

pub const LAYERS: &str = ph::STACK;
pub const STYLES: &str = ph::SWATCHES;
pub const STROKE_CENTER: &str = ph::SQUARE_HALF;
pub const STROKE_INSIDE: &str = ph::CORNERS_IN;
pub const STROKE_OUTSIDE: &str = ph::CORNERS_OUT;
pub const CAP_BUTT: &str = ph::LINE_VERTICAL;
pub const CAP_ROUND: &str = ph::CIRCLE;
pub const CAP_SQUARE: &str = ph::SQUARE;
pub const JOIN_MITER: &str = ph::CARET_UP;
pub const JOIN_ROUND: &str = ph::CIRCLE_HALF_TILT;
pub const JOIN_BEVEL: &str = ph::POLYGON;
pub const PAINT_SOLID: &str = ph::SQUARE;
pub const PAINT_LINEAR: &str = ph::GRADIENT;
pub const PAINT_RADIAL: &str = ph::RADIO_BUTTON;
pub const REVERSE: &str = ph::ARROWS_LEFT_RIGHT;
pub const ASSETS: &str = ph::IMAGES;
pub const VEHICLE: &str = ph::TRUCK;
pub const GROUP: &str = ph::FOLDER_SIMPLE;
pub const SYMBOL: &str = ph::SHAPES;
pub const UNGROUP: &str = ph::FOLDER_SIMPLE_DASHED;
pub const NEW_LAYER: &str = ph::STACK_PLUS;
pub const VISIBLE: &str = ph::EYE;
pub const HIDDEN: &str = ph::EYE_SLASH;
pub const LOCKED: &str = ph::LOCK_SIMPLE;
pub const UNLOCKED: &str = ph::LOCK_SIMPLE_OPEN;
pub const LINKED: &str = ph::LINK_SIMPLE;
pub const UNLINKED: &str = ph::LINK_BREAK;
pub const ADD: &str = ph::PLUS;
/// The local MCP server: listening, or a client connected.
pub const MCP: &str = ph::PLUGS;
pub const MCP_CONNECTED: &str = ph::PLUGS_CONNECTED;
/// A status bar toggle that is on / off (the knob's side tells them apart).
pub const TOGGLE_ON: &str = ph::TOGGLE_RIGHT;
pub const TOGGLE_OFF: &str = ph::TOGGLE_LEFT;
/// The Marketplace: a package to download, the catalog unreachable, a notice.
pub const DOWNLOAD: &str = ph::DOWNLOAD_SIMPLE;
pub const OFFLINE: &str = ph::CLOUD_SLASH;
pub const INFO: &str = ph::INFO;
