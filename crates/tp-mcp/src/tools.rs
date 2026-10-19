//! The tools: their names, descriptions and input schemas (what the client
//! reads in `tools/list`), and their arguments turned into [`Call`]s.

use serde_json::{Value, json};
use tp_core::document::{BooleanOp, Edge, TextAlign};

use crate::args::{self, Args};
use crate::call::{AlignTarget, ArrangeAction, Call, OrderMove, ShapeSpec, WriteOp};

/// Default longest side of the images returned, in pixels.
pub const DEFAULT_IMAGE_SIZE: u32 = 1024;
/// Largest longest side of the images returned, in pixels.
pub const MAX_IMAGE_SIZE: u32 = 2048;

/// What the server tells the client when it connects: the workflow.
pub const INSTRUCTIONS: &str = "TruckPaint is a livery editor for Euro Truck Simulator 2 and \
American Truck Simulator. The open project is a fleet: trucks and trailers, each painted \
through one or more textures. A texture is the UV layout of one vehicle part: its template \
shows where the cab front, sides, roof and doors land. Positions never carry over from one \
vehicle to another.\n\
Workflow: 1) get_project to list the textures; 2) get_template of the texture to paint, and \
find where each part of the truck is; 3) draw with add_path (SVG path data), add_shape, \
add_text and import_image, in texture pixels; 4) render_texture and compare with the \
reference; 5) correct with update_object, arrange_objects or delete_objects. Reuse the \
palette (add_swatch, colors as {\"swatch\": name}) and symbols for what the whole fleet \
shares. Every change is one undo step the user can revert; nothing is saved or exported \
for the user.";

/// The tools, as `tools/list` returns them.
pub fn list() -> Value {
    let texture = json!({"type": "string", "description": "Texture id from get_project (\"<vehicle package>/<texture>\")."});
    let ids = json!({"type": "array", "items": {"type": "integer"}, "description": "Object ids (from list_objects or a previous call)."});
    let fill = json!({"description": "A color (see color), \"none\", or a gradient {\"gradient\": \"linear\"|\"radial\", \"angle\": degrees (0 = left to right, 90 = top to bottom), \"stops\": [{\"offset\": 0..1, \"color\": color}, ...]} with 2 stops or more.",
        "anyOf": [{"type": "string"}, {"type": "object"}]});
    let stroke = json!({"description": "An outline {\"color\": color, \"width\": px, \"align\": \"center\"|\"inside\"|\"outside\"}, or null for none.",
        "type": ["object", "null"]});
    let shadow = json!({"description": "A drop shadow {\"color\", \"opacity\": 0..1, \"offset_x\", \"offset_y\" (px), \"blur\": 0..200 px}, or null for none.",
        "type": ["object", "null"]});
    let num = |d: &str| json!({"type": "number", "description": d});
    let look = |extra: Value| {
        let mut p = json!({
            "fill": fill.clone(),
            "stroke": stroke.clone(),
            "opacity": num("0..1, default 1."),
            "shadow": shadow.clone(),
            "name": {"type": "string", "description": "Layer name, shown in the Layers panel."},
            "parent": {"type": "integer", "description": "A group to add the object into (default: top of the texture's layers)."},
        });
        if let (Value::Object(p), Value::Object(e)) = (&mut p, extra) {
            p.extend(e);
        }
        p
    };
    let tool = |name: &str,
                description: &str,
                properties: Value,
                required: &[&str],
                read_only: bool| {
        json!({
            "name": name,
            "description": description,
            "inputSchema": {"type": "object", "properties": properties, "required": required},
            "annotations": {"readOnlyHint": read_only, "destructiveHint": false, "openWorldHint": false},
        })
    };
    let boxed = |extra: Value| {
        let mut p = json!({
            "x": num("Left of the box, texture pixels (origin top-left, y down)."),
            "y": num("Top of the box, texture pixels."),
            "width": num("Width in texture pixels."),
            "height": num("Height in texture pixels."),
            "rotation": num("Degrees, clockwise, around the box center (default 0)."),
        });
        if let (Value::Object(p), Value::Object(e)) = (&mut p, extra) {
            p.extend(e);
        }
        p
    };
    json!([
        tool(
            "get_project",
            "Lists the open project: its game, vehicles, and each vehicle's textures (id, name, main texture or accessory, width and height in texture pixels), the palette swatches, the graphic and text styles, and the symbols. Start here.",
            json!({}),
            &[],
            true
        ),
        tool(
            "list_objects",
            "Lists the objects of one texture, top to bottom, with their id, kind, name, box (x, y, width, height in texture pixels), rotation, visibility and lock. Groups list their children.",
            json!({"texture": texture}),
            &["texture"],
            true
        ),
        tool(
            "get_object",
            "Returns one object's properties: kind, name, box, rotation, fill, stroke, opacity, shadow, the swatch and style it follows, its text for a text, its outline as SVG path data in texture pixels for a path.",
            json!({"texture": texture, "id": {"type": "integer"}}),
            &["texture", "id"],
            true
        ),
        tool(
            "get_template",
            "Returns the texture's template as a PNG: the vehicle's UV layout, showing where each part of the vehicle (cab front, sides, roof, doors, grille…) lands on the texture. Look at it before drawing: positions are in texture pixels on this layout. The text says how many texture pixels one image pixel covers.",
            json!({"texture": texture, "max_size": {"type": "integer", "description": "Longest side of the image in pixels, default 1024, at most 2048."}}),
            &["texture"],
            true
        ),
        tool(
            "render_texture",
            "Renders the texture's artwork as a PNG, as Export Texture draws it, with the template over it as the canvas shows it (unless with_template is false). Use it to compare your work with the reference image, then correct. The text says how many texture pixels one image pixel covers.",
            json!({"texture": texture, "max_size": {"type": "integer", "description": "Longest side in pixels, default 1024, at most 2048."}, "with_template": {"type": "boolean", "description": "Default true."}}),
            &["texture"],
            true
        ),
        tool(
            "add_shape",
            "Adds a rectangle (with corner_radius), an ellipse, a polygon (sides 3..12) or a star (points 3..12, inner_ratio 0.1..0.9) at a box on a texture, at the top of its layers. Returns the new object's id.",
            json!(look(boxed(json!({
                "texture": texture,
                "shape": {"type": "string", "enum": ["rectangle", "ellipse", "polygon", "star"]},
                "corner_radius": num("Rectangles: corner radius in px."),
                "sides": {"type": "integer", "description": "Polygons: 3 to 12, default 6."},
                "points": {"type": "integer", "description": "Stars: 3 to 12, default 5."},
                "inner_ratio": num("Stars: inner radius / outer radius, 0.1..0.9, default 0.5."),
            })))),
            &["texture", "shape", "x", "y", "width", "height"],
            false
        ),
        tool(
            "add_path",
            "Adds a path from SVG path data `d` (M, L, H, V, C, S, Q, T, A, Z, absolute or relative) in texture pixels: stripes, swooshes, flames, lettering outlines. Several subpaths make one path; with fill_rule \"evenodd\" inner contours become holes. The path stays editable. Returns its id.",
            json!(look(json!({
                "texture": texture,
                "d": {"type": "string", "description": "SVG path data in texture pixels."},
                "fill_rule": {"type": "string", "enum": ["nonzero", "evenodd"], "description": "Default nonzero."},
            }))),
            &["texture", "d"],
            false
        ),
        tool(
            "add_text",
            "Adds an editable text with its top-left at (x, y). The fill is the text color. Returns its id.",
            json!(look(json!({
                "texture": texture,
                "text": {"type": "string", "description": "The text; new lines allowed."},
                "font": {"type": "string", "description": "Font family (default Inter)."},
                "size": num("Font size in texture pixels (default 200)."),
                "weight": {"type": "integer", "description": "100..900 (default 700)."},
                "italic": {"type": "boolean"},
                "align": {"type": "string", "enum": ["left", "center", "right"]},
                "x": num("Left of the text box."),
                "y": num("Top of the text box."),
                "rotation": num("Degrees, clockwise."),
            }))),
            &["texture", "text", "x", "y"],
            false
        ),
        tool(
            "import_image",
            "Imports a PNG, JPEG or SVG image (a logo, a photo) onto a texture, from `data` (base64, with a `name` such as \"logo.png\") or from an absolute local file `path`. Placed at the box when given, else centered at its natural size (at most half the texture). An SVG stays vector. Returns its id.",
            json!(boxed(json!({
                "texture": texture,
                "data": {"type": "string", "description": "The file's content in base64."},
                "name": {"type": "string", "description": "File name with extension, required with data."},
                "path": {"type": "string", "description": "Absolute path to a .png, .jpg, .jpeg or .svg file."},
                "parent": {"type": "integer"},
            }))),
            &["texture"],
            false
        ),
        tool(
            "update_object",
            "Changes an object: any of its box (x, y, width, height), rotation, fill, stroke, opacity, shadow, name, visibility and lock. Changing the look detaches it from its graphic style.",
            json!(look(boxed(json!({
                "texture": texture,
                "id": {"type": "integer"},
                "visible": {"type": "boolean"},
                "locked": {"type": "boolean"},
            })))),
            &["texture", "id"],
            false
        ),
        tool(
            "arrange_objects",
            "Arranges objects: forward, backward, front or back in the layers; group or ungroup; align (left, hcenter, right, top, vcenter, bottom) to each other or to the texture; or combine closed shapes into one path (unite, subtract (the top ones from the bottom one), intersect, exclude).",
            json!({
                "texture": texture,
                "ids": ids,
                "action": {"type": "string", "enum": ["forward", "backward", "front", "back", "group", "ungroup", "align", "combine"]},
                "edge": {"type": "string", "enum": ["left", "hcenter", "right", "top", "vcenter", "bottom"], "description": "For align."},
                "to": {"type": "string", "enum": ["selection", "texture"], "description": "For align, default selection (the texture for one object)."},
                "operation": {"type": "string", "enum": ["unite", "subtract", "intersect", "exclude"], "description": "For combine."},
            }),
            &["texture", "ids", "action"],
            false
        ),
        tool(
            "delete_objects",
            "Deletes objects (and their children).",
            json!({"texture": texture, "ids": ids}),
            &["texture", "ids"],
            false
        ),
        tool(
            "add_swatch",
            "Adds a named color to the project's palette, shared by the whole fleet. Colors given as {\"swatch\": name} then follow later edits of the swatch.",
            json!({"name": {"type": "string"}, "color": {"type": "string", "description": "\"#rrggbb\"."}}),
            &["name", "color"],
            false
        ),
        tool(
            "apply_style",
            "Applies a graphic style (fill, stroke, opacity, shadow) or a text style (to texts) from get_project to objects.",
            json!({"texture": texture, "style": {"type": "string", "description": "Style name or id."}, "ids": ids}),
            &["texture", "style", "ids"],
            false
        ),
        tool(
            "place_symbol",
            "Places an instance of a project symbol (a logo or lettering defined once for the fleet) at a box on a texture. Returns its id.",
            json!(boxed(
                json!({"texture": texture, "symbol": {"type": "string", "description": "Symbol name or id."}})
            )),
            &["texture", "symbol", "x", "y", "width", "height"],
            false
        ),
    ])
}

/// The call of tool `name` with `arguments`.
pub fn parse(name: &str, arguments: &Value) -> Result<Call, String> {
    let a = Args::new(arguments)?;
    let texture = || a.string("texture");
    let max_size = || -> Result<u32, String> {
        let size = a.opt_in("max_size", 1.0..=f64::from(u32::MAX))?;
        Ok(size.map_or(DEFAULT_IMAGE_SIZE, |s| (s as u32).min(MAX_IMAGE_SIZE)))
    };
    let write = |op: WriteOp| -> Result<Call, String> {
        Ok(Call::Write {
            texture: texture()?,
            op,
        })
    };
    match name {
        "get_project" => Ok(Call::GetProject),
        "list_objects" => Ok(Call::ListObjects {
            texture: texture()?,
        }),
        "get_object" => Ok(Call::GetObject {
            texture: texture()?,
            id: a.id("id")?,
        }),
        "get_template" => Ok(Call::GetTemplate {
            texture: texture()?,
            max_size: max_size()?,
        }),
        "render_texture" => Ok(Call::RenderTexture {
            texture: texture()?,
            max_size: max_size()?,
            with_template: a.opt_bool("with_template")?.unwrap_or(true),
        }),
        "add_shape" => {
            let kind = a.choice(
                "shape",
                &[
                    ("rectangle", 0),
                    ("ellipse", 1),
                    ("polygon", 2),
                    ("star", 3),
                ],
                None,
            )?;
            let count = |key: &str, default: f64| -> Result<u8, String> {
                Ok(a.opt_in(key, 3.0..=12.0)?.unwrap_or(default).round() as u8)
            };
            let shape = match kind {
                0 => ShapeSpec::Rectangle {
                    corner_radius: a
                        .opt_in("corner_radius", 0.0..=args::MAX_COORD)?
                        .unwrap_or(0.0),
                },
                1 => ShapeSpec::Ellipse,
                2 => ShapeSpec::Polygon {
                    sides: count("sides", 6.0)?,
                },
                _ => ShapeSpec::Star {
                    points: count("points", 5.0)?,
                    inner_ratio: a.opt_in("inner_ratio", 0.1..=0.9)?.unwrap_or(0.5),
                },
            };
            write(WriteOp::AddShape {
                shape,
                frame: a.box_spec()?,
                look: a.look()?,
                parent: a.opt_id("parent")?,
            })
        }
        "add_path" => {
            let path = args::path_data(&a.string("d")?)?;
            let even_odd = a.choice(
                "fill_rule",
                &[("nonzero", false), ("evenodd", true)],
                Some(false),
            )?;
            write(WriteOp::AddPath {
                path,
                even_odd,
                look: a.look()?,
                parent: a.opt_id("parent")?,
            })
        }
        "add_text" => {
            let content = a.string("text")?;
            if content.trim().is_empty() {
                return Err("`text` must not be empty".to_owned());
            }
            write(WriteOp::AddText {
                content,
                family: a.opt_string("font")?.filter(|f| !f.trim().is_empty()),
                weight: a.opt_in("weight", 100.0..=900.0)?.map(|w| w.round() as u16),
                italic: a.opt_bool("italic")?.unwrap_or(false),
                size: a.opt_in("size", 1.0..=20_000.0)?.unwrap_or(200.0),
                align: a.choice(
                    "align",
                    &[
                        ("left", TextAlign::Left),
                        ("center", TextAlign::Center),
                        ("right", TextAlign::Right),
                    ],
                    Some(TextAlign::Left),
                )?,
                x: a.number("x")?,
                y: a.number("y")?,
                rotation: a.opt_number("rotation")?.unwrap_or(0.0),
                look: a.look()?,
                parent: a.opt_id("parent")?,
            })
        }
        "import_image" => {
            let texture = texture()?;
            let frame = a.opt_box_spec()?;
            let parent = a.opt_id("parent")?;
            let (name, bytes) = args::image_source(&a)?;
            Ok(Call::Write {
                texture,
                op: WriteOp::ImportImage {
                    name,
                    bytes,
                    frame,
                    parent,
                },
            })
        }
        "update_object" => {
            let op = WriteOp::UpdateObject {
                id: a.id("id")?,
                frame: a.box_change()?,
                look: a.look()?,
                visible: a.opt_bool("visible")?,
                locked: a.opt_bool("locked")?,
            };
            write(op)
        }
        "arrange_objects" => {
            let action = match a.choice(
                "action",
                &[
                    ("forward", 0),
                    ("backward", 1),
                    ("front", 2),
                    ("back", 3),
                    ("group", 4),
                    ("ungroup", 5),
                    ("align", 6),
                    ("combine", 7),
                ],
                None,
            )? {
                0 => ArrangeAction::Order(OrderMove::Forward),
                1 => ArrangeAction::Order(OrderMove::Backward),
                2 => ArrangeAction::Order(OrderMove::Front),
                3 => ArrangeAction::Order(OrderMove::Back),
                4 => ArrangeAction::Group,
                5 => ArrangeAction::Ungroup,
                6 => ArrangeAction::Align {
                    edge: a.choice(
                        "edge",
                        &[
                            ("left", Edge::Left),
                            ("hcenter", Edge::HCenter),
                            ("right", Edge::Right),
                            ("top", Edge::Top),
                            ("vcenter", Edge::VCenter),
                            ("bottom", Edge::Bottom),
                        ],
                        None,
                    )?,
                    to: a.choice(
                        "to",
                        &[
                            ("selection", AlignTarget::Selection),
                            ("texture", AlignTarget::Texture),
                        ],
                        Some(AlignTarget::Selection),
                    )?,
                },
                _ => ArrangeAction::Combine(a.choice(
                    "operation",
                    &[
                        ("unite", BooleanOp::Unite),
                        ("subtract", BooleanOp::MinusFront),
                        ("intersect", BooleanOp::Intersect),
                        ("exclude", BooleanOp::Exclude),
                    ],
                    None,
                )?),
            };
            write(WriteOp::Arrange {
                ids: a.ids("ids")?,
                action,
            })
        }
        "delete_objects" => write(WriteOp::Delete { ids: a.ids("ids")? }),
        "add_swatch" => {
            let name = a.string("name")?.trim().to_owned();
            if name.is_empty() {
                return Err("`name` must not be empty".to_owned());
            }
            let crate::call::ColorRef::Hex(color) =
                args::color(a.get("color").ok_or("`color` is required")?, "color")?
            else {
                return Err("`color` must be a hex color".to_owned());
            };
            Ok(Call::AddSwatch { name, color })
        }
        "apply_style" => write(WriteOp::ApplyStyle {
            style: a.string("style")?,
            ids: a.ids("ids")?,
        }),
        "place_symbol" => write(WriteOp::PlaceSymbol {
            symbol: a.string("symbol")?,
            frame: a.box_spec()?,
        }),
        other => Err(format!("unknown tool `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call::ColorRef;

    /// Preferences tells the player the assistant cannot save, export or open
    /// a project: that list must change with the tools.
    #[test]
    fn no_tool_saves_exports_or_opens_a_project() {
        let tools = list();
        for t in tools.as_array().unwrap() {
            let name = t["name"].as_str().unwrap();
            for word in ["save", "export", "open"] {
                assert!(!name.contains(word), "{name}");
            }
        }
    }

    #[test]
    fn every_tool_has_a_schema_and_a_description() {
        let tools = list();
        let tools = tools.as_array().unwrap();
        assert_eq!(tools.len(), 15);
        for t in tools {
            let name = t["name"].as_str().unwrap();
            assert!(
                t["description"].as_str().is_some_and(|d| d.len() > 30),
                "{name}"
            );
            assert_eq!(t["inputSchema"]["type"], "object", "{name}");
            for required in t["inputSchema"]["required"].as_array().unwrap() {
                let key = required.as_str().unwrap();
                assert!(
                    t["inputSchema"]["properties"].get(key).is_some(),
                    "{name}: {key} required but not described"
                );
            }
        }
    }

    #[test]
    fn descriptions_teach_the_livery_rules() {
        let text = format!("{}{}", list(), INSTRUCTIONS);
        for needle in ["template", "texture pixels", "undo", "UV layout"] {
            assert!(text.contains(needle), "{needle}");
        }
    }

    #[test]
    fn stripe() {
        let call = parse(
            "add_path",
            &json!({"texture": "daf/main", "d": "M 0 1800 C 1200 1500 2800 2100 4096 1800 L 4096 2000 C 2800 2300 1200 1700 0 2000 Z", "fill": "#d4202c"}),
        )
        .unwrap();
        let Call::Write {
            texture,
            op: WriteOp::AddPath { path, look, .. },
        } = call
        else {
            panic!("add_path");
        };
        assert_eq!(texture, "daf/main");
        let b = tp_core::kurbo::Shape::bounding_box(&path);
        assert!(b.x0.abs() < 1e-9 && (b.x1 - 4096.0).abs() < 1e-9);
        assert!(matches!(
            look.fill,
            Some(crate::call::PaintSpec::Color(ColorRef::Hex(_)))
        ));
    }

    #[test]
    fn errors_name_the_field() {
        let err = parse(
            "add_shape",
            &json!({"texture": "t", "shape": "rectangle", "x": 0, "y": 0, "width": 10}),
        )
        .unwrap_err();
        assert_eq!(err, "`height` is required");
        let err = parse(
            "add_shape",
            &json!({"texture": "t", "shape": "blob", "x": 0, "y": 0, "width": 10, "height": 5}),
        )
        .unwrap_err();
        assert!(err.starts_with("`shape` must be one of"), "{err}");
        assert!(parse("nope", &json!({})).is_err());
    }

    #[test]
    fn max_size_is_capped() {
        let Call::RenderTexture { max_size, .. } =
            parse("render_texture", &json!({"texture": "t", "max_size": 8000})).unwrap()
        else {
            panic!()
        };
        assert_eq!(max_size, MAX_IMAGE_SIZE);
        let Call::GetTemplate { max_size, .. } =
            parse("get_template", &json!({"texture": "t"})).unwrap()
        else {
            panic!()
        };
        assert_eq!(max_size, DEFAULT_IMAGE_SIZE);
    }
}
