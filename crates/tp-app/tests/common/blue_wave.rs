//! The example livery of the documentation: Blue Wave Biarritz, a fictional
//! carrier, painted on the sample truck and the sample trailer.
//!
//! The ideas are in `tests/fixtures/blue-wave/brief.md`; the placement is
//! [`LAYOUT`], a table of elements per labeled zone of each template, in
//! fractions of the zone. The documentation media (`tests/doc_media.rs`) and
//! the headless test (`tests/blue_wave.rs`) both build the livery here.

use std::sync::Arc;

use tp_app::workspace::Workspace;
use tp_core::AssetKind;
use tp_core::document::tree;
use tp_core::document::{
    AssetId, CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind, StyleId, SwatchId, SymbolId,
    TextBlock,
};
use tp_core::kurbo::{Affine, Point, Rect, Size};

const MARK: &[u8] = include_bytes!("../fixtures/blue-wave/mark.svg");
const WAVES: &[u8] = include_bytes!("../fixtures/blue-wave/waves.svg");
const COAST: &[u8] = include_bytes!("../fixtures/blue-wave/coast.svg");

pub const TAGLINE: &str = "L'excellence du transport sur la Côte Basque";
const FAMILY: &str = "Montserrat";

/// The three colors of the palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    LightBlue,
    Blue,
}

impl Color {
    const ALL: [Color; 3] = [Color::White, Color::LightBlue, Color::Blue];

    pub fn rgba(self) -> Rgba {
        match self {
            Color::White => Rgba::rgb(0xFF, 0xFF, 0xFF),
            Color::LightBlue => Rgba::rgb(0x3E, 0x9E, 0xDB),
            Color::Blue => Rgba::rgb(0x03, 0x31, 0x63),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Color::White => "White",
            Color::LightBlue => "Light blue",
            Color::Blue => "Blue",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// What a row of the layout paints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Element {
    /// The whole box in a palette color.
    Fill(Color),
    /// A thin rounded line.
    Rule(Color),
    /// An instance of the logo symbol, fitted in the box.
    Logo,
    /// The words "Blue Wave", fitted in the box.
    Wordmark,
    /// The tagline, fitted in the box, with the "Tagline" text style.
    Tagline,
    /// The wave bands, stretched over the box.
    Waves { mirrored: bool },
    /// The lighthouse and the rocks, fitted in the box, standing on its
    /// bottom edge.
    Coast { mirrored: bool },
}

/// One element in one zone of one texture.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    /// The package texture id (`standard`, `body_13_6`, `base`).
    pub texture: &'static str,
    /// The label of the zone in the template.
    pub zone: &'static str,
    /// Which of the zones with that label, in template order.
    pub nth: usize,
    pub element: Element,
    /// `x y w h`: a box in fractions of the zone, from its top left corner.
    pub at: [f64; 4],
}

const fn row(
    texture: &'static str,
    zone: &'static str,
    nth: usize,
    element: Element,
    at: [f64; 4],
) -> Row {
    Row {
        texture,
        zone,
        nth,
        element,
        at,
    }
}

use Color::{Blue, LightBlue, White};
use Element::{Coast, Fill, Logo, Rule, Tagline, Waves, Wordmark};

const ALL: [f64; 4] = [0.0, 0.0, 1.0, 1.0];

/// The livery. A zone that is not listed stays unpainted.
pub const LAYOUT: &[Row] = &[
    // Truck, Standard cab: the doors carry the small logo, the front the
    // name above the windshield, the rear wall the centered logo.
    row("standard", "DOOR", 0, Fill(White), ALL),
    row("standard", "DOOR", 0, Logo, [0.10, 0.64, 0.80, 0.22]),
    row(
        "standard",
        "DOOR",
        0,
        Waves { mirrored: false },
        [0.0, 0.88, 1.0, 0.12],
    ),
    row("standard", "DOOR", 1, Fill(White), ALL),
    row("standard", "DOOR", 1, Logo, [0.10, 0.64, 0.80, 0.22]),
    row(
        "standard",
        "DOOR",
        1,
        Waves { mirrored: true },
        [0.0, 0.88, 1.0, 0.12],
    ),
    row("standard", "FRONT", 0, Fill(White), ALL),
    row("standard", "FRONT", 0, Wordmark, [0.20, 0.005, 0.60, 0.04]),
    row(
        "standard",
        "FRONT",
        0,
        Waves { mirrored: false },
        [0.0, 0.78, 1.0, 0.22],
    ),
    row("standard", "ROOF", 0, Fill(White), ALL),
    row("standard", "REAR WALL", 0, Fill(White), ALL),
    row("standard", "REAR WALL", 0, Logo, [0.25, 0.22, 0.50, 0.30]),
    row("standard", "BUMPER", 0, Fill(Blue), ALL),
    row(
        "standard",
        "BUMPER",
        0,
        Rule(LightBlue),
        [0.04, 0.78, 0.92, 0.06],
    ),
    // Trailer, Curtain body 13.6 m: the logo, the tagline and the waves
    // rising to the rear, where the coast stands. The second side is
    // mirrored so that the coast is at the rear on both.
    row("body_13_6", "CURTAIN", 0, Fill(White), ALL),
    row(
        "body_13_6",
        "CURTAIN",
        0,
        Waves { mirrored: false },
        [0.0, 0.55, 1.0, 0.45],
    ),
    row(
        "body_13_6",
        "CURTAIN",
        0,
        Coast { mirrored: false },
        [0.84, 0.08, 0.15, 0.66],
    ),
    row("body_13_6", "CURTAIN", 0, Logo, [0.20, 0.06, 0.46, 0.46]),
    row("body_13_6", "CURTAIN", 0, Tagline, [0.20, 0.56, 0.46, 0.10]),
    row(
        "body_13_6",
        "CURTAIN",
        0,
        Rule(LightBlue),
        [0.20, 0.675, 0.46, 0.015],
    ),
    row("body_13_6", "CURTAIN", 1, Fill(White), ALL),
    row(
        "body_13_6",
        "CURTAIN",
        1,
        Waves { mirrored: true },
        [0.0, 0.55, 1.0, 0.45],
    ),
    row(
        "body_13_6",
        "CURTAIN",
        1,
        Coast { mirrored: true },
        [0.01, 0.08, 0.15, 0.66],
    ),
    row("body_13_6", "CURTAIN", 1, Logo, [0.34, 0.06, 0.46, 0.46]),
    row("body_13_6", "CURTAIN", 1, Tagline, [0.34, 0.56, 0.46, 0.10]),
    row(
        "body_13_6",
        "CURTAIN",
        1,
        Rule(LightBlue),
        [0.34, 0.675, 0.46, 0.015],
    ),
    row("body_13_6", "REAR DOORS", 0, Fill(White), ALL),
    row("body_13_6", "REAR DOORS", 0, Logo, [0.12, 0.25, 0.76, 0.40]),
    row(
        "body_13_6",
        "REAR DOORS",
        0,
        Waves { mirrored: false },
        [0.0, 0.86, 1.0, 0.14],
    ),
    row("body_13_6", "FRONT BULKHEAD", 0, Fill(White), ALL),
    row(
        "body_13_6",
        "FRONT BULKHEAD",
        0,
        Rule(LightBlue),
        [0.0, 0.88, 1.0, 0.04],
    ),
    row("body_13_6", "ROOF", 0, Fill(White), ALL),
    // Truck, Side skirts: Blue with a Light blue rule.
    row("side_skirts", "SIDE SKIRT", 0, Fill(Blue), ALL),
    row(
        "side_skirts",
        "SIDE SKIRT",
        0,
        Rule(LightBlue),
        [0.04, 0.45, 0.92, 0.10],
    ),
    row("side_skirts", "SIDE SKIRT", 1, Fill(Blue), ALL),
    row(
        "side_skirts",
        "SIDE SKIRT",
        1,
        Rule(LightBlue),
        [0.04, 0.45, 0.92, 0.10],
    ),
    // Trailer, Base: the chassis frames in Blue.
    row("base", "CHASSIS FRAME", 0, Fill(Blue), ALL),
    row("base", "CHASSIS FRAME", 1, Fill(Blue), ALL),
    row("base", "REAR BUMPER", 0, Fill(White), ALL),
];

/// The textures the livery needs, per vehicle: the truck with its default
/// textures and its high roof cabin, the trailer with its main texture and the 13.6 m body.
pub fn fleet_textures() -> [(&'static [u8], Vec<String>); 2] {
    let truck = tp_vehicles::Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
    // Both cabins, as the documentation's truck projects have them.
    let mut cabins = tp_app::vehicle_project::default_textures(&truck.manifest);
    cabins.insert(1, "high_roof".to_owned());
    [
        (tp_app::vehicles::SAMPLES[0].bytes, cabins),
        (
            tp_app::vehicles::SAMPLES[1].bytes,
            vec!["base".to_owned(), "body_13_6".to_owned()],
        ),
    ]
}

/// What the livery made, for the tests and the scenarios.
#[derive(Clone, Debug)]
pub struct Livery {
    /// White, Light blue and Blue.
    pub swatches: [SwatchId; 3],
    pub logo: SymbolId,
    pub white_base: StyleId,
    pub blue_panel: StyleId,
    /// None on a project without the trailer.
    pub tagline: Option<StyleId>,
}

/// A zone of a template: its label and rectangle in texture pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct Zone {
    pub label: String,
    pub rect: Rect,
}

/// The labeled zones of a template SVG, in order: every `<rect>` or `<polygon>` (its bounding box)
/// followed by a `<text>` label with no other text in between. The border drawn around the whole
/// texture is not a zone.
pub fn zones(svg: &str, size: Size) -> Vec<Zone> {
    fn attr(tag: &str, name: &str) -> f64 {
        let key = format!(" {name}=\"");
        let at = tag.find(&key).expect("attribute") + key.len();
        let end = tag[at..].find('"').expect("closing quote");
        tag[at..at + end].parse().expect("number")
    }
    let mut out = Vec::new();
    let mut last: Option<Rect> = None;
    let mut rest = svg;
    while let Some(start) = rest.find('<') {
        rest = &rest[start..];
        let end = rest.find('>').expect("closed tag") + 1;
        let tag = &rest[..end];
        if tag.starts_with("<rect ") {
            let (x, y) = (attr(tag, "x"), attr(tag, "y"));
            let (w, h) = (attr(tag, "width"), attr(tag, "height"));
            last = (w < size.width - 100.0).then(|| Rect::new(x, y, x + w, y + h));
        } else if tag.starts_with("<polygon ") {
            let points = tag
                .split("points=\"")
                .nth(1)
                .and_then(|t| t.split('"').next())
                .expect("points");
            let numbers: Vec<f64> = points
                .split([' ', ','])
                .filter_map(|n| n.parse().ok())
                .collect();
            let (xs, ys): (Vec<f64>, Vec<f64>) = numbers.chunks(2).map(|p| (p[0], p[1])).unzip();
            let fold = |v: &[f64], f: fn(f64, f64) -> f64, init| v.iter().copied().fold(init, f);
            last = Some(Rect::new(
                fold(&xs, f64::min, f64::MAX),
                fold(&ys, f64::min, f64::MAX),
                fold(&xs, f64::max, f64::MIN),
                fold(&ys, f64::max, f64::MIN),
            ));
        } else if tag.starts_with("<text ") {
            let close = rest.find("</text>").expect("closed text");
            if let Some(rect) = last.take() {
                out.push(Zone {
                    label: rest[end..close].to_owned(),
                    rect,
                });
            }
        }
        rest = &rest[end..];
    }
    out
}

/// The zone `nth` labeled `label` of the template of surface `surface`.
pub fn zone(ws: &Workspace, surface: usize, label: &str, nth: usize) -> Option<Rect> {
    let s = &ws.project.surfaces[surface];
    let asset = ws.project.assets.get(&s.template.as_ref()?.asset)?;
    let svg = std::str::from_utf8(&asset.bytes).ok()?;
    zones(svg, s.size)
        .into_iter()
        .filter(|z| z.label == label)
        .nth(nth)
        .map(|z| z.rect)
}

/// The index of the surface of the texture `texture`.
pub fn surface_of(ws: &Workspace, texture: &str) -> Option<usize> {
    ws.project
        .surfaces
        .iter()
        .position(|s| s.template.as_ref().is_some_and(|t| t.texture_id == texture))
}

/// The box of a row in texture pixels, inside `zone`.
pub fn boxed(zone: Rect, at: [f64; 4]) -> Rect {
    let [x, y, w, h] = at;
    let (zw, zh) = (zone.width(), zone.height());
    Rect::new(
        zone.x0 + x * zw,
        zone.y0 + y * zh,
        zone.x0 + (x + w) * zw,
        zone.y0 + (y + h) * zh,
    )
}

fn style(weight: u16, italic: bool, size: f64) -> CharStyle {
    CharStyle {
        family: FAMILY.to_owned(),
        weight,
        italic,
        size,
        ..CharStyle::default()
    }
}

/// A text object laid out and centered on `center`.
fn text(
    ws: &mut Workspace,
    content: &str,
    style: CharStyle,
    color: Color,
    swatches: &[SwatchId; 3],
    center: Point,
) -> Object {
    let mut o = Object::text(ObjectId(0), TextBlock::new(content, style), center);
    o.fill = color.rgba().into();
    o.fill_swatch = Some(swatches[color.index()]);
    ws.text.place_at(&mut o, center);
    o.frame.center = center;
    o
}

fn measure(ws: &mut Workspace, content: &str, style: &CharStyle) -> Size {
    ws.text.layout(content, style).size
}

/// `base` with the size that makes `content` fit `target` (width and height).
fn fitted(ws: &mut Workspace, content: &str, mut base: CharStyle, target: Size) -> CharStyle {
    let m = measure(ws, content, &base);
    // A whole size, which the Brand space shows as such.
    base.size = (base.size * (target.width / m.width).min(target.height / m.height)).floor();
    base
}

fn fill_shape(
    kind: ShapeKind,
    name: &str,
    rect: Rect,
    color: Color,
    swatches: &[SwatchId; 3],
) -> Object {
    let mut o = Object::new(ObjectId(0), kind, Frame::from_rect(rect));
    o.name = name.to_owned();
    o.fill = color.rgba().into();
    o.fill_swatch = Some(swatches[color.index()]);
    o
}

/// "Blue" (Blue) and "Wave" (Light blue) side by side, `width` wide in
/// total at most and `height` high at most, centered on `center`.
fn wordmark(
    ws: &mut Workspace,
    swatches: &[SwatchId; 3],
    center: Point,
    width: f64,
    height: f64,
) -> Vec<Object> {
    const GAP: f64 = 0.28;
    let base = style(800, false, 100.0);
    let blue = measure(ws, "Blue", &base);
    let wave = measure(ws, "Wave", &base);
    let total = blue.width + wave.width + GAP * base.size;
    let k = (width / total).min(height / blue.height.max(wave.height));
    let style = style(800, false, base.size * k);
    let (b, w) = (blue.width * k, wave.width * k);
    let left = center.x - (b + w + GAP * style.size) / 2.0;
    let mut out = Vec::new();
    for (word, color, x, span) in [
        ("Blue", Color::Blue, left, b),
        ("Wave", Color::LightBlue, left + b + GAP * style.size, w),
    ] {
        let mut o = text(
            ws,
            word,
            style.clone(),
            color,
            swatches,
            Point::new(x + span / 2.0, center.y),
        );
        o.name = word.to_owned();
        out.push(o);
    }
    out
}

/// Builds the logo (the mark, the name, "BIARRITZ" between two rules) at
/// 100% on the active texture and turns it into the symbol "Logo". Returns
/// the symbol and the size of its content.
fn make_logo(ws: &mut Workspace, swatches: &[SwatchId; 3], mark: AssetId) -> (SymbolId, Size) {
    let origin = Point::new(1000.0, 1000.0);
    let (mark_w, mark_h) = (600.0, 450.0);
    let name_w = 1300.0;
    let name_h = 230.0;
    let mut ids = Vec::new();
    let mut add = |ws: &mut Workspace, o: Object| ids.push(ws.project.add(o));
    // The mark.
    let mut m = Object::new(
        ObjectId(0),
        ShapeKind::Image { asset: mark },
        Frame::new(
            Point::new(origin.x + mark_w / 2.0, origin.y + mark_h / 2.0),
            Size::new(mark_w, mark_h),
            0.0,
        ),
    );
    m.name = "Mark".to_owned();
    add(ws, m);
    // The name, on the right of the mark.
    let name_left = origin.x + mark_w + 50.0;
    let name_center = Point::new(name_left + name_w / 2.0, origin.y + 150.0);
    for o in wordmark(ws, swatches, name_center, name_w, name_h) {
        add(ws, o);
    }
    // BIARRITZ between two rules, under the name.
    let town_y = origin.y + 360.0;
    let town_style = CharStyle {
        letter_spacing: 450.0,
        ..style(600, false, 78.0)
    };
    let town = measure(ws, "BIARRITZ", &town_style);
    let town_x = name_left + name_w / 2.0;
    let o = text(
        ws,
        "BIARRITZ",
        town_style,
        Color::Blue,
        swatches,
        Point::new(town_x, town_y),
    );
    add(ws, o);
    let rule = (name_w - town.width) / 2.0 - 40.0;
    for left in [name_left, name_left + name_w - rule] {
        let rect = Rect::new(left, town_y - 3.0, left + rule, town_y + 3.0);
        let mut o = fill_shape(ShapeKind::rectangle(), "Rule", rect, Color::Blue, swatches);
        o.name = "Rule".to_owned();
        add(ws, o);
    }
    let size = ids
        .iter()
        .filter_map(|id| tp_core::document::tree::get(&ws.project.surface().objects, *id))
        .map(|o| o.bounding_box())
        .reduce(|a, b| a.union(b))
        .expect("logo content")
        .size();
    let (symbol, instance) = ws
        .project
        .convert_to_symbol(&ids, "Logo")
        .expect("logo symbol");
    ws.project.rename_symbol(symbol, "Logo");
    // Only the symbol stays: the instances come from the layout.
    ws.project.surface_mut().remove(&[instance]);
    (symbol, size)
}

/// Paints the livery on the open project, a fleet of the sample truck and
/// the sample trailer (see [`fleet_textures`]). The first texture is left
/// active.
pub fn paint(ws: &mut Workspace) -> Livery {
    let mut swatches = [SwatchId(0); 3];
    for color in Color::ALL {
        let (id, _) = ws.project.add_swatch(color.rgba(), "Color");
        ws.project.rename_swatch(id, color.name());
        swatches[color.index()] = id;
    }
    let mut asset = |name: &str, bytes: &[u8], size: Size| {
        ws.project
            .add_asset(name, AssetKind::Svg, Arc::from(bytes), size)
            .0
    };
    let mark = asset("mark", MARK, Size::new(400.0, 300.0));
    let waves = asset("waves", WAVES, Size::new(2000.0, 600.0));
    let coast = asset("coast", COAST, Size::new(360.0, 500.0));

    ws.project.active_surface = 0;
    let (logo, logo_size) = make_logo(ws, &swatches, mark);
    let half = ws
        .project
        .symbol(logo)
        .expect("logo")
        .surface
        .size
        .to_vec2()
        / 2.0;

    // The ids of the base fills, to follow a graphic style per color.
    let mut fills: Vec<(usize, Color, ObjectId)> = Vec::new();
    let mut taglines: Vec<(usize, ObjectId)> = Vec::new();
    // What each zone received, to group it under a layer.
    let mut layers: Vec<(ZoneKey, Vec<ObjectId>)> = Vec::new();
    for r in LAYOUT {
        // A project of one vehicle has only its textures.
        let Some(surface) = surface_of(ws, r.texture) else {
            continue;
        };
        let zone = zone(ws, surface, r.zone, r.nth).expect("a zone of the template");
        let area = boxed(zone, r.at);
        ws.project.active_surface = surface;
        let before = ws.project.surface().objects.len();
        let name = r.zone.to_lowercase();
        match r.element {
            Element::Fill(color) => {
                let o = fill_shape(
                    ShapeKind::rectangle(),
                    &format!("{name} {}", color.name().to_lowercase()),
                    area,
                    color,
                    &swatches,
                );
                fills.push((surface, color, ws.project.add(o)));
            }
            Element::Rule(color) => {
                let radius = area.height() / 2.0;
                let o = fill_shape(
                    ShapeKind::Rectangle {
                        corner_radius: radius,
                    },
                    "Rule",
                    area,
                    color,
                    &swatches,
                );
                ws.project.add(o);
            }
            Element::Logo => {
                let k = (area.width() / logo_size.width).min(area.height() / logo_size.height);
                let placement = Affine::translate(area.center().to_vec2())
                    * Affine::scale(k)
                    * Affine::translate(-half);
                let o = ws
                    .project
                    .new_instance(logo, placement)
                    .expect("a logo instance");
                ws.project.add(o);
            }
            Element::Wordmark => {
                for o in wordmark(ws, &swatches, area.center(), area.width(), area.height()) {
                    ws.project.add(o);
                }
            }
            Element::Tagline => {
                let base = style(500, true, 100.0);
                let s = fitted(ws, TAGLINE, base, area.size());
                let mut o = text(ws, TAGLINE, s, Color::Blue, &swatches, area.center());
                o.name = "Tagline".to_owned();
                taglines.push((surface, ws.project.add(o)));
            }
            Element::Waves { mirrored } => {
                let mut o = image(waves, area, mirrored);
                o.name = "Waves".to_owned();
                ws.project.add(o);
            }
            Element::Coast { mirrored } => {
                // Fitted, standing on the bottom edge of the box.
                let k = (area.width() / 360.0).min(area.height() / 500.0);
                let (w, h) = (360.0 * k, 500.0 * k);
                let x = if mirrored {
                    area.x0 + w / 2.0
                } else {
                    area.x1 - w / 2.0
                };
                let rect =
                    Rect::from_center_size(Point::new(x, area.y1 - h / 2.0), Size::new(w, h));
                let mut o = image(coast, rect, mirrored);
                o.name = "Coast".to_owned();
                ws.project.add(o);
            }
        }
        let added: Vec<ObjectId> = ws.project.surface().objects[before..]
            .iter()
            .map(|o| o.id)
            .collect();
        let key = (surface, r.zone, r.nth);
        match layers.iter_mut().find(|l| l.0 == key) {
            Some(layer) => layer.1.extend(added),
            None => layers.push((key, added)),
        }
    }

    // The shared styles: one per kind of base, and the tagline.
    let mut white_base = None;
    let mut blue_panel = None;
    for color in [Color::White, Color::Blue] {
        let ids: Vec<_> = fills.iter().filter(|f| f.1 == color).collect();
        let (surface, _, first) = *ids[0];
        ws.project.active_surface = surface;
        let (slot, name) = match color {
            Color::White => (&mut white_base, "White base"),
            _ => (&mut blue_panel, "Blue panel"),
        };
        let id = ws
            .project
            .new_graphic_style(first, "Style")
            .expect("a style");
        ws.project.rename_style(id, name);
        *slot = Some(id);
        for (s, _, o) in ids {
            ws.project.active_surface = *s;
            ws.project.apply_graphic_style(id, &[*o]);
        }
    }
    // The tagline is on the trailer only.
    let tagline = taglines.first().map(|&(first_surface, first)| {
        ws.project.active_surface = first_surface;
        let id = ws.project.new_text_style(first, "Style").expect("a style");
        ws.project.rename_style(id, "Tagline");
        for (s, o) in &taglines {
            ws.project.active_surface = *s;
            ws.project.apply_text_style(id, &[*o]);
        }
        id
    });

    // One layer per zone, named after it.
    for ((surface, label, nth), ids) in layers {
        ws.project.active_surface = surface;
        let Some(group) = ws.project.group(&ids) else {
            continue;
        };
        let mut layer =
            (**tree::get(&ws.project.surface().objects, group).expect("the layer")).clone();
        layer.name = layer_name(label, nth);
        ws.project.surface_mut().replace(&[layer]);
    }

    ws.project.refresh_instances();
    ws.relayout_all_texts();
    ws.set_active_surface(0);
    ws.selection.clear();
    Livery {
        swatches,
        logo,
        white_base: white_base.expect("white base style"),
        blue_panel: blue_panel.expect("blue panel style"),
        tagline,
    }
}

/// A zone of a texture: its surface, label and place among the labels.
type ZoneKey<'a> = (usize, &'a str, usize);

/// The name of the layer holding a zone's artwork.
fn layer_name(label: &str, nth: usize) -> String {
    let side = ["left", "right"][nth.min(1)];
    match label {
        "DOOR" => format!("Door ({side})"),
        "CURTAIN" => format!("Side ({side})"),
        "CHASSIS FRAME" => format!("Chassis frame ({side})"),
        other => {
            let mut name = other.to_lowercase();
            name[..1].make_ascii_uppercase();
            name
        }
    }
}

/// An image of `asset` over `rect`.
fn image(asset: AssetId, rect: Rect, mirrored: bool) -> Object {
    let mut o = Object::new(
        ObjectId(0),
        ShapeKind::Image { asset },
        Frame::from_rect(rect),
    );
    o.mirrored = mirrored;
    o
}

/// The open project's fleet with the livery: the sample truck and trailer,
/// painted, the Workshop shown. The textures' templates are hidden, as in
/// `common::create_project`.
pub fn blue_wave_fleet(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) -> Livery {
    let packages: Vec<_> = fleet_textures()
        .into_iter()
        .map(|(bytes, textures)| (tp_vehicles::Package::read(bytes).unwrap(), textures))
        .collect();
    let fleet: Vec<_> = packages.iter().map(|(p, t)| (p, t.as_slice())).collect();
    let mut project = tp_app::vehicle_project::fleet_project("Blue Wave Biarritz", &fleet).unwrap();
    for surface in &mut project.surfaces {
        if let Some(t) = surface.template.as_mut() {
            t.visible = false;
        }
    }
    super::open_project(h, project);
    h.run();
    let livery = paint(h.state_mut().workspace_mut().expect("project open"));
    super::scenes::wait_for_images(h);
    super::settle_renders(h);
    livery
}

/// Surface `surface` of the open project rendered at `side` pixels wide,
/// without the GPU.
pub fn render(ws: &Workspace, surface: usize, side: u32) -> image::RgbaImage {
    let options = tp_render::RenderOptions {
        width: side,
        height: side,
        background: None,
    };
    let mut fonts = tp_text::FontLibrary::bundled();
    let pixmap = tp_render::render(&ws.project, surface, options, &mut fonts, &mut |_, _| true)
        .expect("not cancelled");
    tp_render::to_rgba(&pixmap)
}
