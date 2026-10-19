//! Native file dialogs, behind a trait so tests can script them. A dialog is
//! a [`DialogRequest`], started with [`FileDialogs::start`] and answered
//! through [`FileDialogs::poll`]; on Linux it runs on its own thread, so that
//! the window keeps answering the system while the dialog is open.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use tp_i18n::tr;

/// Which dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    /// A `.truckpaint` file to open.
    OpenProject,
    /// Where to save a project.
    SaveProject,
    /// Images to place (Place…).
    PickImages,
    /// Where to export a texture.
    SaveExport,
    /// Vehicle packages to install.
    PickPackages,
    /// Template images of a custom vehicle (PNG, DDS or SVG).
    PickTemplates,
    /// Where to export a vehicle package.
    SavePackage,
    /// Where to export a mod.
    SaveMod,
    /// A PNG or JPEG picture for the mod.
    PickModImage,
    /// A folder.
    PickFolder,
}

/// A dialog to show: which one, and its words and defaults, translated when
/// the request is built (the language belongs to the UI thread).
#[derive(Clone, Debug)]
pub struct DialogRequest {
    pub which: Which,
    title: String,
    filter: Option<(String, Vec<String>)>,
    file_name: Option<String>,
    directory: Option<PathBuf>,
}

impl DialogRequest {
    fn new(which: Which, title: impl Into<String>) -> Self {
        Self {
            which,
            title: title.into(),
            filter: None,
            file_name: None,
            directory: None,
        }
    }

    fn filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filter = Some((
            name.into(),
            extensions.iter().map(|e| (*e).to_owned()).collect(),
        ));
        self
    }

    fn named(mut self, file_name: &str) -> Self {
        self.file_name = Some(file_name.to_owned());
        self
    }

    /// The file name proposed, for the save dialogs.
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    /// The folder the dialog opens in, when one was asked for.
    pub fn directory(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    pub fn open_project() -> Self {
        Self::new(Which::OpenProject, tr("dialog-open-project"))
            .filter(tr("filter-project"), &[tp_file::EXTENSION])
    }

    /// Where to save, proposing `suggested` as the file name.
    pub fn save_project(suggested: &str) -> Self {
        Self::new(Which::SaveProject, tr("dialog-save-project"))
            .filter(tr("filter-project"), &[tp_file::EXTENSION])
            .named(suggested)
    }

    pub fn pick_images() -> Self {
        Self::new(Which::PickImages, tr("cmd-place"))
            .filter(tr("filter-images"), &["png", "jpg", "jpeg", "svg"])
    }

    /// Where to export a texture, proposing `suggested` (with its extension).
    pub fn save_export(suggested: &str) -> Self {
        let ext = Path::new(suggested)
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self::new(Which::SaveExport, tr("dialog-export-texture"))
            .filter(ext.to_uppercase(), &[ext.as_str()])
            .named(suggested)
    }

    pub fn pick_packages() -> Self {
        Self::new(Which::PickPackages, tr("cmd-vehicle-library"))
            .filter(tr("filter-packages"), &[tp_vehicles::EXTENSION])
    }

    pub fn pick_templates() -> Self {
        Self::new(Which::PickTemplates, tr("custom-add-templates"))
            .filter(tr("filter-templates"), &["png", "dds", "svg"])
    }

    /// Where to export a vehicle package, proposing `suggested`.
    pub fn save_package(suggested: &str) -> Self {
        Self::new(Which::SavePackage, tr("dialog-export-package"))
            .filter(tr("filter-packages"), &[tp_vehicles::EXTENSION])
            .named(suggested)
    }

    /// Where to export a mod, proposing `suggested` in `folder`.
    pub fn save_mod(suggested: &str, folder: Option<&Path>) -> Self {
        let mut request = Self::new(Which::SaveMod, tr("dialog-export-mod"))
            .filter(tr("filter-mods"), &[crate::mod_export::EXTENSION])
            .named(suggested);
        request.directory = folder.map(Path::to_path_buf);
        request
    }

    pub fn pick_mod_image() -> Self {
        Self::new(Which::PickModImage, tr("mod-choose-image"))
            .filter(tr("filter-mod-images"), &["png", "jpg", "jpeg"])
    }

    /// A folder, with `title` in the dialog.
    pub fn pick_folder(title: &str) -> Self {
        Self::new(Which::PickFolder, title)
    }
}

/// What the user answered.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum DialogAnswer {
    #[default]
    Cancelled,
    /// The chosen files or folder, never empty.
    Paths(Vec<PathBuf>),
}

impl DialogAnswer {
    fn one(path: Option<PathBuf>) -> Self {
        path.map_or(Self::Cancelled, |p| Self::Paths(vec![p]))
    }

    fn many(paths: Vec<PathBuf>) -> Self {
        if paths.is_empty() {
            Self::Cancelled
        } else {
            Self::Paths(paths)
        }
    }

    /// The first chosen path.
    pub fn path(self) -> Option<PathBuf> {
        self.paths().into_iter().next()
    }

    /// Every chosen path (none when cancelled).
    pub fn paths(self) -> Vec<PathBuf> {
        match self {
            Self::Cancelled => Vec::new(),
            Self::Paths(paths) => paths,
        }
    }
}

/// Shows `request` with the operating system's dialog and waits for the
/// answer. The only place that builds an `rfd` dialog.
pub fn show(request: &DialogRequest) -> DialogAnswer {
    let mut dialog = rfd::FileDialog::new().set_title(request.title.as_str());
    if let Some((name, extensions)) = &request.filter {
        dialog = dialog.add_filter(name.as_str(), extensions);
    }
    if let Some(file_name) = &request.file_name {
        dialog = dialog.set_file_name(file_name.as_str());
    }
    if let Some(directory) = &request.directory {
        dialog = dialog.set_directory(directory);
    }
    match request.which {
        Which::OpenProject | Which::PickModImage => DialogAnswer::one(dialog.pick_file()),
        Which::PickImages | Which::PickPackages | Which::PickTemplates => {
            DialogAnswer::many(dialog.pick_files().unwrap_or_default())
        }
        Which::SaveProject | Which::SaveExport | Which::SavePackage | Which::SaveMod => {
            DialogAnswer::one(dialog.save_file())
        }
        Which::PickFolder => DialogAnswer::one(dialog.pick_folder()),
    }
}

pub trait FileDialogs {
    /// Starts `request`; `wake` asks the UI for a frame once it is answered.
    /// Only one dialog at a time: the application asks for one only when
    /// [`busy`](Self::busy) is false.
    fn start(&mut self, request: DialogRequest, wake: Box<dyn Fn() + Send>);
    /// The answer, once, when the dialog is over.
    fn poll(&mut self) -> Option<DialogAnswer>;
    /// Whether a dialog is open (started and not yet polled).
    fn busy(&self) -> bool;
    /// Shows `folder` in the system's file manager (Finder, Explorer…).
    fn show_folder(&mut self, folder: &Path);
    /// Names proposed so far (scripted dialogs only).
    fn suggested(&self) -> Vec<String> {
        Vec::new()
    }
    /// Folders shown in the file manager so far (scripted dialogs only).
    fn shown_folders(&self) -> Vec<PathBuf> {
        Vec::new()
    }
}

/// The operating system's dialogs. On Linux a dialog runs on a thread of its
/// own, because the portal's dialog waits on the thread that opens it and
/// the window would stop answering the system (GNOME offers to force quit);
/// the other systems keep the window alive themselves, so the dialog runs
/// where it is started and its answer is read at once.
#[derive(Default)]
pub struct NativeDialogs {
    answer: Option<DialogAnswer>,
    running: Option<mpsc::Receiver<DialogAnswer>>,
}

impl FileDialogs for NativeDialogs {
    fn start(&mut self, request: DialogRequest, wake: Box<dyn Fn() + Send>) {
        if cfg!(target_os = "linux") {
            let (tx, rx) = mpsc::channel();
            let spawned = std::thread::Builder::new()
                .name("file-dialog".into())
                .spawn(move || {
                    let _ = tx.send(show(&request));
                    wake();
                });
            match spawned {
                Ok(_) => self.running = Some(rx),
                Err(err) => {
                    tracing::error!(%err, "cannot start the file dialog");
                    self.answer = Some(DialogAnswer::Cancelled);
                }
            }
        } else {
            self.answer = Some(show(&request));
        }
    }

    fn poll(&mut self) -> Option<DialogAnswer> {
        if let Some(rx) = &self.running {
            return match rx.try_recv() {
                Ok(answer) => {
                    self.running = None;
                    Some(answer)
                }
                Err(mpsc::TryRecvError::Empty) => None,
                // The thread ended without an answer.
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.running = None;
                    Some(DialogAnswer::Cancelled)
                }
            };
        }
        self.answer.take()
    }

    fn busy(&self) -> bool {
        self.running.is_some()
    }

    fn show_folder(&mut self, folder: &Path) {
        if let Err(err) = opener::open(folder) {
            tracing::warn!("showing {}: {err}", folder.display());
        }
    }
}

/// Dialogs answering from queues (tests). Empty queues mean "cancelled".
#[derive(Debug, Default)]
pub struct ScriptedDialogs {
    pub open: VecDeque<PathBuf>,
    pub save: VecDeque<PathBuf>,
    pub images: VecDeque<Vec<PathBuf>>,
    pub export: VecDeque<PathBuf>,
    pub packages: VecDeque<Vec<PathBuf>>,
    pub templates: VecDeque<Vec<PathBuf>>,
    pub package_save: VecDeque<PathBuf>,
    pub mod_save: VecDeque<PathBuf>,
    pub mod_images: VecDeque<PathBuf>,
    pub folders: VecDeque<PathBuf>,
    /// File names proposed by save dialogs, in order.
    pub suggested: Vec<String>,
    /// Folders the mod save dialogs opened in, in order.
    pub mod_folders: Vec<Option<PathBuf>>,
    /// Folders shown in the file manager, in order.
    pub shown: Vec<PathBuf>,
    /// The answer of the dialog started, read by the next `poll`.
    pub ready: Option<DialogAnswer>,
}

impl FileDialogs for ScriptedDialogs {
    fn start(&mut self, request: DialogRequest, _wake: Box<dyn Fn() + Send>) {
        if let Some(name) = request.file_name() {
            self.suggested.push(name.to_owned());
        }
        if request.which == Which::SaveMod {
            self.mod_folders
                .push(request.directory().map(Path::to_path_buf));
        }
        self.ready = Some(match request.which {
            Which::OpenProject => DialogAnswer::one(self.open.pop_front()),
            Which::SaveProject => DialogAnswer::one(self.save.pop_front()),
            Which::PickImages => DialogAnswer::many(self.images.pop_front().unwrap_or_default()),
            Which::SaveExport => DialogAnswer::one(self.export.pop_front()),
            Which::PickPackages => {
                DialogAnswer::many(self.packages.pop_front().unwrap_or_default())
            }
            Which::PickTemplates => {
                DialogAnswer::many(self.templates.pop_front().unwrap_or_default())
            }
            Which::SavePackage => DialogAnswer::one(self.package_save.pop_front()),
            Which::SaveMod => DialogAnswer::one(self.mod_save.pop_front()),
            Which::PickModImage => DialogAnswer::one(self.mod_images.pop_front()),
            Which::PickFolder => DialogAnswer::one(self.folders.pop_front()),
        });
    }

    fn poll(&mut self) -> Option<DialogAnswer> {
        self.ready.take()
    }

    fn busy(&self) -> bool {
        false
    }

    fn show_folder(&mut self, folder: &Path) {
        self.shown.push(folder.to_path_buf());
    }

    fn suggested(&self) -> Vec<String> {
        self.suggested.clone()
    }

    fn shown_folders(&self) -> Vec<PathBuf> {
        self.shown.clone()
    }
}

/// Adds the `.truckpaint` extension when the user left it out.
pub fn with_extension(path: PathBuf) -> PathBuf {
    let has = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(tp_file::EXTENSION));
    if has {
        path
    } else {
        let mut name = path.into_os_string();
        name.push(".");
        name.push(tp_file::EXTENSION);
        PathBuf::from(name)
    }
}

/// File name proposed when saving a project named `name`.
pub fn suggested_file_name(name: &str) -> String {
    let safe: String = name
        .chars()
        .map(|c| if "/\\:*?\"<>|".contains(c) { '-' } else { c })
        .collect();
    format!("{}.{}", safe.trim(), tp_file::EXTENSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scripted_dialog_answers_at_once_and_only_once() {
        let mut dialogs = ScriptedDialogs::default();
        dialogs.save.push_back("/a/b.truckpaint".into());
        dialogs.start(DialogRequest::save_project("x.truckpaint"), Box::new(|| {}));
        assert!(!dialogs.busy());
        assert_eq!(
            dialogs.poll(),
            Some(DialogAnswer::Paths(vec!["/a/b.truckpaint".into()]))
        );
        assert_eq!(dialogs.poll(), None);
        assert_eq!(dialogs.suggested(), ["x.truckpaint"]);
        // An empty queue means the dialog was cancelled.
        dialogs.start(DialogRequest::open_project(), Box::new(|| {}));
        assert_eq!(dialogs.poll(), Some(DialogAnswer::Cancelled));
    }

    #[test]
    fn answers_give_their_paths() {
        let answer = DialogAnswer::Paths(vec!["/a".into(), "/b".into()]);
        assert_eq!(answer.clone().path(), Some("/a".into()));
        assert_eq!(answer.paths(), [PathBuf::from("/a"), PathBuf::from("/b")]);
        assert_eq!(DialogAnswer::Cancelled.path(), None);
        assert!(DialogAnswer::Cancelled.paths().is_empty());
    }

    #[test]
    fn extension_and_suggested_name() {
        assert_eq!(
            with_extension("/a/ace".into()),
            PathBuf::from("/a/ace.truckpaint")
        );
        assert_eq!(
            with_extension("/a/ace.TruckPaint".into()),
            PathBuf::from("/a/ace.TruckPaint")
        );
        assert_eq!(
            suggested_file_name("ACE Logistics"),
            "ACE Logistics.truckpaint"
        );
        assert_eq!(suggested_file_name("A/B"), "A-B.truckpaint");
    }
}
