use kurbo::{Affine, Point, Size};

use super::*;
use crate::document::{
    AssetId, CharStyle, ColorStop, Frame, Gradient, GradientKind, StrokeStyle, SwatchId, SymbolId,
    TextBlock,
};
use crate::project::{SurfaceTemplate, TexturePart};

const RED: Rgba = Rgba::rgb(0xC2, 0x3B, 0x2A);
const BLACK: Rgba = Rgba::rgb(0x15, 0x15, 0x15);
const WHITE: Rgba = Rgba::rgb(255, 255, 255);

fn rect(id: u64, fill: Rgba) -> Object {
    let mut o = Object::new(
        ObjectId(id),
        ShapeKind::rectangle(),
        Frame::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0), 0.0),
    );
    o.fill = Paint::Solid(fill);
    o
}

fn group(id: u64, kind: ShapeKind, children: Vec<Object>) -> Object {
    let mut o = rect(id, BLACK);
    o.kind = kind;
    o.children = children.into_iter().map(Arc::new).collect();
    o
}

fn instance(id: u64, children: Vec<Object>) -> Object {
    group(
        id,
        ShapeKind::Instance {
            symbol: SymbolId(1),
            placement: Affine::IDENTITY,
        },
        children,
    )
}

fn hidden(mut o: Object) -> Object {
    o.visible = false;
    o
}

fn surface(objects: Vec<Object>) -> Surface {
    let mut s = Surface::new("Chassis", Size::new(1024.0, 1024.0));
    s.objects = objects.into_iter().map(Arc::new).collect();
    s.template = Some(SurfaceTemplate {
        package_id: "truck".to_owned(),
        texture_id: "chassis".to_owned(),
        part: TexturePart::Accessory,
        asset: AssetId(1),
        layout_version: 1,
        game_ids: Vec::new(),
        also_paints: Vec::new(),
        main_index: None,
        opacity: SurfaceTemplate::DEFAULT_OPACITY,
        visible: true,
        status: TemplateStatus::Current,
        flagged: Default::default(),
    });
    s
}

fn flagged(objects: Vec<Object>, status: TemplateStatus) -> Surface {
    let mut s = surface(objects);
    s.template.as_mut().unwrap().status = status;
    s
}

fn colors(objects: Vec<Object>) -> OffPalette {
    off_palette(&surface(objects).objects)
}

// Texture states

#[test]
fn one_rectangle_is_modified() {
    assert_eq!(surface(vec![rect(1, RED)]).state(), TextureState::Modified);
}

#[test]
fn every_object_hidden_is_empty() {
    let s = surface(vec![hidden(rect(1, RED)), hidden(rect(2, RED))]);
    assert_eq!(s.state(), TextureState::Empty);
}

#[test]
fn group_with_hidden_children_is_empty() {
    let g = group(
        1,
        ShapeKind::Group,
        vec![hidden(rect(2, RED)), hidden(rect(3, RED))],
    );
    let s = surface(vec![g, hidden(rect(4, RED))]);
    assert_eq!(s.state(), TextureState::Empty);
    let i = instance(5, vec![hidden(rect(6, RED))]);
    assert_eq!(surface(vec![i]).state(), TextureState::Empty);
}

#[test]
fn group_with_a_drawn_child_is_modified() {
    let g = group(
        1,
        ShapeKind::Group,
        vec![hidden(rect(2, RED)), rect(3, RED)],
    );
    assert_eq!(surface(vec![g]).state(), TextureState::Modified);
}

#[test]
fn flags_win_over_objects() {
    let s = flagged(vec![rect(1, RED)], TemplateStatus::LayoutChanged);
    assert_eq!(s.state(), TextureState::ToCheck(CheckReason::LayoutChanged));
    let s = flagged(vec![], TemplateStatus::Removed);
    assert_eq!(s.state(), TextureState::ToCheck(CheckReason::NotInVersion));
}

#[test]
fn opacity_and_locking_keep_it_modified() {
    let mut o = rect(1, RED);
    o.opacity = 0.0;
    o.locked = true;
    assert_eq!(surface(vec![o]).state(), TextureState::Modified);
    let mut s = surface(vec![rect(2, RED)]);
    s.template.as_mut().unwrap().visible = false;
    assert_eq!(s.state(), TextureState::Modified);
}

// Filters

#[test]
fn filters_match_states() {
    let check = TextureState::ToCheck(CheckReason::LayoutChanged);
    let all = [TextureState::Empty, TextureState::Modified, check];
    let matching = |f: TextureFilter| all.iter().map(|s| f.matches(*s)).collect::<Vec<_>>();
    assert_eq!(matching(TextureFilter::All), [true, true, true]);
    assert_eq!(matching(TextureFilter::ToDo), [true, false, true]);
    assert_eq!(matching(TextureFilter::ToCheck), [false, false, true]);
    assert!(TextureFilter::ToDo.matches(TextureState::ToCheck(CheckReason::NotInVersion)));
}

// Off-palette colors

#[test]
fn linked_colors_dont_count() {
    let mut r = rect(1, RED);
    r.fill_swatch = Some(SwatchId(1));
    r.stroke = Some(StrokeStyle {
        paint: Paint::Solid(Rgba::rgb(0, 0, 0)),
        ..StrokeStyle::default()
    });
    let mut t = Object::text(
        ObjectId(2),
        TextBlock::new("ACE", CharStyle::default()),
        Point::new(10.0, 10.0),
    );
    t.fill = Paint::Solid(WHITE);
    let off = colors(vec![r, t]);
    assert_eq!(off.colors, [Rgba::rgb(0, 0, 0), WHITE]);
    assert_eq!(off.objects, [ObjectId(1), ObjectId(2)]);
}

#[test]
fn same_value_not_linked_counts() {
    let mut e = rect(1, RED);
    e.kind = ShapeKind::Ellipse;
    assert_eq!(colors(vec![e]).colors, [RED]);
}

#[test]
fn unlinked_gradient_stops_count() {
    let mut first = ColorStop::new(0.0, RED);
    first.swatch = Some(SwatchId(1));
    let mut r = rect(1, RED);
    r.fill = Paint::Gradient(Gradient::new(
        GradientKind::Linear,
        &[first, ColorStop::new(1.0, BLACK)],
    ));
    // A gradient fill never has a solid fill link.
    r.fill_swatch = None;
    let off = colors(vec![r]);
    assert_eq!(off.colors, [BLACK]);
    assert_eq!(off.objects, [ObjectId(1)]);
}

#[test]
fn colors_are_distinct() {
    let off = colors(vec![rect(1, BLACK), rect(2, BLACK), rect(3, BLACK)]);
    assert_eq!(off.colors, [BLACK]);
    assert_eq!(off.objects, [ObjectId(1), ObjectId(2), ObjectId(3)]);
}

#[test]
fn instances_are_left_out() {
    let i = instance(1, vec![rect(2, RED), rect(3, BLACK)]);
    assert_eq!(colors(vec![i]), OffPalette::default());
}

#[test]
fn hidden_objects_are_left_out() {
    let g = group(1, ShapeKind::Group, vec![hidden(rect(2, RED))]);
    assert_eq!(
        colors(vec![hidden(rect(3, BLACK)), g]),
        OffPalette::default()
    );
}

#[test]
fn groups_are_walked_innermost_objects_listed() {
    let g = group(1, ShapeKind::Group, vec![rect(2, RED), rect(3, BLACK)]);
    let off = colors(vec![g, rect(4, RED)]);
    assert_eq!(off.colors, [RED, BLACK]);
    assert_eq!(off.objects, [ObjectId(2), ObjectId(3), ObjectId(4)]);
}

#[test]
fn stroke_only_and_transparent_colors() {
    let mut r = rect(1, Rgba::with_alpha(0, 0, 0, 0));
    r.stroke = Some(StrokeStyle {
        paint: Paint::Solid(RED),
        ..StrokeStyle::default()
    });
    let mut linked = rect(2, Rgba::with_alpha(0, 0, 0, 0));
    linked.stroke = Some(StrokeStyle {
        paint: Paint::Solid(BLACK),
        swatch: Some(SwatchId(2)),
        ..StrokeStyle::default()
    });
    let off = colors(vec![r, linked]);
    assert_eq!(off.colors, [RED]);
    assert_eq!(off.objects, [ObjectId(1)]);
}

#[test]
fn images_have_no_colors() {
    let mut i = rect(1, RED);
    i.kind = ShapeKind::Image { asset: AssetId(3) };
    assert_eq!(colors(vec![i]), OffPalette::default());
}

#[test]
fn objects_on_counts_leaves_hidden_ones_and_instances_once() {
    let s = surface(vec![
        rect(1, BLACK),
        hidden(rect(2, BLACK)),
        group(
            3,
            ShapeKind::Group,
            vec![
                rect(4, BLACK),
                group(5, ShapeKind::Group, vec![rect(6, BLACK)]),
            ],
        ),
        instance(7, vec![rect(8, BLACK), rect(9, BLACK)]),
        group(10, ShapeKind::Group, vec![]),
    ]);
    assert_eq!(
        objects_on(&s),
        [1, 2, 4, 6, 7].map(ObjectId),
        "groups are left out, an instance is one object"
    );
    assert!(objects_on(&surface(vec![])).is_empty());
}
