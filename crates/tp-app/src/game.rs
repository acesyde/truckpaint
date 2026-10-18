//! The player's games: finding their folders in the Steam libraries, checking
//! a folder, and making the official vehicle packages from it on a worker
//! thread, one vehicle after another.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use tp_i18n::tr;
use tp_pack::extract::{self, GameInfo, InspectError, Outcome, Scan};
use tp_vehicles::{Game, Package};

use crate::vehicles::VehicleLibrary;

/// The operating systems whose Steam folders are known.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    Windows,
    Linux,
    MacOs,
}

impl Os {
    pub const CURRENT: Os = if cfg!(windows) {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::MacOs
    } else {
        Os::Linux
    };
}

/// The folders detection starts from, given so every system can be tested on any.
#[derive(Clone, Debug, Default)]
pub struct Dirs {
    pub home: Option<PathBuf>,
    /// `Program Files (x86)` and `Program Files` (Windows).
    pub program_files: Vec<PathBuf>,
}

impl Dirs {
    /// This computer's folders.
    pub fn of_user() -> Self {
        let program_files = ["ProgramFiles(x86)", "ProgramFiles"]
            .into_iter()
            .filter_map(std::env::var_os)
            .map(PathBuf::from)
            .collect();
        Self {
            home: directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf()),
            program_files,
        }
    }
}

/// Where Steam may be installed, by system. The Windows registry is not read:
/// a Steam elsewhere than in `Program Files` is found by choosing the folder.
pub fn steam_roots(os: Os, dirs: &Dirs) -> Vec<PathBuf> {
    let home = |relative: &str| dirs.home.iter().map(|h| h.join(relative)).collect();
    match os {
        Os::Windows => dirs.program_files.iter().map(|p| p.join("Steam")).collect(),
        Os::Linux => [
            ".local/share/Steam",
            ".steam/steam",
            ".var/app/com.valvesoftware.Steam/.local/share/Steam",
        ]
        .into_iter()
        .flat_map(home)
        .collect(),
        Os::MacOs => home("Library/Application Support/Steam"),
    }
}

/// The library folders Steam lists in `libraryfolders.vdf` (`"path"` entries), after its
/// own folder.
pub fn libraries(root: &Path) -> Vec<PathBuf> {
    let mut found = vec![root.to_path_buf()];
    let text =
        std::fs::read_to_string(root.join("steamapps/libraryfolders.vdf")).unwrap_or_default();
    for line in text.lines() {
        let tokens = quoted(line);
        if let [key, value, ..] = tokens.as_slice()
            && key == "path"
        {
            let library = PathBuf::from(value);
            if !found.contains(&library) {
                found.push(library);
            }
        }
    }
    found
}

/// The quoted strings of a line of Valve's key-value text, backslashes unescaped.
fn quoted(line: &str) -> Vec<String> {
    let (mut tokens, mut current, mut inside) = (Vec::new(), String::new(), false);
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match (c, inside) {
            ('"', false) => inside = true,
            ('"', true) => {
                tokens.push(std::mem::take(&mut current));
                inside = false;
            }
            ('\\', true) => current.extend(chars.next()),
            (c, true) => current.push(c),
            _ => {}
        }
    }
    tokens
}

/// The folder name Steam gives `game`.
fn steam_name(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "Euro Truck Simulator 2",
        Game::Ats => "American Truck Simulator",
    }
}

/// The existing folders where `game` may be, in Steam's libraries.
pub fn candidates(os: Os, dirs: &Dirs, game: Game) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in steam_roots(os, dirs) {
        for library in libraries(&root) {
            let folder = library.join("steamapps/common").join(steam_name(game));
            // The archives sit in the app bundle on macOS.
            let bundle = folder.join(format!("{}.app/Contents/Resources", steam_name(game)));
            let places = [Some(folder), (os == Os::MacOs).then_some(bundle)];
            found.extend(places.into_iter().flatten().filter(|p| p.is_dir()));
        }
    }
    found
}

/// The first of `game`'s Steam folders that really holds the game.
pub fn detect_in(os: Os, dirs: &Dirs, game: Game) -> Option<(PathBuf, GameInfo)> {
    candidates(os, dirs, game)
        .into_iter()
        .find_map(|folder| check(game, &folder).ok().map(|info| (folder, info)))
}

/// [`detect_in`] on this computer.
pub fn detect(game: Game) -> Option<(PathBuf, GameInfo)> {
    detect_in(Os::CURRENT, &Dirs::of_user(), game)
}

/// Why a folder is refused as `game`'s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FolderProblem {
    Inspect(InspectError),
    /// The folder holds the other game.
    OtherGame(Game),
}

/// Checks that `folder` holds `game`, and reads its version.
pub fn check(game: Game, folder: &Path) -> Result<GameInfo, FolderProblem> {
    let info = extract::inspect(folder).map_err(FolderProblem::Inspect)?;
    if info.game == game {
        Ok(info)
    } else {
        Err(FolderProblem::OtherGame(info.game))
    }
}

/// Why a folder is refused, in the player's language.
pub fn problem_text(problem: &FolderProblem) -> String {
    match problem {
        FolderProblem::Inspect(InspectError::NotAFolder) => tr("games-not-found"),
        FolderProblem::Inspect(InspectError::Missing(file)) => {
            tr!("games-missing-file", file = file.as_str())
        }
        FolderProblem::Inspect(InspectError::NotAGame) => tr("games-not-a-game"),
        FolderProblem::Inspect(InspectError::Unreadable(cause)) => {
            tr!("games-unreadable", cause = cause.as_str())
        }
        FolderProblem::OtherGame(Game::Ets2) => tr("games-is-ets2"),
        FolderProblem::OtherGame(Game::Ats) => tr("games-is-ats"),
    }
}

/// A game's folder as Preferences shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FolderStatus {
    Unset,
    Valid(GameInfo),
    Problem(FolderProblem),
}

impl FolderStatus {
    pub fn of(game: Game, folder: Option<&Path>) -> Self {
        match folder {
            None => Self::Unset,
            Some(folder) => match check(game, folder) {
                Ok(info) => Self::Valid(info),
                Err(problem) => Self::Problem(problem),
            },
        }
    }
}

/// On a first run with no folder set, sets the ones found in Steam.
pub fn detect_first_run(folders: &mut BTreeMap<String, PathBuf>) {
    detect_first_run_in(Os::CURRENT, &Dirs::of_user(), folders);
}

fn detect_first_run_in(os: Os, dirs: &Dirs, folders: &mut BTreeMap<String, PathBuf>) {
    if !folders.is_empty() {
        return;
    }
    for game in Game::ALL {
        if let Some((folder, _)) = detect_in(os, dirs, game) {
            folders.insert(game.code().to_owned(), folder);
        }
    }
}

/// A vehicle's package state against the installed library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Generation {
    NotGenerated,
    /// The installed package covers the game's version.
    Generated(semver::Version),
    /// The game's version is outside the installed package's.
    GameUpdated(semver::Version),
}

pub fn package_id(game: Game, vehicle: &str) -> String {
    format!("official.{}.{vehicle}", game.code())
}

pub fn generation(
    library: &VehicleLibrary,
    game: Game,
    vehicle: &str,
    info: &GameInfo,
) -> Generation {
    let Some(installed) = library.get(&package_id(game, vehicle)) else {
        return Generation::NotGenerated;
    };
    let m = &installed.newest().manifest;
    match info.semver() {
        Ok(version) if !m.game.versions.matches(&version) => {
            Generation::GameUpdated(m.version.clone())
        }
        _ => Generation::Generated(m.version.clone()),
    }
}

/// The game read in a worker thread.
pub struct ScanJob {
    done: Receiver<Result<Scan, String>>,
}

impl ScanJob {
    pub fn start(folder: PathBuf, notify: impl Fn() + Send + 'static) -> Self {
        let (tx, done) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("game-scan".into())
            .spawn(move || {
                let _ = tx.send(extract::scan(&folder));
                notify();
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot start reading the game");
        }
        Self { done }
    }

    fn poll(&self) -> Option<Result<Scan, String>> {
        match self.done.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err("stopped".into())),
        }
    }
}

/// A vehicle to make, with the newest installed package of it.
pub struct Item {
    pub id: String,
    pub previous: Option<PathBuf>,
}

/// What the worker tells, in order.
pub enum Event {
    /// Vehicle number `index` of the run starts.
    Started(usize),
    Made(usize, Result<Option<Vec<u8>>, String>),
    /// The run ended (all done, or cancelled before the next vehicle).
    Finished,
}

/// Generation of a list of vehicles, strictly one after another.
pub struct GenerateJob {
    events: Receiver<Event>,
}

impl GenerateJob {
    /// `cancel` is checked before each vehicle starts; `notify` is called after each event.
    pub fn start(
        scan: Arc<Scan>,
        items: Vec<Item>,
        cancel: Arc<AtomicBool>,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let (tx, events) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("game-generate".into())
            .spawn(move || {
                let send = |event| {
                    let _ = tx.send(event);
                    notify();
                };
                for (index, item) in items.iter().enumerate() {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    send(Event::Started(index));
                    send(Event::Made(index, make(&scan, item)));
                }
                send(Event::Finished);
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot start generating");
        }
        Self { events }
    }
}

/// The package bytes of one vehicle, or `None` when it is up to date.
fn make(scan: &Scan, item: &Item) -> Result<Option<Vec<u8>>, String> {
    let previous = match &item.previous {
        Some(path) => {
            let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
            Some(Package::read(&bytes).map_err(|e| format!("{}: {e}", path.display()))?)
        }
        None => None,
    };
    match extract::generate(scan, &item.id, previous.as_ref())? {
        Outcome::Package(packed) => Ok(Some(packed.bytes)),
        Outcome::UpToDate => Ok(None),
    }
}

/// A checked vehicle's state during a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowState {
    Waiting,
    InProgress,
    /// Installed, with its version.
    Installed(String),
    UpToDate,
    Failed(String),
}

/// A run in progress.
pub struct Run {
    job: GenerateJob,
    cancel: Arc<AtomicBool>,
    /// The vehicles being made, in order: id, name, state.
    pub rows: Vec<(String, String, RowState)>,
    pub cancelling: bool,
}

impl Run {
    /// Vehicles finished (installed, up to date or failed).
    pub fn done(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| !matches!(r.2, RowState::Waiting | RowState::InProgress))
            .count()
    }

    /// The name of the vehicle being made.
    pub fn current(&self) -> Option<&str> {
        self.rows
            .iter()
            .find(|r| r.2 == RowState::InProgress)
            .map(|r| r.1.as_str())
    }

    pub fn state_of(&self, id: &str) -> Option<&RowState> {
        self.rows.iter().find(|r| r.0 == id).map(|r| &r.2)
    }
}

/// How the last run ended.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub installed: usize,
    pub skipped: usize,
    /// Vehicle name and why.
    pub failed: Vec<(String, String)>,
    pub cancelled: bool,
}

/// How a run ended, in a sentence.
pub fn summary_text(summary: &Summary) -> String {
    tr!(
        if summary.cancelled {
            "game-result-cancelled"
        } else {
            "game-result"
        },
        installed = summary.installed.to_string(),
        skipped = summary.skipped.to_string(),
        failed = summary.failed.len().to_string()
    )
}

/// What reading the game gave.
pub enum ScanState {
    Idle,
    Scanning { folder: PathBuf, job: ScanJob },
    Ready { folder: PathBuf, scan: Arc<Scan> },
    Failed { folder: PathBuf, message: String },
}

impl ScanState {
    fn folder(&self) -> Option<&Path> {
        match self {
            Self::Idle => None,
            Self::Scanning { folder, .. }
            | Self::Ready { folder, .. }
            | Self::Failed { folder, .. } => Some(folder),
        }
    }
}

/// The games in the application: their folders, the scan of ETS2 and the generation run.
pub struct GameState {
    /// The game the My Game tab shows.
    pub game: Game,
    /// The check of each game's folder, as Preferences shows it.
    pub folders: BTreeMap<Game, FolderStatus>,
    /// Why the last folder chosen for a game was refused (the previous one stays).
    pub refused: BTreeMap<Game, String>,
    /// The folder found in Steam for a game with no folder set (looked for once).
    pub detected: BTreeMap<Game, Option<PathBuf>>,
    pub scan: ScanState,
    /// Vehicles checked in the My Game tab.
    pub checked: BTreeSet<String>,
    pub run: Option<Run>,
    pub last: Option<Summary>,
    /// A run ended and nobody has been told yet.
    pub announce: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            game: Game::Ets2,
            folders: BTreeMap::new(),
            refused: BTreeMap::new(),
            detected: BTreeMap::new(),
            scan: ScanState::Idle,
            checked: BTreeSet::new(),
            run: None,
            last: None,
            announce: false,
        }
    }
}

impl GameState {
    /// Checks the folders of Preferences again.
    pub fn refresh_folders(&mut self, folders: &BTreeMap<String, PathBuf>) {
        self.refused.clear();
        for game in Game::ALL {
            let folder = folders.get(game.code()).map(PathBuf::as_path);
            self.folders.insert(game, FolderStatus::of(game, folder));
        }
    }

    /// Sets `game`'s folder to `folder` if it holds that game; says why when it does not.
    pub fn set_folder(
        &mut self,
        folders: &mut BTreeMap<String, PathBuf>,
        game: Game,
        folder: &Path,
    ) {
        match check(game, folder) {
            Ok(info) => {
                folders.insert(game.code().to_owned(), folder.to_path_buf());
                self.folders.insert(game, FolderStatus::Valid(info));
                self.refused.remove(&game);
            }
            Err(problem) => {
                self.refused.insert(game, problem_text(&problem));
            }
        }
    }

    /// Forgets `game`'s folder.
    pub fn clear_folder(&mut self, folders: &mut BTreeMap<String, PathBuf>, game: Game) {
        folders.remove(game.code());
        self.folders.insert(game, FolderStatus::Unset);
        self.refused.remove(&game);
    }

    /// Reads the game again.
    pub fn refresh_scan(&mut self) {
        self.detected.clear();
        self.scan = ScanState::Idle;
    }

    /// Starts reading `folder` unless it is the one already read or being read.
    pub fn ensure_scan(&mut self, folder: &Path, notify: impl Fn() + Send + 'static) {
        if self.scan.folder() == Some(folder) {
            return;
        }
        self.checked.clear();
        self.scan = ScanState::Scanning {
            folder: folder.to_path_buf(),
            job: ScanJob::start(folder.to_path_buf(), notify),
        };
    }

    /// The scan, once read.
    pub fn scanned(&self) -> Option<&Arc<Scan>> {
        match &self.scan {
            ScanState::Ready { scan, .. } => Some(scan),
            _ => None,
        }
    }

    /// Starts making the checked vehicles of the scan, if no run is going.
    pub fn generate(&mut self, library: &VehicleLibrary, notify: impl Fn() + Send + 'static) {
        let Some(scan) = self.scanned().cloned() else {
            return;
        };
        if self.run.is_some() {
            return;
        }
        let game = scan.info.game;
        let chosen = scan
            .vehicles
            .iter()
            .filter(|v| self.checked.contains(&v.id))
            .collect::<Vec<_>>();
        if chosen.is_empty() {
            return;
        }
        let items = chosen
            .iter()
            .map(|v| Item {
                id: v.id.clone(),
                previous: library
                    .get(&package_id(game, &v.id))
                    .map(|i| i.newest().path.clone()),
            })
            .collect();
        let rows = chosen
            .iter()
            .map(|v| (v.id.clone(), v.name.clone(), RowState::Waiting))
            .collect();
        let cancel = Arc::new(AtomicBool::new(false));
        self.last = None;
        self.run = Some(Run {
            job: GenerateJob::start(scan, items, cancel.clone(), notify),
            cancel,
            rows,
            cancelling: false,
        });
    }

    /// Stops before the next vehicle; the ones installed stay.
    pub fn cancel(&mut self) {
        if let Some(run) = &mut self.run {
            run.cancel.store(true, Ordering::Relaxed);
            run.cancelling = true;
        }
    }

    /// Takes what the workers sent; installs each package in `library`.
    pub fn poll(&mut self, library: &mut VehicleLibrary) {
        if let ScanState::Scanning { folder, job } = &self.scan
            && let Some(result) = job.poll()
        {
            let folder = folder.clone();
            self.scan = match result {
                Ok(scan) => {
                    self.checked
                        .retain(|id| scan.vehicles.iter().any(|v| v.id == *id));
                    ScanState::Ready {
                        folder,
                        scan: Arc::new(scan),
                    }
                }
                Err(message) => ScanState::Failed { folder, message },
            };
        }
        let Some(run) = &mut self.run else {
            return;
        };
        let mut finished = false;
        let mut stopped = false;
        loop {
            match run.job.events.try_recv() {
                Ok(Event::Started(index)) => run.rows[index].2 = RowState::InProgress,
                Ok(Event::Made(index, result)) => {
                    let name = run.rows[index].1.clone();
                    run.rows[index].2 = match result {
                        Ok(Some(bytes)) => match library.install_bytes(&bytes) {
                            Ok(m) => RowState::Installed(m.version.to_string()),
                            Err(err) => RowState::Failed(crate::vehicles::install_error_message(
                                &err, &name,
                            )),
                        },
                        Ok(None) => RowState::UpToDate,
                        Err(reason) => RowState::Failed(reason),
                    };
                }
                Ok(Event::Finished) => {
                    finished = true;
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    stopped = true;
                    break;
                }
            }
        }
        if finished || stopped {
            let run = self.run.take().expect("a run is going");
            let mut summary = Summary {
                cancelled: run.cancelling || stopped,
                ..Summary::default()
            };
            for (_, name, state) in run.rows {
                match state {
                    RowState::Installed(_) => summary.installed += 1,
                    RowState::UpToDate => summary.skipped += 1,
                    RowState::Failed(reason) => summary.failed.push((name, reason)),
                    RowState::Waiting | RowState::InProgress => {}
                }
            }
            self.checked.clear();
            self.last = Some(summary);
            self.announce = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tp_pack::synthetic::{Vehicle, write_game};

    fn game_folder(parent: &Path, name: &str, application: &str, version: &str) -> PathBuf {
        let folder = parent.join("steamapps/common").join(name);
        std::fs::create_dir_all(&folder).unwrap();
        write_game(&folder, application, version, &[]);
        folder
    }

    fn steam(root: &Path, libraries: &[&Path]) {
        let entries = libraries
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let path = p.display().to_string().replace('\\', "\\\\");
                format!(
                    "\t\"{i}\"\n\t{{\n\t\t\"path\"\t\t\"{path}\"\n\t\t\"label\"\t\t\"\"\n\t}}\n"
                )
            })
            .collect::<String>();
        std::fs::create_dir_all(root.join("steamapps")).unwrap();
        std::fs::write(
            root.join("steamapps/libraryfolders.vdf"),
            format!("\"libraryfolders\"\n{{\n{entries}}}\n"),
        )
        .unwrap();
    }

    #[test]
    fn library_files_are_read_with_their_escapes() {
        assert_eq!(
            quoted("\t\t\"path\"\t\t\"C:\\\\Games\\\\Steam\""),
            ["path", "C:\\Games\\Steam"]
        );
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(libraries(temp.path()), [temp.path()]);
        steam(temp.path(), &[Path::new("/mnt/a"), Path::new("/mnt/b")]);
        assert_eq!(
            libraries(temp.path()),
            [temp.path(), Path::new("/mnt/a"), Path::new("/mnt/b")]
        );
    }

    #[test]
    fn the_game_is_found_in_the_steam_folders_of_each_system() {
        for (os, home_steam) in [
            (Os::Linux, ".local/share/Steam"),
            (Os::Linux, ".steam/steam"),
            (
                Os::Linux,
                ".var/app/com.valvesoftware.Steam/.local/share/Steam",
            ),
            (Os::MacOs, "Library/Application Support/Steam"),
            (Os::Windows, "Program Files (x86)/Steam"),
        ] {
            let temp = tempfile::tempdir().unwrap();
            let dirs = Dirs {
                home: Some(temp.path().to_path_buf()),
                program_files: vec![temp.path().join("Program Files (x86)")],
            };
            let root = temp.path().join(home_steam);
            let ets2 = game_folder(&root, "Euro Truck Simulator 2", "eut2", "1.61.1.9");
            game_folder(&root, "American Truck Simulator", "ats", "1.55.0.1");
            steam(&root, &[]);
            let (folder, info) = detect_in(os, &dirs, Game::Ets2).unwrap();
            assert_eq!(
                (folder, info.version.as_str()),
                (ets2, "1.61.1.9"),
                "{home_steam}"
            );
            let (_, info) = detect_in(os, &dirs, Game::Ats).unwrap();
            assert_eq!(info.game, Game::Ats);
        }
    }

    #[test]
    fn a_second_library_a_mac_bundle_and_a_wrong_folder() {
        let temp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            home: Some(temp.path().join("home")),
            program_files: Vec::new(),
        };
        let root = temp.path().join("home/Library/Application Support/Steam");
        let second = temp.path().join("second");
        steam(&root, &[&second]);
        // The bundle holds the archives; a folder with no archives is not taken.
        let bundle = second.join(
            "steamapps/common/Euro Truck Simulator 2/Euro Truck Simulator 2.app/Contents/Resources",
        );
        std::fs::create_dir_all(&bundle).unwrap();
        write_game(&bundle, "eut2", "1.61.1.9", &[]);
        assert_eq!(detect_in(Os::MacOs, &dirs, Game::Ets2).unwrap().0, bundle);
        assert!(detect_in(Os::Linux, &dirs, Game::Ets2).is_none());
        // ATS folder given as ETS2's.
        assert_eq!(
            check(
                Game::Ets2,
                &game_folder(temp.path(), "ats", "ats", "1.55.0.1")
            ),
            Err(FolderProblem::OtherGame(Game::Ats))
        );
    }

    #[test]
    fn the_first_run_sets_what_it_finds_and_nothing_when_steam_is_absent() {
        let temp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            home: Some(temp.path().to_path_buf()),
            program_files: Vec::new(),
        };
        let mut folders = BTreeMap::new();
        detect_first_run_in(Os::Linux, &dirs, &mut folders);
        assert!(folders.is_empty());
        let root = temp.path().join(".steam/steam");
        let ets2 = game_folder(&root, "Euro Truck Simulator 2", "eut2", "1.61.1.9");
        detect_first_run_in(Os::Linux, &dirs, &mut folders);
        assert_eq!(folders["ets2"], ets2);
        assert!(!folders.contains_key("ats"));
        // Folders already set are left alone.
        folders.insert("ats".into(), "/somewhere".into());
        folders.remove("ets2");
        detect_first_run_in(Os::Linux, &dirs, &mut folders);
        assert!(!folders.contains_key("ets2"));
    }

    #[test]
    fn no_steam_means_nothing_found() {
        let temp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            home: Some(temp.path().to_path_buf()),
            program_files: Vec::new(),
        };
        for os in [Os::Windows, Os::Linux, Os::MacOs] {
            assert!(detect_in(os, &dirs, Game::Ets2).is_none());
        }
    }

    fn folder(vehicles: &[Vehicle]) -> tempfile::TempDir {
        let temp = tempfile::tempdir().unwrap();
        write_game(temp.path(), "eut2", "1.61.1.9", vehicles);
        temp
    }

    fn three() -> [Vehicle<'static>; 3] {
        [
            Vehicle::truck("example.a", "A"),
            Vehicle {
                broken: true,
                ..Vehicle::truck("example.b", "B")
            },
            Vehicle::truck("example.c", "C"),
        ]
    }

    /// A state holding the scan of `game` with `ids` checked.
    fn ready(game: &Path, ids: &[&str]) -> GameState {
        GameState {
            scan: ScanState::Ready {
                folder: game.to_path_buf(),
                scan: Arc::new(extract::scan(game).unwrap()),
            },
            checked: ids.iter().map(|s| (*s).to_string()).collect(),
            ..GameState::default()
        }
    }

    fn finish(state: &mut GameState, library: &mut VehicleLibrary) {
        while state.run.is_some() {
            state.poll(library);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[test]
    fn a_failing_vehicle_does_not_stop_the_others_and_installs_nothing() {
        let game = folder(&three());
        let vehicles = tempfile::tempdir().unwrap();
        let mut library = VehicleLibrary::open(vehicles.path());
        let mut state = ready(game.path(), &["example.a", "example.b", "example.c"]);
        state.generate(&library, || {});
        assert!(state.run.is_some());
        // Only one run at a time.
        state.generate(&library, || {});
        finish(&mut state, &mut library);
        let last = state.last.clone().unwrap();
        assert_eq!((last.installed, last.skipped, last.failed.len()), (2, 0, 1));
        assert_eq!(last.failed[0].0, "Example B");
        assert!(!last.cancelled && state.announce);
        let ids = library
            .vehicles()
            .iter()
            .map(|v| v.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["official.ets2.example.a", "official.ets2.example.c"]);
        assert!(state.checked.is_empty());
        // Again with nothing changed: both are up to date, nothing new is installed.
        state.checked = ["example.a".to_owned()].into();
        state.generate(&library, || {});
        finish(&mut state, &mut library);
        let last = state.last.clone().unwrap();
        assert_eq!((last.installed, last.skipped), (0, 1));
        assert_eq!(
            library
                .get("official.ets2.example.a")
                .unwrap()
                .versions
                .len(),
            1
        );
    }

    #[test]
    fn cancel_stops_before_the_next_vehicle_and_keeps_the_installed_ones() {
        let vehicles = [
            Vehicle::truck("example.a", "A"),
            Vehicle::truck("example.b", "B"),
            Vehicle::truck("example.c", "C"),
        ];
        let game = folder(&vehicles);
        let scan = Arc::new(extract::scan(game.path()).unwrap());
        let items = ["example.a", "example.b", "example.c"]
            .map(|id| Item {
                id: id.into(),
                previous: None,
            })
            .into();
        // The worker cancels itself after its first vehicle's last event (Started, Made).
        let cancel = Arc::new(AtomicBool::new(false));
        let (flag, calls) = (cancel.clone(), std::sync::atomic::AtomicUsize::new(0));
        let job = GenerateJob::start(scan, items, cancel, move || {
            if calls.fetch_add(1, Ordering::Relaxed) == 1 {
                flag.store(true, Ordering::Relaxed);
            }
        });
        let mut seen = Vec::new();
        while let Ok(event) = job.events.recv() {
            seen.push(match event {
                Event::Started(i) => format!("start {i}"),
                Event::Made(i, result) => format!("made {i} {}", result.unwrap().is_some()),
                Event::Finished => "finished".into(),
            });
        }
        assert_eq!(seen, ["start 0", "made 0 true", "finished"]);
    }

    #[test]
    fn a_cancelled_run_is_reported_and_the_game_is_scanned_off_thread() {
        let game = folder(&three());
        let vehicles = tempfile::tempdir().unwrap();
        let mut library = VehicleLibrary::open(vehicles.path());
        let mut state = GameState::default();
        state.ensure_scan(game.path(), || {});
        while state.scanned().is_none() {
            state.poll(&mut library);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(state.scanned().unwrap().vehicles.len(), 3);
        state.checked = ["example.a".to_owned(), "example.c".to_owned()].into();
        state.generate(&library, || {});
        state.cancel();
        finish(&mut state, &mut library);
        let last = state.last.unwrap();
        assert!(last.cancelled);
        // Cancelled before the first vehicle: nothing was installed.
        assert_eq!(last.installed, 0);
        assert!(library.vehicles().is_empty());
    }

    #[test]
    fn folders_are_checked_for_their_game() {
        let game = folder(&[]);
        let mut folders = BTreeMap::new();
        folders.insert("ets2".to_owned(), game.path().to_path_buf());
        folders.insert("ats".to_owned(), game.path().to_path_buf());
        let mut state = GameState::default();
        state.refresh_folders(&folders);
        assert!(matches!(state.folders[&Game::Ets2], FolderStatus::Valid(_)));
        assert_eq!(
            state.folders[&Game::Ats],
            FolderStatus::Problem(FolderProblem::OtherGame(Game::Ets2))
        );
        folders.insert("ets2".to_owned(), game.path().join("gone"));
        state.refresh_folders(&folders);
        assert_eq!(
            state.folders[&Game::Ets2],
            FolderStatus::Problem(FolderProblem::Inspect(InspectError::NotAFolder))
        );
    }
}
