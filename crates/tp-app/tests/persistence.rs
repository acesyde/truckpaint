//! Headless tests for persistence: saving, opening, the unsaved-changes
//! prompt, recent projects and crash recovery.

mod common;

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers, ViewportEvent, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::layout::Space;
use tp_app::state::{Modal, Screen};
use tp_app::workspace::{SaveState, Workspace};
use tp_core::document::{
    CharStyle, Frame, Node, Object, ObjectId, PathData, Rgba, ShapeKind, StrokeStyle, Subpath,
    TextBlock,
};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

/// Runs a few frames. Background threads (saves, recovery copies, SVG
/// renders) wake the UI at unpredictable times, so `Harness::run`, which
/// waits until nothing asks for a repaint, would be flaky.
fn settle(h: &mut H) {
    for _ in 0..4 {
        h.step();
    }
    // A project file is read in the background.
    for _ in 0..400 {
        if !h.state().is_opening() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
}

fn harness_in(recovery: Option<&Path>) -> H {
    let mut state = AppState::with_prefs(tp_app::prefs::Prefs::default(), None);
    if let Some(dir) = recovery {
        state.enable_recovery(dir);
    }
    common::builder()
        .with_size(common::SIZE)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state)
}

fn open() -> H {
    let mut h = harness_in(None);
    common::create_project(&mut h);
    settle(&mut h);
    h
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn dialogs(h: &mut H, save: &[PathBuf], open: &[PathBuf]) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        save: save.iter().cloned().collect(),
        open: open.iter().cloned().collect(),
        ..Default::default()
    });
}

/// Steps until background saves are applied.
fn wait_saves(h: &mut H) {
    for _ in 0..400 {
        h.step();
        let busy = h.state().workspace().is_some_and(|w| w.saving.is_some());
        if !busy {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    settle(h);
}

fn save_shortcut(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::S);
    wait_saves(h);
}

fn add_rect(h: &mut H, x: f64) -> ObjectId {
    let now = h.ctx.input(|i| i.time);
    let frame = Frame::new(Point::new(x, 500.0), Size::new(200.0, 100.0), 0.0);
    let id = ws_mut(h).create_shape(ShapeKind::rectangle(), frame, now);
    settle(h);
    id
}

fn status(h: &H) -> SaveState {
    ws(h).save_state()
}

fn message_text(h: &H) -> Option<String> {
    match &h.state().modal {
        Some(Modal::Message { text, .. }) => Some(text.clone()),
        _ => None,
    }
}

// ------------------------------------------------------------------ saving

#[test]
fn first_save() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    ws_mut(&mut h).project.name = "ACE Logistics".into();
    // The extension is added when the user leaves it out.
    dialogs(&mut h, &[dir.path().join("ace")], &[]);
    assert_eq!(status(&h), SaveState::Unsaved);
    save_shortcut(&mut h);
    let path = dir.path().join("ace.truckpaint");
    assert!(path.is_file());
    assert_eq!(status(&h), SaveState::Saved);
    assert_eq!(ws(&h).path.as_deref(), Some(path.as_path()));
    assert!(h.query_by_label("Saved").is_some());
    assert_eq!(h.state().prefs.recent[0].path, path);
}

#[test]
fn save_after_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    let mut h = open();
    dialogs(&mut h, std::slice::from_ref(&path), &[]);
    save_shortcut(&mut h);
    add_rect(&mut h, 300.0);
    assert_eq!(status(&h), SaveState::Unsaved);
    // No second dialog: the scripted save queue is empty now.
    save_shortcut(&mut h);
    assert_eq!(status(&h), SaveState::Saved);
    let reread = tp_file::read(&path).unwrap().project;
    assert_eq!(reread.surface().objects.len(), 1);
}

#[test]
fn undo_back_to_the_saved_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    dialogs(&mut h, &[dir.path().join("a.truckpaint")], &[]);
    save_shortcut(&mut h);
    add_rect(&mut h, 300.0);
    assert_eq!(status(&h), SaveState::Unsaved);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    settle(&mut h);
    assert_eq!(status(&h), SaveState::Saved);
    assert!(h.query_by_label("Saved").is_some());
}

#[test]
fn saving_status_is_shown() {
    let mut h = open();
    ws_mut(&mut h).saving = Some(tp_app::workspace::PendingSave {
        id: 999,
        snapshot: ws(&h).snapshot(),
        path: "/tmp/x.truckpaint".into(),
    });
    settle(&mut h);
    assert!(h.query_by_label("Saving…").is_some());
}

#[cfg(unix)]
#[test]
fn save_to_a_read_only_location() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let locked = dir.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    let mut h = open();
    dialogs(&mut h, &[locked.join("ace.truckpaint")], &[]);
    save_shortcut(&mut h);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    let text = message_text(&h).expect("error message");
    assert!(text.contains("ace.truckpaint could not be saved"), "{text}");
    assert_eq!(status(&h), SaveState::Unsaved);
    assert!(!locked.join("ace.truckpaint").exists());
    h.key_press(Key::Enter);
    settle(&mut h);
    assert!(h.state().modal.is_none());
}

// ----------------------------------------------------------------- opening

/// A saved project with a group, a text and an image, at `path`.
fn rich_file(path: &Path) -> Workspace {
    let mut h = open();
    {
        let ws = ws_mut(&mut h);
        ws.project.name = "ACE Logistics".into();
        let mut a = Object::new(
            ObjectId(0),
            ShapeKind::Rectangle {
                corner_radius: 12.0,
            },
            Frame::new(Point::new(800.0, 600.0), Size::new(400.0, 200.0), 15.0),
        );
        a.fill = Rgba::rgb(0x7A, 0x1F, 0x2B).into();
        let a = ws.project.add(a);
        let b = ws.project.add(Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            Frame::new(Point::new(1200.0, 600.0), Size::new(300.0, 300.0), 0.0),
        ));
        ws.project.group(&[a, b]);
        let anchor = Point::new(500.0, 1500.0);
        let mut text = Object::text(
            ObjectId(0),
            TextBlock::new("ACE", CharStyle::default()),
            anchor,
        );
        ws.text.place_at(&mut text, anchor);
        ws.project.add(text);
        let png = tp_app::import::solid_png(80, 40, [20, 60, 220, 255]);
        let file = tp_app::import::read_bytes("logo.png", png).unwrap();
        ws.place_files(vec![Ok(file)], Some(Point::new(2000.0, 2000.0)), 0.0);
        ws.project.add_to_palette(Rgba::rgb(1, 2, 3));
        add_vector_objects(ws);
    }
    dialogs(&mut h, &[path.to_path_buf()], &[]);
    save_shortcut(&mut h);
    assert_eq!(status(&h), SaveState::Saved);
    let Screen::Workspace(ws) = std::mem::replace(&mut h.state_mut().screen, Screen::Home) else {
        unreachable!()
    };
    *ws
}

/// A star, a curved path with a hole and an open line.
fn add_vector_objects(ws: &mut Workspace) {
    ws.project.add(Object::new(
        ObjectId(0),
        ShapeKind::Polygon {
            sides: 5,
            star: Some(0.4),
        },
        Frame::new(Point::new(3000.0, 600.0), Size::new(500.0, 480.0), -12.0),
    ));
    let corner = |x: f64, y: f64| Node::corner(Point::new(x, y));
    let mut swoosh = Object::from_path(
        ObjectId(0),
        PathData::new(vec![
            Subpath::new(
                vec![
                    Node::smooth(Point::new(300.0, 2600.0), Point::new(1200.0, 1700.0)),
                    Node::smooth(Point::new(3600.0, 2100.0), Point::new(3900.0, 2300.0)),
                    corner(3500.0, 2600.0),
                ],
                true,
            ),
            Subpath::new(
                vec![
                    corner(1600.0, 2250.0),
                    corner(1600.0, 2450.0),
                    corner(1900.0, 2450.0),
                ],
                true,
            ),
        ]),
    );
    swoosh.frame.rotation_deg = 5.0;
    // Gradients round-trip too: a radial fill and a gradient outline.
    let stops = [
        tp_core::document::ColorStop::new(0.0, Rgba::rgb(0xF0, 0xB4, 0x4C)),
        tp_core::document::ColorStop::new(0.6, Rgba::with_alpha(0x7A, 0x1F, 0x2B, 180)),
        tp_core::document::ColorStop::new(1.0, Rgba::with_alpha(0x7A, 0x1F, 0x2B, 0)),
    ];
    let mut glow =
        tp_core::document::Gradient::new(tp_core::document::GradientKind::Radial, &stops);
    glow.minor = Point::new(0.5, 0.8);
    swoosh.fill = tp_core::document::Paint::Gradient(glow);
    ws.project.add(swoosh);
    let mut line = Object::from_path(
        ObjectId(0),
        PathData::new(vec![Subpath::new(
            vec![corner(400.0, 3200.0), corner(3700.0, 3300.0)],
            false,
        )]),
    );
    line.name = "Line".into();
    line.fill = Rgba::rgb(255, 255, 255).into();
    line.edit_path(|p| p.line_width = 30.0);
    line.stroke = Some(StrokeStyle {
        paint: tp_core::document::Paint::Gradient(tp_core::document::Gradient::new(
            tp_core::document::GradientKind::Linear,
            &stops,
        )),
        width: 4.0,
        ..Default::default()
    });
    ws.project.add(line);
}

#[test]
fn round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    let original = rich_file(&path);
    let mut h = harness_in(None);
    dialogs(&mut h, &[], std::slice::from_ref(&path));
    h.key_press_modifiers(Modifiers::COMMAND, Key::O);
    settle(&mut h);
    let opened = &ws(&h).project;
    assert_eq!(opened.name, "ACE Logistics");
    assert_eq!(opened.palette, original.project.palette);
    assert_eq!(opened.assets, original.project.assets);
    assert_eq!(opened.surfaces, original.project.surfaces);
    let gradients = opened
        .surface()
        .objects
        .iter()
        .filter(|o| {
            o.fill.gradient().is_some() || o.stroke.is_some_and(|s| s.paint.gradient().is_some())
        })
        .count();
    assert_eq!(gradients, 2, "the radial fill and the gradient outline");
    assert_eq!(status(&h), SaveState::Saved);
    assert!(ws(&h).selection.is_empty());
    assert!(!ws(&h).history.can_undo());
}

#[test]
fn open_from_the_menu() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    rich_file(&path);
    let mut h = open();
    assert_eq!(ws(&h).space, Space::Workshop);
    // Replacing an unsaved project asks first.
    dialogs(&mut h, &[], std::slice::from_ref(&path));
    h.get_by_label("File").click();
    settle(&mut h);
    h.get_by_label("Open Project…").click();
    settle(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::UnsavedChanges(_))));
    h.get_by_label("Don't Save").click();
    settle(&mut h);
    assert_eq!(ws(&h).project.name, "ACE Logistics");
    assert_eq!(h.state().title(), "ACE Logistics — TruckPaint");
    assert!(h.query_by_label("Saved").is_some());
    // An opened file is shown in the Project space, whatever space the
    // previous project was in (workspace-spaces, Opening a file).
    assert_eq!(ws(&h).space, Space::Project);
}

#[test]
fn damaged_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    rich_file(&path);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.truncate(bytes.len() / 2);
    std::fs::write(&path, bytes).unwrap();
    let mut h = open();
    let before = ws(&h).project.clone();
    dialogs(&mut h, &[], &[path]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::O);
    settle(&mut h);
    h.get_by_label("Don't Save").click();
    settle(&mut h);
    assert_eq!(
        message_text(&h).as_deref(),
        Some("ace.truckpaint is damaged and cannot be opened.")
    );
    assert_eq!(ws(&h).project, before);
}

#[test]
fn newer_format() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("future.truckpaint");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    let o = zip::write::SimpleFileOptions::default();
    zip.start_file("mimetype", o).unwrap();
    zip.write_all(b"application/x-truckpaint").unwrap();
    zip.start_file("format", o).unwrap();
    zip.write_all(b"99").unwrap();
    zip.start_file("project.ron", o).unwrap();
    zip.write_all(b"(name: \"x\")").unwrap();
    zip.finish().unwrap();
    let mut h = harness_in(None);
    dialogs(&mut h, &[], &[path]);
    h.get_by_label("Open Project…").click();
    settle(&mut h);
    assert_eq!(
        message_text(&h).as_deref(),
        Some("future.truckpaint was created with a newer version of TruckPaint.")
    );
    assert!(!h.state().has_project());
}

#[test]
fn open_from_the_home_screen_and_recent_entries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    rich_file(&path);
    let mut h = harness_in(None);
    dialogs(&mut h, &[], std::slice::from_ref(&path));
    h.get_by_label("Open Project…").click();
    settle(&mut h);
    assert_eq!(ws(&h).project.name, "ACE Logistics");
    // Saved project closes without a prompt and is listed first.
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    settle(&mut h);
    assert!(matches!(h.state().screen, Screen::Home));
    assert_eq!(h.state().prefs.recent[0].path, path);
    h.get_by_label("ACE Logistics").click();
    settle(&mut h);
    assert_eq!(ws(&h).path.as_deref(), Some(path.as_path()));
}

#[test]
fn an_unreadable_file_leaves_the_open_project_alone() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.truckpaint");
    std::fs::write(&bad, b"not a project").unwrap();
    let mut h = open();
    add_rect(&mut h, 300.0);
    let ctx = h.ctx.clone();
    h.state_mut().open_file(&ctx, &bad);
    settle(&mut h);
    assert_eq!(
        message_text(&h).as_deref(),
        Some("bad.truckpaint is not a TruckPaint project.")
    );
    // Still the project that was open, with its unsaved rectangle.
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
    assert_eq!(status(&h), SaveState::Unsaved);
}

#[test]
fn a_second_open_while_reading_starts_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (first, second) = (
        dir.path().join("a.truckpaint"),
        dir.path().join("b.truckpaint"),
    );
    rich_file(&first);
    rich_file(&second);
    let mut h = harness_in(None);
    let ctx = h.ctx.clone();
    h.state_mut().open_file(&ctx, &first);
    assert!(h.state().is_opening());
    h.state_mut().open_file(&ctx, &second);
    settle(&mut h);
    assert_eq!(ws(&h).path.as_deref(), Some(first.as_path()));
}

/// A quick open shows no box; one that is still reading after a moment
/// shows "Opening <file>…" and the window keeps answering. The slow file is
/// a named pipe, which blocks the reader until the test writes to it.
#[cfg(unix)]
#[test]
fn the_box_shows_only_for_a_slow_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    rich_file(&path);

    // Frames of a hundredth of a second: the time a quick read takes.
    let mut h = common::builder()
        .with_size(common::SIZE)
        .with_step_dt(0.01)
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(tp_app::prefs::Prefs::default(), None),
        );
    let ctx = h.ctx.clone();
    h.state_mut().open_file(&ctx, &path);
    h.step();
    assert!(h.query_by_label_contains("Opening").is_none());
    settle(&mut h);
    assert_eq!(ws(&h).project.name, "ACE Logistics");

    let slow = dir.path().join("slow.truckpaint");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&slow)
            .status()
            .unwrap()
            .success()
    );
    let mut h = harness_in(None);
    let ctx = h.ctx.clone();
    h.state_mut().open_file(&ctx, &slow);
    for _ in 0..8 {
        h.step();
    }
    assert!(h.state().is_opening());
    assert!(
        h.query_by_label_contains("Opening slow.truckpaint")
            .is_some()
    );
    let (bytes, pipe) = (std::fs::read(&path).unwrap(), slow.clone());
    let writer = std::thread::spawn(move || std::fs::write(pipe, bytes).unwrap());
    settle(&mut h);
    writer.join().unwrap();
    assert!(h.query_by_label_contains("Opening").is_none());
    assert_eq!(ws(&h).project.name, "ACE Logistics");
}

// ------------------------------------------------------- unsaved changes

#[test]
fn cancel_keeps_working() {
    let mut h = open();
    add_rect(&mut h, 300.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    settle(&mut h);
    assert!(h.query_by_label_contains("Save changes to").is_some());
    h.key_press(Key::Escape);
    settle(&mut h);
    assert!(h.state().modal.is_none());
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
    assert_eq!(status(&h), SaveState::Unsaved);
}

#[test]
fn no_prompt_when_saved() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    dialogs(&mut h, &[dir.path().join("a.truckpaint")], &[]);
    save_shortcut(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    settle(&mut h);
    assert!(matches!(h.state().screen, Screen::Home));
}

#[test]
fn save_from_the_prompt_then_close() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.truckpaint");
    let mut h = open();
    add_rect(&mut h, 300.0);
    dialogs(&mut h, std::slice::from_ref(&path), &[]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    settle(&mut h);
    h.get_by_role_and_label(Role::Button, "Save").click();
    settle(&mut h);
    for _ in 0..200 {
        h.step();
        if !h.state().has_project() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(matches!(h.state().screen, Screen::Home));
    assert_eq!(
        tp_file::read(&path)
            .unwrap()
            .project
            .surface()
            .objects
            .len(),
        1
    );
}

#[test]
fn new_project_asks_first() {
    let mut h = open();
    add_rect(&mut h, 300.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::N);
    settle(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::UnsavedChanges(_))));
    h.get_by_label("Don't Save").click();
    settle(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::NewProject(_))));
}

#[test]
fn quit_with_unsaved_changes() {
    let mut h = open();
    add_rect(&mut h, 300.0);
    h.input_mut()
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    h.step();
    assert!(matches!(h.state().modal, Some(Modal::UnsavedChanges(_))));
    assert!(h.state().has_project());
    settle(&mut h);
    h.get_by_label("Cancel").click();
    settle(&mut h);
    assert!(h.state().modal.is_none());
    assert!(h.state().has_project());
}

// ----------------------------------------------------------- crash recovery

fn copies(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "truckpaint"))
        .collect()
}

/// Edits for more than the recovery interval.
fn edit_for_a_while(h: &mut H) {
    h.input_mut().time = Some(1000.0);
    h.step();
    add_rect(h, 300.0);
    h.input_mut().time = Some(1000.0 + tp_app::recovery::INTERVAL + 1.0);
    h.step();
    h.input_mut().time = None;
    settle(h);
    // Recovery copies are written in the background.
    for _ in 0..200 {
        if !copies(h.state().recovery_dir().unwrap()).is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn recovery_copy_while_editing_and_normal_close() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness_in(Some(dir.path()));
    common::create_project(&mut h);
    edit_for_a_while(&mut h);
    assert_eq!(copies(dir.path()).len(), 1);
    assert_eq!(status(&h), SaveState::Unsaved);
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    settle(&mut h);
    h.get_by_label("Don't Save").click();
    settle(&mut h);
    assert!(copies(dir.path()).is_empty());
}

/// Leaves a recovery copy of "ACE Logistics" as a crashed session would.
fn crashed_session(dir: &Path) {
    let mut h = harness_in(Some(dir));
    common::create_project(&mut h);
    ws_mut(&mut h).project.name = "ACE Logistics".into();
    edit_for_a_while(&mut h);
    assert_eq!(copies(dir).len(), 1);
    // Dropped without shutdown: like a crash.
    drop(h);
}

#[test]
fn restore_unsaved_work() {
    let dir = tempfile::tempdir().unwrap();
    crashed_session(dir.path());
    let mut h = harness_in(Some(dir.path()));
    settle(&mut h);
    let recovered = h.get_by_label("Recovered projects").rect();
    let recent = h.get_by_label("Recent projects").rect();
    assert!(recovered.top() < recent.top(), "recovery shown first");
    assert!(h.query_by_label("Recovered ACE Logistics").is_some());
    h.get_by_label("Restore").click();
    settle(&mut h);
    assert_eq!(ws(&h).project.name, "ACE Logistics");
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
    assert_eq!(status(&h), SaveState::Unsaved);
    assert!(h.state().recovered.is_empty());
    // Shown in the Project space, the status bar saying it is unsaved.
    assert_eq!(ws(&h).space, Space::Project);
    assert!(h.query_by_label("Unsaved changes").is_some());
    // The copy now belongs to this session: closing removes it.
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    settle(&mut h);
    h.get_by_label("Don't Save").click();
    settle(&mut h);
    assert!(copies(dir.path()).is_empty());
}

#[test]
fn discard_a_recovered_project() {
    let dir = tempfile::tempdir().unwrap();
    crashed_session(dir.path());
    let mut h = harness_in(Some(dir.path()));
    settle(&mut h);
    h.get_by_label("Discard").click();
    settle(&mut h);
    assert!(h.query_by_label("Recovered projects").is_none());
    assert!(copies(dir.path()).is_empty());
    drop(h);
    let h = harness_in(Some(dir.path()));
    assert!(h.state().recovered.is_empty());
}

#[test]
fn vector_project_survives_a_crash_and_exports() {
    let dir = tempfile::tempdir().unwrap();
    let surfaces = {
        let mut h = harness_in(Some(dir.path()));
        common::create_project(&mut h);
        add_vector_objects(ws_mut(&mut h));
        edit_for_a_while(&mut h);
        assert_eq!(copies(dir.path()).len(), 1);
        ws(&h).project.surfaces.clone()
    };
    let mut h = harness_in(Some(dir.path()));
    settle(&mut h);
    h.get_by_label("Restore").click();
    settle(&mut h);
    assert_eq!(ws(&h).project.surfaces, surfaces);
    // And the restored paths export.
    let out = dir.path().join("vector.png");
    let job = tp_app::export::ExportJob::start(
        ws(&h).project.clone(),
        tp_app::export::ExportSettings {
            divisor: 4,
            ..Default::default()
        },
        ws(&h).text.fonts.fork(),
        out.clone(),
        || {},
    );
    assert_eq!(
        job.wait(),
        tp_app::export::ExportOutcome::Written(out.clone())
    );
    let img = image::open(&out).unwrap().to_rgba8();
    assert_eq!(img.dimensions(), (1024, 1024));
    // Inside the hole of the swoosh: the white background shows.
    let hole = img.get_pixel(1700 / 4 + 5, 2400 / 4).0;
    assert_eq!(hole, [255, 255, 255, 255]);
    // Just left of the hole: the swoosh fill.
    let fill = img.get_pixel(1450 / 4, 2400 / 4).0;
    assert_ne!(
        fill,
        [255, 255, 255, 255],
        "the swoosh is drawn around the hole"
    );
}

// --- Preferences ---------------------------------------------------------------

/// A preferences file of the build before the spaces: a panel stack with
/// the Assets panel closed, a column width and a hidden Vehicles sidebar.
const PREFS_BEFORE_SPACES: &str = include_str!("fixtures/prefs-before-spaces.ron");

#[test]
fn preferences_from_a_build_before_the_spaces() {
    use tp_app::layout::WorkspaceLayout;
    use tp_app::prefs::PrefsStore;
    let dir = tempfile::tempdir().unwrap();
    let store = PrefsStore::new(dir.path());
    std::fs::write(store.path(), PREFS_BEFORE_SPACES).unwrap();
    let loaded = store.load();
    assert!(loaded.issue.is_none(), "{:?}", loaded.issue);
    assert!(loaded.backup.is_none());
    let prefs = loaded.prefs;
    assert_eq!(prefs.ui_scale, 1.25);
    assert_eq!(prefs.layout, WorkspaceLayout::default());
    // Every other preference is kept.
    assert!(prefs.view_aids.grid && !prefs.view_aids.snapping);
    assert_eq!(prefs.view_aids.grid_spacing, 128.0);
    assert_eq!(prefs.language(), Some(tp_i18n::Language::French));
    assert_eq!(prefs.recent_colors, vec![[200, 30, 40, 255]]);
    // Nothing else was written next to the file, and no message appears.
    let files: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert_eq!(files.len(), 1);
    let mut h = common::builder().with_size(common::SIZE).build_ui_state(
        |ui, state: &mut AppState| state.show(ui),
        AppState::with_prefs(prefs, None),
    );
    h.run();
    assert!(h.state().modal.is_none());
}

#[test]
fn workspace_layout_survives_restart() {
    use tp_app::layout::LeftTab;
    use tp_app::prefs::PrefsStore;
    let dir = tempfile::tempdir().unwrap();
    let store = || Some(PrefsStore::new(dir.path()));
    let mut state = AppState::new(store());
    state.system_fonts = false;
    let mut h = common::builder()
        .with_size(common::SIZE)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    common::create_project(&mut h);
    h.key_press(Key::Num2);
    h.run();
    h.key_press(Key::Tab);
    h.run();
    // The inspector's width as left by dragging its edge.
    h.state_mut().prefs.layout.inspector_width = 360.0;
    h.run();
    h.state_mut().persist_prefs(f64::MAX, true);
    let layout = AppState::new(store()).prefs.layout;
    assert_eq!(layout.left_tab, LeftTab::Layers);
    assert!(layout.panels_hidden);
    assert_eq!(layout.inspector_width, 360.0);
}
