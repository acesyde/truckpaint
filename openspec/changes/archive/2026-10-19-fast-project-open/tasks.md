## 1. File format: the version in its own entry

- [x] 1.1 Write a `format` entry (the version as text) right after `mimetype`, remove the `format` field from the document, read the entry before inflating the document, and refuse a missing or newer version; verify with `tp-file` tests for the round trip, a missing entry (damaged) and a newer version (newer-version message)
- [x] 1.2 Delete `Header`, `is_development_v1`, the `Error::Unsupported` variant and its use in `migrate`, the message `file-dev-format` in the four locales, and the three `dev-*.truckpaint` fixtures with their tests; verify `mise run test` has no reference left (`grep -r "dev-format\|development format" crates` returns nothing)
- [x] 1.3 Write `v1.truckpaint` and `v1-palette-colors.truckpaint` again with the new entry, keeping their content; verify the existing tests that read them pass unchanged
- [x] 1.4 Add a test that writes and reads a project of several megabytes (about 1,300 text objects with outlines across 30 textures) and asserts reading takes well under a second; verify it fails when a whole-document parse that ignores fields is put back

## 2. Opening in the background

- [x] 2.1 Add `OpenJob` in `tp-app` (thread `project-open`, channel, repaint call, `poll`) after `export.rs`, and keep `opening: Option<OpenJob>` in the app state; verify with a unit test that a job over a temp file delivers `Opened` and one over a damaged file delivers the error
- [x] 2.2 Split `open_file`: the guards (same file open, already opening) and the job start, then the existing tail (game data, `open_document`, texts laid out again, recent entry) run when the frame polls the finished job; the project that was open is replaced only then. Verify with tests that an unreadable file keeps the open project and shows the damaged-file message, and that a second open while reading starts nothing
- [x] 2.3 Show the box "Opening <name>…" (modal with a spinner, as the other dialogs) only after a quarter of a second, and add `opening-project` to the four locales; verify with a headless test that no box exists in the first frames of a quick open, that it exists once the delay has passed with a slow job, and that German fits (`mise run screenshots` once)
- [x] 2.4 Update the `tp-app` tests that open a file and expect the project at once (`persistence.rs` and the others that call `PendingAction::Open`) to run frames until the job is done, with one helper in `tests/common`; verify `mise run test` passes

## 3. Check and wrap up

- [x] 3.1 Profile the opening of a large project again (the profile of the exploration: about 1,300 objects, 2.7 MB) in the debug build and note the time of each step; verify the window keeps answering while it reads and that the part left on the UI thread (text layout, workspace) is below a few tens of milliseconds, or note the follow-up
- [x] 3.2 Run `mise run ci`; verify it passes, and that `mise run doc:media` is not needed (no documented screen changed)
