//! Headless tests for file dialogs that take their time: while one is open
//! the window is closed to commands, its answer is used once by what asked
//! for it, and a stale answer is dropped. The dialogs are a test double
//! that answers when the test releases it, as the Linux dialog does from its
//! own thread.

mod common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::dialog_flow::Purpose;
use tp_app::file_dialogs::{DialogAnswer, DialogRequest, FileDialogs, Which};
use tp_app::state::{Modal, Screen};
use tp_app::workspace::SaveState;

type H = Harness<'static, AppState>;

#[derive(Default)]
struct Shared {
    started: Vec<Which>,
    answer: Option<DialogAnswer>,
    open: bool,
    woken: usize,
    wake: Option<Box<dyn Fn() + Send>>,
}

/// What a test holds to answer the dialog.
#[derive(Clone, Default)]
struct Gate(Arc<Mutex<Shared>>);

impl Gate {
    /// The user answers the dialog.
    fn release(&self, answer: DialogAnswer) {
        let mut shared = self.0.lock().unwrap();
        shared.answer = Some(answer);
        shared.woken += 1;
        if let Some(wake) = &shared.wake {
            wake();
        }
    }

    fn started(&self) -> Vec<Which> {
        self.0.lock().unwrap().started.clone()
    }

    fn woken(&self) -> usize {
        self.0.lock().unwrap().woken
    }
}

struct SlowDialogs(Gate);

impl FileDialogs for SlowDialogs {
    fn start(&mut self, request: DialogRequest, wake: Box<dyn Fn() + Send>) {
        let mut shared = self.0.0.lock().unwrap();
        shared.started.push(request.which);
        shared.open = true;
        shared.wake = Some(wake);
    }

    fn poll(&mut self) -> Option<DialogAnswer> {
        let mut shared = self.0.0.lock().unwrap();
        let answer = shared.answer.take();
        if answer.is_some() {
            shared.open = false;
        }
        answer
    }

    fn busy(&self) -> bool {
        self.0.0.lock().unwrap().open
    }

    fn show_folder(&mut self, _folder: &std::path::Path) {}
}

fn settle(h: &mut H) {
    for _ in 0..6 {
        h.step();
    }
}

fn open_slow() -> (H, Gate) {
    let mut h = common::harness();
    common::create_project(&mut h);
    let gate = Gate::default();
    h.state_mut().dialogs = Box::new(SlowDialogs(gate.clone()));
    settle(&mut h);
    (h, gate)
}

fn key(h: &mut H, modifiers: Modifiers, key: Key) {
    h.key_press_modifiers(modifiers, key);
    settle(h);
}

fn waiting(h: &H) -> bool {
    h.query_by_label_contains("Waiting for the file dialog")
        .is_some()
}

fn status(h: &H) -> SaveState {
    h.state().workspace().expect("project open").save_state()
}

#[test]
fn the_window_is_closed_to_commands_while_a_dialog_is_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    let (mut h, gate) = open_slow();
    key(&mut h, Modifiers::COMMAND | Modifiers::SHIFT, Key::S);
    assert_eq!(gate.started(), [Which::SaveProject]);
    assert!(h.state().dialog_pending());
    assert!(waiting(&h), "the box is shown");
    // Neither New Project, nor Close, nor a second Save As does anything.
    key(&mut h, Modifiers::COMMAND, Key::N);
    assert!(h.state().modal.is_none());
    key(&mut h, Modifiers::COMMAND, Key::W);
    assert!(h.state().has_project() && h.state().modal.is_none());
    key(&mut h, Modifiers::COMMAND | Modifiers::SHIFT, Key::S);
    assert_eq!(gate.started(), [Which::SaveProject], "no second dialog");
    // The user chooses a file: the project is saved to it, once.
    gate.release(DialogAnswer::Paths(vec![path.clone()]));
    for _ in 0..200 {
        h.step();
        if status(&h) == SaveState::Saved {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(status(&h), SaveState::Saved);
    assert!(path.exists());
    assert!(!h.state().dialog_pending());
    assert!(!waiting(&h), "the box is gone");
    assert_eq!(gate.woken(), 1);
}

#[test]
fn cancelling_continues_nothing() {
    let (mut h, gate) = open_slow();
    key(&mut h, Modifiers::COMMAND | Modifiers::SHIFT, Key::S);
    assert!(waiting(&h));
    gate.release(DialogAnswer::Cancelled);
    settle(&mut h);
    assert!(!waiting(&h) && !h.state().dialog_pending());
    assert!(h.state().modal.is_none() && h.state().has_project());
}

#[test]
fn save_before_closing_waits_for_the_file_and_stops_when_cancelled() {
    let (mut h, gate) = open_slow();
    let now = h.ctx.input(|i| i.time);
    let frame = tp_core::document::Frame::new(
        tp_core::kurbo::Point::new(300.0, 500.0),
        tp_core::kurbo::Size::new(200.0, 100.0),
        0.0,
    );
    h.state_mut().workspace_mut().unwrap().create_shape(
        tp_core::document::ShapeKind::rectangle(),
        frame,
        now,
    );
    settle(&mut h);
    key(&mut h, Modifiers::COMMAND, Key::W);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Save")
        .click();
    settle(&mut h);
    assert_eq!(gate.started(), [Which::SaveProject]);
    gate.release(DialogAnswer::Cancelled);
    settle(&mut h);
    // Still open, with its changes: nothing was closed.
    assert!(matches!(h.state().screen, Screen::Workspace(_)));
    assert_eq!(status(&h), SaveState::Unsaved);
}

#[test]
fn save_before_closing_closes_once_the_file_is_chosen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.truckpaint");
    let (mut h, gate) = open_slow();
    let now = h.ctx.input(|i| i.time);
    let frame = tp_core::document::Frame::new(
        tp_core::kurbo::Point::new(300.0, 500.0),
        tp_core::kurbo::Size::new(200.0, 100.0),
        0.0,
    );
    h.state_mut().workspace_mut().unwrap().create_shape(
        tp_core::document::ShapeKind::rectangle(),
        frame,
        now,
    );
    settle(&mut h);
    key(&mut h, Modifiers::COMMAND, Key::W);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Save")
        .click();
    settle(&mut h);
    gate.release(DialogAnswer::Paths(vec![path.clone()]));
    for _ in 0..400 {
        h.step();
        if !h.state().has_project() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(matches!(h.state().screen, Screen::Home));
    assert!(path.exists());
}

#[test]
fn an_answer_for_a_dialog_that_was_closed_is_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let target: PathBuf = dir.path().join("texture.png");
    let (mut h, gate) = open_slow();
    key(&mut h, Modifiers::COMMAND | Modifiers::SHIFT, Key::E);
    assert!(matches!(h.state().modal, Some(Modal::Export(_))));
    // Export… asks for a file; the Export Texture dialog is closed meanwhile.
    h.state_mut().want_dialog(
        DialogRequest::save_export("texture.png"),
        Purpose::ExportTexture,
    );
    settle(&mut h);
    assert_eq!(gate.started(), [Which::SaveExport]);
    h.state_mut().modal = None;
    gate.release(DialogAnswer::Paths(vec![target.clone()]));
    settle(&mut h);
    assert!(h.state().modal.is_none(), "no message, no dialog");
    assert!(!target.exists(), "nothing was exported");
    assert!(!h.state().dialog_pending());
}

#[test]
fn a_second_request_while_one_is_open_is_dropped() {
    let (mut h, gate) = open_slow();
    h.state_mut()
        .want_dialog(DialogRequest::pick_images(), Purpose::OpenProject);
    settle(&mut h);
    assert_eq!(gate.started(), [Which::PickImages]);
    h.state_mut()
        .want_dialog(DialogRequest::open_project(), Purpose::OpenProject);
    settle(&mut h);
    assert_eq!(gate.started(), [Which::PickImages]);
    gate.release(DialogAnswer::Cancelled);
    settle(&mut h);
    // Free again: the next request starts.
    h.state_mut()
        .want_dialog(DialogRequest::open_project(), Purpose::OpenProject);
    settle(&mut h);
    assert_eq!(gate.started(), [Which::PickImages, Which::OpenProject]);
}
