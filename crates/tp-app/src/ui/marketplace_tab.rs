//! The Marketplace tab of the Vehicle Library: the catalog's vehicles, to
//! check and install or update, with the legal notice always in view.

use std::collections::BTreeSet;

use egui::{Align, Layout, RichText, ScrollArea, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{SquareCheckbox, primary_button, secondary_button};
use tp_vehicles::catalog::{Catalog, Entry};

use super::vehicle_dialogs::{VehicleFilter, game_label, kind_label};
use crate::marketplace::{Failure, Marketplace};
use crate::vehicles::VehicleLibrary;

/// A catalog vehicle's status for this app and the installed vehicles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarketStatus {
    Install,
    /// A lower version is installed.
    Update,
    Installed,
    /// Every version needs a newer TruckPaint.
    TooNew,
}

impl MarketStatus {
    fn can_install(self) -> bool {
        matches!(self, Self::Install | Self::Update)
    }

    /// Icon, text and color: the icon and the text say it.
    fn look(self) -> (&'static str, String, egui::Color32) {
        match self {
            Self::Install => (icons::DOWNLOAD, tr("market-status-install"), color::LINK),
            Self::Update => (icons::UPDATE, tr("vehicles-status-update"), color::SIGNAL),
            Self::Installed => (
                icons::CHECK,
                tr("market-status-installed"),
                color::TEXT_SECONDARY,
            ),
            Self::TooNew => (icons::WARNING, tr("market-status-too-new"), color::WARNING),
        }
    }
}

/// A catalog vehicle as listed: the version offered (the newest this app
/// can install, else the newest) and its status.
pub struct MarketRow<'a> {
    pub entry: &'a Entry,
    pub status: MarketStatus,
}

/// This app's version, compared with the entries' `min_app`.
pub fn app_version() -> semver::Version {
    env!("CARGO_PKG_VERSION")
        .parse()
        .unwrap_or_else(|_| semver::Version::new(0, 0, 0))
}

/// The catalog's vehicles passing `filter`, one row each.
pub fn rows<'a>(
    catalog: &'a Catalog,
    library: &VehicleLibrary,
    filter: &VehicleFilter,
) -> Vec<MarketRow<'a>> {
    let app = app_version();
    catalog
        .ids()
        .into_iter()
        .filter_map(|id| {
            let compatible = catalog.newest_compatible(id, &app);
            let entry = compatible.or_else(|| catalog.versions(id).into_iter().next())?;
            let status = match (compatible, library.get(id)) {
                (None, _) => MarketStatus::TooNew,
                (Some(_), None) => MarketStatus::Install,
                (Some(e), Some(v)) if v.newest().manifest.version < e.version => {
                    MarketStatus::Update
                }
                (Some(_), Some(_)) => MarketStatus::Installed,
            };
            filter
                .matches_vehicle(&entry.name, &entry.brand, entry.game, entry.kind)
                .then_some(MarketRow { entry, status })
        })
        .collect()
}

/// "12.3 MB".
pub fn size_text(bytes: u64) -> String {
    let mb = bytes as f64 / 1_000_000.0;
    if mb < 0.1 {
        tr!("market-size-mb", size = "0.1")
    } else {
        tr!("market-size-mb", size = format!("{mb:.1}"))
    }
}

/// What the player asked for in a frame.
#[derive(Default)]
pub struct MarketClicks {
    pub refresh: bool,
    pub cancel: bool,
    /// Vehicle ids to install or update.
    pub install: Vec<String>,
}

/// The message for a failed download.
pub fn failure_text(failure: &Failure) -> String {
    match failure {
        Failure::Unreachable(reason) => tr!("market-unreachable", reason = reason.as_str()),
        Failure::Invalid(reason) => tr!("market-invalid", reason = reason.as_str()),
        Failure::NewerApp => tr("market-newer-app"),
        Failure::Damaged => tr("market-damaged"),
    }
}

/// The tab's body, `height` tall: notice, status, list and detail pane.
#[allow(clippy::too_many_arguments)]
pub fn body(
    ui: &mut Ui,
    market: &Marketplace,
    library: &VehicleLibrary,
    filter: &mut VehicleFilter,
    selected: &mut Option<String>,
    checked: &mut BTreeSet<String>,
    height: f32,
    detail_width: f32,
    clicks: &mut MarketClicks,
) {
    notice_line(
        ui,
        icons::INFO,
        &tr("market-legal-notice"),
        color::TEXT_SECONDARY,
    );
    ui.add_space(space::XS);
    status_line(ui, market, clicks);
    ui.add_space(space::SM);
    let Some(catalog) = &market.catalog else {
        ui.allocate_space(egui::vec2(ui.available_width(), height - 60.0));
        return;
    };
    let can_install = market.can_install();
    let listed = rows(catalog, library, filter);
    let shown = listed
        .iter()
        .find(|r| selected.as_deref() == Some(r.entry.id.as_str()))
        .or(listed.first());
    let list_height = height - 60.0;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::XL;
        let list_width = ui.available_width() - detail_width - space::XL;
        ui.allocate_ui_with_layout(
            egui::vec2(list_width, list_height),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(list_width);
                ui.set_min_height(list_height);
                filter.show(ui, "marketplace");
                ui.add_space(space::SM);
                if listed.is_empty() {
                    ui.label(
                        RichText::new(tr("new-project-no-match")).color(color::TEXT_SECONDARY),
                    );
                    return;
                }
                ScrollArea::vertical()
                    .id_salt("marketplace_list")
                    .max_height(list_height - 90.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for row in &listed {
                            let is_shown = shown.is_some_and(|s| s.entry.id == row.entry.id);
                            if market_row(ui, row, is_shown, can_install, checked) {
                                *selected = Some(row.entry.id.clone());
                            }
                        }
                    });
                ui.add_space(space::SM);
                let picked: Vec<String> = listed
                    .iter()
                    .filter(|r| r.status.can_install() && checked.contains(&r.entry.id))
                    .map(|r| r.entry.id.clone())
                    .collect();
                let label = tr!("market-install-checked", count = picked.len());
                if ui
                    .add_enabled(can_install && !picked.is_empty(), primary_button(&label))
                    .clicked()
                {
                    clicks.install = picked;
                }
            },
        );
        ui.allocate_ui_with_layout(
            egui::vec2(detail_width, list_height),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(detail_width);
                if let Some(row) = shown {
                    detail(ui, catalog, row, can_install, clicks);
                }
            },
        );
    });
}

fn notice_line(ui: &mut Ui, icon: &str, text: &str, tint: egui::Color32) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.label(icons::rich(icon).color(tint));
        ui.add(egui::Label::new(RichText::new(text).small().color(tint)).wrap())
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    });
}

/// Loading, offline, failure or progress, with Refresh, Retry or Cancel.
fn status_line(ui: &mut Ui, market: &Marketplace, clicks: &mut MarketClicks) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        if let Some(job) = &market.installing {
            ui.add(egui::Spinner::new());
            ui.label(tr!(
                "market-installing",
                done = (job.done + 1).min(job.total),
                total = job.total
            ));
            clicks.cancel |= ui.add(secondary_button(&tr("button-cancel"))).clicked();
            return;
        }
        if market.loading {
            ui.add(egui::Spinner::new());
            ui.label(RichText::new(tr("market-loading")).color(color::TEXT_SECONDARY));
            return;
        }
        let retry = match (&market.offline_since, &market.failure, &market.catalog) {
            (Some(saved), _, _) => {
                let now = std::time::SystemTime::now();
                let secs = |t: std::time::SystemTime| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs())
                };
                let when = super::home::relative_time(secs(now), secs(*saved));
                ui.label(icons::rich(icons::OFFLINE).color(color::WARNING));
                ui.label(
                    RichText::new(tr!("market-offline", when = when.as_str()))
                        .color(color::WARNING),
                );
                true
            }
            (None, Some(failure), None) => {
                ui.label(icons::rich(icons::WARNING).color(color::ERROR));
                let text = failure_text(failure);
                ui.add(egui::Label::new(RichText::new(&text).color(color::ERROR)).wrap())
                    .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
                true
            }
            (None, Some(failure), Some(_)) => {
                ui.label(icons::rich(icons::WARNING).color(color::ERROR));
                ui.label(RichText::new(failure_text(failure)).color(color::ERROR));
                true
            }
            (None, None, _) => false,
        };
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = tr(if retry {
                "market-retry"
            } else {
                "market-refresh"
            });
            clicks.refresh |= ui.add(secondary_button(&label)).clicked();
        });
    });
}

/// A list row: checkbox, name, brand · kind · game, version, size and
/// status. Returns whether the row was clicked (to show it).
fn market_row(
    ui: &mut Ui,
    row: &MarketRow,
    shown: bool,
    can_install: bool,
    checked: &mut BTreeSet<String>,
) -> bool {
    let e = row.entry;
    let mut clicked = false;
    let frame = egui::Frame::new()
        .fill(if shown {
            color::CONTROL
        } else {
            egui::Color32::TRANSPARENT
        })
        .inner_margin(egui::Margin::symmetric(space::SM as i8, space::XS as i8));
    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let mut on = checked.contains(&e.id);
            let enabled = can_install && row.status.can_install();
            if ui
                .add_enabled(enabled, SquareCheckbox::new(&mut on, ""))
                .changed()
            {
                if on {
                    checked.insert(e.id.clone());
                } else {
                    checked.remove(&e.id);
                }
            }
            let name = ui.add(
                egui::Label::new(RichText::new(&e.name).color(if shown {
                    color::TEXT_PRIMARY
                } else {
                    color::TEXT_SECONDARY
                }))
                .sense(egui::Sense::click()),
            );
            name.widget_info(|| {
                WidgetInfo::selected(WidgetType::RadioButton, true, shown, &e.name)
            });
            clicked |= name.clicked();
            ui.label(
                RichText::new(format!(
                    "{} · {} · {}",
                    e.brand,
                    kind_label(e.kind),
                    game_label(e.game)
                ))
                .small()
                .color(color::TEXT_MUTED),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let (icon, text, tint) = row.status.look();
                ui.label(RichText::new(&text).small().color(tint));
                ui.label(icons::rich(icon).color(tint));
                ui.label(
                    RichText::new(size_text(e.size))
                        .small()
                        .color(color::TEXT_MUTED),
                );
                ui.label(
                    RichText::new(e.version.to_string())
                        .monospace()
                        .color(color::TEXT_SECONDARY),
                );
            });
        });
    });
    clicked
}

/// The detail pane: what the vehicle is, its rights, changelog and older
/// versions, and Install or Update.
fn detail(
    ui: &mut Ui,
    catalog: &Catalog,
    row: &MarketRow,
    can_install: bool,
    clicks: &mut MarketClicks,
) {
    let e = row.entry;
    ui.label(RichText::new(&e.name).heading().color(color::TEXT_PRIMARY));
    ui.label(
        RichText::new(format!(
            "{} · {} · {} · {}",
            e.brand,
            kind_label(e.kind),
            game_label(e.game),
            size_text(e.size)
        ))
        .color(color::TEXT_SECONDARY),
    );
    ui.add_space(space::SM);
    let (icon, text, tint) = row.status.look();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.label(icons::rich(icon).color(tint));
        ui.label(RichText::new(text).color(tint));
    });
    if row.status.can_install() {
        let label = tr!(
            if row.status == MarketStatus::Update {
                "market-update-to"
            } else {
                "market-install-version"
            },
            version = e.version.to_string()
        );
        if ui
            .add_enabled(can_install, primary_button(&label))
            .clicked()
        {
            clicks.install = vec![e.id.clone()];
        }
    }
    ui.add_space(space::SM);
    let field = |ui: &mut Ui, label: &str, value: &str| {
        ui.label(RichText::new(label).small().color(color::TEXT_MUTED));
        ui.add(egui::Label::new(RichText::new(value).color(color::TEXT_SECONDARY)).wrap());
    };
    field(ui, &tr("market-license"), &e.license);
    if !e.authors.is_empty() {
        field(ui, &tr("market-authors"), &e.authors.join(", "));
    }
    if let Some(changelog) = &e.changelog {
        field(ui, &tr("market-changelog"), changelog);
    }
    if let Some(notice) = &e.notice {
        ui.add_space(space::XS);
        notice_line(ui, icons::INFO, notice, color::TEXT_SECONDARY);
    }
    let older: Vec<String> = catalog
        .versions(&e.id)
        .into_iter()
        .filter(|v| v.version < e.version)
        .map(|v| v.version.to_string())
        .collect();
    if !older.is_empty() {
        field(ui, &tr("market-older-versions"), &older.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, version: &str, min_app: &str) -> Entry {
        Entry {
            id: id.into(),
            version: version.parse().unwrap(),
            name: id.into(),
            brand: "Volvo".into(),
            kind: tp_vehicles::Kind::Truck,
            game: tp_vehicles::Game::Ets2,
            size: 1_234_567,
            sha256: String::new(),
            url: String::new(),
            license: "MIT".into(),
            min_app: min_app.parse().unwrap(),
            game_versions: None,
            preview: None,
            authors: Vec::new(),
            changelog: None,
            notice: None,
        }
    }

    #[test]
    fn statuses_and_filter() {
        let dir = tempfile::tempdir().unwrap();
        let mut library = VehicleLibrary::open(dir.path());
        library
            .install_bytes(&tp_vehicles::sample::package(
                "a.installed",
                "Old",
                "1.0.0",
                &tp_vehicles::sample::truck_textures(),
            ))
            .unwrap();
        library
            .install_bytes(&tp_vehicles::sample::package(
                "a.current",
                "Same",
                "1.0.0",
                &tp_vehicles::sample::truck_textures(),
            ))
            .unwrap();
        let catalog = Catalog {
            schema: 1,
            packages: vec![
                entry("a.new", "1.0.0", "0.1.0"),
                entry("a.installed", "1.1.0", "0.1.0"),
                entry("a.current", "1.0.0", "0.1.0"),
                entry("a.future", "1.0.0", "99.0.0"),
            ],
        };
        let status = |filter: &VehicleFilter| -> Vec<(String, MarketStatus)> {
            rows(&catalog, &library, filter)
                .into_iter()
                .map(|r| (r.entry.id.clone(), r.status))
                .collect()
        };
        assert_eq!(
            status(&VehicleFilter::default()),
            [
                ("a.new".into(), MarketStatus::Install),
                ("a.installed".into(), MarketStatus::Update),
                ("a.current".into(), MarketStatus::Installed),
                ("a.future".into(), MarketStatus::TooNew),
            ]
        );
        let filter = VehicleFilter {
            query: "FUTURE".into(),
            ..Default::default()
        };
        assert_eq!(status(&filter).len(), 1);
        assert_eq!(size_text(1_234_567), "1.2 MB");
    }
}
