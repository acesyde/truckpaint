//! Renders key screens to PNG for visual QA.
//!
//! Ignored by default (needs a GPU). Run with:
//! `cargo test -p tp-app --test screenshots -- --ignored`
//! Images are written to `target/screenshots/`.

mod common;

use std::path::PathBuf;

use common::scenes::*;
use egui::Vec2;
use egui_kittest::kittest::Queryable;
use tp_app::layout::LeftTab;
use tp_app::prefs::{Prefs, RecentProject};

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/screenshots");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn save(harness: &mut egui_kittest::Harness<'static, tp_app::AppState>, name: &str) {
    harness.run();
    wait_for_thumbnails(harness);
    let image = harness.render().expect("render");
    image.save(out_dir().join(format!("{name}.png"))).unwrap();
}

/// The screens of the "TruckPaint 5" mockup: home, New Project, the three
/// spaces (the Workshop with nothing selected, with a text selected, and
/// its left tabs), the Export Mod dialog and the Vehicle Library, in English
/// and German (the longest labels) at 100 % and 200 %, as on macOS (the
/// menus in the system menu bar, the top bar in the title strip). Files are
/// named `screen_<language>_<screen>_<scale>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_screens() {
    use tp_app::layout::Space;
    use tp_app::state::Modal;
    use tp_core::document::{CharStyle, Rgba};
    use tp_i18n::Language;
    // A recent project with artwork, for its card's thumbnail.
    let dir = tempfile::tempdir().unwrap();
    let ace = dir.path().join("ace.truckpaint");
    {
        let mut h = common::harness();
        common::create_project(&mut h);
        demo_scene(&mut h);
        tp_file::write(&h.state().workspace().unwrap().project, &ace).unwrap();
    }
    for language in [Language::English, Language::German] {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let code = language.code();
            let name = |screen: &str| format!("screen_{code}_{screen}_{suffix}");
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            prefs.recent = vec![
                RecentProject {
                    name: "ACE Logistics".into(),
                    path: ace.clone(),
                    last_opened: tp_app::prefs::now_unix() - 7_200,
                },
                RecentProject {
                    name: "Old fleet".into(),
                    path: "/missing/old-fleet.truckpaint".into(),
                    last_opened: 0,
                },
            ];
            let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
            let mut h = common::wgpu_harness_with(prefs, size);
            mac_native(&mut h);
            save(&mut h, &name("home"));

            h.get_by_label(&tp_i18n::tr("home-new-project")).click();
            h.run();
            h.get_by_role_and_label(egui::accesskit::Role::CheckBox, common::SAMPLE)
                .click();
            h.run();
            save(&mut h, &name("new_project"));

            h.get_by_label(&tp_i18n::tr("new-project-create")).click();
            // New Project opens the Project space; then the Workshop.
            save(&mut h, &name("project"));
            common::show_space(&mut h, Space::Workshop);
            let ids = demo_scene(&mut h);
            let text = {
                let ws = h.state_mut().workspace_mut().unwrap();
                let style = CharStyle {
                    size: 260.0,
                    ..CharStyle::default()
                };
                let fill = Rgba::rgb(0x1B, 0x23, 0x40);
                add_text(ws, "ACE Logistics", style, (300.0, 3950.0), fill, None)
            };
            save(&mut h, &name("workshop"));
            h.state_mut().workspace_mut().unwrap().selection = vec![text];
            save(&mut h, &name("workshop_text"));
            h.state_mut().workspace_mut().unwrap().selection.clear();
            h.state_mut().prefs.layout.left_tab = LeftTab::Layers;
            save(&mut h, &name("workshop_layers"));
            h.state_mut().prefs.layout.left_tab = LeftTab::Resources;
            save(&mut h, &name("workshop_resources"));
            // The Polygon tool's options bar.
            h.state_mut().workspace_mut().unwrap().tool = tp_app::tool::Tool::Polygon;
            save(&mut h, &name("workshop_polygon"));
            h.state_mut().workspace_mut().unwrap().tool = tp_app::tool::Tool::Select;
            h.state_mut().prefs.layout.left_tab = LeftTab::Textures;
            demo_brand(&mut h, &ids);
            // The Resources tab with content, and the Layers tab's linked
            // rows (objects following a style, a symbol instance).
            h.state_mut().prefs.layout.left_tab = LeftTab::Resources;
            save(&mut h, &name("workshop_resources_content"));
            h.state_mut().prefs.layout.left_tab = LeftTab::Layers;
            save(&mut h, &name("workshop_layers_linked"));
            h.state_mut().prefs.layout.left_tab = LeftTab::Textures;
            common::show_space(&mut h, Space::Brand);
            save(&mut h, &name("brand"));

            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::E);
            settle_dialog(&mut h);
            save(&mut h, &name("export_mod"));
            h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
            save(&mut h, &name("vehicle_library"));
            h.state_mut().modal = None;

            // The size pill while a selection is resized (mid-drag on the
            // band's bottom right handle).
            common::show_space(&mut h, Space::Workshop);
            h.state_mut().workspace_mut().unwrap().selection = vec![ids[0]];
            h.run();
            resize_mid_drag(&mut h, (3948.0, 1350.0), Vec2::new(-60.0, 40.0));
            let image = h.render().expect("render");
            image
                .save(out_dir().join(format!("{}.png", name("workshop_resize"))))
                .unwrap();
            h.event(egui::Event::PointerButton {
                pos: h
                    .state()
                    .workspace()
                    .unwrap()
                    .screen_map(1.0)
                    .unwrap()
                    .to_screen(tp_core::kurbo::Point::new(3948.0, 1350.0)),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            });
            h.run();

            // An update installed for the project's vehicle: its card and
            // the Vehicle Library say so.
            h.state_mut().workspace_mut().unwrap().project.vehicles[0].version = "1.0.0".into();
            common::show_space(&mut h, Space::Project);
            save(&mut h, &name("project_update"));
            h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
            save(&mut h, &name("vehicle_library_update"));
        }
    }

    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    h.get_by_label("Edit").click();
    h.run();
    save(&mut h, "menu_edit");
}

/// The window as on macOS once the menus are in the system menu bar: no
/// menu row, the top bar in the title strip after the traffic lights.
fn mac_native(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) {
    h.state_mut().macos = true;
    h.state_mut().native_menu_installed = true;
}

/// The top of the window on Windows and Linux: the drawn title bar (home
/// screen, Project space, the Workshop with the Object menu open, as
/// artboard 05) and the system title bar with the menu row, in English and
/// German at 100 % and 200 %; then, at 960 px wide in German, the drawn bar
/// and the macOS top bar. Files are named
/// `title_bar_<language>_<mode>_<screen>_<scale>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_title_bars() {
    use tp_app::layout::Space;
    use tp_i18n::Language;
    for language in [Language::English, Language::German] {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let code = language.code();
            let name = |screen: &str| format!("title_bar_{code}_{screen}_{suffix}");
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
            for system in [false, true] {
                let mode = if system { "system" } else { "drawn" };
                let mut h = common::wgpu_harness_with(
                    Prefs {
                        system_title_bar: system,
                        ..prefs.clone()
                    },
                    size,
                );
                save(&mut h, &name(&format!("{mode}_home")));
                title_bar_project(&mut h);
                common::show_space(&mut h, Space::Project);
                save(&mut h, &name(&format!("{mode}_project")));
                common::show_space(&mut h, Space::Workshop);
                save(&mut h, &name(&format!("{mode}_workshop")));
                open_menu(&mut h, "menu-object");
                save(&mut h, &name(&format!("{mode}_workshop_object")));
            }
        }
    }

    // German at 960 px: every label whole.
    let mut prefs = Prefs::default();
    prefs.set_language(Some(Language::German));
    for mac in [false, true] {
        let mode = if mac { "mac" } else { "drawn" };
        let mut h = common::wgpu_harness_with(prefs.clone(), Vec2::new(960.0, 700.0));
        if mac {
            mac_native(&mut h);
        }
        title_bar_project(&mut h);
        for space in [Space::Project, Space::Workshop, Space::Brand] {
            common::show_space(&mut h, space);
            let screen = format!("{space:?}").to_lowercase();
            save(&mut h, &format!("title_bar_de_{mode}_{screen}_960"));
        }
    }
}

/// The long-text check of the top of the window: every interface text 40 %
/// longer, at the default window size, in the drawn title bar (home screen,
/// Workshop, the Object menu open), the macOS top bar and the system title
/// bar's menu row.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_title_bars_long_text() {
    tp_i18n::set_lengthening(40);
    for mode in ["drawn", "mac", "system"] {
        let prefs = Prefs {
            system_title_bar: mode == "system",
            ..Prefs::default()
        };
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        if mode == "mac" {
            mac_native(&mut h);
        }
        save(&mut h, &format!("title_bar_long_{mode}_home"));
        title_bar_project(&mut h);
        save(&mut h, &format!("title_bar_long_{mode}_workshop"));
        if mode != "mac" {
            open_menu(&mut h, "menu-object");
            save(&mut h, &format!("title_bar_long_{mode}_workshop_object"));
        }
    }
    tp_i18n::set_lengthening(0);
}

/// Opens the sample project, named "ACE Logistics", with the demo scene,
/// in the Workshop.
fn title_bar_project(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) {
    common::create_project(h);
    h.state_mut().workspace_mut().unwrap().project.name = "ACE Logistics".into();
    demo_scene(h);
    h.run();
}

/// Presses at document point `at` (a selection handle) and drags by `by`
/// points without releasing.
fn resize_mid_drag(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
    at: (f64, f64),
    by: Vec2,
) {
    let from = h
        .state()
        .workspace()
        .unwrap()
        .screen_map(1.0)
        .unwrap()
        .to_screen(tp_core::kurbo::Point::new(at.0, at.1));
    h.event(egui::Event::PointerMoved(from));
    h.event(egui::Event::PointerButton {
        pos: from,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    for i in 1..=8 {
        h.event(egui::Event::PointerMoved(from + by * (i as f32 / 8.0)));
        h.step();
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_gallery_in_color_and_grayscale() {
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(900.0, 900.0));
    h.run();
    h.state_mut().show_gallery = true;
    h.run();
    let mut image = h.render().expect("render");
    image.save(out_dir().join("gallery.png")).unwrap();
    for pixel in image.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        let luma = (0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b)) as u8;
        pixel.0 = [luma, luma, luma, a];
    }
    image.save(out_dir().join("gallery_grayscale.png")).unwrap();
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_canvas_scene() {
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let ids = demo_scene(&mut h);
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[1]];
        save(&mut h, &format!("canvas_single_{suffix}"));
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[2], ids[3]];
        save(&mut h, &format!("canvas_multi_{suffix}"));
    }
    // Zoomed in: overlay keeps its size.
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    let ids = demo_scene(&mut h);
    {
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.selection = vec![ids[1]];
        let view = ws.viewport.as_mut().unwrap();
        view.center = tp_core::kurbo::Point::new(1300.0, 2300.0);
        view.set_zoom_centered(0.25);
    }
    save(&mut h, "canvas_zoom_25");
    {
        let view = h
            .state_mut()
            .workspace_mut()
            .unwrap()
            .viewport
            .as_mut()
            .unwrap();
        view.set_zoom_centered(4.0);
        view.center = tp_core::kurbo::Point::new(540.0, 2300.0);
    }
    save(&mut h, "canvas_zoom_400");
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_editing_panels() {
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        prefs.layout.inspector_width = 300.0;
        prefs.recent_colors = vec![
            [0x7A, 0x1F, 0x2B, 255],
            [255, 255, 255, 255],
            [0xF0, 0xB4, 0x4C, 255],
        ];
        let size = Vec2::new(1440.0, 1000.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs.clone(), size);
        h.state_mut().prefs.layout.left_tab = LeftTab::Layers;
        common::create_project(&mut h);
        let ids = livery_tree(&mut h);
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[3]];
        save(&mut h, &format!("panels_layers_transform_{suffix}"));

        // The color popover, then the stroke popover.
        common::open_color_popover(&mut h, tp_app::workspace::ColorTarget::Fill);
        save(&mut h, &format!("panels_colors_stroke_{suffix}"));
        common::open_stroke_popover(&mut h);
        save(&mut h, &format!("panels_stroke_popover_{suffix}"));
    }
}

/// The inspector as in the "TruckPaint 5" artboards 04 and 05: nothing
/// selected on the Side skirts, then a text selected, then its color
/// popover open.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_inspector() {
    use tp_core::document::{CharStyle, Rgba, StrokeAlign, StrokeStyle};
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        prefs.recent_colors = vec![
            [0x5B, 0x8D, 0xEF, 255],
            [0xD9, 0x36, 0x36, 255],
            [0x1B, 0x23, 0x40, 255],
        ];
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let text = {
            let ws = h.state_mut().workspace_mut().unwrap();
            let side_skirts = ws
                .project
                .surfaces
                .iter()
                .position(|s| s.name == "Side skirts")
                .unwrap();
            ws.set_active_surface(side_skirts);
            for (name, hex) in [
                ("Ardent green", (0x1F, 0x4D, 0x3A)),
                ("Cream", (0xEF, 0xE6, 0xCF)),
                ("Signal red", (0xC2, 0x3B, 0x2A)),
            ] {
                let (id, _) = ws
                    .project
                    .add_swatch(Rgba::rgb(hex.0, hex.1, hex.2), "Color");
                ws.project.rename_swatch(id, name);
            }
            let style = CharStyle {
                size: 180.0,
                ..CharStyle::default()
            };
            add_text(
                ws,
                "test",
                style,
                (200.0, 650.0),
                Rgba::rgb(0x5B, 0x8D, 0xEF),
                Some(StrokeStyle {
                    paint: Rgba::rgb(0xD9, 0x36, 0x36).into(),
                    width: 6.0,
                    align: StrokeAlign::Outside,
                    ..Default::default()
                }),
            )
        };
        save(&mut h, &format!("inspector_nothing_selected_{suffix}"));
        h.state_mut().workspace_mut().unwrap().selection = vec![text];
        save(&mut h, &format!("inspector_text_{suffix}"));
        common::open_color_popover(&mut h, tp_app::workspace::ColorTarget::Fill);
        save(&mut h, &format!("inspector_color_popover_{suffix}"));
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_text_and_images() {
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        prefs.layout.inspector_width = 300.0;
        let size = Vec2::new(1440.0, 1000.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        h.state_mut().prefs.layout.left_tab = LeftTab::Layers;
        common::create_project(&mut h);
        let ids = lettering_scene(&mut h);
        wait_for_images(&mut h);
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[0]];
        save(&mut h, &format!("text_livery_{suffix}"));

        // Editing on the canvas, with part of the text selected.
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.start_editing(ids[0], None, 0.0);
            ws.text_move(tp_text::Motion::WordLeft, true, 0.0);
        }
        save(&mut h, &format!("text_editing_{suffix}"));
        h.state_mut().workspace_mut().unwrap().end_text_session(0.0);

        // Font picker.
        h.get_by_label_contains("Font family").click();
        h.run();
        save(&mut h, &format!("font_picker_{suffix}"));
        h.key_press(egui::Key::Escape);
        h.run();

        // Image selected: image information, no fill/stroke.
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[5]];
        save(&mut h, &format!("image_selected_{suffix}"));
    }
    // Zoomed out and in on the images.
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    lettering_scene(&mut h);
    for (zoom, center, name) in [
        (0.25, (2048.0, 2048.0), "images_zoom_25"),
        (8.0, (1700.0, 2900.0), "images_zoom_800_svg"),
        (8.0, (3000.0, 1300.0), "images_zoom_800_png"),
        (8.0, (700.0, 1450.0), "text_zoom_800"),
    ] {
        {
            let view = h
                .state_mut()
                .workspace_mut()
                .unwrap()
                .viewport
                .as_mut()
                .unwrap();
            view.center = tp_core::kurbo::Point::new(center.0, center.1);
            view.set_zoom_centered(zoom);
        }
        h.step();
        wait_for_images(&mut h);
        save(&mut h, name);
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_persistence() {
    use tp_app::recovery::{RecoveryMeta, RecoveryStore};
    use tp_app::state::{Modal, PendingAction};
    let dir = tempfile::tempdir().unwrap();
    let recovery = dir.path().join("recovery");
    {
        // A copy left by a crashed session.
        let store = RecoveryStore::open(&recovery).unwrap();
        tp_file::write(
            &tp_core::Project::new("ACE Logistics", tp_core::TextureResolution::R4096),
            &store.copy_path(),
        )
        .unwrap();
        let (path, text) = store.meta_file(&RecoveryMeta {
            name: "ACE Logistics".into(),
            original: Some("/home/user/Liveries/ace.truckpaint".into()),
            saved_at: tp_app::prefs::now_unix() - 300,
        });
        std::fs::write(path, text).unwrap();
    }
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        prefs.recent = vec![RecentProject {
            name: "Scania fleet".into(),
            path: "/home/user/Liveries/scania.truckpaint".into(),
            last_opened: tp_app::prefs::now_unix() - 86_400 * 3,
        }];
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        h.state_mut().enable_recovery(&recovery);
        save(&mut h, &format!("home_recovered_{suffix}"));

        common::create_project(&mut h);
        h.state_mut().modal = Some(Modal::UnsavedChanges(PendingAction::CloseProject));
        save(&mut h, &format!("unsaved_prompt_{suffix}"));
        h.state_mut().modal = Some(Modal::Message {
            title: "Cannot open project".into(),
            text: "ace.truckpaint was created with a newer version of TruckPaint.".into(),
        });
        save(&mut h, &format!("message_{suffix}"));
        h.state_mut().modal = None;
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.saving = Some(tp_app::workspace::PendingSave {
            id: 0,
            snapshot: ws.snapshot(),
            path: "/tmp/a.truckpaint".into(),
        });
        save(&mut h, &format!("status_saving_{suffix}"));
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_export() {
    use tp_app::state::Modal;
    use tp_app::ui::export_dialog::ExportDialog;
    let wait_preview = |h: &mut egui_kittest::Harness<'static, tp_app::AppState>| {
        for _ in 0..400 {
            h.step();
            let ready = matches!(&h.state().modal, Some(Modal::Export(d)) if d.preview_ready());
            if ready {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        h.step();
    };
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        lettering_scene(&mut h);
        wait_for_images(&mut h);
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::E,
        );
        wait_preview(&mut h);
        save(&mut h, &format!("export_png_{suffix}"));
        if let Some(Modal::Export(d)) = &mut h.state_mut().modal {
            d.settings.format = tp_app::export::ExportFormat::Dds;
            d.settings.background = None;
        }
        wait_preview(&mut h);
        save(&mut h, &format!("export_dds_transparent_{suffix}"));
        let _ = ExportDialog::preview_ready;
    }
    // The exported texture itself, to compare with the canvas.
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    lettering_scene(&mut h);
    let ws = h.state_mut().workspace_mut().unwrap();
    let pixmap = tp_render::render(
        &ws.project,
        0,
        tp_render::RenderOptions {
            width: 4096,
            height: 4096,
            background: Some(tp_core::document::Rgba::rgb(255, 255, 255)),
        },
        &mut ws.text.fonts,
        &mut |_, _| true,
    )
    .unwrap();
    std::fs::write(
        out_dir().join("export_4k.png"),
        tp_render::encode_png(&pixmap).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_mod_export() {
    use egui::accesskit::Role;
    use tp_app::layout::Space;
    use tp_i18n::Language;
    for language in [Language::English, Language::German] {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let code = language.code();
            let name = |screen: &str| format!("mod_export_{code}_{screen}_{suffix}");
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            let dir = tempfile::tempdir().unwrap();
            let documents = dir.path().join("Documents");
            std::fs::create_dir_all(&documents).unwrap();
            let folders = tp_app::mod_export::Folders {
                documents: Some(documents.clone()),
                data: Some(dir.path().join("data")),
                home: Some(dir.path().to_path_buf()),
            };

            // The Export Mod dialog: summary, destination, buttons.
            let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
            let mut h = common::wgpu_harness_with(prefs.clone(), size);
            h.state_mut().mod_folders = folders.clone();
            common::create_project(&mut h);
            lettering_scene(&mut h);
            wait_for_images(&mut h);
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::E);
            settle_dialog(&mut h);
            save(&mut h, &name("dialog"));
            // An existing file: the replace question.
            let file = documents.join(tp_app::mod_export::suggested_name(
                &h.state().workspace().unwrap().project.mod_settings.name,
            ));
            std::fs::write(&file, b"earlier export").unwrap();
            common::last(&h, &tp_i18n::tr("mod-export-start")).click();
            settle_dialog(&mut h);
            save(&mut h, &name("replace"));
            h.state_mut().modal = None;
            // With problems listed, each with Show.
            {
                let ws = h.state_mut().workspace_mut().unwrap();
                ws.project.mod_settings.name.clear();
                ws.project.mod_settings.price = 0;
                ws.project.mod_settings.internal_name = Some("ace_logistic".into());
                ws.project.game_versions = vec!["1.56.x".into(), "1.57.*".into()];
            }
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::E);
            settle_dialog(&mut h);
            save(&mut h, &name("problems"));
            h.state_mut().modal = None;

            // The Project space's editable column, tall enough to show it
            // whole: chips with a badly written version, problems under
            // their fields and in Before exporting, Advanced opened by the
            // internal name's.
            let tall = Vec2::new(1440.0, 1760.0) * scale.max(1.0);
            let mut h = common::wgpu_harness_with(prefs, tall);
            h.state_mut().mod_folders = folders;
            common::create_project(&mut h);
            {
                let ws = h.state_mut().workspace_mut().unwrap();
                let s = &mut ws.project.mod_settings;
                s.author = "Jane Doe".into();
                s.description = "Fleet colors\nTruck and trailer".into();
                s.price = 0;
                s.internal_name = Some("ace_logistic".into());
                ws.project.game_versions = vec!["1.56.*".into(), "1.56.x".into()];
            }
            common::show_space(&mut h, Space::Project);
            common::settle_renders(&mut h);
            save(&mut h, &format!("project_mod_{code}_column_{suffix}"));
            // + Add opened.
            h.get_by_role_and_label(Role::Button, &tp_i18n::tr("project-game-versions-add"))
                .click();
            h.run();
            save(&mut h, &format!("project_mod_{code}_add_{suffix}"));
            // Files dragged over the window, the pointer on the shop icon.
            h.key_press(egui::Key::Escape);
            h.run();
            let icon = h
                .get_by_role_and_label(Role::Image, &tp_i18n::tr("mod-icon"))
                .rect()
                .center();
            h.event(egui::Event::PointerMoved(icon));
            h.input_mut().hovered_files = vec![egui::HoveredFile {
                path: Some("/tmp/logo.png".into()),
                mime: String::new(),
            }];
            h.step();
            h.step();
            let image = h.render().expect("render");
            image
                .save(out_dir().join(format!("project_mod_{code}_drop_{suffix}.png")))
                .unwrap();
            h.input_mut().hovered_files.clear();
        }
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_vector_tools() {
    use tp_app::tool::Tool;
    use tp_core::document::{Node, NodeRef, PointRef};
    use tp_core::kurbo::Point;
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let ids = vector_scene(&mut h);
        save(&mut h, &format!("vector_canvas_{suffix}"));

        // Direct Selection on the swoosh, its second point selected.
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.tool = Tool::DirectSelect;
            ws.selection = vec![ids[3]];
            ws.points = [PointRef::new(ids[3], NodeRef::new(0, 1))].into();
        }
        save(&mut h, &format!("vector_direct_select_{suffix}"));

        // A pen path being drawn, the next segment following the pointer.
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.points.clear();
            ws.selection.clear();
            ws.tool = Tool::Pen;
            ws.pen = Some(tp_app::path_edit::PenSession {
                nodes: vec![
                    Node::corner(Point::new(600.0, 1500.0)),
                    Node::smooth(Point::new(1400.0, 1300.0), Point::new(1800.0, 1300.0)),
                ],
            });
        }
        let pointer = h
            .state()
            .workspace()
            .unwrap()
            .screen_map(1.0)
            .unwrap()
            .to_screen(Point::new(2400.0, 1700.0));
        h.event(egui::Event::PointerMoved(pointer));
        save(&mut h, &format!("vector_pen_{suffix}"));
    }
    // The exported texture, to compare with the canvas.
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    vector_scene(&mut h);
    let ws = h.state_mut().workspace_mut().unwrap();
    let pixmap = tp_render::render(
        &ws.project,
        0,
        tp_render::RenderOptions {
            width: 1024,
            height: 1024,
            background: None,
        },
        &mut ws.text.fonts,
        &mut |_, _| true,
    )
    .unwrap();
    std::fs::write(
        out_dir().join("vector_export_1k.png"),
        tp_render::encode_png(&pixmap).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_precision_aids() {
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    use tp_core::{Axis, Guide};
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        prefs.view_aids.grid = true;
        prefs.view_aids.grid_spacing = 128.0;
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let moving = {
            let ws = h.state_mut().workspace_mut().unwrap();
            let mut a = Object::new(
                ObjectId(0),
                ShapeKind::rectangle(),
                Frame::new(Point::new(900.0, 1200.0), Size::new(800.0, 400.0), 0.0),
            );
            a.fill = Rgba::rgb(0xC0, 0x39, 0x2B).into();
            ws.project.add(a);
            let mut b = Object::new(
                ObjectId(0),
                ShapeKind::Ellipse,
                Frame::new(Point::new(2900.0, 2400.0), Size::new(600.0, 600.0), 0.0),
            );
            b.fill = Rgba::rgb(0x2E, 0x86, 0xDE).into();
            let b = ws.project.add(b);
            // A dark livery area, to check the grid reads on dark paint too.
            let mut dark = Object::new(
                ObjectId(0),
                ShapeKind::rectangle(),
                Frame::new(Point::new(2048.0, 3600.0), Size::new(4096.0, 1000.0), 0.0),
            );
            dark.fill = Rgba::rgb(0x1F, 0x24, 0x2B).into();
            ws.project.add(dark);
            ws.project.add_guide(Guide::new(Axis::Horizontal, 3000.0));
            ws.project.add_guide(Guide::new(Axis::Vertical, 2048.0));
            ws.selection = vec![b];
            b
        };
        let _ = moving;
        h.run();
        save(&mut h, &format!("precision_aids_{suffix}"));

        // Mid-drag: the ellipse's top snaps to the rectangle's top edge.
        let map = |h: &egui_kittest::Harness<'static, tp_app::AppState>, x: f64, y: f64| {
            h.state()
                .workspace()
                .unwrap()
                .screen_map(1.0)
                .unwrap()
                .to_screen(Point::new(x, y))
        };
        let (from, to) = (map(&h, 2900.0, 2400.0), map(&h, 2900.0, 1310.0));
        h.event(egui::Event::PointerMoved(from));
        h.event(egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
        h.step();
        for i in 1..=8 {
            h.event(egui::Event::PointerMoved(
                from + (to - from) * (i as f32 / 8.0),
            ));
            h.step();
        }
        let image = h.render().expect("render");
        image
            .save(out_dir().join(format!("precision_snapping_{suffix}.png")))
            .unwrap();
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_align() {
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            let colors = [
                Rgba::rgb(0xC0, 0x39, 0x2B),
                Rgba::rgb(0x2E, 0x86, 0xDE),
                Rgba::rgb(0xF0, 0xB4, 0x4C),
            ];
            let mut ids = Vec::new();
            for (i, (x, y)) in [(700.0, 1200.0), (1700.0, 1500.0), (2800.0, 1000.0)]
                .into_iter()
                .enumerate()
            {
                let mut o = Object::new(
                    ObjectId(0),
                    if i == 1 {
                        ShapeKind::Ellipse
                    } else {
                        ShapeKind::rectangle()
                    },
                    Frame::new(
                        Point::new(x, y),
                        Size::new(500.0, 300.0 + 100.0 * i as f64),
                        0.0,
                    ),
                );
                o.fill = colors[i].into();
                ids.push(ws.project.add(o));
            }
            ws.selection = ids;
            ws.panels.align_to = tp_app::arrange::AlignTo::KeyObject;
        }
        save(&mut h, &format!("align_key_object_{suffix}"));
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_combine() {
    use tp_core::document::{BooleanOp, Frame, Object, ObjectId, Rgba, ShapeKind};
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            let shape = |kind, c: (f64, f64), s: (f64, f64), fill| {
                let mut o = Object::new(ObjectId(0), kind, Frame::new(c.into(), s.into(), 0.0));
                o.fill = tp_core::document::Paint::Solid(fill);
                o
            };
            // A stripe with a round notch (Minus Front).
            let stripe = ws.project.add(shape(
                ShapeKind::rectangle(),
                (1500.0, 1200.0),
                (2600.0, 500.0),
                Rgba::rgb(0x2E, 0x86, 0xDE),
            ));
            let disk = ws.project.add(shape(
                ShapeKind::Ellipse,
                (2800.0, 1200.0),
                (700.0, 700.0),
                Rgba::rgb(0xC0, 0x39, 0x2B),
            ));
            ws.selection = vec![stripe, disk];
            ws.combine_selection(BooleanOp::MinusFront, 0.0);
            // Two overlapping shapes selected, ready to combine.
            let a = ws.project.add(shape(
                ShapeKind::Ellipse,
                (1200.0, 2700.0),
                (900.0, 900.0),
                Rgba::rgb(0xF0, 0xB4, 0x4C),
            ));
            let b = ws.project.add(shape(
                ShapeKind::rectangle(),
                (1800.0, 2700.0),
                (900.0, 700.0),
                Rgba::rgb(0x27, 0xAE, 0x60),
            ));
            ws.selection = vec![a, b];
        }
        save(&mut h, &format!("combine_{suffix}"));
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_stroke_options() {
    use tp_core::document::{
        Cap, CharStyle, Dash, Frame, Join, LineStyle, Node, Object, ObjectId, PathData, Rgba,
        ShapeKind, StrokeAlign, StrokeStyle, Subpath,
    };
    use tp_core::kurbo::Point;
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let (text, line) = {
            let ws = h.state_mut().workspace_mut().unwrap();
            // A burgundy base with an inside border.
            let mut base = Object::new(
                ObjectId(0),
                ShapeKind::rectangle(),
                Frame::new(Point::new(2048.0, 2048.0), (3400.0, 2200.0).into(), 0.0),
            );
            base.fill = Rgba::rgb(0x7A, 0x1F, 0x2B).into();
            base.stroke = Some(StrokeStyle {
                paint: Rgba::rgb(0xF0, 0xB4, 0x4C).into(),
                width: 40.0,
                align: StrokeAlign::Inside,
                line: LineStyle {
                    join: Join::Round,
                    ..LineStyle::default()
                },
                swatch: None,
            });
            ws.project.add(base);
            // Outside-outlined lettering.
            let text = add_text(
                ws,
                "ACE",
                CharStyle {
                    family: "Barlow Condensed".into(),
                    weight: 700,
                    size: 900.0,
                    ..CharStyle::default()
                },
                (900.0, 1300.0),
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
            // A dashed pinstripe and a dotted line under the lettering.
            let mut stripe = |y: f64, dash: Dash, cap: Cap, width: f64| {
                let mut data = PathData::new(vec![Subpath::new(
                    vec![
                        Node::corner(Point::new(700.0, y)),
                        Node::corner(Point::new(3400.0, y)),
                    ],
                    false,
                )]);
                data.line_width = width;
                data.line_style = LineStyle {
                    dash: Some(dash),
                    cap,
                    ..LineStyle::default()
                };
                let mut o = Object::from_path(ObjectId(0), data);
                o.name = "Pinstripe".into();
                o.fill = Rgba::rgb(0xF0, 0xB4, 0x4C).into();
                ws.project.add(o)
            };
            let line = stripe(
                2700.0,
                Dash {
                    dash: 120.0,
                    gap: 60.0,
                },
                Cap::Butt,
                36.0,
            );
            stripe(
                2900.0,
                Dash {
                    dash: 0.0,
                    gap: 60.0,
                },
                Cap::Round,
                30.0,
            );
            ws.selection = vec![text];
            (text, line)
        };
        let _ = text;
        save(&mut h, &format!("stroke_options_text_{suffix}"));
        h.state_mut().workspace_mut().unwrap().selection = vec![line];
        save(&mut h, &format!("stroke_options_line_{suffix}"));
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_gradients() {
    use tp_app::tool::Tool;
    use tp_app::workspace::ColorTarget;
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let (text, line) = gradient_scene(&mut h);
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.selection = vec![text];
            ws.panels.color_target = ColorTarget::Fill;
        }
        save(&mut h, &format!("gradients_text_{suffix}"));
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.selection = vec![line];
            ws.tool = Tool::Gradient;
        }
        save(&mut h, &format!("gradients_tool_{suffix}"));
    }
}

/// The canvas and the export agree on gradient colors (interior pixels).
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn canvas_matches_export_for_gradients() {
    use tp_core::document::Rgba;
    use tp_core::kurbo::Point;
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    gradient_scene(&mut h);
    h.state_mut().workspace_mut().unwrap().selection.clear();
    h.run();
    let canvas = h.render().expect("render");
    let ws = h.state_mut().workspace_mut().unwrap();
    let map = ws.screen_map(1.0).expect("canvas");
    let side = 2048u32;
    let export = tp_render::to_rgba(
        &tp_render::render(
            &ws.project,
            0,
            tp_render::RenderOptions {
                width: side,
                height: side,
                background: Some(Rgba::rgb(255, 255, 255)),
            },
            &mut ws.text.fonts,
            &mut |_, _| true,
        )
        .unwrap(),
    );
    let surface = ws.project.surface().max_side();
    // Inside the cab fade (away from the glow, text and line) and on dashes.
    let samples = [
        Point::new(500.0, 2300.0),
        Point::new(1200.0, 2400.0),
        Point::new(2600.0, 2500.0),
        Point::new(3500.0, 2300.0),
        Point::new(700.0, 2900.0),
        Point::new(2050.0, 2900.0),
    ];
    let mut worst = 0u8;
    for p in samples {
        let s = map.to_screen(p);
        let c = canvas.get_pixel(s.x.round() as u32, s.y.round() as u32).0;
        let e = export
            .get_pixel(
                (p.x / surface * f64::from(side)) as u32,
                (p.y / surface * f64::from(side)) as u32,
            )
            .0;
        let d = (0..3).map(|i| c[i].abs_diff(e[i])).max().unwrap();
        worst = worst.max(d);
        eprintln!("{p:?}: canvas {c:?} export {e:?}");
    }
    assert!(worst <= 2, "canvas and export differ by {worst} levels");
}

/// The main screens in every language at 100% and 200% (layout check for
/// longer translations).
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_languages() {
    use tp_app::state::Modal;
    use tp_i18n::Language;
    for language in Language::ALL {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
            let code = language.code();
            let mut h = common::wgpu_harness_with(prefs, size);
            save(&mut h, &format!("lang_{code}_home_{suffix}"));
            h.state_mut().modal = Some(Modal::NewProject(Default::default()));
            save(&mut h, &format!("lang_{code}_new_project_{suffix}"));
            h.state_mut().modal = None;
            h.run();
            // A project with a selected stroked text (the inspector).
            h.get_by_label(&tp_i18n::tr("home-new-project")).click();
            h.run();
            h.get_by_role_and_label(egui::accesskit::Role::CheckBox, common::SAMPLE)
                .click();
            h.run();
            h.get_by_label(&tp_i18n::tr("new-project-create")).click();
            h.run();
            save(&mut h, &format!("lang_{code}_project_{suffix}"));
            common::show_space(&mut h, tp_app::layout::Space::Workshop);
            let (text, _) = gradient_scene(&mut h);
            h.state_mut().workspace_mut().unwrap().selection = vec![text];
            save(&mut h, &format!("lang_{code}_workspace_{suffix}"));
            h.state_mut().modal = Some(Modal::Preferences);
            save(&mut h, &format!("lang_{code}_preferences_{suffix}"));
            h.state_mut().modal = None;
            h.run();
            h.state_mut().workspace_mut().unwrap().selection.clear();
            let r = h
                .state_mut()
                .workspace_mut()
                .unwrap()
                .project
                .surface()
                .objects[0]
                .id;
            h.state_mut().workspace_mut().unwrap().selection = vec![r];
            save(&mut h, &format!("lang_{code}_transform_{suffix}"));
        }
    }
}

/// A vehicle project (template overlay, Textures tab with an update) and
/// the New Project vehicle step, in English and German.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_vehicle_screens() {
    use tp_app::state::Modal;
    use tp_vehicles::sample::{self, SampleTexture};
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        let dir = tempfile::tempdir().unwrap();
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        h.state_mut().prefs.layout.left_tab = LeftTab::Layers;
        h.state_mut().vehicles =
            tp_app::vehicles::VehicleLibrary::open(&dir.path().join("library"));
        let tex = |id, name, size, layout| SampleTexture {
            id,
            name,
            size: tp_vehicles::TextureSize::new(size, size),
            layout,
        };
        for (version, cabin_layout) in [("1.2.0", 1), ("1.3.0", 2)] {
            let bytes = sample::package(
                "scs.sample.truck",
                "Sample Truck",
                version,
                &[
                    tex("cabin", "Cabin", 4096, cabin_layout),
                    tex("chassis", "Chassis", 2048, 1),
                    tex("accessories", "Accessories", 1024, 1),
                ],
            );
            h.state_mut().vehicles.install_bytes(&bytes).unwrap();
        }
        let code = language.code();
        h.state_mut().modal = Some(Modal::NewProject(Default::default()));
        if let Some(Modal::NewProject(d)) = &mut h.state_mut().modal {
            d.check(tp_app::ui::vehicle_dialogs::VehicleChoice {
                id: "scs.sample.truck".into(),
                version: "1.3.0".parse().unwrap(),
                textures: vec!["chassis".into(), "accessories".into()],
            });
        }
        save(&mut h, &format!("vehicle_wizard_{code}"));
        h.state_mut().modal = None;
        h.run();
        let package = h
            .state()
            .vehicles
            .load("scs.sample.truck", &"1.2.0".parse().unwrap())
            .unwrap();
        let textures = tp_app::vehicle_project::default_textures(&package.manifest);
        let project =
            tp_app::vehicle_project::fleet_project("Sample Truck", &[(&package, &textures)])
                .unwrap();
        common::open_project(&mut h, project);
        save(&mut h, &format!("vehicle_project_{code}"));
    }
}

/// The empty New Project vehicle step and Vehicle Library with the samples
/// button, then projects on the samples' templates, in English and French.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_sample_vehicle() {
    use tp_app::state::Modal;
    for language in [tp_i18n::Language::English, tp_i18n::Language::French] {
        let dir = tempfile::tempdir().unwrap();
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        h.state_mut().vehicles =
            tp_app::vehicles::VehicleLibrary::open(&dir.path().join("library"));
        h.state_mut().modal = Some(Modal::NewProject(Default::default()));
        save(&mut h, &format!("sample_wizard_empty_{code}"));
        h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
        save(&mut h, &format!("sample_library_empty_{code}"));
        h.state_mut().modal = None;
        h.state_mut().vehicles.install_samples().unwrap();
        let truck = h
            .state()
            .vehicles
            .load(tp_app::vehicles::SAMPLE_ID, &"1.1.0".parse().unwrap())
            .unwrap();
        let trailer = h
            .state()
            .vehicles
            .load(
                tp_app::vehicles::SAMPLE_TRAILER_ID,
                &"1.0.0".parse().unwrap(),
            )
            .unwrap();
        for (name, package, main) in [
            ("standard", &truck, "standard"),
            ("high_roof", &truck, "high_roof"),
            ("trailer", &trailer, "base"),
        ] {
            let mut textures = tp_app::vehicle_project::default_textures(&package.manifest);
            textures[0] = main.to_owned();
            let project =
                tp_app::vehicle_project::fleet_project("Sample", &[(package, &textures)]).unwrap();
            common::open_project(&mut h, project);
            h.run();
            save(&mut h, &format!("sample_{name}_{code}"));
        }
    }
}

/// A fleet: the sample truck's two main textures and the sample trailer,
/// with the Textures tab, the Add Vehicle and Textures dialogs, the Update
/// Template dialog and the wizard's texture checkboxes, in English and
/// German.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_fleet_screens() {
    use tp_app::state::Modal;
    use tp_app::ui::vehicle_dialogs::{AddVehicleDialog, TexturesDialog, VehicleChoice};
    use tp_app::vehicles::{SAMPLE_ID, SAMPLE_TRAILER_ID};
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        // Wizard with both main textures checked.
        let both = vec![
            "standard".to_owned(),
            "high_roof".to_owned(),
            "chassis".to_owned(),
            "cab_accessories".to_owned(),
            "side_skirts".to_owned(),
        ];
        h.state_mut().modal = Some(Modal::NewProject(Default::default()));
        if let Some(Modal::NewProject(d)) = &mut h.state_mut().modal {
            d.check(VehicleChoice {
                id: SAMPLE_ID.into(),
                version: "1.1.0".parse().unwrap(),
                textures: both.clone(),
            });
        }
        save(&mut h, &format!("fleet_wizard_{code}"));
        // The fleet project: the truck and the trailer.
        let load =
            |h: &egui_kittest::Harness<'static, tp_app::AppState>, id: &str, version: &str| {
                h.state()
                    .vehicles
                    .load(id, &version.parse().unwrap())
                    .unwrap()
            };
        let truck = load(&h, SAMPLE_ID, "1.1.0");
        let project =
            tp_app::vehicle_project::fleet_project("ACE Logistics", &[(&truck, &both)]).unwrap();
        h.state_mut().modal = None;
        common::open_project(&mut h, project);
        let trailer = load(&h, SAMPLE_TRAILER_ID, "1.0.0");
        let all = tp_app::vehicle_project::default_textures(&trailer.manifest);
        h.state_mut()
            .workspace_mut()
            .unwrap()
            .add_vehicle(&trailer, &all, 1.0)
            .unwrap();
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.set_active_surface(2);
        // A texture flagged by an update: ⚠ in the Textures tab, the notice
        // in the inspector.
        ws.project.surfaces[2].template.as_mut().unwrap().status =
            tp_core::TemplateStatus::LayoutChanged;
        for _ in 0..20 {
            h.step();
        }
        save(&mut h, &format!("fleet_project_{code}"));
        h.state_mut().modal = Some(Modal::AddVehicle(AddVehicleDialog::for_game(Some("ets2"))));
        save(&mut h, &format!("fleet_add_vehicle_{code}"));
        let dialog = {
            let p = &h.state().workspace().unwrap().project;
            TexturesDialog::new(p, &p.vehicles[0])
        };
        h.state_mut().modal = Some(Modal::Textures(dialog));
        save(&mut h, &format!("fleet_textures_{code}"));
    }
}

/// The Custom Vehicle dialog filled in, with a texture missing its game
/// ids, then the Vehicle Library with the custom vehicle (New Version…,
/// Export…), in English and French.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_custom_vehicle() {
    use tp_app::state::Modal;
    use tp_app::ui::custom_vehicle::{CustomVehicleDialog, Origin};
    for language in [tp_i18n::Language::English, tp_i18n::Language::French] {
        let dir = tempfile::tempdir().unwrap();
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 1000.0));
        h.state_mut().vehicles =
            tp_app::vehicles::VehicleLibrary::open(&dir.path().join("library"));
        let mut dialog = CustomVehicleDialog::new(Origin::NewProject(Default::default()));
        dialog.form.name = "R 2024".into();
        dialog.form.brand = "Scania".into();
        dialog.form.path = "scania.r_2024".into();
        let wide = {
            let mut out = Vec::new();
            image::RgbaImage::new(64, 32)
                .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
                .unwrap();
            out
        };
        dialog.add_files(
            vec![
                ("cabin.png".into(), Ok(tp_vehicles::sample::png(64))),
                ("side_skirts.png".into(), Ok(tp_vehicles::sample::png(32))),
                ("mirrors.png".into(), Ok(wide)),
            ],
            None,
        );
        dialog.form.rows[2].game_ids = "mirror.painted, s_mirror.painted".into();
        h.state_mut().modal = Some(Modal::CustomVehicle(Box::new(dialog)));
        save(&mut h, &format!("custom_vehicle_{code}"));

        // Build it, then show the library.
        let packed = match &h.state().modal {
            Some(Modal::CustomVehicle(d)) => {
                let mut form = d.form.clone();
                form.rows[1].game_ids = "sideskirt.a".into();
                form.pack(&mut || {}).unwrap()
            }
            _ => unreachable!(),
        };
        h.state_mut().vehicles.install_bytes(&packed.bytes).unwrap();
        h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
        save(&mut h, &format!("custom_vehicle_library_{code}"));
        // My vehicles: the custom status among the sample vehicles.
        h.state_mut().vehicles.install_samples().unwrap();
        h.state_mut().modal = Some(Modal::VehicleLibrary(
            tp_app::ui::vehicle_dialogs::LibraryDialog {
                tab: tp_app::ui::vehicle_dialogs::LibraryTab::Mine,
                ..Default::default()
            },
        ));
        save(&mut h, &format!("custom_vehicle_library_mine_{code}"));
        h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
        save(&mut h, &format!("custom_vehicle_library_all_{code}"));
    }
}

/// The brand kit: the palette with a linked swatch marked, the Edit Swatch
/// popup, the Styles section with both groups, and the Copy from Cabin
/// dialog, in English and German.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_brand_kit() {
    use tp_app::state::Modal;
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 1100.0));
        h.state_mut().prefs.layout.left_tab = LeftTab::Resources;
        let (red, _) = brand_fleet(&mut h);
        for _ in 0..20 {
            h.step();
        }
        save(&mut h, &format!("brand_palette_styles_{code}"));
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.start_swatch_edit(red);
        save(&mut h, &format!("brand_edit_swatch_{code}"));
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.cancel_brand_edit();
        let high_roof = ws
            .project
            .surfaces
            .iter()
            .position(|s| s.name == "High roof")
            .unwrap();
        ws.set_active_surface(high_roof);
        let dialog = tp_app::ui::vehicle_dialogs::CopyFromCabinDialog::new(&ws.project);
        h.state_mut().modal = Some(Modal::CopyFromCabin(dialog));
        save(&mut h, &format!("brand_copy_from_cabin_{code}"));
    }
}

/// The before/after editor (artboard 07's right panel) on the brand kit's
/// fleet, the band and the stripe also drawn on the other textures: the
/// Brand space with the swatch editor and with the style editor open, a new
/// value set (the tiles show it), in English and German at 100 % and 200 %,
/// and the editor as a dialog over the Workshop. Files are named
/// `brand_editor_<language>_<screen>_<scale>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_brand_editor() {
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            let code = language.code();
            let name = |screen: &str| format!("brand_editor_{code}_{screen}_{suffix}");
            let size = Vec2::new(1440.0, 900.0) * scale;
            let mut h = common::wgpu_harness_with(prefs, size);
            editor_fleet(&mut h);
            common::show_space(&mut h, tp_app::layout::Space::Brand);
            open_editor(&mut h, EditorTarget::Swatch);
            save_settled(&mut h, &name("swatch"));
            h.state_mut().workspace_mut().unwrap().cancel_brand_edit();
            open_editor(&mut h, EditorTarget::Style);
            save_settled(&mut h, &name("style"));
            h.state_mut().workspace_mut().unwrap().cancel_brand_edit();
            // Edit Swatch… from the Workshop: the same editor as a dialog.
            common::show_space(&mut h, tp_app::layout::Space::Workshop);
            open_editor(&mut h, EditorTarget::Swatch);
            save_settled(&mut h, &name("workshop"));
            h.state_mut().workspace_mut().unwrap().cancel_brand_edit();
        }
    }
}

/// The brand editor and the cards with every interface text 40 % longer
/// (localization: Texts 40% longer), in English at 100 %.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_brand_editor_long_texts() {
    use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    tp_i18n::set_lengthening(40);
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 1200.0));
    editor_fleet(&mut h);
    // A symbol, for its card's usage.
    let ws = h.state_mut().workspace_mut().unwrap();
    let badge = ws.project.add(Object::new(
        ObjectId(0),
        ShapeKind::Ellipse,
        Frame::new(Point::new(900.0, 900.0), Size::new(400.0, 400.0), 0.0),
    ));
    let (logo, _) = ws.project.convert_to_symbol(&[badge], "Symbol").unwrap();
    ws.project.rename_symbol(logo, "ACE badge");
    common::show_space(&mut h, tp_app::layout::Space::Brand);
    save_settled(&mut h, "brand_editor_long_cards");
    open_editor(&mut h, EditorTarget::Swatch);
    save_settled(&mut h, "brand_editor_long_swatch");
    h.state_mut().workspace_mut().unwrap().cancel_brand_edit();
    open_editor(&mut h, EditorTarget::Style);
    save_settled(&mut h, "brand_editor_long_style");
    h.state_mut().workspace_mut().unwrap().cancel_brand_edit();
    common::show_space(&mut h, tp_app::layout::Space::Workshop);
    open_editor(&mut h, EditorTarget::Swatch);
    save_settled(&mut h, "brand_editor_long_workshop");
    tp_i18n::set_lengthening(0);
}

/// Saves once the preview thumbnails of the open editor are rendered.
fn save_settled(h: &mut egui_kittest::Harness<'static, tp_app::AppState>, name: &str) {
    h.run();
    common::settle_renders(h);
    save(h, name);
}

/// Symbols: the Symbols section with two instances of a logo on a texture
/// (one selected, the inspector showing it), then the logo edited in its own
/// view with its bar, in English and French.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_symbols() {
    use tp_core::document::{
        CharStyle, Frame, Object, ObjectId, Paint, Rgba, ShapeKind, StrokeStyle, TextBlock,
    };
    use tp_core::kurbo::{Point, Size};
    for language in [tp_i18n::Language::English, tp_i18n::Language::French] {
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 1000.0));
        h.state_mut().prefs.layout.left_tab = LeftTab::Resources;
        common::create_project(&mut h);
        let ws = h.state_mut().workspace_mut().unwrap();
        let frame = |x, y, w, h| Frame::new(Point::new(x, y), Size::new(w, h), 0.0);
        let mut badge = Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            frame(900.0, 1500.0, 420.0, 420.0),
        );
        badge.fill = Paint::Solid(Rgba::rgb(0xC0, 0x10, 0x20));
        badge.stroke = Some(StrokeStyle {
            paint: Paint::Solid(Rgba::rgb(255, 255, 255)),
            width: 18.0,
            ..StrokeStyle::default()
        });
        let mut letters = Object::text(
            ObjectId(0),
            TextBlock::new(
                "ACE",
                CharStyle {
                    size: 180.0,
                    weight: 900,
                    ..CharStyle::default()
                },
            ),
            Point::new(760.0, 1560.0),
        );
        letters.fill = Paint::Solid(Rgba::rgb(255, 255, 255));
        ws.text.place_at(&mut letters, Point::new(760.0, 1560.0));
        let a = ws.project.add(badge);
        let b = ws.project.add(letters);
        ws.selection = vec![a, b];
        let symbol = ws.convert_to_symbol(1.0).unwrap();
        ws.project.rename_symbol(symbol, "ACE badge");
        ws.place_symbol(symbol, Some(Point::new(2900.0, 1500.0)), 2.0);
        let second = ws.selection[0];
        let rotated = tp_core::document::rotate(
            &ws.selected_objects(),
            Point::new(2900.0, 1500.0),
            -15.0,
            false,
        );
        ws.project.surface_mut().replace(&rotated);
        ws.project.refresh_instances();
        ws.selection = vec![second];
        ws.relayout_all_texts();
        for _ in 0..20 {
            h.step();
        }
        save(&mut h, &format!("symbols_panel_{code}"));
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.edit_symbol(symbol, 3.0);
        for _ in 0..20 {
            h.step();
        }
        save(&mut h, &format!("symbols_edit_{code}"));
    }
}

/// Flip: an SVG logo and a text, with flipped copies (horizontal under each,
/// vertical for the text), the inspector's Flip buttons, and the Object
/// menu showing Flip Horizontal and Flip Vertical.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_flip() {
    use tp_core::document::{CharStyle, FlipAxis, Object, ObjectId, Paint, Rgba, TextBlock};
    use tp_core::kurbo::{Point, Vec2 as V};
    let prefs = Prefs::default();
    let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 1000.0));
    common::create_project(&mut h);
    let ws = h.state_mut().workspace_mut().unwrap();
    // A truck-like logo facing right: an arrow and a red wheel at the back.
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="150">
        <polygon points="0,30 200,30 200,0 300,75 200,150 200,120 0,120" fill="#1F4E9E"/>
        <circle cx="50" cy="75" r="30" fill="#D02030"/>
    </svg>"##;
    ws.place_files(
        vec![tp_app::import::read_bytes("arrow.svg", svg.to_vec())],
        Some(Point::new(1100.0, 1100.0)),
        0.0,
    );
    let logo = ws.selection[0];
    let mut text = Object::text(
        ObjectId(0),
        TextBlock::new(
            "ACE",
            CharStyle {
                size: 260.0,
                weight: 900,
                ..CharStyle::default()
            },
        ),
        Point::new(2400.0, 1150.0),
    );
    text.fill = Paint::Solid(Rgba::rgb(0x1F, 0x4E, 0x9E));
    ws.text.place_at(&mut text, Point::new(2400.0, 1150.0));
    let text = ws.project.add(text);
    // Flipped copies below the originals.
    let copies = ws.project.duplicate(&[logo, text], V::new(0.0, 900.0));
    ws.selection = vec![copies[0]];
    ws.flip_selection(FlipAxis::Horizontal, 1.0);
    let upside_down = ws.project.duplicate(&[text], V::new(0.0, 1800.0));
    ws.selection = vec![copies[1]];
    ws.flip_selection(FlipAxis::Horizontal, 2.0);
    ws.selection = upside_down;
    ws.flip_selection(FlipAxis::Vertical, 3.0);
    ws.selection = vec![copies[0], copies[1]];
    for _ in 0..30 {
        h.step();
    }
    save(&mut h, "flip_canvas");
    h.get_by_label("Object").click();
    for _ in 0..5 {
        h.step();
    }
    save(&mut h, "flip_menu");
}

/// Import from Library…: a library holding the symbol "Logo Ardent" (an
/// SVG badge and its lettering), two swatches and two styles; one swatch is
/// already in the project, and the symbol is checked.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_import_from_library() {
    use tp_app::library::Element;
    use tp_core::document::{CharStyle, Object, ObjectId, Paint, Rgba, StrokeStyle, TextBlock};
    use tp_core::kurbo::Point;
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 1000.0));
    common::create_project(&mut h);
    let vert = Rgba::rgb(0x1E, 0x8C, 0x3A);
    let gris = Rgba::rgb(0x4A, 0x50, 0x58);
    let tp_app::AppState {
        library, screen, ..
    } = h.state_mut();
    let tp_app::state::Screen::Workspace(ws) = screen else {
        panic!("project open");
    };
    let (green, _) = ws.project.add_swatch(vert, "Color");
    ws.project.rename_swatch(green, "Vert Ardent");
    let (grey, _) = ws.project.add_swatch(gris, "Color");
    ws.project.rename_swatch(grey, "Gris Ardent");
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <circle cx="100" cy="100" r="90" fill="#1E8C3A"/>
        <polygon points="100,30 160,150 40,150" fill="#FFFFFF"/>
    </svg>"##;
    ws.place_files(
        vec![tp_app::import::read_bytes("badge.svg", svg.to_vec())],
        Some(Point::new(1400.0, 1050.0)),
        0.0,
    );
    let badge = ws.selection[0];
    let mut text = Object::text(
        ObjectId(0),
        TextBlock::new(
            "ARDENT",
            CharStyle {
                size: 180.0,
                weight: 900,
                ..CharStyle::default()
            },
        ),
        Point::new(1000.0, 1350.0),
    );
    text.fill = Paint::Solid(gris);
    text.fill_swatch = Some(grey);
    ws.text.place_at(&mut text, Point::new(1000.0, 1350.0));
    let text = ws.project.add(text);
    let titre = ws.project.new_text_style(text, "Text style").unwrap();
    ws.project.rename_style(titre, "Titre");
    let mut stripe = Object::new(
        ObjectId(0),
        tp_core::document::ShapeKind::rectangle(),
        tp_core::document::Frame::new(
            Point::new(2000.0, 2000.0),
            tp_core::kurbo::Size::new(800.0, 80.0),
            0.0,
        ),
    );
    stripe.fill = Paint::Solid(vert);
    stripe.fill_swatch = Some(green);
    stripe.stroke = Some(StrokeStyle {
        paint: Paint::Solid(gris),
        width: 8.0,
        swatch: Some(grey),
        ..StrokeStyle::default()
    });
    let stripe = ws.project.add(stripe);
    let bande = ws.project.new_graphic_style(stripe, "Style").unwrap();
    ws.project.rename_style(bande, "Bande");
    let (logo, _) = ws
        .project
        .convert_to_symbol(&[badge, text], "Symbol")
        .unwrap();
    ws.project.rename_symbol(logo, "Logo Ardent");
    ws.relayout_all_texts();
    for element in [
        Element::Symbol(logo),
        Element::Swatch(green),
        Element::Style(bande),
    ] {
        ws.add_to_library(library, element, 1.0);
    }
    common::create_project(&mut h);
    run(&mut h, tp_app::commands::CommandId::ImportFromLibrary);
    h.get_by_role_and_label(egui::accesskit::Role::CheckBox, "Vert Ardent")
        .click();
    h.run();
    h.get_by_label("Import").click();
    h.run();
    run(&mut h, tp_app::commands::CommandId::ImportFromLibrary);
    h.get_by_role_and_label(egui::accesskit::Role::CheckBox, "Logo Ardent")
        .click();
    for _ in 0..30 {
        h.step();
    }
    save(&mut h, "import_from_library");
}

fn run(h: &mut egui_kittest::Harness<'static, tp_app::AppState>, id: tp_app::commands::CommandId) {
    h.state_mut().queue.push(id);
    h.run_steps(3);
    h.run();
}

/// The texture states: the Project space of a truck and trailer fleet with
/// Empty, Modified and To check textures (All, then To do, then Before
/// exporting expanded), the Workshop with nothing selected (On this
/// texture, and a To check notice) and the Export Mod dialog's warnings,
/// in English and German at 100 % and 200 %. Files are named
/// `status_<language>_<screen>_<scale>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_texture_status() {
    use tp_app::layout::Space;
    use tp_app::state::Modal;
    use tp_app::vehicles::{SAMPLE_ID, SAMPLE_TRAILER_ID};
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            let code = language.code();
            let size = Vec2::new(1440.0, 900.0) * scale;
            let mut h = common::wgpu_harness_with(prefs, size);
            let load =
                |h: &egui_kittest::Harness<'static, tp_app::AppState>, id: &str, version: &str| {
                    h.state()
                        .vehicles
                        .load(id, &version.parse().unwrap())
                        .unwrap()
                };
            let truck = load(&h, SAMPLE_ID, "1.1.0");
            let mut textures = tp_app::vehicle_project::default_textures(&truck.manifest);
            textures.push("high_roof".into());
            let project =
                tp_app::vehicle_project::fleet_project("ACE Logistics", &[(&truck, &textures)])
                    .unwrap();
            h.state_mut().open_project(project);
            let trailer = load(&h, SAMPLE_TRAILER_ID, "1.0.0");
            let all = tp_app::vehicle_project::default_textures(&trailer.manifest);
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.add_vehicle(&trailer, &all, 1.0).unwrap();
            // Paint the accessories and the mudflaps; four rectangles on
            // the side skirts, two of their colors off the palette.
            let paint = |ws: &mut tp_app::workspace::Workspace, name: &str, colors: &[Rgba]| {
                let i = ws
                    .project
                    .surfaces
                    .iter()
                    .position(|s| s.name == name)
                    .unwrap();
                ws.project.active_surface = i;
                let side = ws.project.surface().max_side();
                for (k, color) in colors.iter().enumerate() {
                    let mut o = Object::new(
                        ObjectId(0),
                        ShapeKind::rectangle(),
                        Frame::new(
                            Point::new(side / 2.0, side * (0.2 + 0.2 * k as f64)),
                            Size::new(side * 0.9, side * 0.15),
                            0.0,
                        ),
                    );
                    o.fill = (*color).into();
                    ws.project.add(o);
                }
            };
            let navy = Rgba::rgb(0x1E, 0x2A, 0x4A);
            let red = Rgba::rgb(0xC2, 0x3B, 0x2A);
            paint(ws, "Chassis", &[navy]);
            paint(ws, "Cab accessories", &[red]);
            paint(ws, "Mudflaps", &[navy]);
            let gold = Rgba::rgb(0xE8, 0xB0, 0x30);
            paint(ws, "Side skirts", &[navy, red, gold, red]);
            let (swatch, _) = ws.project.add_swatch(red, "Color");
            let ids: Vec<ObjectId> = ws.project.surface().objects.iter().map(|o| o.id).collect();
            for id in &ids {
                ws.edit_object(*id, "undo-change-fill", 1.0, |o| {
                    if o.fill.solid_color() == Some(red) {
                        o.fill_swatch = Some(swatch);
                    }
                });
            }
            let body = ws
                .project
                .surfaces
                .iter()
                .position(|s| s.name == "Curtain body 13.6 m")
                .unwrap();
            ws.project.surfaces[body].template.as_mut().unwrap().status =
                tp_core::TemplateStatus::LayoutChanged;
            ws.selection.clear();
            ws.space = Space::Project;
            common::settle_renders(&mut h);
            save(&mut h, &format!("status_{code}_project_{suffix}"));
            h.get_by_role_and_label(
                egui::accesskit::Role::RadioButton,
                &tp_i18n::tr("project-filter-to-do"),
            )
            .click();
            h.run();
            save(&mut h, &format!("status_{code}_todo_{suffix}"));
            h.state_mut().workspace_mut().unwrap().texture_filter = tp_core::TextureFilter::All;
            // Down to the end of the Mod information column, where Before
            // exporting is, with the empty textures' line expanded.
            let empty = tp_i18n::tr!("mod-warning-empty-many", count = 4);
            let scroll_down = |h: &mut egui_kittest::Harness<'static, tp_app::AppState>| {
                for _ in 0..12 {
                    h.get_by_label(&empty).scroll_down();
                    h.run();
                }
            };
            scroll_down(&mut h);
            h.get_by_label(&empty).click();
            h.run();
            scroll_down(&mut h);
            save(&mut h, &format!("status_{code}_before_exporting_{suffix}"));
            // The Workshop with nothing selected: On this texture.
            common::show_space(&mut h, Space::Workshop);
            h.run();
            save(&mut h, &format!("status_{code}_workshop_{suffix}"));
            // A texture To check.
            h.state_mut()
                .workspace_mut()
                .unwrap()
                .set_active_surface(body);
            common::settle_renders(&mut h);
            save(&mut h, &format!("status_{code}_to_check_{suffix}"));
            // The Export Mod dialog's warnings.
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::E);
            settle_dialog(&mut h);
            assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
            save(&mut h, &format!("status_{code}_export_mod_{suffix}"));
        }
    }
}

/// Saves a render after a few frames: the palette's blinking caret keeps
/// `Harness::run` from settling.
fn save_stepped(h: &mut egui_kittest::Harness<'static, tp_app::AppState>, name: &str) {
    h.run_steps(6);
    let image = h.render().expect("render");
    image.save(out_dir().join(format!("{name}.png"))).unwrap();
}

/// A truck and trailer fleet in the Workshop with Empty, Modified and To
/// check textures, nothing selected.
fn palette_fleet(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) {
    use tp_app::vehicles::{SAMPLE_ID, SAMPLE_TRAILER_ID};
    use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    let load = |h: &egui_kittest::Harness<'static, tp_app::AppState>, id: &str, version: &str| {
        h.state()
            .vehicles
            .load(id, &version.parse().unwrap())
            .unwrap()
    };
    let truck = load(h, SAMPLE_ID, "1.1.0");
    let textures = tp_app::vehicle_project::default_textures(&truck.manifest);
    let project =
        tp_app::vehicle_project::fleet_project("ACE Logistics", &[(&truck, &textures)]).unwrap();
    common::open_project(h, project);
    let trailer = load(h, SAMPLE_TRAILER_ID, "1.0.0");
    let all = tp_app::vehicle_project::default_textures(&trailer.manifest);
    let ws = h.state_mut().workspace_mut().unwrap();
    ws.add_vehicle(&trailer, &all, 1.0).unwrap();
    for name in ["Chassis", "Mudflaps"] {
        let i = ws
            .project
            .surfaces
            .iter()
            .position(|s| s.name == name)
            .unwrap();
        ws.project.active_surface = i;
        let side = ws.project.surface().max_side();
        ws.project.add(Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(
                Point::new(side / 2.0, side / 2.0),
                Size::new(side * 0.8, side * 0.3),
                0.0,
            ),
        ));
    }
    ws.project.surfaces[0].template.as_mut().unwrap().status =
        tp_core::TemplateStatus::LayoutChanged;
    ws.set_active_surface(0);
    ws.selection.clear();
    common::settle_renders(h);
}

/// The command palette (artboard 04's "Search textures ⌘K" field opens
/// it): the Workshop of a truck and trailer fleet with an empty query
/// (Recent, Textures with their state markers, Commands), a query
/// highlighting a disabled command (its reason in the footer), the
/// textures-only mode from the Search textures field, the home screen and
/// a narrow window, in English and German at 100 % and 200 %. Files are
/// named `palette_<language>_<screen>_<scale>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_command_palette() {
    use egui::{Key, Modifiers};
    use tp_app::commands::CommandId;
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            let code = language.code();
            let size = Vec2::new(1440.0, 900.0) * scale;
            let mut h = common::wgpu_harness_with(prefs.clone(), size);
            // The home screen.
            h.run();
            h.key_press_modifiers(Modifiers::COMMAND, Key::K);
            save_stepped(&mut h, &format!("palette_{code}_home_{suffix}"));
            h.key_press(Key::Escape);
            h.run();

            palette_fleet(&mut h);
            h.state_mut().palette_recent = vec![
                CommandId::ShowGrid,
                CommandId::Flip(tp_core::document::FlipAxis::Horizontal),
            ];
            save(&mut h, &format!("palette_{code}_textures_tab_{suffix}"));
            h.key_press_modifiers(Modifiers::COMMAND, Key::K);
            save_stepped(&mut h, &format!("palette_{code}_empty_{suffix}"));
            // A query, its first result a disabled command.
            let query = if language == tp_i18n::Language::German {
                "gruppieren"
            } else {
                "group"
            };
            h.event(egui::Event::Text(query.into()));
            save_stepped(&mut h, &format!("palette_{code}_disabled_{suffix}"));
            h.key_press(Key::Escape);
            h.run_steps(4);
            // Textures only, from the Search textures field.
            h.get_by_label(&tp_i18n::tr("textures-search")).click();
            h.run_steps(2);
            h.event(egui::Event::Text("ch".into()));
            save_stepped(&mut h, &format!("palette_{code}_textures_only_{suffix}"));
            h.key_press(Key::Escape);
            h.run_steps(4);

            // The narrowest window (the minimum size, 960 × 600).
            let mut h = common::wgpu_harness_with(prefs, Vec2::new(960.0, 600.0) * scale);
            h.run();
            h.key_press_modifiers(Modifiers::COMMAND, Key::K);
            save_stepped(&mut h, &format!("palette_{code}_narrow_{suffix}"));
        }
    }
}

/// The palette and the Search textures field with every interface text
/// 40 % longer (localization: Texts 40% longer), in English at 100 %.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_command_palette_long_texts() {
    use egui::{Key, Modifiers};
    tp_i18n::set_lengthening(40);
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    palette_fleet(&mut h);
    h.state_mut().palette_recent = vec![tp_app::commands::CommandId::ShowGrid];
    save(&mut h, "palette_long_textures_tab");
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    save_stepped(&mut h, "palette_long_empty");
    h.event(egui::Event::Text("group".into()));
    save_stepped(&mut h, "palette_long_disabled");
    h.key_press(Key::Escape);
    h.run_steps(4);
    h.get_by_label(&tp_i18n::tr("textures-search")).click();
    h.run_steps(2);
    save_stepped(&mut h, "palette_long_textures_only");
    tp_i18n::set_lengthening(0);
}

/// The lettering scene with drop shadows: a dark, soft shadow under the
/// title and a wide, blurred one under the PNG logo. Returns the title,
/// the logo and the phone number (no shadow), after the shadow renders.
fn shadow_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> [tp_core::document::ObjectId; 3] {
    use tp_core::document::{Rgba, Shadow};
    use tp_core::kurbo::Vec2 as DocVec;
    let ids = lettering_scene(h);
    let (title, phone, logo) = (ids[0], ids[1], ids[4]);
    {
        let ws = h.state_mut().workspace_mut().unwrap();
        let shadowed = |id, shadow| {
            let mut o = (**ws.project.surface().get(id).unwrap()).clone();
            o.shadow = Some(shadow);
            o
        };
        let title = shadowed(
            title,
            Shadow {
                color: Rgba::rgb(0x15, 0x15, 0x18),
                opacity: 0.7,
                offset: DocVec::new(24.0, 24.0),
                blur: 16.0,
                ..Shadow::DEFAULT
            },
        );
        let logo = shadowed(
            logo,
            Shadow {
                opacity: 0.6,
                offset: DocVec::new(40.0, 48.0),
                blur: 60.0,
                ..Shadow::DEFAULT
            },
        );
        ws.project.surface_mut().replace(&[title, logo]);
    }
    wait_for_images(h);
    common::settle_renders(h);
    [title, logo, phone]
}

/// Saves once the shadow and thumbnail renders are done.
fn save_shadowed(h: &mut egui_kittest::Harness<'static, tp_app::AppState>, name: &str) {
    common::settle_renders(h);
    save(h, name);
}

/// Drop shadows in the Workshop, as on macOS, in English and German at
/// 100 % and 200 %: the shadowed lettering and logo with nothing selected,
/// the title selected (the Shadow row), its shadow popover open, the logo
/// selected, and the phone number selected (+ Add a shadow). Then, in
/// English, the title and the logo zoomed in, the shadow under each.
/// Files are named `shadow_<language>_<screen>_<scale>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_drop_shadow() {
    use tp_i18n::Language;
    let popover = tp_app::ui::workspace::inspector::shadow_popover();
    for language in [Language::English, Language::German] {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let code = language.code();
            let name = |screen: &str| format!("shadow_{code}_{screen}_{suffix}");
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
            let mut h = common::wgpu_harness_with(prefs, size);
            mac_native(&mut h);
            common::create_project(&mut h);
            let [title, logo, phone] = shadow_scene(&mut h);
            save_shadowed(&mut h, &name("workshop"));
            h.state_mut().workspace_mut().unwrap().selection = vec![title];
            save_shadowed(&mut h, &name("text"));
            tp_ui::widgets::Popover::open(&h.ctx, popover);
            save_shadowed(&mut h, &name("popover"));
            h.key_press(egui::Key::Escape);
            h.run();
            h.state_mut().workspace_mut().unwrap().selection = vec![logo];
            save_shadowed(&mut h, &name("image"));
            h.state_mut().workspace_mut().unwrap().selection = vec![phone];
            save_shadowed(&mut h, &name("add"));
        }
    }
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    mac_native(&mut h);
    common::create_project(&mut h);
    shadow_scene(&mut h);
    for (zoom, center, name) in [
        (1.0, (1500.0, 1450.0), "shadow_zoom_text"),
        (1.0, (3000.0, 1300.0), "shadow_zoom_image"),
    ] {
        {
            let view = h
                .state_mut()
                .workspace_mut()
                .unwrap()
                .viewport
                .as_mut()
                .unwrap();
            view.center = tp_core::kurbo::Point::new(center.0, center.1);
            view.set_zoom_centered(zoom);
        }
        h.step();
        wait_for_images(&mut h);
        save_shadowed(&mut h, name);
    }
}

/// The Shadow row and its popover with every interface text 40 % longer,
/// in English at 100 %: the title selected, its popover open, and the
/// phone number selected (+ Add a shadow).
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_drop_shadow_long_texts() {
    tp_i18n::set_lengthening(40);
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    mac_native(&mut h);
    common::create_project(&mut h);
    let [title, _, phone] = shadow_scene(&mut h);
    h.state_mut().workspace_mut().unwrap().selection = vec![title];
    save_shadowed(&mut h, "shadow_long_text");
    tp_ui::widgets::Popover::open(&h.ctx, tp_app::ui::workspace::inspector::shadow_popover());
    save_shadowed(&mut h, "shadow_long_popover");
    h.key_press(egui::Key::Escape);
    h.run();
    h.state_mut().workspace_mut().unwrap().selection = vec![phone];
    save_shadowed(&mut h, "shadow_long_add");
    tp_i18n::set_lengthening(0);
}

/// A template-like PNG of `width` × `height` (outline and diagonals on
/// transparency), for a rectangular texture.
fn rect_template(width: u32, height: u32) -> Vec<u8> {
    let mut img = image::RgbaImage::new(width, height);
    let ink = image::Rgba([20, 20, 20, 255]);
    for x in 0..width {
        img.put_pixel(x, 0, ink);
        img.put_pixel(x, height - 1, ink);
        let y = (x as u64 * u64::from(height - 1) / u64::from(width - 1)) as u32;
        img.put_pixel(x, y, ink);
        img.put_pixel(x, height - 1 - y, ink);
    }
    for y in 0..height {
        img.put_pixel(0, y, ink);
        img.put_pixel(width - 1, y, ink);
    }
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// Rectangular textures: the sample truck with a trailer whose main
/// texture "Base" is 8192 × 1024 and accessory "Light bar" 2048 × 256,
/// each with a circle and a square (which must stay round and square):
/// the Workshop fitted on each, the Textures tab, the Project space's
/// texture rows (scrolled to the trailer's) and the Export Texture dialog
/// of each, in English and German at
/// 100 %. Files are named `rect_<language>_<screen>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_rectangular_textures() {
    use tp_app::layout::Space;
    use tp_app::state::Modal;
    use tp_core::document::{CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    use tp_vehicles::TextureSize;
    use tp_vehicles::sample::{self, SampleTexture};
    let base = SampleTexture {
        id: "base",
        name: "Base",
        size: TextureSize::new(8192, 1024),
        layout: 1,
    };
    let light_bar = SampleTexture {
        id: "light_bar",
        name: "Light bar",
        size: TextureSize::new(2048, 256),
        layout: 1,
    };
    let manifest = sample::manifest_with(
        "scs.krone.cool_liner",
        "Krone Cool Liner",
        "1.0.0",
        "trailer",
        &[base],
        &[light_bar],
    );
    let bytes = sample::zip(
        &manifest,
        &[
            ("templates/base.png".into(), rect_template(1024, 128)),
            ("templates/light_bar.png".into(), rect_template(512, 64)),
        ],
    );
    let trailer = tp_vehicles::Package::read(&bytes).unwrap();
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let code = language.code();
        let name = |screen: &str| format!("rect_{code}_{screen}");
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        mac_native(&mut h);
        common::create_project(&mut h);
        let all = tp_app::vehicle_project::default_textures(&trailer.manifest);
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.project.name = "ACE Logistics".into();
        ws.add_vehicle(&trailer, &all, 1.0).unwrap();
        let base_index = ws.project.active_surface;
        // A circle and a square of the same size on each texture.
        let shapes = |ws: &mut tp_app::workspace::Workspace, side: f64, at: (f64, f64)| {
            for (i, kind, fill) in [
                (0.0, ShapeKind::Ellipse, Rgba::rgb(0xF0, 0xB4, 0x4C)),
                (1.5, ShapeKind::rectangle(), Rgba::rgb(0x7A, 0x1F, 0x2B)),
            ] {
                let center = Point::new(at.0 + i * side, at.1);
                let mut o = Object::new(
                    ObjectId(0),
                    kind,
                    Frame::new(center, Size::new(side, side), 0.0),
                );
                o.fill = fill.into();
                ws.project.add(o);
            }
        };
        shapes(ws, 800.0, (700.0, 512.0));
        let style = CharStyle {
            size: 420.0,
            ..CharStyle::default()
        };
        add_text(
            ws,
            "ACE Logistics",
            style,
            (3000.0, 660.0),
            Rgba::rgb(0x1B, 0x23, 0x40),
            None,
        );
        ws.set_active_surface(base_index + 1);
        shapes(ws, 200.0, (180.0, 128.0));
        ws.set_active_surface(base_index);
        common::settle_renders(&mut h);
        h.state_mut().prefs.layout.left_tab = LeftTab::Layers;
        run(&mut h, tp_app::commands::CommandId::FitToScreen);
        save(&mut h, &name("workshop"));
        h.state_mut().prefs.layout.left_tab = LeftTab::Textures;
        save(&mut h, &name("textures_tab"));
        h.state_mut()
            .workspace_mut()
            .unwrap()
            .set_active_surface(base_index + 1);
        common::settle_renders(&mut h);
        run(&mut h, tp_app::commands::CommandId::FitToScreen);
        save(&mut h, &name("workshop_accessory"));
        h.state_mut().prefs.layout.left_tab = LeftTab::Layers;
        common::show_space(&mut h, Space::Project);
        save(&mut h, &name("project"));
        // Scrolled down to the trailer's rows.
        h.event(egui::Event::PointerMoved(egui::pos2(500.0, 500.0)));
        h.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: Vec2::new(0.0, -2000.0),
            modifiers: egui::Modifiers::NONE,
            phase: egui::TouchPhase::Move,
        });
        h.run_steps(10);
        save(&mut h, &name("project_trailer"));
        common::show_space(&mut h, Space::Workshop);
        for (index, screen) in [(base_index, "export"), (base_index + 1, "export_accessory")] {
            h.state_mut()
                .workspace_mut()
                .unwrap()
                .set_active_surface(index);
            h.run();
            h.key_press_modifiers(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::E,
            );
            for _ in 0..400 {
                h.step();
                let ready = matches!(&h.state().modal, Some(Modal::Export(d)) if d.preview_ready());
                if ready {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            settle_dialog(&mut h);
            save(&mut h, &name(screen));
            h.state_mut().modal = None;
            h.run();
        }
    }
}

/// Several vehicles at once, in English and German at 100 %: New Project
/// with three vehicles checked (the first card open, the others folded) and
/// Add Vehicle… with two vehicles checked. Files are named
/// `fleet_choice_<language>_<screen>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_multi_vehicle_choice() {
    use egui::accesskit::Role;
    use tp_app::state::Modal;
    use tp_i18n::Language;
    // A third installed vehicle beside the sample truck and trailer.
    let volvo = tp_vehicles::sample::package(
        "community.jdoe.volvo_fh16",
        "Volvo FH16 2012",
        "1.0.0",
        &tp_vehicles::sample::truck_textures(),
    );
    for language in [Language::English, Language::German] {
        let code = language.code();
        let name = |screen: &str| format!("fleet_choice_{code}_{screen}");
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        mac_native(&mut h);
        h.state_mut().vehicles.install_bytes(&volvo).unwrap();
        h.run();
        h.get_by_label(&tp_i18n::tr("home-new-project")).click();
        h.run();
        for vehicle in [
            common::SAMPLE,
            "Volvo FH16 2012",
            "TruckPaint Sample Trailer",
        ] {
            h.get_by_role_and_label(Role::CheckBox, vehicle).click();
            settle_dialog(&mut h);
        }
        // The first card open, as in the mockup.
        h.get_by_role_and_label(Role::Button, common::SAMPLE)
            .click();
        settle_dialog(&mut h);
        save(&mut h, &name("new_project"));

        h.state_mut().modal = None;
        common::create_project(&mut h);
        h.state_mut()
            .queue
            .push(tp_app::commands::CommandId::AddVehicle);
        settle_dialog(&mut h);
        for vehicle in ["TruckPaint Sample Trailer", "Volvo FH16 2012"] {
            h.get_by_role_and_label(Role::CheckBox, vehicle).click();
            settle_dialog(&mut h);
        }
        assert!(matches!(h.state().modal, Some(Modal::AddVehicle(_))));
        save(&mut h, &name("add_vehicle"));
    }
}

/// Posts one tools/call to the local MCP server at `port` (from another
/// thread, while the harness steps) and waits for its answer.
fn mcp_call(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
    port: u16,
    name: &str,
    arguments: serde_json::Value,
) {
    use std::io::{Read, Write};
    let body = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": name, "arguments": arguments}})
    .to_string();
    let client = std::thread::spawn(move || {
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            stream,
            "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
    });
    while !client.is_finished() {
        h.step();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    client.join().unwrap();
}

/// The local MCP server: Preferences' AI assistant (MCP) section off, on
/// with a client connected, and with its port in use, and the Workshop's
/// status bar while a client draws, in English and German at 100 % and
/// 200 %; then the same at 100 % with every text 40 % longer. Files are
/// named `mcp_<language>_<screen>_<scale>.png`.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_mcp() {
    use tp_app::state::Modal;
    use tp_i18n::Language;
    let free_port = || {
        std::net::TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    };
    let runs = [
        (Language::English, 1.0, "100", 0),
        (Language::English, 2.0, "200", 0),
        (Language::German, 1.0, "100", 0),
        (Language::German, 2.0, "200", 0),
        (Language::English, 1.0, "long", 40),
    ];
    for (language, scale, suffix, lengthening) in runs {
        tp_i18n::set_lengthening(lengthening);
        let code = language.code();
        let name = |screen: &str| format!("mcp_{code}_{screen}_{suffix}");
        let port = free_port();
        let mut prefs = Prefs {
            ui_scale: scale,
            mcp_port: port,
            ..Prefs::default()
        };
        prefs.set_language(Some(language));
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        mac_native(&mut h);
        common::create_project(&mut h);
        demo_scene(&mut h);
        // Off.
        h.state_mut().modal = Some(Modal::Preferences);
        h.state_mut().preferences_tab = tp_app::state::PrefsTab::Ai;
        h.run();
        save(&mut h, &name("prefs_off"));
        // On, a client drawing.
        h.state_mut().prefs.mcp_enabled = true;
        h.run();
        let texture = {
            let p = &h.state().workspace().unwrap().project;
            let t = p.surfaces[0].template.as_ref().unwrap();
            format!("{}/{}", t.package_id, t.texture_id)
        };
        for x in [600, 1400, 2200] {
            mcp_call(
                &mut h,
                port,
                "add_shape",
                serde_json::json!({"texture": texture, "shape": "star", "x": x, "y": 3000,
                    "width": 600, "height": 600, "fill": "#F2B705"}),
            );
        }
        h.state_mut().preferences_tab = tp_app::state::PrefsTab::Ai;
        h.run();
        save(&mut h, &name("prefs_connected"));
        h.state_mut().modal = None;
        h.run();
        save(&mut h, &name("status_bar"));
        // The port taken by another program.
        h.state_mut().prefs.mcp_enabled = false;
        h.run();
        let taken = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        h.state_mut().prefs.mcp_port = taken.local_addr().unwrap().port();
        h.state_mut().prefs.mcp_enabled = true;
        h.state_mut().modal = Some(Modal::Preferences);
        h.state_mut().preferences_tab = tp_app::state::PrefsTab::Ai;
        h.run();
        save(&mut h, &name("prefs_port_in_use"));
        h.state_mut().modal = None;
        h.run();
        save(&mut h, &name("status_bar_port_in_use"));
        drop(taken);
    }
    tp_i18n::set_lengthening(0);
}
