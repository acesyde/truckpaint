## Context

`tp_file::from_bytes` (`crates/tp-file/src/lib.rs`) opens a project in four steps: `open_zip` (checks `mimetype`, bounds the unpacked size, inflates `project.ron`), `header` (the format version), `is_development_v1` (spots files of unreleased builds) and `migrate` (the real parse). `header` and `is_development_v1` parse the whole document into structures that name only a few fields, so `ron` walks everything else with `IgnoredAny`. In `ron` 0.12.2 that path looks for `".."` in the rest of the file at every number, which makes it quadratic: a 2.7 MB document spent about 80 s of CPU there (a `perf` profile: 95 % of all samples). The defect is known and fixed upstream but not released; the typed parse in `migrate` is not affected. See proposal.md for motivation and specs/project-files for the behavior.

`open_file` (`tp-app/src/project_io.rs`) runs on the UI thread: it reads the file, fills the game data, replaces the workspace and lays out every text again. The texture export already shows the pattern for slow work (`export.rs`): a named thread, an `mpsc` channel, a progress value, a repaint call, and a polled outcome shown in a dialog.

## Goals / Non-Goals

**Goals:**
- No whole-document parse that ignores fields, so reading is linear in the file size.
- The window never waits for the file; a slow open says what it is doing.

**Non-Goals:**
- Fixing or patching `ron`. Nothing left in TruckPaint depends on the defect; the typed parse stays.
- Reading the file once instead of twice (the recent-projects thumbnail reads it again in its own thread) and restoring a recovered project in the background. Both use the same read function, so they benefit from the fix, not from the box.
- A cancel button, a progress bar or reading in several steps.
- Moving the work that needs the workspace (game data, text layout) off the UI thread. It is measured at the end (see tasks); a follow-up change handles it only if it is still visible.

## Decisions

**The format version goes in its own entry of the zip, `format`, written first after `mimetype`** (the version as text, like `1`). The reader checks `mimetype`, reads `format`, refuses a newer or missing version, and only then inflates and parses the document. The document no longer holds a `format` field, so `Header` is deleted. Alternatives: scan the first bytes of the document for `format:` (no file change, but it depends on the exact layout of the pretty-printed text); update `ron` to the upstream commit (a git dependency until the next release, and the two extra parses would stay). A separate entry costs one small zip entry and keeps the check independent of the document's syntax.

**`is_development_v1` and the `Error::Unsupported` variant are deleted**, with their message in the four languages and the three `dev-*.truckpaint` test fixtures. TruckPaint is not released, so no player has such a file; a leftover development file has no `format` entry and is refused as damaged. `v1.truckpaint` and `v1-palette-colors.truckpaint` (fixtures of the format that will be the first released) are written again with the new entry; their content does not change.

**Opening runs in a job, modeled on the texture export:** a new `OpenJob` in `tp-app` (`open_job.rs`) with a thread named `project-open`, a channel and the repaint call. The thread does `tp_file::read(path)` and sends `Result<Opened, Error>`. The app keeps `opening: Option<OpenJob>`; the frame polls it. Nothing about the workspace is touched by the thread.

**`open_file` splits in two.** The first half (same file already open → keep it; already opening → ignore) starts the job. The second half, run when the job delivers, is today's tail unchanged: fill the game data, replace the workspace (`open_document`), lay the texts out again, push the recent entry. The project that was open is replaced only here, so an unreadable file leaves it in place, and the unsaved-changes question (already asked before `PendingAction::Open`) is not asked again. A failure shows the same message as today.

**The box appears after a quarter of a second and only then.** A quick open of a small file must not flash a box. The box is an `egui::Modal` like the other dialogs (`ui::dialogs::modal`), small, with the title "Opening <name>…" and a spinner; the backdrop keeps clicks from reaching the window, so no second open can start by hand, and `open_file` ignores one that arrives by another route (a recent project clicked in the same frame, a command). Closing the window while a file is being read quits as usual; the thread ends on its own and its result is dropped.

**A message for the box, in the four languages** (`new-project`-style Fluent id `opening-project`, with the file name). English: "Opening {name}…", sentence case as for other messages in progress.

## Risks / Trade-offs

- [Development files stop opening, including the ones on the author's machine (`DAF NGD.truckpaint`, and the test file `heavy.truckpaint`)] → Accepted: the app is not released and the project rules say to skip backward compatibility. Such a file can be converted once by an older build, or recreated.
- [A frame loop that polls a job is new for opening; tests that call `open_file` and expect the project at once now see it a frame later] → `tp-app` tests use a helper that runs frames until the job is done; `common` gets it in one place.
- [The part that stays on the UI thread (text layout, workspace) may still be slow on very large projects] → Measured at the end on `heavy.truckpaint`-like data; if it takes more than a few tens of milliseconds, a follow-up moves it.
- [A file read twice in parallel at open (project and recent thumbnail)] → Left as is; once reading is linear it costs milliseconds.
- [`ron` stays at 0.12.2 with its defect] → No typed read hits it. If a future change adds a read that ignores most of a document, the same slowdown returns; the large-file scenario in the spec and a test with a multi-megabyte document are there to catch that.

## Migration Plan

None for players. Development files are not converted. Order of work: the file format and its tests first, then the job and the box.
