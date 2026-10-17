//! What is left to do on a texture: its state (Empty, Modified, To check)
//! and its off-palette colors. Both are derived from the project, never
//! stored.

use std::sync::Arc;

use crate::document::{Object, ObjectId, Paint, Rgba, ShapeKind};
use crate::project::{Surface, TemplateStatus};

/// The state of a texture, from its objects and its template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureState {
    /// Nothing is drawn: it is exported transparent.
    Empty,
    /// At least one object is drawn.
    Modified,
    /// An update flagged it; wins over the other two.
    ToCheck(CheckReason),
}

/// Why an update flagged a texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CheckReason {
    LayoutChanged,
    NotInVersion,
}

impl TextureState {
    pub fn is_to_check(self) -> bool {
        matches!(self, Self::ToCheck(_))
    }
}

/// Whether an object is drawn: visible and, for a group or an instance, at
/// least one of its children drawn. Opacity and locking don't matter.
pub fn is_drawn(o: &Object) -> bool {
    o.visible && (!o.kind.has_content() || o.children.iter().any(|c| is_drawn(c)))
}

/// The objects a template update asks to reposition on `surface`: every
/// object, hidden ones too, inside groups but not the groups themselves; a
/// symbol instance is one object. In layer order, bottom first.
pub fn objects_on(surface: &Surface) -> Vec<ObjectId> {
    fn walk(objects: &[Arc<Object>], out: &mut Vec<ObjectId>) {
        for o in objects {
            match o.kind {
                ShapeKind::Group => walk(&o.children, out),
                _ => out.push(o.id),
            }
        }
    }
    let mut out = Vec::new();
    walk(&surface.objects, &mut out);
    out
}

impl Surface {
    /// The texture's state (see [`TextureState`]).
    pub fn state(&self) -> TextureState {
        match self.template.as_ref().map(|t| t.status) {
            Some(TemplateStatus::LayoutChanged) => {
                TextureState::ToCheck(CheckReason::LayoutChanged)
            }
            Some(TemplateStatus::Removed) => TextureState::ToCheck(CheckReason::NotInVersion),
            _ if self.objects.iter().any(|o| is_drawn(o)) => TextureState::Modified,
            _ => TextureState::Empty,
        }
    }
}

/// Which textures a list shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureFilter {
    #[default]
    All,
    /// Empty or To check.
    ToDo,
    ToCheck,
}

impl TextureFilter {
    pub fn matches(self, state: TextureState) -> bool {
        match self {
            Self::All => true,
            Self::ToDo => state != TextureState::Modified,
            Self::ToCheck => state.is_to_check(),
        }
    }
}

/// The colors of a list of objects that aren't linked to a swatch, and the
/// objects holding them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OffPalette {
    /// Distinct colors, in first-seen order.
    pub colors: Vec<Rgba>,
    /// The innermost objects holding at least one, in tree order.
    pub objects: Vec<ObjectId>,
}

/// The off-palette colors of `objects`: unlinked solid fills and strokes
/// and unlinked gradient stops of the drawn shapes and texts, inside
/// groups but not inside instances (their colors belong to the symbol).
/// Fully transparent colors are left out.
pub fn off_palette(objects: &[Arc<Object>]) -> OffPalette {
    let mut out = OffPalette::default();
    walk(objects, &mut out);
    out
}

fn walk(objects: &[Arc<Object>], out: &mut OffPalette) {
    for o in objects.iter().filter(|o| is_drawn(o)) {
        match o.kind {
            ShapeKind::Group => walk(&o.children, out),
            ShapeKind::Instance { .. } | ShapeKind::Image { .. } => {}
            _ => {
                let mut found = false;
                let fill = (&o.fill, o.fill_swatch.is_some());
                let stroke = o.stroke.as_ref().map(|s| (&s.paint, s.swatch.is_some()));
                for (paint, linked) in std::iter::once(fill).chain(stroke) {
                    found |= add_paint(paint, linked, &mut out.colors);
                }
                if found {
                    out.objects.push(o.id);
                }
            }
        }
    }
}

/// Adds the unlinked colors of `paint`; whether it held any.
fn add_paint(paint: &Paint, linked: bool, colors: &mut Vec<Rgba>) -> bool {
    let mut found = false;
    let mut add = |c: Rgba| {
        if c.a > 0 {
            found = true;
            if !colors.contains(&c) {
                colors.push(c);
            }
        }
    };
    match paint {
        Paint::Solid(c) if !linked => add(*c),
        Paint::Solid(_) => {}
        Paint::Gradient(g) => g
            .stops()
            .iter()
            .filter(|s| s.swatch.is_none())
            .for_each(|s| add(s.color)),
    }
    found
}

#[cfg(test)]
mod tests;
