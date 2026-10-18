//! Preferences in three tabs: the tab it opens on, the size it keeps, and
//! how it fits the smallest window and the longest language.

mod common;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::prefs::Prefs;
use tp_app::state::{Modal, PrefsTab};

type H = Harness<'static, AppState>;

fn open_with(prefs: Prefs, size: Vec2) -> H {
    let state = AppState::with_prefs(prefs, None);
    let mut h = common::builder()
        .with_size(size)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::Preferences)));
    h
}

fn open() -> H {
    open_with(Prefs::default(), Vec2::new(1440.0, 900.0))
}

fn pick(h: &mut H, tab: &str) {
    h.get_by_role_and_label(Role::RadioButton, tab).click();
    h.run();
}

#[test]
fn opens_on_general() {
    let h = open();
    assert_eq!(h.state().preferences_tab, PrefsTab::General);
    assert!(
        h.query_by_role_and_label(Role::SpinButton, "Grid spacing")
            .is_some()
    );
    assert!(h.query_by_label("Reset to defaults").is_some());
    assert!(h.query_by_label("I play this game").is_none());
}

#[test]
fn each_tab_shows_its_settings() {
    let mut h = open();
    pick(&mut h, "Games");
    assert_eq!(h.state().preferences_tab, PrefsTab::Games);
    assert_eq!(h.query_all_by_label("I play this game").count(), 2);
    assert!(h.query_by_label("Reset to defaults").is_none());
    assert!(
        h.query_by_role_and_label(Role::SpinButton, "Grid spacing")
            .is_none()
    );
    pick(&mut h, "AI Assistant");
    assert_eq!(h.state().preferences_tab, PrefsTab::Ai);
    assert!(h.query_by_label("Copy").is_some());
    assert!(h.query_by_label("I play this game").is_none());
    pick(&mut h, "General");
    assert_eq!(h.state().preferences_tab, PrefsTab::General);
    assert!(h.query_by_label("Reset to defaults").is_some());
}

#[test]
fn the_dialog_keeps_its_size_on_every_tab() {
    let mut h = open();
    let done = h.get_by_label("Done").rect();
    let tabs = h.get_by_role_and_label(Role::RadioButton, "General").rect();
    for tab in ["Games", "AI Assistant", "General"] {
        pick(&mut h, tab);
        assert_eq!(h.get_by_label("Done").rect(), done, "{tab}");
        assert_eq!(
            h.get_by_role_and_label(Role::RadioButton, "General").rect(),
            tabs,
            "{tab}"
        );
    }
}

#[test]
fn a_value_survives_switching_tabs() {
    let mut h = open();
    h.state_mut().prefs.view_aids.grid_spacing = 128.0;
    pick(&mut h, "Games");
    pick(&mut h, "General");
    assert_eq!(h.state().prefs.view_aids.grid_spacing, 128.0);
    assert!(
        h.query_by_role_and_label(Role::SpinButton, "Grid spacing")
            .is_some()
    );
}

#[test]
fn the_smallest_window_keeps_the_tabs_reset_and_done_in_view() {
    let mut h = open_with(Prefs::default(), Vec2::new(960.0, 600.0));
    for _ in 0..3 {
        h.run();
    }
    for label in ["Done", "Reset to defaults"] {
        let rect = h.get_by_label(label).rect();
        assert!(rect.bottom() <= 600.0, "{label} {rect:?}");
    }
    let tabs = h.get_by_role_and_label(Role::RadioButton, "General").rect();
    assert!(tabs.top() >= 0.0 && tabs.bottom() <= 600.0, "{tabs:?}");
    // Every tab keeps Done in view too.
    for tab in ["Games", "AI Assistant"] {
        pick(&mut h, tab);
        assert!(h.get_by_label("Done").rect().bottom() <= 600.0, "{tab}");
    }
}

#[test]
fn german_fits_the_games_tab_without_scrolling() {
    let mut prefs = Prefs::default();
    prefs.set_language(Some(tp_i18n::Language::German));
    let mut h = open_with(prefs, Vec2::new(1440.0, 900.0));
    pick(&mut h, "Spiele");
    for _ in 0..3 {
        h.run();
    }
    let done = h.get_by_label("Fertig").rect();
    let hint = h.get_by_label_contains("Es wird nichts versendet").rect();
    assert!(hint.bottom() <= done.top(), "{hint:?} {done:?}");
    // The three tabs are whole at the dialog's width.
    for tab in ["Allgemein", "Spiele", "KI-Assistent"] {
        let rect = h.get_by_role_and_label(Role::RadioButton, tab).rect();
        assert!(rect.width() > 40.0, "{tab} {rect:?}");
    }
}
