//! The small box shown over the window while a project file is read, once
//! the read has lasted a little (a quick open shows nothing).

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
    let text = tr!("opening-project", name = file_name(&job.path));
    modal("opening_project_modal").show(ctx, |ui| {
        // A very long file name ends with "…" instead of widening the box.
        ui.set_max_width(MAX_WIDTH);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::MD;
            ui.add(egui::Spinner::new().color(color::TEXT_SECONDARY));
            let label = ui
                .add(egui::Label::new(RichText::new(&text).color(color::TEXT_PRIMARY)).truncate());
            label.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &text));
        });
    });
}
