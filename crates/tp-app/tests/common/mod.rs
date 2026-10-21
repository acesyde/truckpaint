//! Shared helpers for UI tests.

#![allow(dead_code)]

pub mod blue_wave;
pub mod scenes;

use egui::Vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::layout::{LeftTab, Space};
use tp_app::prefs::Prefs;
use tp_app::vehicles::VehicleLibrary;
use tp_app::workspace::ColorTarget;
use tp_vehicles::Package;

pub const SIZE: Vec2 = Vec2::new(1440.0, 900.0);
/// Name of the built-in sample vehicle every test project is made for.
pub const SAMPLE: &str = "TruckPaint Sample Truck";

/// A vehicle library in a temporary folder (kept for the whole test run)
/// with the built-in sample truck and trailer installed.
pub fn sample_library() -> VehicleLibrary {
    let dir = tempfile::tempdir().expect("temporary folder").keep();
    let mut library = VehicleLibrary::open(&dir);
    library.install_samples().expect("sample vehicles");
    library
}

/// The application with in-memory preferences and the sample vehicles
/// installed.
fn state(prefs: Prefs) -> AppState {
    let mut state = AppState::with_prefs(prefs, None);
    state.vehicles = sample_library();
    state
}

/// Frames `Harness::run` may take before it gives up. Thumbnails and mod
/// pictures render on worker threads and ask for a repaint when done; on a
/// slow CI machine that can land a few frames later than egui's default of
/// 4 allows. A UI that never settles still fails, just later.
const MAX_STEPS: u64 = 32;

/// `Harness::builder()` with the frame budget above: test files that build
/// their own harness start from this.
pub fn builder<S>() -> egui_kittest::HarnessBuilder<S> {
    Harness::builder().with_max_steps(MAX_STEPS)
}

/// Harness running the whole application with in-memory preferences.
pub fn harness_with(prefs: Prefs) -> Harness<'static, AppState> {
    builder()
        .with_size(SIZE)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state(prefs))
}

pub fn harness() -> Harness<'static, AppState> {
    harness_with(Prefs::default())
}

/// Same, rendering with wgpu (for screenshots).
pub fn wgpu_harness_with(prefs: Prefs, size: Vec2) -> Harness<'static, AppState> {
    builder()
        .with_size(size)
        .wgpu()
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state(prefs))
}

/// Same, at `pixels_per_point`, with each step advancing time by `step_dt`
/// seconds (documentation media: sharp images, animations at a fixed frame
/// rate).
pub fn wgpu_harness_hidpi(
    prefs: Prefs,
    size: Vec2,
    pixels_per_point: f32,
    step_dt: f32,
) -> Harness<'static, AppState> {
    builder()
        .with_size(size)
        .with_pixels_per_point(pixels_per_point)
        .with_step_dt(step_dt)
        .wgpu()
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state(prefs))
}

/// Opens a project for the sample truck with its default textures, as New
/// Project would make it: the Standard cab (4096 px), Chassis, Cab
/// accessories and Side skirts; the Standard cab is active. New Project
/// shows the Project space; this then shows the Workshop, where most tests
/// work (`open_project` keeps the Project space). Its templates are hidden:
/// the sample's SVG templates render in a background thread whose repaints
/// would make editing tests depend on timing (tests/vehicles.rs covers
/// templates and the wizard).
pub fn create_project(harness: &mut Harness<'static, AppState>) {
    let has_sample = harness
        .state()
        .vehicles
        .get(tp_app::vehicles::SAMPLE_ID)
        .is_some();
    if !has_sample {
        harness.state_mut().vehicles = sample_library();
    }
    let package = Package::read(tp_app::vehicles::SAMPLES[0].bytes).expect("sample package");
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let mut project = tp_app::vehicle_project::fleet_project(SAMPLE, &[(&package, &textures)])
        .expect("sample textures");
    for surface in &mut project.surfaces {
        if let Some(t) = surface.template.as_mut() {
            t.visible = false;
        }
    }
    open_project(harness, project);
    harness.run();
    assert!(harness.state().has_project(), "project should be open");
    settle_renders(harness);
}

/// Runs frames until the background renders of the open project (texture
/// and template thumbnails, the before/after editor's previews, the mod's
/// pictures, the canvas's shadows) are shown, so that their
/// repaints don't land in the middle of a test's interaction.
pub fn settle_renders(harness: &mut Harness<'static, AppState>) {
    // A time budget, not a frame count: CI machines render much slower.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while std::time::Instant::now() < deadline {
        harness.step();
        let busy = harness.state().workspace().is_some_and(|ws| {
            ws.thumbnails.is_rendering()
                || ws.preview_thumbs.is_rendering()
                || ws.mod_previews.is_rendering()
                || ws.shadows.is_rendering()
                || ws.side_view.is_rendering()
        });
        if !busy {
            harness.run();
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let ws = harness.state().workspace().unwrap();
    panic!(
        "the background renders never finished: thumbnails {} previews {} mod {} shadows {}",
        ws.thumbnails.is_rendering(),
        ws.preview_thumbs.is_rendering(),
        ws.mod_previews.is_rendering(),
        ws.shadows.is_rendering()
    );
}

/// Opens `project` and shows the Workshop.
pub fn open_project(harness: &mut Harness<'static, AppState>, project: tp_core::Project) {
    harness.state_mut().open_project(project);
    show_space(harness, Space::Workshop);
}

/// Shows `space` of the open project.
pub fn show_space(harness: &mut Harness<'static, AppState>, space: Space) {
    harness
        .state_mut()
        .workspace_mut()
        .expect("open project")
        .space = space;
}

/// `prefs` with the left panel showing `tab`.
pub fn prefs_with_tab(mut prefs: Prefs, tab: LeftTab) -> Prefs {
    prefs.layout.left_tab = tab;
    prefs
}

/// Shows `tab` in the left panel.
pub fn show_tab(harness: &mut Harness<'static, AppState>, tab: LeftTab) {
    harness.state_mut().prefs.layout.left_tab = tab;
    harness.run();
}

/// The last node labelled `label`: the one of a menu or dialog drawn over
/// the workspace when the workspace has one with the same label (the top
/// bar's Export…, the inspector's align and flip buttons).
pub fn last<'h>(harness: &'h Harness<'static, AppState>, label: &'h str) -> egui_kittest::Node<'h> {
    harness
        .get_all_by_label(label)
        .last()
        .unwrap_or_else(|| panic!("no node labelled {label:?}"))
}

/// Whether the inspector's color popover is open.
pub fn color_popover_open(harness: &Harness<'static, AppState>) -> bool {
    tp_ui::widgets::Popover::is_open(
        &harness.ctx,
        tp_app::ui::workspace::inspector::color_popover(),
    )
}

/// Opens the color popover on the inspector's Fill or Stroke row, as
/// clicking the row's swatch does (the row becomes the color target).
pub fn open_color_popover(harness: &mut Harness<'static, AppState>, target: ColorTarget) {
    let current = harness
        .state()
        .workspace()
        .expect("open project")
        .panels
        .color_target;
    if color_popover_open(harness) && current == target {
        return;
    }
    let row = match target {
        ColorTarget::Fill => "Fill color",
        ColorTarget::Stroke => "Stroke color",
    };
    harness.get_by_label(row).click();
    harness.run();
}

/// Opens the stroke popover from the inspector's Stroke row summary.
pub fn open_stroke_popover(harness: &mut Harness<'static, AppState>) {
    let id = tp_app::ui::workspace::inspector::stroke_popover();
    if !tp_ui::widgets::Popover::is_open(&harness.ctx, id) {
        harness.get_by_label("Stroke options").click();
        harness.run();
    }
}

/// Opens the line settings popover (dashes, caps and joins of lines).
pub fn open_line_settings(harness: &mut Harness<'static, AppState>) {
    let id = tp_app::ui::workspace::inspector::line_popover();
    if !tp_ui::widgets::Popover::is_open(&harness.ctx, id) {
        harness.get_by_label("Line settings").click();
        harness.run();
    }
}

/// The window commands (`ViewportCommand`) the last frame sent.
pub fn viewport_commands(harness: &Harness<'static, AppState>) -> Vec<egui::ViewportCommand> {
    harness
        .output()
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|o| o.commands.clone())
        .unwrap_or_default()
}
