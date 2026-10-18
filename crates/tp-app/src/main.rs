// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::process::ExitCode;

use tp_app::paths::{APP_NAME, AppDirs};
use tp_app::prefs::PrefsStore;
use tp_app::{AppState, TruckPaintApp};

fn main() -> ExitCode {
    let dirs = AppDirs::resolve();
    let _log_guard = tp_app::logging::init(dirs.as_ref().map(AppDirs::logs).as_deref());
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting {APP_NAME}");
    if dirs.is_none() {
        tracing::warn!("no home directory found: preferences will not be saved");
    }

    let store = dirs.as_ref().map(|d| PrefsStore::new(&d.config));
    // The preferences say how the window opens (its title bar).
    let state = AppState::new(store);
    let mode = state.title_bar_mode();
    tracing::info!(?mode, "title bar");
    let options = eframe::NativeOptions {
        viewport: tp_app::title_bar::initial_viewport(
            egui::ViewportBuilder::default()
                .with_title(APP_NAME)
                .with_app_id("truckpaint")
                .with_inner_size([1440.0, 900.0])
                .with_min_inner_size([960.0, 600.0]),
            mode,
        ),
        persist_window: true,
        persistence_path: dirs.as_ref().map(|d| d.config.join("window.ron")),
        // Text glyphs and image edges are drawn as raw triangles (egui only
        // smooths its own shapes): multisampling gives them smooth edges.
        multisampling: 4,
        ..Default::default()
    };

    let recovery_dir = dirs.as_ref().map(|d| d.recovery());
    let vehicles_dir = dirs.as_ref().map(|d| d.vehicles());
    let library_path = dirs.as_ref().map(|d| d.library());
    let result = eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| {
            Ok(Box::new(TruckPaintApp::new(
                cc,
                state,
                recovery_dir,
                vehicles_dir,
                library_path,
            )))
        }),
    );

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!(%err, "failed to start");
            let bin = std::env::args()
                .next()
                .unwrap_or_else(|| "truckpaint".into());
            let hint = if cfg!(target_os = "macos") {
                "Make sure macOS is up to date (Metal is required).".to_owned()
            } else {
                format!(
                    "Update your graphics drivers, or try the OpenGL backend:\n\n    WGPU_BACKEND=gl {bin}"
                )
            };
            eprintln!(
                "\n{APP_NAME} could not start: {err}\n\n\
                 This usually means no compatible graphics adapter was found.\n{hint}\n"
            );
            ExitCode::FAILURE
        }
    }
}
