//! Reading a project file in a background thread, so that the window keeps
//! answering while a large project opens. The thread only reads the file
//! (`tp_file::read`); the project is shown by the UI thread when it is done.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

/// Seconds a read takes before the "Opening…" box shows; a quick open
/// must not flash it.
pub const BOX_DELAY: f64 = 0.25;

pub struct OpenJob {
    pub path: PathBuf,
    /// The UI clock when the read started.
    pub started: f64,
    done: Receiver<Result<tp_file::Opened, tp_file::Error>>,
}

impl OpenJob {
    /// Starts reading `path`; `notify` wakes the UI when the read ends.
    pub fn start(path: PathBuf, started: f64, notify: impl Fn() + Send + 'static) -> Self {
        let (tx, done) = mpsc::channel();
        let target = path.clone();
        let spawned = thread::Builder::new()
            .name("project-open".into())
            .spawn(move || {
                let _ = tx.send(tp_file::read(&target));
                notify();
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot start reading the project");
        }
        Self {
            path,
            started,
            done,
        }
    }

    /// The result once the file is read.
    pub fn poll(&self) -> Option<Result<tp_file::Opened, tp_file::Error>> {
        match self.done.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            // The thread could not start or stopped without a result.
            Err(TryRecvError::Disconnected) => Some(Err(tp_file::Error::Io(
                std::io::Error::other("reading stopped"),
            ))),
        }
    }

    /// Whether the "Opening…" box shows at UI time `now`.
    pub fn box_shown(&self, now: f64) -> bool {
        now - self.started >= BOX_DELAY
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    fn wait(job: &OpenJob) -> Result<tp_file::Opened, tp_file::Error> {
        let until = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(result) = job.poll() {
                return result;
            }
            assert!(Instant::now() < until, "the job never ended");
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn delivers_the_project_of_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.truckpaint");
        tp_file::write(&crate::vehicle_project::test_project("a"), &path).unwrap();
        let job = OpenJob::start(path, 0.0, || {});
        assert_eq!(wait(&job).unwrap().project.name, "a");
    }

    #[test]
    fn delivers_the_error_of_a_damaged_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.truckpaint");
        std::fs::write(&path, b"not a project").unwrap();
        let job = OpenJob::start(path, 0.0, || {});
        assert!(matches!(wait(&job), Err(tp_file::Error::NotAProject)));
    }

    #[test]
    fn the_box_waits_for_its_delay() {
        let job = OpenJob::start(PathBuf::from("/missing.truckpaint"), 10.0, || {});
        assert!(!job.box_shown(10.1));
        assert!(job.box_shown(10.0 + BOX_DELAY));
    }
}
