//! Interaction tests for design-system widgets.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_ui::icons;
use tp_ui::tokens::size;
use tp_ui::widgets::{IconButton, MenuRow, ToolButton};

/// Harness whose first frame only installs the theme (fonts registered in a
/// frame become usable from the next one).
pub fn themed<S>(
    mut app: impl FnMut(&mut egui::Ui, &mut S) + 'static,
    state: S,
) -> Harness<'static, S> {
    let mut installed = false;
    Harness::new_ui_state(
        move |ui, state| {
            if !installed {
                tp_ui::theme::install(ui.ctx(), tp_ui::ThemeSettings::default());
                installed = true;
                ui.ctx().request_repaint();
                return;
            }
            app(ui, state);
        },
        state,
    )
}

#[test]
fn tool_button_click_active_and_hit_area() {
    let mut h = themed(
        |ui, clicks: &mut i32| {
            if ui
                .add(ToolButton::new(icons::RECTANGLE, "Rectangle").active(true))
                .clicked()
            {
                *clicks += 1;
            }
            ui.add_enabled(false, ToolButton::new(icons::PEN, "Pen"));
        },
        0,
    );
    h.run();

    let rect = h.get_by_label("Rectangle").rect();
    assert!(rect.width() >= size::HIT_MIN && rect.height() >= size::HIT_MIN);
    // The active state is exposed to assistive technology, not only drawn.
    assert_eq!(
        h.get_by_label("Rectangle").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );

    h.get_by_label("Rectangle").click();
    h.run();
    h.get_by_label("Pen").click();
    h.run();
    assert_eq!(*h.state(), 1, "disabled tool button must ignore clicks");
    assert!(h.get_by_label("Pen").accesskit_node().is_disabled());
}

#[test]
fn icon_button_hit_area_is_at_least_minimum() {
    let mut h = themed(
        |ui, _: &mut ()| {
            ui.add(IconButton::new(icons::CLOSE, "Close"));
        },
        (),
    );
    h.run();
    let rect = h.get_by_label("Close").rect();
    assert!(rect.width() >= size::HIT_MIN && rect.height() >= size::HIT_MIN);
}

#[test]
fn disabled_menu_row_ignores_clicks() {
    let mut h = themed(
        |ui, clicked: &mut bool| {
            *clicked |= ui
                .add_enabled(false, MenuRow::new("Undo").shortcut(Some("Ctrl+Z")))
                .clicked();
        },
        false,
    );
    h.run();
    h.get_by_label("Undo").click();
    h.run();
    assert!(!*h.state());
}

#[test]
fn hover_shows_tooltip_with_shortcut() {
    let mut h = themed(
        |ui, _: &mut ()| {
            ui.add(ToolButton::new(icons::ELLIPSE, "Ellipse").shortcut(Some("E")));
        },
        (),
    );
    h.run();
    h.get_by_label("Ellipse").hover();
    // Tooltips appear after the hover delay.
    for _ in 0..60 {
        h.step();
    }
    h.run();
    h.get_by_label("E");
}

mod editing_widgets {
    use egui::accesskit::Role;
    use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
    use egui_kittest::kittest::Queryable;
    use tp_ui::widgets::{
        ColorSwatch, FieldEvent, FillOrStroke, FillStrokeSwatches, Hsv, NumericField, SwatchColor,
        hue_slider, sv_square,
    };

    use super::themed;

    #[derive(Default)]
    struct FieldState {
        value: f64,
        events: Vec<FieldEvent>,
    }

    fn field_harness() -> egui_kittest::Harness<'static, FieldState> {
        themed(
            |ui, state: &mut FieldState| {
                let event = NumericField::new("W", "Width", Some(state.value))
                    .suffix("px")
                    .range(1.0..=10_000.0)
                    .show(ui);
                if event != FieldEvent::None {
                    state.events.push(event);
                    if let FieldEvent::Live(v) | FieldEvent::Commit(v) = event {
                        state.value = v;
                    }
                }
            },
            FieldState {
                value: 400.0,
                events: Vec::new(),
            },
        )
    }

    #[test]
    fn typing_and_enter_commits() {
        let mut h = field_harness();
        h.run();
        let field = h.get_by_role_and_label(Role::TextInput, "Width");
        field.focus();
        h.run();
        h.key_press_modifiers(Modifiers::COMMAND, Key::A);
        h.get_by_role_and_label(Role::TextInput, "Width")
            .type_text("600");
        h.run();
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(h.state().events.last(), Some(&FieldEvent::Commit(600.0)));
        assert_eq!(h.state().value, 600.0);
    }

    #[test]
    fn escape_reverts() {
        let mut h = field_harness();
        h.run();
        h.get_by_role_and_label(Role::TextInput, "Width").focus();
        h.run();
        h.get_by_role_and_label(Role::TextInput, "Width")
            .type_text("999");
        h.run();
        h.key_press(Key::Escape);
        h.run();
        assert_eq!(h.state().events.last(), Some(&FieldEvent::Revert));
        assert_eq!(h.state().value, 400.0);
    }

    #[test]
    fn scrubbing_the_label_reports_live_then_commit() {
        let mut h = field_harness();
        h.run();
        let label = h.get_by_label("W").rect().center();
        h.event(Event::PointerMoved(label));
        h.event(Event::PointerButton {
            pos: label,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.step();
        for i in 1..=5 {
            h.event(Event::PointerMoved(label + Vec2::new(i as f32 * 10.0, 0.0)));
            h.step();
        }
        let end = label + Vec2::new(50.0, 0.0);
        h.event(Event::PointerButton {
            pos: end,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run();
        let events = &h.state().events;
        assert!(events.iter().any(|e| matches!(e, FieldEvent::Live(_))));
        match events.last() {
            Some(FieldEvent::Commit(v)) => assert!(*v > 400.0, "{v}"),
            other => panic!("expected commit, got {other:?}"),
        }
    }

    #[test]
    fn mixed_value_shows_placeholder() {
        let mut h = themed(
            |ui, _: &mut ()| {
                NumericField::new("R", "Rotation", None).show(ui);
            },
            (),
        );
        h.run();
        let node = h.get_by_role_and_label(Role::TextInput, "Rotation");
        assert_eq!(node.value().unwrap_or_default(), "");
    }

    #[test]
    fn swatches_report_clicks() {
        let mut h = themed(
            |ui, clicked: &mut Vec<String>| {
                if ColorSwatch::new(SwatchColor::None, "No stroke")
                    .show(ui)
                    .clicked()
                {
                    clicked.push("none".into());
                }
                let pair = FillStrokeSwatches {
                    fill: SwatchColor::Solid(egui::Color32::RED),
                    stroke: SwatchColor::Mixed,
                    active: FillOrStroke::Fill,
                };
                if let Some(which) = pair.show(ui) {
                    clicked.push(format!("{which:?}"));
                }
            },
            Vec::new(),
        );
        h.run();
        h.get_by_label("No stroke").click();
        h.run();
        h.get_by_label("Stroke").click();
        h.run();
        assert_eq!(h.state(), &vec!["none".to_owned(), "Stroke".to_owned()]);
    }

    #[test]
    fn picker_drag_changes_values() {
        let mut h = themed(
            |ui, hsv: &mut (Hsv, bool)| {
                ui.set_width(200.0);
                if sv_square(ui, &mut hsv.0, 120.0).changed() {
                    hsv.1 = true;
                }
                hue_slider(ui, &mut hsv.0);
            },
            (Hsv::new(0.0, 0.0, 0.0, 1.0), false),
        );
        h.run();
        let square = h.get_by_label("Saturation and value").rect();
        let target = Pos2::new(square.right() - 1.0, square.top() + 1.0);
        h.event(Event::PointerMoved(target));
        h.event(Event::PointerButton {
            pos: target,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.event(Event::PointerButton {
            pos: target,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run();
        let (hsv, changed) = *h.state();
        assert!(changed);
        assert!(hsv.s > 0.95 && hsv.v > 0.95, "{hsv:?}");
        let hue = h.get_by_label("Hue").rect();
        let mid = Pos2::new(hue.center().x, hue.center().y);
        h.event(Event::PointerMoved(mid));
        h.event(Event::PointerButton {
            pos: mid,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.event(Event::PointerButton {
            pos: mid,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run();
        assert!((h.state().0.h - 0.5).abs() < 0.02, "{:?}", h.state().0);
    }

    #[test]
    fn hsv_to_color() {
        assert_eq!(
            Hsv::new(0.0, 1.0, 1.0, 1.0).to_color32(),
            egui::Color32::RED
        );
        assert_eq!(
            Hsv::new(1.0 / 3.0, 1.0, 1.0, 1.0).to_color32(),
            egui::Color32::GREEN
        );
        assert_eq!(
            Hsv::new(0.0, 0.0, 1.0, 1.0).to_color32(),
            egui::Color32::WHITE
        );
    }
}

/// Every shape painted in the last frame, nested shapes flattened.
fn painted_shapes<S>(h: &Harness<'static, S>) -> Vec<egui::Shape> {
    fn flatten(shape: &egui::Shape, out: &mut Vec<egui::Shape>) {
        match shape {
            egui::Shape::Vec(shapes) => shapes.iter().for_each(|s| flatten(s, out)),
            other => out.push(other.clone()),
        }
    }
    let mut out = Vec::new();
    for clipped in &h.output().shapes {
        flatten(&clipped.shape, &mut out);
    }
    out
}

/// Text shapes painted in the last frame: (text, color, font family).
fn painted_texts<S>(h: &Harness<'static, S>) -> Vec<(String, egui::Color32, egui::FontFamily)> {
    painted_shapes(h)
        .into_iter()
        .filter_map(|shape| match shape {
            egui::Shape::Text(text) => {
                let section = text.galley.job.sections.first()?;
                Some((
                    text.galley.text().to_owned(),
                    section.format.color,
                    section.format.font_id.family.clone(),
                ))
            }
            _ => None,
        })
        .collect()
}

mod segmented_controls {
    use egui::Color32;
    use egui::accesskit::Toggled;
    use egui_kittest::kittest::{NodeT, Queryable};
    use tp_ui::tokens::color;
    use tp_ui::widgets::SegmentedControl;

    use super::{painted_shapes, painted_texts, themed};

    fn harness() -> egui_kittest::Harness<'static, u8> {
        themed(
            |ui, current: &mut u8| {
                if let Some(v) = SegmentedControl::new()
                    .segment(0u8, "", "Solid", None)
                    .segment(1, "", "Linear", None)
                    .segment(2, "", "Radial", None)
                    .show(ui, *current)
                {
                    *current = v;
                }
            },
            0,
        )
    }

    /// Rectangles filled with `fill` in the last frame.
    fn rects_filled(h: &egui_kittest::Harness<'static, u8>, fill: Color32) -> Vec<egui::Rect> {
        painted_shapes(h)
            .into_iter()
            .filter_map(|shape| match shape {
                egui::Shape::Rect(r) if r.fill == fill => Some(r.rect),
                _ => None,
            })
            .collect()
    }

    fn luma(c: Color32) -> f32 {
        0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b())
    }

    #[test]
    fn choosing_an_option_moves_the_white_pill() {
        let mut h = harness();
        h.run();
        h.get_by_label("Linear").click();
        h.run();
        assert_eq!(*h.state(), 1);
        assert_eq!(
            h.get_by_label("Linear").accesskit_node().toggled(),
            Some(Toggled::True)
        );
        assert_eq!(
            h.get_by_label("Solid").accesskit_node().toggled(),
            Some(Toggled::False)
        );

        // One white pill, under Linear.
        let pills = rects_filled(&h, color::ACCENT_PRIMARY);
        assert_eq!(pills.len(), 1, "{pills:?}");
        assert!(pills[0].contains(h.get_by_label("Linear").rect().center()));
        // Linear in dark text on the pill, Solid as plain text on the track.
        let texts = painted_texts(&h);
        let color_of = |label: &str| {
            texts
                .iter()
                .find(|(text, ..)| text == label)
                .map(|(_, c, _)| *c)
                .unwrap_or_else(|| panic!("{label} not painted"))
        };
        assert_eq!(color_of("Linear"), color::TEXT_ON_PRIMARY);
        assert_eq!(color_of("Solid"), color::TEXT_MUTED);
        // The options sit in one track on the control surface.
        let track = rects_filled(&h, color::CONTROL);
        assert!(
            track.iter().any(|t| t.contains_rect(pills[0])
                && t.contains(h.get_by_label("Solid").rect().center())),
            "{track:?}"
        );
    }

    /// The active option is a fill change: in grayscale the pill stands out
    /// from the track, and its text from the pill.
    #[test]
    fn active_option_in_grayscale() {
        let mut h = harness();
        h.run();
        assert_eq!(rects_filled(&h, color::ACCENT_PRIMARY).len(), 1);
        assert!(luma(color::ACCENT_PRIMARY) - luma(color::CONTROL) > 150.0);
        assert!(luma(color::ACCENT_PRIMARY) - luma(color::TEXT_ON_PRIMARY) > 150.0);
    }

    #[test]
    fn options_meet_the_minimum_hit_area() {
        let mut h = harness();
        h.run();
        for label in ["Solid", "Linear", "Radial"] {
            let rect = h.get_by_label(label).rect();
            assert!(rect.height() >= tp_ui::tokens::size::HIT_MIN, "{label}");
        }
    }
}

mod interface_fonts {
    use egui::FontFamily;
    use tp_ui::widgets::NumericField;

    use super::{painted_texts, themed};

    #[test]
    fn labels_in_geist_and_values_in_jetbrains_mono() {
        let mut h = themed(
            |ui, _: &mut ()| {
                ui.label("Opacity");
                NumericField::new("X", "Position X", Some(512.0)).show(ui);
            },
            (),
        );
        h.run();
        let texts = painted_texts(&h);
        let family_of = |label: &str| {
            texts
                .iter()
                .find(|(text, ..)| text == label)
                .map(|(.., f)| f.clone())
                .unwrap_or_else(|| panic!("{label} not painted: {texts:?}"))
        };
        assert_eq!(family_of("Opacity"), FontFamily::Proportional);
        assert_eq!(family_of("512"), FontFamily::Monospace);

        // The families resolve first to the embedded Geist and JetBrains Mono.
        let first_font = |family: FontFamily| {
            h.ctx.fonts(|fonts| {
                let defs = fonts.definitions();
                let name = defs.families[&family][0].clone();
                defs.font_data[&name].font.to_vec()
            })
        };
        let geist: &[u8] = include_bytes!("../../../assets/fonts/Geist-Regular.ttf");
        let mono: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf");
        assert!(first_font(FontFamily::Proportional) == geist);
        assert!(first_font(FontFamily::Monospace) == mono);
    }
}

mod popovers {
    use egui::{Key, Modifiers, PointerButton, Pos2};
    use egui_kittest::kittest::Queryable;
    use tp_ui::widgets::{MenuRow, Popover};

    use super::themed;

    #[derive(Default)]
    struct State {
        /// Draw the rows (and their popovers).
        hide_rows: bool,
        text: String,
        menu_item: bool,
        inside_clicks: u32,
    }

    /// Two rows, "Fill" and "Stroke", each opening its popover; the Fill
    /// popover holds a button, a text field and a context menu.
    fn harness() -> egui_kittest::Harness<'static, State> {
        themed(
            |ui, state: &mut State| {
                ui.add_space(40.0);
                if state.hide_rows {
                    return;
                }
                for name in ["Fill", "Stroke"] {
                    let row = ui.button(name);
                    let id = Popover::id(name);
                    if row.clicked() {
                        Popover::toggle(ui.ctx(), id);
                    }
                    Popover::new(id, row.rect).show(ui, |ui| {
                        ui.label(format!("{name} settings"));
                        if name == "Fill" {
                            if ui.button("Inside").clicked() {
                                state.inside_clicks += 1;
                            }
                            ui.text_edit_singleline(&mut state.text);
                            ui.button("More").context_menu(|ui| {
                                if ui.add(MenuRow::new("Menu item")).clicked() {
                                    state.menu_item = true;
                                    ui.close();
                                }
                            });
                        }
                    });
                }
            },
            State::default(),
        )
    }

    fn open(h: &egui_kittest::Harness<'static, State>, name: &str) -> bool {
        h.query_by_label(&format!("{name} settings")).is_some()
    }

    fn press_escape(h: &mut egui_kittest::Harness<'static, State>) {
        h.key_press(Key::Escape);
        h.run();
    }

    /// With room on the left of its row, the popover opens there, its top
    /// on the row's.
    #[test]
    fn opens_beside_its_row_when_there_is_room() {
        let mut h = themed(
            |ui, _: &mut ()| {
                ui.add_space(40.0);
                ui.horizontal(|ui| {
                    ui.add_space(400.0);
                    let row = ui.button("Fill");
                    let id = Popover::id("beside");
                    if row.clicked() {
                        Popover::toggle(ui.ctx(), id);
                    }
                    Popover::new(id, row.rect).show(ui, |ui| ui.label("Fill settings"));
                });
            },
            (),
        );
        h.run();
        h.get_by_label("Fill").click();
        h.run();
        let row = h.get_by_label("Fill").rect();
        let label = h.get_by_label("Fill settings").rect();
        assert!(label.right() <= row.left(), "{row:?} {label:?}");
        assert!(label.top() >= row.top() - 1.0, "{row:?} {label:?}");
    }

    /// Without room on the left (the row at the window's edge), it opens
    /// under its row.
    #[test]
    fn opens_under_its_row_and_stays_open_for_its_own_widgets() {
        let mut h = harness();
        h.run();
        assert!(!open(&h, "Fill"));
        h.get_by_label("Fill").click();
        h.run();
        assert!(open(&h, "Fill"));
        let row = h.get_by_label("Fill").rect();
        let label = h.get_by_label("Fill settings").rect();
        assert!(label.top() >= row.bottom(), "{row:?} {label:?}");

        // Its own widgets work without closing it.
        h.get_by_label("Inside").click();
        h.run();
        assert_eq!(h.state().inside_clicks, 1);
        assert!(open(&h, "Fill"));

        // Clicking its row again closes it.
        h.get_by_label("Fill").click();
        h.run();
        assert!(!open(&h, "Fill"));
    }

    #[test]
    fn escape_closes_it_after_a_field_lets_go() {
        let mut h = harness();
        h.run();
        h.get_by_label("Fill").click();
        h.run();
        press_escape(&mut h);
        assert!(!open(&h, "Fill"));

        h.get_by_label("Fill").click();
        h.run();
        h.get_by_role(egui::accesskit::Role::TextInput).click();
        h.run();
        assert!(h.ctx.text_edit_focused());
        // The first Escape leaves the field, the second closes the popover.
        press_escape(&mut h);
        assert!(open(&h, "Fill"));
        press_escape(&mut h);
        assert!(!open(&h, "Fill"));
    }

    #[test]
    fn a_press_outside_closes_it() {
        let mut h = harness();
        h.run();
        h.get_by_label("Fill").click();
        h.run();
        let far = Pos2::new(5.0, 5.0);
        h.hover_at(far);
        h.event(egui::Event::PointerButton {
            pos: far,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.event(egui::Event::PointerButton {
            pos: far,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run();
        assert!(!open(&h, "Fill"));
    }

    #[test]
    fn one_at_a_time() {
        let mut h = harness();
        h.run();
        h.get_by_label("Fill").click();
        h.run();
        // Escape-free switch: opening the other closes the first. The Fill
        // popover covers the Stroke row, so open it as the row would.
        Popover::open(&h.ctx, Popover::id("Stroke"));
        h.run();
        assert!(open(&h, "Stroke"));
        assert!(!open(&h, "Fill"));
        assert_eq!(Popover::open_id(&h.ctx), Some(Popover::id("Stroke")));
    }

    #[test]
    fn a_menu_opened_inside_does_not_close_it() {
        let mut h = harness();
        h.run();
        h.get_by_label("Fill").click();
        h.run();
        h.get_by_label("More").click_secondary();
        h.run();
        h.get_by_label("Menu item").click();
        h.run();
        assert!(h.state().menu_item);
        assert!(open(&h, "Fill"));
    }

    #[test]
    fn it_closes_when_its_row_goes_away() {
        let mut h = harness();
        h.run();
        h.get_by_label("Fill").click();
        h.run();
        h.state_mut().hide_rows = true;
        h.run();
        h.state_mut().hide_rows = false;
        h.run();
        assert!(!open(&h, "Fill"));
        assert_eq!(Popover::open_id(&h.ctx), None);
    }

    #[test]
    fn close_on_escape_takes_the_key() {
        let mut h = harness();
        h.run();
        assert!(!Popover::close_on_escape(&h.ctx));
        h.get_by_label("Fill").click();
        h.run();
        assert!(Popover::is_open(&h.ctx, Popover::id("Fill")));
        Popover::close(&h.ctx);
        assert_eq!(Popover::open_id(&h.ctx), None);
    }
}

mod truncated_labels {
    use egui_kittest::kittest::Queryable;
    use tp_ui::widgets::truncated_label;

    use super::{painted_shapes, themed};

    const LONG: &str = "Grey stripe along the lower edge of the cabin";

    #[test]
    fn a_long_text_ends_with_an_ellipsis_and_shows_whole_in_a_tooltip() {
        let mut h = themed(
            |ui, _: &mut ()| {
                truncated_label(ui, LONG, 120.0);
            },
            (),
        );
        h.run();
        // Named by its whole text, laid out within the width given.
        let rect = h.get_by_label(LONG).rect();
        assert!(rect.width() <= 120.0, "{rect:?}");
        // One line, ending with "…".
        let galley = painted_shapes(&h)
            .into_iter()
            .find_map(|shape| match shape {
                egui::Shape::Text(t) if t.galley.text() == LONG => Some(t.galley),
                _ => None,
            })
            .expect("label painted");
        assert!(galley.elided);
        assert_eq!(galley.rows.len(), 1);
        let shown: String = galley.rows[0].glyphs.iter().map(|g| g.chr).collect();
        assert!(shown.ends_with('…') && shown.len() < LONG.len(), "{shown}");

        h.get_by_label(LONG).hover();
        for _ in 0..60 {
            h.step();
        }
        h.run();
        assert_eq!(h.get_all_by_label(LONG).count(), 2, "label and tooltip");
    }

    #[test]
    fn a_short_text_is_shown_whole() {
        let mut h = themed(
            |ui, _: &mut ()| {
                truncated_label(ui, "Logo", 120.0);
            },
            (),
        );
        h.run();
        let galley = painted_shapes(&h)
            .into_iter()
            .find_map(|shape| match shape {
                egui::Shape::Text(t) if t.galley.text() == "Logo" => Some(t.galley),
                _ => None,
            })
            .expect("label painted");
        assert!(!galley.elided);
    }
}

mod checkboxes {
    use egui::accesskit::{Role, Toggled};
    use egui_kittest::kittest::{NodeT, Queryable};
    use tp_ui::tokens::color;
    use tp_ui::widgets::SquareCheckbox;

    use super::{painted_shapes, themed};

    #[test]
    fn checked_and_unchecked_square_boxes() {
        let mut h = themed(
            |ui, on: &mut bool| {
                ui.add(SquareCheckbox::new(on, "Transparent"));
            },
            false,
        );
        h.run();
        let node = h.get_by_role_and_label(Role::CheckBox, "Transparent");
        assert_eq!(node.accesskit_node().toggled(), Some(Toggled::False));
        // Unchecked: an outlined square with small rounded corners, no fill.
        let square = |h: &egui_kittest::Harness<'static, bool>| {
            painted_shapes(h).into_iter().find_map(|shape| match shape {
                egui::Shape::Rect(r) if (r.rect.width() - r.rect.height()).abs() < 0.01 => Some(r),
                _ => None,
            })
        };
        let empty = square(&h).expect("box painted");
        assert_eq!(empty.fill, egui::Color32::TRANSPARENT);
        assert!(f32::from(empty.corner_radius.nw) < empty.rect.width() / 2.0);

        h.get_by_label("Transparent").click();
        h.run();
        assert!(*h.state());
        assert_eq!(
            h.get_by_role_and_label(Role::CheckBox, "Transparent")
                .accesskit_node()
                .toggled(),
            Some(Toggled::True)
        );
        // Checked: the square filled, with a check mark on it.
        let full = square(&h).expect("box painted");
        assert_eq!(full.fill, color::ACCENT_PRIMARY);
        assert!(f32::from(full.corner_radius.nw) < full.rect.width() / 2.0);
        assert!(painted_shapes(&h).iter().any(|s| matches!(s,
            egui::Shape::Text(t) if t.galley.text() == tp_ui::icons::CHECK
                && full.rect.contains(t.visual_bounding_rect().center()))));
    }

    #[test]
    fn mixed_checkbox_is_a_filled_square_with_a_dash() {
        let mut h = themed(
            |ui, on: &mut bool| {
                ui.add(SquareCheckbox::new(on, "Stroke").indeterminate(true));
            },
            false,
        );
        h.run();
        let shapes = painted_shapes(&h);
        let full = shapes
            .iter()
            .find_map(|shape| match shape {
                egui::Shape::Rect(r) if (r.rect.width() - r.rect.height()).abs() < 0.01 => Some(r),
                _ => None,
            })
            .expect("box painted");
        assert_eq!(full.fill, color::ACCENT_PRIMARY);
        assert!(shapes.iter().any(|s| matches!(s,
            egui::Shape::LineSegment { points, .. } if full.rect.contains(points[0])
                && full.rect.contains(points[1]))));
        h.get_by_label("Stroke").click();
        h.run();
        assert!(*h.state(), "a click toggles it");
    }

    #[test]
    fn disabled_checkbox_ignores_clicks() {
        let mut h = themed(
            |ui, on: &mut bool| {
                ui.add_enabled(false, SquareCheckbox::new(on, "In the project"));
            },
            true,
        );
        h.run();
        h.get_by_label("In the project").click();
        h.run();
        assert!(*h.state());
        assert!(
            h.get_by_label("In the project")
                .accesskit_node()
                .is_disabled()
        );
    }
}

#[test]
fn checkbox_with_a_long_label_stays_in_its_container() {
    let mut h = themed(
        |ui, width: &mut f32| {
            ui.set_max_width(200.0);
            let mut on = true;
            let long = "chs_f_l.side02p, chs_f_l.side02sp, chs_f_l.side02tp, chs_f_l.side02tsp";
            *width = ui
                .add(tp_ui::widgets::SquareCheckbox::new(&mut on, long))
                .rect
                .width();
        },
        0.0,
    );
    h.run();
    assert!(*h.state() <= 200.0, "{}", h.state());
}

#[test]
fn chips_wrap_in_a_narrow_row_even_when_one_is_disabled() {
    let mut h = themed(
        |ui, bottom: &mut f32| {
            ui.set_max_width(200.0);
            ui.horizontal_wrapped(|ui| {
                for (i, name) in [
                    "High Roof (highline)",
                    "High Roof (highline_8x4)",
                    "Normal Roof",
                ]
                .into_iter()
                .enumerate()
                {
                    let r = tp_ui::widgets::chip_toggle(ui, i == 0, i != 0, name);
                    assert!(r.rect.right() <= 200.5, "{name} sticks out: {:?}", r.rect);
                    *bottom = bottom.max(r.rect.bottom());
                }
            });
        },
        0.0,
    );
    h.run();
    assert!(*h.state() > 60.0, "the chips did not wrap: {}", h.state());
}
