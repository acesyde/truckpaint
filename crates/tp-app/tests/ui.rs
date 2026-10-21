//! Headless UI tests for the app-shell spec scenarios.

mod common;

use std::path::PathBuf;

use egui::accesskit::{Role, Toggled};
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::layout::{LeftTab, Space};
use tp_app::prefs::{Prefs, RecentProject};
use tp_app::state::{Modal, Screen};
use tp_app::tool::Tool;
use tp_ui::tokens::size;

use common::{create_project, harness, harness_with};

fn tool(h: &Harness<'static, AppState>) -> Tool {
    h.state().workspace().expect("workspace").tool
}

fn open_menu(h: &mut Harness<'static, AppState>, title: &str) {
    h.get_by_label(title).click();
    h.run();
}

// --- start-screen ---------------------------------------------------------

#[test]
fn home_screen_on_launch_with_empty_recent_state() {
    let h = harness();
    assert!(matches!(h.state().screen, Screen::Home));
    h.get_by_label("New Project");
    h.get_by_label("Open Project…");
    h.get_by_label("No recent projects");
}

#[test]
fn empty_recent_projects_centered_with_a_create_button() {
    let mut h = harness();
    h.run();
    // Centered on the list's width: the column of the title and the
    // Preferences button, not on a narrow box at its left.
    let title = h.get_by_label("Recent projects").rect();
    let preferences = h.get_by_label("Preferences").rect();
    let empty = h.get_by_label("No recent projects").rect();
    let middle = (title.left() + preferences.right()) / 2.0;
    assert!(
        (empty.center().x - middle).abs() < 2.0,
        "empty state at {}, list middle at {middle}",
        empty.center().x
    );
    h.get_by_label("Create Your First Project").click();
    h.run();
    assert!(h.query_by_label("Create Project").is_some());
}

#[test]
fn closing_project_returns_home() {
    let mut h = harness();
    create_project(&mut h);
    open_menu(&mut h, "File");
    // File › Close, under the title bar's Close control.
    h.get_all_by_label("Close")
        .max_by(|a, b| a.rect().top().total_cmp(&b.rect().top()))
        .unwrap()
        .click();
    h.run();
    // A new project has unsaved changes: the prompt comes first.
    h.get_by_label("Don't Save").click();
    h.run();
    assert!(matches!(h.state().screen, Screen::Home));
}

#[test]
fn missing_recent_file_can_be_removed() {
    let prefs = Prefs {
        recent: vec![RecentProject {
            name: "Gone".into(),
            path: PathBuf::from("/definitely/not/here.truckpaint"),
            last_opened: 0,
        }],
        ..Prefs::default()
    };
    let mut h = harness_with(prefs);
    h.get_by_label("Gone (file not found)");
    h.get_by_label("Remove from list").click();
    h.run();
    assert!(h.state().prefs.recent.is_empty());
    h.get_by_label("No recent projects");
}

/// Runs frames until the recent projects' thumbnails are read.
fn read_thumbnails(h: &mut Harness<'static, AppState>) {
    for _ in 0..2000 {
        h.step();
        if !h.state().recent_thumbnails.is_reading() {
            h.run();
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the thumbnails were never read");
}

#[test]
fn recent_cards_show_the_first_main_texture() {
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};

    let dir = tempfile::tempdir().unwrap();
    let package = tp_vehicles::Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let mut project =
        tp_app::vehicle_project::fleet_project("Red fleet", &[(&package, &textures)]).unwrap();
    // The Standard cab, first main texture, covered by a red rectangle.
    let side = project.surface().max_side();
    let mut o = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(
            Point::new(side / 2.0, side / 2.0),
            Size::new(side, side),
            0.0,
        ),
    );
    o.fill = Rgba::rgb(220, 20, 20).into();
    project.add(o);
    let red = dir.path().join("red.truckpaint");
    tp_file::write(&project, &red).unwrap();
    let broken = dir.path().join("broken.truckpaint");
    std::fs::write(&broken, b"not a project").unwrap();
    let entry = |name: &str, path: &std::path::Path| RecentProject {
        name: name.into(),
        path: path.to_path_buf(),
        last_opened: 0,
    };
    let prefs = Prefs {
        recent: vec![entry("Red fleet", &red), entry("Broken", &broken)],
        ..Prefs::default()
    };
    let mut h = harness_with(prefs);
    h.run();
    h.get_by_label("Red fleet");
    read_thumbnails(&mut h);
    let thumbnails = &h.state().recent_thumbnails;
    assert_eq!(
        thumbnails.color_at(&red, [0.5, 0.5]),
        Some(Rgba::rgb(220, 20, 20)),
        "the card's thumbnail is red"
    );
    assert!(thumbnails.texture(&red).is_some());
    // An unreadable file keeps the placeholder, and still opens on click.
    assert!(thumbnails.texture(&broken).is_none());
    h.get_by_label("Broken");
}

#[test]
fn cancelled_open_dialog_does_nothing() {
    let mut h = harness();
    h.get_by_label("Open Project…").click();
    h.run();
    assert!(matches!(h.state().screen, Screen::Home));
    assert!(h.state().modal.is_none());
}

#[test]
fn new_project_defaults_to_the_vehicle_name() {
    let mut h = harness();
    h.get_by_label("New Project").click();
    h.run();
    pick_sample(&mut h);
    h.get_by_label("Create Project").click();
    h.run();
    let ws = h.state().workspace().unwrap();
    assert_eq!(ws.project.name, "TruckPaint Sample Truck");
    assert_eq!(ws.project.surface().name, "Standard cab");
    assert_eq!(
        ws.project.surface().size,
        tp_core::kurbo::Size::new(4096.0, 4096.0)
    );
    assert_eq!(h.state().title(), "TruckPaint Sample Truck — TruckPaint");
}

/// Picks the sample vehicle in the New Project vehicle list.
fn pick_sample(h: &mut Harness<'static, AppState>) {
    h.get_by_role_and_label(Role::CheckBox, common::SAMPLE)
        .click();
    h.run();
}

#[test]
fn new_project_with_name_via_keyboard() {
    let mut h = harness();
    h.get_by_label("New Project").click();
    h.run();
    pick_sample(&mut h);
    h.get_by_role_and_label(Role::TextInput, "Project name")
        .click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Project name")
        .type_text("ACE Logistics");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    let ws = h.state().workspace().expect("created with Enter");
    assert_eq!(ws.project.name, "ACE Logistics");
    assert_eq!(ws.project.surfaces.len(), 4, "the Standard cab's textures");
    assert_eq!(ws.space, Space::Project, "opens in the Project space");
}

#[test]
fn escape_cancels_new_project() {
    let mut h = harness();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_label("Your fleet");
    h.key_press(Key::Escape);
    h.run();
    assert!(h.state().modal.is_none());
    assert!(matches!(h.state().screen, Screen::Home));
}

#[test]
fn new_project_dialog_tab_moves_focus() {
    let mut h = harness();
    h.get_by_label("New Project").click();
    h.run();
    assert!(
        h.get_by_role_and_label(Role::TextInput, "Project name")
            .is_focused(),
        "name field focused on open"
    );
    h.key_press(Key::Tab);
    h.run();
    assert!(
        !h.get_by_role_and_label(Role::TextInput, "Project name")
            .is_focused()
    );
}

// --- workspace-layout -----------------------------------------------------

#[test]
fn menus_are_in_order() {
    let mut h = harness();
    create_project(&mut h);
    let menus: Vec<f32> = tp_app::ui::menu_bar::MENUS
        .iter()
        .map(|title| {
            // Menu titles sit in the menu bar (a tab may share the label).
            h.get_all_by_label(&tp_i18n::tr(title))
                .map(|n| n.rect())
                .min_by(|a, b| a.top().total_cmp(&b.top()))
                .unwrap()
                .left()
        })
        .collect();
    assert!(menus.windows(2).all(|w| w[0] < w[1]), "{menus:?}");
    // No Export menu: its items are in File.
    assert!(h.query_by_label("Export").is_none());
}

/// workspace-layout: Export items in File.
#[test]
fn export_items_in_file() {
    let mut h = harness();
    create_project(&mut h);
    open_menu(&mut h, "File");
    let top = |label| h.get_by_role_and_label(Role::Button, label).rect().top();
    let (texture, export, quit) = (top("Export Texture…"), top("Export Mod…"), top("Quit"));
    assert!(
        texture < export && export < quit,
        "{texture} {export} {quit}"
    );
    use tp_app::commands::CommandId;
    let shortcut = |id: CommandId| id.shortcut().map(|s| (s.modifiers, s.logical_key));
    assert_eq!(
        shortcut(CommandId::ExportTexture),
        Some((Modifiers::COMMAND | Modifiers::SHIFT, Key::E))
    );
    assert_eq!(
        shortcut(CommandId::ExportMod),
        Some((Modifiers::COMMAND, Key::E))
    );
}

#[test]
fn disabled_menu_item_does_nothing() {
    let mut h = harness();
    create_project(&mut h);
    open_menu(&mut h, "Edit");
    h.get_by_label("Undo").click();
    h.run();
    assert!(h.state().has_project());
    assert!(h.state().modal.is_none());
}

#[test]
fn selection_is_default_and_clicking_rectangle_activates_it() {
    let mut h = harness();
    create_project(&mut h);
    assert_eq!(tool(&h), Tool::Select);
    h.get_by_label("Rectangle").click();
    h.run();
    assert_eq!(tool(&h), Tool::Rectangle);
}

#[test]
fn tool_buttons_have_minimum_hit_area() {
    let mut h = harness();
    create_project(&mut h);
    let rect = h.get_by_label("Ellipse").rect();
    assert!(rect.width() >= size::HIT_MIN && rect.height() >= size::HIT_MIN);
    let tab = h.get_by_role_and_label(Role::RadioButton, "Layers").rect();
    assert!(tab.width() >= size::HIT_MIN && tab.height() >= size::HIT_MIN);
}

#[test]
fn artboard_stays_visible_when_resizing() {
    let mut h = harness();
    create_project(&mut h);
    for size in [
        Vec2::new(1440.0, 900.0),
        Vec2::new(1000.0, 640.0),
        Vec2::new(1800.0, 700.0),
    ] {
        h.set_size(size);
        h.run();
        let canvas = h.get_by_label("Canvas").rect();
        let artboard = h.state().workspace().unwrap().artboard_rect(1.0).unwrap();
        assert!(canvas.contains_rect(artboard), "{size:?}");
        assert!((artboard.width() - artboard.height()).abs() < 0.5);
    }
}

#[test]
fn inspector_width_is_clamped() {
    let mut prefs = Prefs::default();
    prefs.layout.inspector_width = 5_000.0;
    let mut h = harness_with(prefs);
    create_project(&mut h);
    let width = h.state().prefs.layout.inspector_width;
    assert!(width <= size::INSPECTOR_MAX, "{width}");
}

#[test]
fn the_view_menu_has_no_3d_preview() {
    let mut h = harness();
    create_project(&mut h);
    h.get_by_label("Canvas");
    h.get_by_label("View").click();
    h.run();
    for item in ["Zoom In", "Actual Size", "Show Grid", "Show Guides"] {
        assert!(h.query_by_label_contains(item).is_some(), "{item}");
    }
    for gone in ["2D Canvas", "3D Preview", "Split View", "Show 3D Preview"] {
        assert!(h.query_by_label_contains(gone).is_none(), "{gone}");
    }
    // Nothing in the menu bar switches views any more.
    assert!(h.query_by_label("Split").is_none());
    assert!(h.query_by_label("3D").is_none());
}

#[test]
fn status_bar_shows_unsaved_for_new_project() {
    let mut h = harness();
    create_project(&mut h);
    h.get_by_label("Unsaved changes");
    h.get_by_label_contains("Zoom ");
}

#[test]
fn reset_workspace_restores_the_layout() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press(Key::Num3);
    h.run();
    h.key_press(Key::Tab);
    h.run();
    h.state_mut().prefs.layout.inspector_width = 400.0;
    let layout = h.state().prefs.layout.clone();
    assert!(layout.panels_hidden);
    assert_eq!(layout.left_tab, LeftTab::Resources);
    open_menu(&mut h, "View");
    h.get_by_label("Reset Workspace").click();
    h.run();
    let layout = &h.state().prefs.layout;
    assert!(!layout.panels_hidden);
    assert_eq!(layout.left_tab, LeftTab::Textures);
    assert_eq!(layout.left_width, size::LEFT_PANEL_DEFAULT);
    assert_eq!(layout.inspector_width, size::INSPECTOR_DEFAULT);
    assert_eq!(space(&h), Space::Workshop, "the space is kept");
}

// --- command-system -------------------------------------------------------

#[test]
fn menu_and_shortcut_trigger_same_command() {
    let mut h = harness();
    create_project(&mut h);
    open_menu(&mut h, "View");
    h.get_by_role_and_label(Role::CheckBox, "Layers").click();
    h.run();
    assert_eq!(h.state().prefs.layout.left_tab, LeftTab::Layers);
    h.state_mut().prefs.layout.left_tab = LeftTab::Textures;
    h.key_press(Key::Num2);
    h.run();
    assert_eq!(h.state().prefs.layout.left_tab, LeftTab::Layers);
}

fn space(h: &Harness<'static, AppState>) -> Space {
    h.state().workspace().expect("workspace").space
}

fn toggled(h: &Harness<'static, AppState>, label: &str) -> bool {
    h.get_by_role_and_label(Role::CheckBox, label)
        .accesskit_node()
        .toggled()
        .is_some_and(|t| t == Toggled::True)
}

#[test]
fn view_menu_lists_spaces_tabs_and_view_settings_in_order() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press(Key::Num2);
    h.run();
    open_menu(&mut h, "View");
    let toggles = [
        "Project",
        "Workshop",
        "Brand",
        "Textures",
        "Layers",
        "Resources",
        "Hide Panels",
        "Show Template",
        "Show Grid",
        "Show Guides",
    ];
    // The rows of the menu, not other controls of the same name.
    let left = h
        .get_by_role_and_label(Role::CheckBox, "Hide Panels")
        .rect()
        .left();
    let mut ys: Vec<f32> = toggles
        .iter()
        .map(|label| {
            h.query_all_by_role_and_label(Role::CheckBox, label)
                .map(|n| n.rect())
                .find(|r| (r.left() - left).abs() < 1.0)
                .expect(label)
                .center()
                .y
        })
        .collect();
    // Command Palette… comes first, with its shortcut.
    ys.insert(0, h.get_by_label("Command Palette…").rect().center().y);
    assert!(
        tp_app::commands::CommandId::CommandPalette
            .shortcut()
            .is_some()
    );
    ys.push(h.get_by_label("Clear Guides").rect().center().y);
    ys.push(
        h.query_all_by_role_and_label(Role::CheckBox, "Snapping")
            .map(|n| n.rect())
            .find(|r| (r.left() - left).abs() < 1.0)
            .expect("Snapping")
            .center()
            .y,
    );
    // Show Side View follows the snapping item, then the zoom commands.
    ys.push(
        h.query_all_by_role_and_label(Role::CheckBox, "Show Side View")
            .map(|n| n.rect())
            .find(|r| (r.left() - left).abs() < 1.0)
            .expect("Show Side View")
            .center()
            .y,
    );
    for label in [
        "Zoom In",
        "Zoom Out",
        "Fit to Screen",
        "Actual Size (100%)",
        "Reset Workspace",
    ] {
        ys.push(h.get_by_label(label).rect().center().y);
    }
    assert!(ys.windows(2).all(|w| w[0] < w[1]), "{ys:?}");
    assert!(toggled(&h, "Workshop") && toggled(&h, "Layers"));
    for label in ["Project", "Brand", "Textures", "Resources", "Hide Panels"] {
        assert!(!toggled(&h, label), "{label}");
    }
    for gone in ["Sidebar", "Colors", "Properties", "Assets"] {
        assert!(
            h.query_by_role_and_label(Role::CheckBox, gone).is_none(),
            "{gone}"
        );
    }
}

#[test]
fn no_sidebar_nor_vehicle_information() {
    let mut h = harness();
    create_project(&mut h);
    open_menu(&mut h, "Vehicle");
    assert!(h.query_by_label("Vehicle Information").is_none());
    h.key_press(Key::Escape);
    h.run();
    let before = h.state().prefs.layout.clone();
    for key in [Key::F5, Key::F6, Key::F7, Key::F8] {
        h.key_press(key);
        h.run();
        assert_eq!(h.state().prefs.layout, before, "{key:?}");
        assert!(h.state().queue.is_empty());
        assert!(h.state().modal.is_none());
    }
}

#[test]
fn spaces_by_key_keep_the_work() {
    let mut h = harness();
    create_project(&mut h);
    let id = h.state_mut().workspace_mut().unwrap().create_shape(
        tp_core::document::ShapeKind::rectangle(),
        tp_core::document::Frame::new(
            tp_core::kurbo::Point::new(500.0, 500.0),
            tp_core::kurbo::Size::new(100.0, 100.0),
            0.0,
        ),
        1.0,
    );
    h.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    h.run();
    let ws = |h: &Harness<'static, AppState>| {
        let ws = h.state().workspace().unwrap();
        (
            ws.selection.clone(),
            ws.viewport.unwrap().zoom,
            ws.project.active_surface,
        )
    };
    let before = ws(&h);
    assert_eq!(before.0, vec![id]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num3);
    h.run();
    assert_eq!(space(&h), Space::Brand);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    h.run();
    assert_eq!(space(&h), Space::Project);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num2);
    h.run();
    assert_eq!(space(&h), Space::Workshop);
    assert_eq!(ws(&h), before);
}

#[test]
fn tab_keys_from_another_space_show_the_workshop() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press(Key::Tab);
    h.run();
    assert!(h.state().prefs.layout.panels_hidden);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    h.run();
    h.key_press(Key::Num3);
    h.run();
    assert_eq!(space(&h), Space::Workshop);
    let layout = &h.state().prefs.layout;
    assert_eq!(layout.left_tab, LeftTab::Resources);
    assert!(!layout.panels_hidden, "the panels are shown again");
}

#[test]
fn tab_and_g_act_in_the_workshop_only() {
    let mut h = harness();
    create_project(&mut h);
    let template = |h: &Harness<'static, AppState>| {
        let ws = h.state().workspace().unwrap();
        ws.project.surface().template.as_ref().unwrap().visible
    };
    let shown = template(&h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    h.run();
    h.key_press(Key::Tab);
    h.run();
    h.key_press(Key::G);
    h.run();
    assert!(!h.state().prefs.layout.panels_hidden);
    assert_eq!(template(&h), shown);
    // Hide Panels is enabled only in the Workshop.
    open_menu(&mut h, "View");
    assert!(
        h.get_by_role_and_label(Role::CheckBox, "Hide Panels")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn hide_panels_with_tab() {
    let mut h = harness();
    create_project(&mut h);
    // The left panel's tabs and the inspector's Fill row.
    let panels = |h: &Harness<'static, AppState>| {
        let left = h.query_by_role_and_label(Role::RadioButton, "Layers");
        let inspector = h.query_by_label("Fill color");
        assert_eq!(left.is_some(), inspector.is_some());
        left.is_some()
    };
    assert!(panels(&h));
    h.key_press(Key::Tab);
    h.run();
    assert!(h.state().prefs.layout.panels_hidden);
    assert!(!panels(&h));
    // The tool rail and the bars stay, and no control took the keyboard
    // focus.
    h.get_by_label("Ellipse");
    h.get_by_label("Unsaved changes");
    assert!(h.ctx.memory(|m| m.focused()).is_none());
    h.key_press(Key::Tab);
    h.run();
    assert!(!h.state().prefs.layout.panels_hidden);
    assert!(panels(&h));
}

#[test]
fn single_keys_type_in_a_focused_field() {
    let mut h = harness();
    create_project(&mut h);
    // The hex field of the color popover.
    common::open_color_popover(&mut h, tp_app::workspace::ColorTarget::Fill);
    let field = "Hex color";
    h.get_by_role_and_label(Role::TextInput, field).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    for (key, text) in [
        (Key::Num1, "1"),
        (Key::Num2, "2"),
        (Key::G, "g"),
        (Key::Num3, "3"),
    ] {
        h.event(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        });
        h.event(egui::Event::Text(text.into()));
        h.run();
    }
    let value = h
        .get_by_role_and_label(Role::TextInput, field)
        .value()
        .unwrap_or_default();
    assert!(value.contains("12g3"), "{value:?}");
    assert_eq!(h.state().prefs.layout.left_tab, LeftTab::Textures);
    let ws = h.state().workspace().unwrap();
    assert!(!ws.project.surface().template.as_ref().unwrap().visible);
    // Tab leaves the field without hiding the panels.
    h.key_press(Key::Tab);
    h.run();
    assert!(!h.state().prefs.layout.panels_hidden);
}

#[test]
fn tab_in_a_dialog_moves_the_focus() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    h.run_steps(4);
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
    let focused = |h: &Harness<'static, AppState>| h.ctx.memory(|m| m.focused());
    h.key_press(Key::Tab);
    h.run_steps(4);
    let first = focused(&h).expect("a control of the dialog has the focus");
    h.key_press(Key::Tab);
    h.run_steps(4);
    assert_ne!(focused(&h), Some(first), "the focus moved on");
    assert!(!h.state().prefs.layout.panels_hidden);
}

#[test]
fn tool_shortcuts_and_temporary_hand() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press(Key::E);
    h.run();
    assert_eq!(tool(&h), Tool::Ellipse);
    h.key_press_modifiers(Modifiers::SHIFT, Key::I);
    h.run();
    assert_eq!(tool(&h), Tool::Image);
    h.key_press(Key::I);
    h.run();
    assert_eq!(tool(&h), Tool::Eyedropper);

    h.key_press(Key::R);
    h.run();
    h.key_down(Key::Space);
    h.run();
    assert_eq!(tool(&h), Tool::Hand);
    h.key_up(Key::Space);
    h.run();
    assert_eq!(tool(&h), Tool::Rectangle);
}

#[test]
fn plain_tool_key_does_not_fire_with_command_modifier() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::R);
    h.run();
    assert_eq!(tool(&h), Tool::Select);
}

#[test]
fn typing_in_text_field_does_not_switch_tools() {
    let mut h = common::builder().with_size(common::SIZE).build_ui_state(
        |ui, (state, text, frames): &mut (AppState, String, u32)| {
            state.show(ui);
            // The theme's fonts are usable from the second frame on.
            if *frames > 0 {
                egui::Window::new("Scratch").show(ui.ctx(), |ui| {
                    ui.add(egui::TextEdit::singleline(text).hint_text("scratch field"))
                        .widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Scratch")
                        });
                });
            }
            *frames += 1;
        },
        (
            AppState::with_prefs(Prefs::default(), None),
            String::new(),
            0,
        ),
    );
    h.run();
    h.state_mut().0.open_project(tp_core::Project::new(
        "Test",
        tp_core::TextureResolution::R2048,
    ));
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Scratch").focus();
    h.run();
    // A real keyboard sends both the key event and the text event.
    h.key_press(Key::R);
    h.get_by_role_and_label(Role::TextInput, "Scratch")
        .type_text("R");
    h.run();
    assert_eq!(h.state().1, "R");
    assert_eq!(h.state().0.workspace().unwrap().tool, Tool::Select);
}

#[test]
fn disabled_command_shortcut_does_nothing() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert!(h.state().has_project());
    assert!(h.state().modal.is_none());
}

// --- app-preferences / ui scaling -----------------------------------------

#[test]
fn preferences_reset_to_defaults() {
    let prefs = Prefs {
        ui_scale: 1.5,
        text_scale: 1.25,
        system_title_bar: true,
        ..Prefs::default()
    };
    let mut h = harness_with(prefs);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::Preferences)));
    h.get_by_label("Reset to defaults").click();
    h.run();
    assert_eq!(h.state().prefs.ui_scale, 1.0);
    assert_eq!(h.state().prefs.text_scale, 1.0);
    // The title bar choice stays.
    assert!(h.state().prefs.system_title_bar);
}

#[test]
fn ui_scale_applies_without_restart() {
    let mut h = harness();
    h.run();
    let ctx = h.ctx.clone();
    assert!((ctx.zoom_factor() - 1.0).abs() < 1e-4);
    let body_before = ctx.global_style().text_styles[&egui::TextStyle::Body].size;

    h.state_mut().prefs.ui_scale = 1.5;
    h.state_mut().prefs.text_scale = 1.2;
    h.run();
    assert!((ctx.zoom_factor() - 1.5).abs() < 1e-4);
    let body_after = ctx.global_style().text_styles[&egui::TextStyle::Body].size;
    assert!((body_after - body_before * 1.2).abs() < 1e-3);
}

#[test]
fn macos_menus_show_symbol_shortcuts() {
    let mut h = common::builder()
        .with_size(common::SIZE)
        .with_os(egui::os::OperatingSystem::Mac)
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(Prefs::default(), None),
        );
    create_project(&mut h);
    open_menu(&mut h, "Edit");
    let formatter = tp_app::commands::ShortcutFormatter::new(&h.ctx);
    let undo = tp_app::commands::CommandId::Undo;
    let redo = tp_app::commands::CommandId::Redo;
    assert_eq!(formatter.command(undo).as_deref(), Some("⌘Z"));
    assert_eq!(formatter.command(redo).as_deref(), Some("⇧⌘Z"));
}

#[test]
fn every_command_has_a_menu_entry() {
    use egui_kittest::kittest::By;
    use std::collections::HashSet;
    use tp_app::commands::CommandId;
    let mut h = harness();
    create_project(&mut h);
    let labels = |h: &Harness<'static, AppState>| -> HashSet<String> {
        h.query_all(
            By::new()
                .predicate(|n| matches!(n.role(), Role::Button | Role::CheckBox | Role::MenuItem)),
        )
        .filter_map(|n| n.accesskit_node().label())
        .collect()
    };
    // The rows each menu adds to the window (the same label may also be a
    // button elsewhere, e.g. a tab).
    let outside = labels(&h);
    let count =
        |h: &Harness<'static, AppState>, label: &str| h.query_all(By::new().label(label)).count();
    let mut menus = HashSet::new();
    let mut add = |h: &Harness<'static, AppState>| {
        for label in labels(h) {
            if !outside.contains(&label) || count(h, &label) > 1 {
                menus.insert(label);
            }
        }
    };
    for title in tp_app::ui::menu_bar::MENUS {
        open_menu(&mut h, &tp_i18n::tr(title));
        add(&h);
        for sub in ["menu-combine", "menu-align"] {
            let label = tp_i18n::tr(sub);
            if let Some(node) = h.query_by_label(&label) {
                node.click();
                h.run();
                add(&h);
            }
        }
        h.key_press(Key::Escape);
        h.run();
    }
    let missing: Vec<CommandId> = CommandId::all()
        .into_iter()
        .filter(|id| {
            // Tools are in the tool bar; nudging, the color keys and Done
            // Editing Symbol have no menu, as before.
            !matches!(
                id,
                CommandId::SelectTool(_)
                    | CommandId::Nudge(..)
                    | CommandId::SwapColorTarget
                    | CommandId::SwapFillStroke
                    | CommandId::DefaultColors
                    | CommandId::FinishSymbol
            )
        })
        // The design gallery is in the Help menu of debug builds only.
        .filter(|id| cfg!(debug_assertions) || *id != CommandId::DesignGallery)
        .filter(|id| !menus.contains(&tp_i18n::tr(id.meta().label)))
        .collect();
    assert!(missing.is_empty(), "{missing:?}");
}

#[test]
fn keyboard_shortcuts_window_lists_the_workspace_keys() {
    use tp_app::commands::{CommandId, ShortcutFormatter};
    let mut h = harness();
    create_project(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Slash);
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::KeyboardShortcuts)));
    let formatter = ShortcutFormatter::new(&h.ctx);
    for id in [
        CommandId::ShowSpace(Space::Project),
        CommandId::ShowSpace(Space::Brand),
        CommandId::ShowLeftTab(LeftTab::Resources),
        CommandId::TogglePanels,
        CommandId::ActualSize,
        CommandId::FitToScreen,
        CommandId::BringForward,
        CommandId::NextTexture,
        CommandId::PreviousTexture,
        CommandId::ShowTemplate,
        CommandId::SelectTool(Tool::Gradient),
        CommandId::ExportMod,
        CommandId::ExportTexture,
    ] {
        let keys: Vec<String> = id
            .meta()
            .shortcuts
            .iter()
            .map(|s| formatter.format(s))
            .collect();
        let keys = keys.join(", ");
        assert!(h.query_all_by_label(&keys).count() >= 1, "{id:?}: {keys}");
        let label = tp_i18n::tr(id.meta().label);
        assert!(h.query_all_by_label(&label).count() >= 1, "{label}");
    }
}

// --- ui-design-system: Checkboxes -------------------------------------------

/// Every checkbox of the app is the square `SquareCheckbox`: egui's
/// `Checkbox`, which the theme's corner radius turns into a circle, is not
/// used in tp-app.
#[test]
fn no_egui_checkbox_in_the_app() {
    fn rust_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    rust_files(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 50, "only {} files found", files.len());
    let mut found = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        for (n, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            let egui_box = line.match_indices("Checkbox").any(|(at, _)| {
                let before = &line[..at];
                !before.ends_with("Square") && !before.ends_with("WidgetType::")
            });
            if egui_box || line.contains(".checkbox(") {
                found.push(format!("{}:{}: {}", file.display(), n + 1, line.trim()));
            }
        }
    }
    assert!(found.is_empty(), "egui checkboxes: {found:#?}");
}
