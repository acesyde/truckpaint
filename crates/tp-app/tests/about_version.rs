//! Headless test for the version in the About dialog.

mod common;

use egui_kittest::kittest::Queryable;
use tp_app::commands::CommandId;

/// About shows the app's version: `dev` in a build that is not a release, the
/// release tag without its `v` when the build sets `TRUCKPAINT_VERSION`.
#[test]
fn about_shows_the_version_of_the_build() {
    let mut h = common::harness();
    h.run();
    h.state_mut().queue.push(CommandId::About);
    h.run_steps(3);
    h.run();
    assert!(h.state().modal.is_some(), "About did not open");
    let expected = format!(
        "Version {}",
        option_env!("TRUCKPAINT_VERSION").unwrap_or("dev")
    );
    assert!(
        h.query_by_label(&expected).is_some(),
        "About does not show {expected:?}"
    );
}
