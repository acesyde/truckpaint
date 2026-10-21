//! Scenes and waits shared by the screenshot and documentation media
//! targets (`tests/screenshots.rs`, `tests/doc_media.rs`).

use egui_kittest::kittest::Queryable;

/// Adds a varied scene to the open project and returns the ids.
pub fn demo_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> Vec<tp_core::document::ObjectId> {
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle};
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().expect("project open");
    let mut ids = Vec::new();
    let mut add = |kind,
                   center: (f64, f64),
                   size: (f64, f64),
                   rot,
                   fill: Rgba,
                   stroke: Option<StrokeStyle>,
                   opacity| {
        let mut o = Object::new(
            ObjectId(0),
            kind,
            Frame::new(
                Point::new(center.0, center.1),
                Size::new(size.0, size.1),
                rot,
            ),
        );
        o.fill = fill.into();
        o.stroke = stroke;
        o.opacity = opacity;
        ids.push(ws.project.add(o));
    };
    add(
        ShapeKind::rectangle(),
        (2048.0, 900.0),
        (3800.0, 900.0),
        0.0,
        Rgba::rgb(0x7A, 0x1F, 0x2B),
        None,
        1.0,
    );
    add(
        ShapeKind::Rectangle {
            corner_radius: 120.0,
        },
        (1300.0, 2300.0),
        (1600.0, 900.0),
        -12.0,
        Rgba::rgb(0xF2, 0xF2, 0xF2),
        Some(StrokeStyle {
            paint: Rgba::rgb(0x20, 0x20, 0x20).into(),
            width: 24.0,
            ..Default::default()
        }),
        1.0,
    );
    add(
        ShapeKind::Ellipse,
        (2900.0, 2600.0),
        (1400.0, 1400.0),
        0.0,
        Rgba::rgb(0xF0, 0xB4, 0x4C),
        None,
        0.7,
    );
    add(
        ShapeKind::rectangle(),
        (2900.0, 3500.0),
        (1800.0, 160.0),
        25.0,
        Rgba::rgb(0x30, 0x30, 0x36),
        None,
        1.0,
    );
    ids
}

/// Fills the Brand space from the demo scene `ids`: five swatches, two
/// graphic styles (the selection follows the second), a text style, a
/// symbol and an image.
pub fn demo_brand(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
    ids: &[tp_core::document::ObjectId],
) {
    use tp_core::document::{CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind, TextBlock};
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().expect("project open");
    let p = &mut ws.project;
    for (name, c) in [
        ("Burgundy", Rgba::rgb(0x7A, 0x1F, 0x2B)),
        ("Cream", Rgba::rgb(0xEF, 0xE6, 0xCF)),
        ("Forest", Rgba::rgb(0x1F, 0x4D, 0x3A)),
        ("Gold", Rgba::rgb(0xB8, 0x91, 0x3E)),
        ("Ink", Rgba::rgb(0x15, 0x15, 0x15)),
    ] {
        let (id, _) = p.add_swatch(c, "Color");
        p.rename_swatch(id, name);
    }
    for (object, name) in [(ids[0], "Burgundy band"), (ids[1], "Plate")] {
        let style = p.new_graphic_style(object, "Style").unwrap();
        p.rename_style(style, name);
    }
    let mut lettering = Object::new(
        ObjectId(0),
        ShapeKind::Text,
        Frame::new(Point::new(2048.0, 3000.0), Size::new(1200.0, 200.0), 0.0),
    );
    lettering.text = Some(TextBlock::new("ACE", CharStyle::default()));
    let lettering = p.add(lettering);
    let style = p.new_text_style(lettering, "Text style").unwrap();
    p.rename_style(style, "Lettering");
    let (logo, _) = p.convert_to_symbol(&[ids[3]], "Symbol").unwrap();
    p.rename_symbol(logo, "Logo");
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" rx="4" fill="#1E8C3A"/></svg>"##;
    p.add_asset(
        "badge",
        tp_core::AssetKind::Svg,
        std::sync::Arc::from(&svg[..]),
        Size::new(40.0, 20.0),
    );
    ws.relayout_all_texts();
    ws.selection = vec![ids[1]];
}

/// A text object laid out with the workspace's text engine.
pub fn add_text(
    ws: &mut tp_app::workspace::Workspace,
    content: &str,
    style: tp_core::document::CharStyle,
    anchor: (f64, f64),
    fill: tp_core::document::Rgba,
    stroke: Option<tp_core::document::StrokeStyle>,
) -> tp_core::document::ObjectId {
    use tp_core::document::{Object, ObjectId, TextBlock};
    let anchor = tp_core::kurbo::Point::new(anchor.0, anchor.1);
    let mut o = Object::text(ObjectId(0), TextBlock::new(content, style), anchor);
    o.fill = fill.into();
    o.stroke = stroke;
    ws.text.place_at(&mut o, anchor);
    ws.project.add(o)
}

/// Steps until the texture thumbnails of the open project are rendered.
pub fn wait_for_thumbnails(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) {
    for _ in 0..200 {
        let state = h.state();
        let rendering = state.workspace().is_some_and(|ws| {
            ws.thumbnails.is_rendering()
                || ws.mod_previews.is_rendering()
                || ws.side_view.is_rendering()
        }) || state.recent_thumbnails.is_reading()
            || state.package_previews.is_reading();
        if !rendering {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.step();
    }
    h.run();
}

/// Steps until SVG renders from the background thread are uploaded.
pub fn wait_for_images(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) {
    for _ in 0..200 {
        h.step();
        if !h.state().workspace().unwrap().images.is_rendering() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    h.step();
}

/// Opens the menu titled with the text of `key`.
pub fn open_menu(h: &mut egui_kittest::Harness<'static, tp_app::AppState>, key: &str) {
    h.get_by_role_and_label(egui::accesskit::Role::Button, &tp_i18n::tr(key))
        .click();
    h.run();
}

/// Steps until the open Export Mod dialog has settled in place (egui sizes
/// and centers it over a few frames).
pub fn settle_dialog(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) {
    for _ in 0..8 {
        h.step();
    }
}

/// A livery with lettering, a PNG logo and an SVG badge.
pub fn lettering_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> Vec<tp_core::document::ObjectId> {
    use tp_core::document::TextAlign;
    use tp_core::document::{CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle};
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().expect("project open");
    let mut base = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(2048.0, 1700.0), Size::new(4096.0, 1800.0), 0.0),
    );
    base.fill = Rgba::rgb(0x7A, 0x1F, 0x2B).into();
    ws.project.add(base);
    let white = Rgba::rgb(255, 255, 255);
    let black = Rgba::rgb(0x15, 0x15, 0x18);
    let title = add_text(
        ws,
        "ACE LOGISTICS",
        CharStyle {
            family: "Barlow Condensed".into(),
            weight: 700,
            italic: true,
            size: 420.0,
            ..CharStyle::default()
        },
        (300.0, 1500.0),
        white,
        Some(StrokeStyle {
            paint: black.into(),
            width: 14.0,
            ..Default::default()
        }),
    );
    let phone = add_text(
        ws,
        "+33 1 23 45 67 89",
        CharStyle {
            family: "Oswald".into(),
            weight: 500,
            size: 160.0,
            letter_spacing: 80.0,
            ..CharStyle::default()
        },
        (320.0, 1850.0),
        Rgba::rgb(0xF0, 0xB4, 0x4C),
        None,
    );
    let slogan = add_text(
        ws,
        "EUROPE\nWIDE",
        CharStyle {
            family: "Bebas Neue".into(),
            weight: 400,
            size: 300.0,
            align: TextAlign::Center,
            line_height: 90.0,
            ..CharStyle::default()
        },
        (3300.0, 3000.0),
        Rgba::rgb(0x30, 0x30, 0x36),
        None,
    );
    let rotated = add_text(
        ws,
        "Montserrat",
        CharStyle {
            family: "Montserrat".into(),
            weight: 800,
            size: 220.0,
            ..CharStyle::default()
        },
        (500.0, 3300.0),
        Rgba::rgb(0x1F, 0x6F, 0xFF),
        None,
    );
    {
        let mut o = (**ws.project.surface().get(rotated).unwrap()).clone();
        o.frame.rotation_deg = -15.0;
        o.frame.size.width *= 1.4;
        o.sync_text_scale();
        ws.project.surface_mut().replace(&[o]);
    }
    // A PNG logo (radial gradient) and an SVG badge.
    let png = {
        let img = image::RgbaImage::from_fn(512, 512, |x, y| {
            let (dx, dy) = (x as f32 - 256.0, y as f32 - 256.0);
            let d = (dx * dx + dy * dy).sqrt() / 256.0;
            if d > 1.0 {
                image::Rgba([0, 0, 0, 0])
            } else {
                image::Rgba([20, (60.0 + 150.0 * d) as u8, 220, 255])
            }
        });
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    };
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="300" viewBox="0 0 600 300">
  <rect x="10" y="10" width="580" height="280" rx="60" fill="#F0B44C" stroke="#151518" stroke-width="16"/>
  <path d="M80 220 L180 80 L280 220 Z" fill="#7A1F2B"/>
  <circle cx="420" cy="150" r="90" fill="none" stroke="#151518" stroke-width="24"/>
</svg>"##;
    let files = vec![
        tp_app::import::read_bytes("logo.png", png),
        tp_app::import::read_bytes("badge.svg", svg.to_vec()),
    ];
    ws.place_files(files, Some(Point::new(3000.0, 1300.0)), 0.0);
    let images = ws.selection.clone();
    {
        let mut badge = ws.selected_objects()[1].clone();
        badge.frame.center = Point::new(1900.0, 3000.0);
        badge.frame.size = Size::new(1200.0, 600.0);
        badge.frame.rotation_deg = 8.0;
        ws.project.surface_mut().replace(&[badge]);
    }
    ws.selection.clear();
    let mut ids = vec![title, phone, slogan, rotated];
    ids.extend(images);
    ids
}

/// A gradient livery: a linear cab fade, a radial glow behind gradient
/// lettering with an Outside stroke, and a gradient dashed line. Returns the
/// lettering and the line.
pub fn gradient_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> (tp_core::document::ObjectId, tp_core::document::ObjectId) {
    use tp_core::document::{
        Cap, CharStyle, ColorStop, Dash, Frame, Gradient, GradientKind, Join, LineStyle, Node,
        Object, ObjectId, Paint, PathData, Rgba, ShapeKind, StrokeAlign, StrokeStyle, Subpath,
    };
    use tp_core::kurbo::Point;
    let ws = h.state_mut().workspace_mut().unwrap();
    let stops = |a: Rgba, b: Rgba| [ColorStop::new(0.0, a), ColorStop::new(1.0, b)];
    // Cab side fading from burgundy to near black, left to right.
    let mut base = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(2048.0, 2048.0), (3600.0, 2400.0).into(), 0.0),
    );
    base.name = "Fade".into();
    base.fill = Paint::Gradient(Gradient::new(
        GradientKind::Linear,
        &stops(Rgba::rgb(0x9B, 0x22, 0x35), Rgba::rgb(0x1A, 0x08, 0x0C)),
    ));
    ws.project.add(base);
    // A gold glow behind the lettering.
    let mut glow = Object::new(
        ObjectId(0),
        ShapeKind::Ellipse,
        Frame::new(Point::new(2048.0, 1500.0), (2600.0, 1200.0).into(), 0.0),
    );
    glow.name = "Glow".into();
    glow.fill = Paint::Gradient(Gradient::new(
        GradientKind::Radial,
        &stops(
            Rgba::with_alpha(0xF0, 0xB4, 0x4C, 200),
            Rgba::with_alpha(0xF0, 0xB4, 0x4C, 0),
        ),
    ));
    ws.project.add(glow);
    // Gold-to-bronze lettering with a black Outside outline.
    let text = add_text(
        ws,
        "ACE",
        CharStyle {
            family: "Barlow Condensed".into(),
            weight: 700,
            size: 900.0,
            ..CharStyle::default()
        },
        (1300.0, 1050.0),
        Rgba::rgb(255, 255, 255),
        Some(StrokeStyle {
            paint: Rgba::rgb(0x11, 0x11, 0x11).into(),
            width: 30.0,
            align: StrokeAlign::Outside,
            line: LineStyle {
                join: Join::Round,
                ..LineStyle::default()
            },
            swatch: None,
        }),
    );
    let mut lettering = (**ws.project.surface().get(text).unwrap()).clone();
    let mut gold = Gradient::new(
        GradientKind::Linear,
        &stops(Rgba::rgb(0xFF, 0xE0, 0x8A), Rgba::rgb(0xA8, 0x6A, 0x1E)),
    );
    gold.set_angle(&lettering.frame, 90.0);
    lettering.fill = Paint::Gradient(gold);
    ws.project.surface_mut().replace(&[lettering]);
    // A dashed line going from yellow to red.
    let mut data = PathData::new(vec![Subpath::new(
        vec![
            Node::corner(Point::new(500.0, 2900.0)),
            Node::corner(Point::new(3600.0, 2900.0)),
        ],
        false,
    )]);
    data.line_width = 40.0;
    data.line_style = LineStyle {
        dash: Some(Dash {
            dash: 160.0,
            gap: 60.0,
        }),
        cap: Cap::Butt,
        ..LineStyle::default()
    };
    let mut line = Object::from_path(ObjectId(0), data);
    line.name = "Stripe".into();
    line.fill = Paint::Gradient(Gradient::new(
        GradientKind::Linear,
        &stops(Rgba::rgb(0xFF, 0xD0, 0x20), Rgba::rgb(0xE0, 0x20, 0x20)),
    ));
    let line = ws.project.add(line);
    (text, line)
}

/// Stars, a polygon, a swoosh with a hole and open lines.
pub fn vector_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> Vec<tp_core::document::ObjectId> {
    use tp_core::document::{
        Frame, Node, Object, ObjectId, PathData, Rgba, ShapeKind, StrokeStyle, Subpath,
    };
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().unwrap();
    let mut ids = Vec::new();
    let mut add = |o: Object| ids.push(ws.project.add(o));
    let base = |kind, c: (f64, f64), s: (f64, f64), rot: f64, fill: Rgba| {
        let mut o = Object::new(ObjectId(0), kind, Frame::new(c.into(), s.into(), rot));
        o.fill = fill.into();
        o
    };
    add(base(
        ShapeKind::rectangle(),
        (2048.0, 2048.0),
        (4096.0, 4096.0),
        0.0,
        Rgba::rgb(0x1B, 0x2A, 0x41),
    ));
    let mut star = base(
        ShapeKind::Polygon {
            sides: 5,
            star: Some(0.42),
        },
        (800.0, 900.0),
        (900.0, 860.0),
        -8.0,
        Rgba::rgb(0xF0, 0xB4, 0x4C),
    );
    star.stroke = Some(StrokeStyle {
        paint: Rgba::rgb(255, 255, 255).into(),
        width: 24.0,
        ..Default::default()
    });
    add(star);
    add(base(
        ShapeKind::Polygon {
            sides: 6,
            star: None,
        },
        (2000.0, 900.0),
        (800.0, 700.0),
        0.0,
        Rgba::rgb(0xC0, 0x39, 0x2B),
    ));
    let corner = |x: f64, y: f64| Node::corner(Point::new(x, y));
    let swoosh = Subpath::new(
        vec![
            Node::smooth(Point::new(300.0, 2600.0), Point::new(1200.0, 1700.0)),
            Node::smooth(Point::new(3600.0, 2100.0), Point::new(3900.0, 2300.0)),
            Node::smooth(Point::new(3500.0, 2600.0), Point::new(2400.0, 2300.0)),
        ],
        true,
    );
    let hole = Subpath::new(
        vec![
            corner(1600.0, 2250.0),
            corner(1600.0, 2450.0),
            corner(1900.0, 2450.0),
            corner(1900.0, 2250.0),
        ],
        true,
    );
    let mut s = Object::from_path(ObjectId(0), PathData::new(vec![swoosh, hole]));
    s.fill = Rgba::rgb(0x2E, 0x86, 0xDE).into();
    s.name = "Swoosh".into();
    add(s);
    for (i, y) in [3200.0, 3400.0, 3600.0].into_iter().enumerate() {
        let mut line = Object::from_path(
            ObjectId(0),
            PathData::new(vec![Subpath::new(
                vec![
                    corner(400.0, y),
                    Node::smooth(Point::new(2000.0, y - 150.0), Point::new(2600.0, y - 150.0)),
                    corner(3700.0, y),
                ],
                false,
            )]),
        );
        line.fill = Rgba::rgb(255, 255, 255).into();
        line.edit_path(|p| p.line_width = 20.0 + 20.0 * i as f64);
        if i == 2 {
            // The thickest line outlined in black.
            line.stroke = Some(StrokeStyle {
                paint: Rgba::rgb(0, 0, 0).into(),
                width: 12.0,
                ..Default::default()
            });
        }
        line.name = "Line".into();
        add(line);
    }
    let _ = Size::ZERO;
    ids
}

/// The brand kit's fleet: the sample truck with its default textures and
/// the high roof, three swatches (Company red linked from a gradient stop,
/// a stroke and a lettering; Company grey from a fill), the Stripe and
/// Band graphic styles, the Lettering text style, the band selected.
/// Returns Company red and the Band style.
pub fn brand_fleet(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> (tp_core::document::SwatchId, tp_core::document::StyleId) {
    use tp_core::document::{
        CharStyle, ColorStop, Frame, Gradient, GradientKind, Object, ObjectId, Paint, Rgba,
        ShapeKind, StrokeStyle, TextBlock,
    };
    use tp_core::kurbo::{Point, Size};
    let package = tp_vehicles::Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
    let mut textures = tp_app::vehicle_project::default_textures(&package.manifest);
    textures.push("high_roof".into());
    let project =
        tp_app::vehicle_project::fleet_project("ACE Logistics", &[(&package, &textures)]).unwrap();
    super::open_project(h, project);
    let ws = h.state_mut().workspace_mut().unwrap();
    let p = &mut ws.project;
    let (red, _) = p.add_swatch(Rgba::rgb(0xC0, 0x10, 0x20), "Color");
    p.rename_swatch(red, "Company red");
    let (grey, _) = p.add_swatch(Rgba::rgb(0x50, 0x55, 0x5A), "Color");
    p.rename_swatch(grey, "Company grey");
    p.add_swatch(Rgba::rgb(0xF0, 0xB4, 0x4C), "Color");
    let frame = |x, y, w, h| Frame::new(Point::new(x, y), Size::new(w, h), 0.0);
    let mut stripe = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        frame(2048.0, 2600.0, 3600.0, 260.0),
    );
    stripe.name = "Stripe".into();
    let mut stop = ColorStop::new(0.0, Rgba::rgb(0xC0, 0x10, 0x20));
    stop.swatch = Some(red);
    stripe.fill = Paint::Gradient(Gradient::new(
        GradientKind::Linear,
        &[stop, ColorStop::new(1.0, Rgba::rgb(0x50, 0x55, 0x5A))],
    ));
    let stripe = p.add(stripe);
    p.new_graphic_style(stripe, "Style");
    p.rename_style(p.graphic_styles[0].id, "Stripe");
    let mut band = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        frame(2048.0, 3000.0, 3600.0, 120.0),
    );
    band.fill = Paint::Solid(Rgba::rgb(0x50, 0x55, 0x5A));
    band.fill_swatch = Some(grey);
    band.stroke = Some(StrokeStyle {
        paint: Paint::Solid(Rgba::rgb(0xC0, 0x10, 0x20)),
        swatch: Some(red),
        width: 12.0,
        ..StrokeStyle::default()
    });
    let band = p.add(band);
    p.new_graphic_style(band, "Style");
    p.rename_style(p.graphic_styles[1].id, "Band");
    let mut lettering = Object::new(
        ObjectId(0),
        ShapeKind::Text,
        frame(1200.0, 1500.0, 10.0, 10.0),
    );
    lettering.fill = Paint::Solid(Rgba::rgb(0xC0, 0x10, 0x20));
    lettering.fill_swatch = Some(red);
    lettering.text = Some(TextBlock::new(
        "ACE LOGISTICS",
        CharStyle {
            size: 260.0,
            ..CharStyle::default()
        },
    ));
    let lettering = p.add(lettering);
    p.new_text_style(lettering, "Text style");
    p.rename_style(p.text_styles[0].id, "Lettering");
    ws.relayout_all_texts();
    ws.selection = vec![band];
    ws.panels.color_target = tp_app::workspace::ColorTarget::Fill;
    let band_style = ws.project.graphic_styles[1].id;
    (red, band_style)
}

/// The brand kit's fleet with the band and the stripe copied on every
/// other texture, so the impact box shows several tiles.
pub fn editor_fleet(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) {
    brand_fleet(h);
    let ws = h.state_mut().workspace_mut().unwrap();
    let copies: Vec<_> = ws.project.surfaces[0]
        .objects
        .iter()
        .filter(|o| o.text.is_none())
        .cloned()
        .collect();
    let side = ws.project.surfaces[0].max_side();
    for i in 1..ws.project.surfaces.len() {
        ws.project.active_surface = i;
        // Scaled to the texture's size.
        let k = ws.project.surface().max_side() / side;
        for o in &copies {
            let mut o = tp_core::document::Object::clone(o);
            o.frame.center = (o.frame.center.to_vec2() * k).to_point();
            o.frame.size *= k;
            ws.project.add(o);
        }
    }
    ws.set_active_surface(0);
    ws.selection.clear();
    super::settle_renders(h);
}

/// Opens the editor on Company red (a new green) or on the Band style (a
/// new green fill), as the picker would set it.
pub fn open_editor(h: &mut egui_kittest::Harness<'static, tp_app::AppState>, target: EditorTarget) {
    use tp_app::brand_ops::EditValue;
    use tp_core::document::{Paint, Rgba};
    let green = Rgba::rgb(0x24, 0x5E, 0x45);
    let ws = h.state_mut().workspace_mut().unwrap();
    let value = match target {
        EditorTarget::Swatch => {
            let red = ws.project.palette[0].id;
            ws.start_swatch_edit(red);
            EditValue::Color(green)
        }
        EditorTarget::Style => {
            let band = &ws.project.graphic_styles[1];
            let mut look = band.look;
            look.fill = Paint::Solid(green);
            look.fill_swatch = None;
            ws.start_style_edit(band.id);
            EditValue::Look(look)
        }
    };
    ws.set_edit_value(value);
    let edit = ws.panels.brand_edit.as_mut().unwrap();
    let hsva = tp_core::document::Hsva::from(green);
    edit.hex = green.to_hex();
    edit.hsv = tp_ui::widgets::Hsv::new(hsva.h / 360.0, hsva.s, hsva.v, hsva.a);
}

/// A small livery tree: Background, Graphics (3 stripes), Branding (logo).
pub fn livery_tree(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> Vec<tp_core::document::ObjectId> {
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle};
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().expect("project open");
    let shape = |kind, name: &str, c: (f64, f64), s: (f64, f64), fill: Rgba| {
        let mut o = Object::new(
            ObjectId(0),
            kind,
            Frame::new(Point::new(c.0, c.1), Size::new(s.0, s.1), 0.0),
        );
        o.name = name.into();
        o.fill = fill.into();
        o
    };
    let bg = ws.project.add(shape(
        ShapeKind::rectangle(),
        "Background",
        (2048.0, 2048.0),
        (4096.0, 4096.0),
        Rgba::rgb(0xF2, 0xF2, 0xF2),
    ));
    let base = ws.project.add(shape(
        ShapeKind::rectangle(),
        "Burgundy base",
        (2048.0, 1500.0),
        (4096.0, 1200.0),
        Rgba::rgb(0x7A, 0x1F, 0x2B),
    ));
    let white = ws.project.add(shape(
        ShapeKind::rectangle(),
        "White stripe",
        (2048.0, 2200.0),
        (4096.0, 160.0),
        Rgba::rgb(0xFF, 0xFF, 0xFF),
    ));
    let grey = ws.project.add(shape(
        ShapeKind::rectangle(),
        "Grey stripe",
        (2048.0, 2400.0),
        (4096.0, 120.0),
        Rgba::rgb(0x70, 0x70, 0x78),
    ));
    let graphics = ws.project.group(&[base, white, grey]).unwrap();
    let mut logo = shape(
        ShapeKind::Ellipse,
        "Logo",
        (1200.0, 3200.0),
        (900.0, 900.0),
        Rgba::rgb(0xF0, 0xB4, 0x4C),
    );
    logo.stroke = Some(StrokeStyle {
        paint: Rgba::rgb(0x20, 0x20, 0x20).into(),
        width: 24.0,
        ..Default::default()
    });
    let logo = ws.project.add(logo);
    let mut name = shape(
        ShapeKind::Rectangle {
            corner_radius: 40.0,
        },
        "Company name",
        (2700.0, 3200.0),
        (1600.0, 300.0),
        Rgba::rgb(0x30, 0x30, 0x36),
    );
    name.opacity = 0.8;
    let name = ws.project.add(name);
    let branding = ws.project.group(&[logo, name]).unwrap();
    for (id, label) in [(graphics, "Graphics"), (branding, "Branding")] {
        let mut g = (**ws.project.surface().get(id).unwrap()).clone();
        g.name = label.into();
        ws.project.surface_mut().replace(&[g]);
    }
    ws.panels.expanded.extend([graphics, branding]);
    vec![bg, graphics, branding, logo]
}

/// What [`open_editor`] edits.
pub enum EditorTarget {
    Swatch,
    Style,
}
