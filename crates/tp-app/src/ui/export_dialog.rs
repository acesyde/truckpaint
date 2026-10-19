//! Export Texture dialog: settings, live preview, progress.

use egui::{
    Color32, CornerRadius, Key, Rect, RichText, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_core::document::Rgba;
use tp_i18n::tr;
use tp_render::DdsEncoding;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{SegmentedControl, primary_button, secondary_button};

use crate::dialog_flow::Purpose;
use crate::export::{
    ExportFormat, ExportJob, ExportOutcome, ExportSettings, PreviewJob, size_text,
};
use crate::file_dialogs::DialogRequest;
use crate::state::{AppState, Modal};

/// State of the open Export Texture dialog.
pub struct ExportDialog {
    pub settings: ExportSettings,
    preview: Option<egui::TextureHandle>,
    /// Settings the shown preview was rendered with.
    pub preview_for: Option<ExportSettings>,
    preview_job: Option<PreviewJob>,
    pub job: Option<ExportJob>,
    /// The file chosen in the save dialog, to export to.
    pub chosen: Option<std::path::PathBuf>,
}

impl ExportDialog {
    pub fn new(settings: ExportSettings) -> Self {
        Self {
            settings,
            preview: None,
            preview_for: None,
            preview_job: None,
            job: None,
            chosen: None,
        }
    }

    /// Whether a preview render matches the current settings.
    pub fn preview_ready(&self) -> bool {
        self.preview.is_some()
            && self
                .preview_for
                .is_some_and(|p| same_image(&p, &self.settings))
    }
}

/// Settings that change the rendered pixels (not the encoding).
fn same_image(a: &ExportSettings, b: &ExportSettings) -> bool {
    a.divisor == b.divisor && a.background == b.background
}

const PREVIEW: f32 = 300.0;

/// Shows the dialog; returns false when it should close.
pub fn show(ctx: &egui::Context, state: &mut AppState, dialog: &mut ExportDialog) -> bool {
    let Some(ws) = state.workspace_mut() else {
        return false;
    };
    // The active surface's own size, named after its vehicle, variant and
    // texture.
    let texture_size = ws.project.surface().size;
    let name = crate::export::export_name(&ws.project);
    let texture = format!(
        "{} · {} × {} px",
        ws.project.surface().name,
        texture_size.width.round(),
        texture_size.height.round()
    );

    // Preview: pick up a finished render, start one when settings changed.
    if let Some(job) = &dialog.preview_job
        && let Some(image) = job.poll()
    {
        let settings = job.settings;
        dialog.preview =
            Some(ctx.load_texture("export_preview", image, egui::TextureOptions::LINEAR));
        dialog.preview_for = Some(settings);
        dialog.preview_job = None;
    }
    let requested = dialog
        .preview_job
        .as_ref()
        .map(|j| j.settings)
        .or(dialog.preview_for);
    if requested.is_none_or(|r| !same_image(&r, &dialog.settings)) {
        let ctx2 = ctx.clone();
        dialog.preview_job = Some(PreviewJob::start(
            ws.project.clone(),
            dialog.settings,
            ws.text.fonts.fork(),
            move || ctx2.request_repaint(),
        ));
    }

    // A running export.
    if let Some(job) = &dialog.job
        && let Some(outcome) = job.poll()
    {
        dialog.job = None;
        match outcome {
            ExportOutcome::Written(path) => {
                let file = path
                    .file_name()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                let now = ctx.input(|i| i.time);
                ws.show_hint(tr!("export-done", file = file), now);
                return false;
            }
            ExportOutcome::Cancelled => {}
            ExportOutcome::Failed { path, reason } => {
                let file = path
                    .file_name()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                state.modal = Some(Modal::Message {
                    title: tr("export-failed-title"),
                    text: tr!("export-failed", file = file, reason = reason),
                });
                return false;
            }
        }
    }

    let mut keep = true;
    let mut start_export = false;
    let exporting = dialog.job.as_ref().map(|j| (j.progress(), j.path.clone()));
    super::dialogs::modal("export_modal").show(ctx, |ui| {
        ui.set_width(660.0);
        super::dialogs::title(ui, &tr("dialog-export-texture"), Some(&texture));
        ui.add_space(space::LG);
        ui.horizontal_top(|ui| {
            // Preview.
            let (area, response) = ui.allocate_exact_size(Vec2::splat(PREVIEW), Sense::hover());
            response
                .widget_info(|| WidgetInfo::labeled(WidgetType::Image, true, tr("export-preview")));
            // The texture's shape, never stretched.
            let rect = tp_ui::widgets::fit_rect(
                area,
                Vec2::new(texture_size.width as f32, texture_size.height as f32),
            );
            tp_ui::widgets::paint_checkerboard(&ui.painter_at(rect), rect, 10.0);
            match &dialog.preview {
                Some(texture) => {
                    ui.painter().image(
                        texture.id(),
                        rect,
                        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                None => {
                    // Painted, not added as a widget: adding one only while
                    // loading would shift the ids of the controls next to
                    // it, and a click landing when the preview arrives would
                    // be lost.
                    egui::Spinner::new()
                        .paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(24.0)));
                    ui.ctx().request_repaint();
                }
            }
            ui.painter().rect_stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, color::BORDER_STRONG),
                egui::StrokeKind::Outside,
            );
            ui.add_space(space::XL);

            ui.vertical(|ui| {
                ui.add_enabled_ui(exporting.is_none(), |ui| {
                    settings_ui(ui, &mut dialog.settings, texture_size)
                });
                ui.add_space(space::LG);
                let (width, height) = dialog.settings.size(texture_size);
                let (bytes, exact) = dialog.settings.estimated_bytes(texture_size);
                let format = match dialog.settings.format {
                    ExportFormat::Png => "PNG",
                    ExportFormat::Dds => match dialog.settings.dds {
                        DdsEncoding::Bc3 => "DDS (DXT5)",
                        DdsEncoding::Rgba => "DDS (RGBA)",
                    },
                };
                let info = tr!(
                    "export-info",
                    width = width.to_string(),
                    height = height.to_string(),
                    format = format,
                    bytes = size_text(bytes, exact)
                );
                let label = ui.label(RichText::new(&info).small().color(color::TEXT_SECONDARY));
                label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &info));
            });
        });
        match &exporting {
            Some((progress, path)) => {
                let file = path
                    .file_name()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                ui.add(egui::ProgressBar::new(*progress).text(tr!("export-progress", file = file)));
                super::dialogs::footer(ui, |ui| {
                    if ui.add(secondary_button(&tr("button-cancel"))).clicked()
                        && let Some(job) = &dialog.job
                    {
                        job.cancel();
                    }
                });
            }
            None => {
                super::dialogs::footer(ui, |ui| {
                    start_export |= ui.add(primary_button(&tr("export-start"))).clicked();
                    if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                        keep = false;
                    }
                });
            }
        }
    });
    if exporting.is_none() && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
        keep = false;
    }
    state.export_settings = dialog.settings;

    if start_export {
        let suggested = dialog.settings.suggested_name(&name);
        state.want_dialog(
            DialogRequest::save_export(&suggested),
            Purpose::ExportTexture,
        );
    }
    if let Some(path) = dialog.chosen.take() {
        let ext = dialog.settings.format.extension();
        let path = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
        {
            path
        } else {
            let mut p = path.into_os_string();
            p.push(".");
            p.push(ext);
            p.into()
        };
        if let Some(ws) = state.workspace_mut() {
            let ctx2 = ctx.clone();
            dialog.job = Some(ExportJob::start(
                ws.project.clone(),
                dialog.settings,
                ws.text.fonts.fork(),
                path,
                move || ctx2.request_repaint(),
            ));
        }
    }
    keep
}

fn settings_ui(ui: &mut Ui, settings: &mut ExportSettings, texture: tp_core::kurbo::Size) {
    super::dialogs::field_label(ui, &tr("export-format"));
    if let Some(format) = SegmentedControl::new()
        .segment(ExportFormat::Png, tp_ui::icons::IMAGE, "PNG", None)
        .segment(ExportFormat::Dds, tp_ui::icons::VEHICLE, "DDS", None)
        .show(ui, settings.format)
    {
        settings.format = format;
    }
    if settings.format == ExportFormat::Dds {
        ui.add_space(space::XS);
        let label = |e: DdsEncoding| match e {
            DdsEncoding::Bc3 => tr("export-dds-bc3"),
            DdsEncoding::Rgba => tr("export-dds-rgba"),
        };
        let combo = egui::ComboBox::from_id_salt("dds_encoding")
            .icon(tp_ui::widgets::dropdown_icon)
            .width(220.0)
            .selected_text(label(settings.dds))
            .show_ui(ui, |ui| {
                for e in [DdsEncoding::Bc3, DdsEncoding::Rgba] {
                    ui.selectable_value(&mut settings.dds, e, label(e));
                }
            });
        combo.response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::ComboBox, true, tr("export-dds-encoding"))
        });
    }
    ui.add_space(space::MD);
    super::dialogs::field_label(ui, &tr("export-size"));
    ui.horizontal(|ui| {
        for divisor in [1, 2, 4] {
            let (w, h) = ExportSettings {
                divisor,
                ..*settings
            }
            .size(texture);
            ui.radio_value(&mut settings.divisor, divisor, format!("{w} × {h}"));
        }
    });
    ui.add_space(space::MD);
    super::dialogs::field_label(ui, &tr("export-background"));
    ui.horizontal(|ui| {
        let mut transparent = settings.background.is_none();
        if ui
            .add(tp_ui::widgets::SquareCheckbox::new(
                &mut transparent,
                tr("export-transparent"),
            ))
            .changed()
        {
            settings.background = if transparent {
                None
            } else {
                Some(Rgba::rgb(255, 255, 255))
            };
        }
        if let Some(bg) = &mut settings.background {
            let mut c = Color32::from_rgb(bg.r, bg.g, bg.b);
            let response = ui.color_edit_button_srgba(&mut c);
            response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::ColorButton, true, tr("export-background-color"))
            });
            if response.changed() {
                *bg = Rgba::rgb(c.r(), c.g(), c.b());
            }
        }
    });
}
