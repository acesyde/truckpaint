//! The small boxes shown over the window while a project file is read (once
//! the read has lasted a little: a quick open shows nothing) and while a file
//! dialog is open.

use egui::RichText;
use tp_i18n::tr;
use tp_ui::tokens::{color, space};

use super::dialogs::modal;
use crate::open_job::OpenJob;
use crate::project_io::file_name;

/// Widest the box gets, in points.
const MAX_WIDTH: f32 = 420.0;

/// Shows the box for `job` if its delay has passed.
pub fn show(ctx: &egui::Context, job: Option<&OpenJob>) {
    let Some(job) = job else {
        return;
    };
    if !job.box_shown(ctx.input(|i| i.time)) {
        return;
    }
    busy_box(
        ctx,
        "opening_project_modal",
        &tr!("opening-project", name = file_name(&job.path)),
    );
}

/// The box shown while a file dialog is open: what the application is
/// waiting for, in case the dialog is hidden behind the window.
pub fn waiting_for_dialog(ctx: &egui::Context) {
    busy_box(ctx, "file_dialog_modal", &tr("file-dialog-waiting"));
}

/// A small modal with a spinner and `text`.
fn busy_box(ctx: &egui::Context, id: &str, text: &str) {
    modal(id).show(ctx, |ui| {
        // A very long file name ends with "…" instead of widening the box.
        ui.set_max_width(MAX_WIDTH);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::MD;
            ui.add(egui::Spinner::new().color(color::TEXT_SECONDARY));
            let label =
                ui.add(egui::Label::new(RichText::new(text).color(color::TEXT_PRIMARY)).truncate());
            label.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
        });
    });
}
