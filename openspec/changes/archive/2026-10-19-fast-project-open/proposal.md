## Why

Opening a large project freezes TruckPaint. A project with about 1,300 objects (a 2.7 MB file) took over a minute to open, and the window did not answer meanwhile. A profile showed that 95 % of the time went into reading the file twice before the real read, and that each of those reads gets slower as the square of the file size. Real projects with many imported shapes get there too. Even once reading is fast, a project that takes a moment to open should say so instead of looking frozen.

## What Changes

- Reading a project file no longer parses the whole document just to learn its format version or to spot files from development builds. The version is stored in its own entry of the file and read first, alone.
- **BREAKING (development builds only)**: project files written by earlier, unreleased builds are not recognized any more. They open as "damaged", not as "development format". TruckPaint is not released, so no player has such a file.
- Opening a project reads the file in the background. A small box, "Opening <name>…", shows while it reads; the window keeps answering. Nothing else can be started meanwhile.
- The project is shown only when it is ready. A file that cannot be opened shows the same messages as today, and the project already open stays open.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-files`: the format version is recorded in its own entry; files of development builds are no longer told apart from damaged files ("Versioned format", "Invalid files"); opening a project shows a loading box and does not freeze the window ("Open a project").

## Impact

- `tp-file`: `from_bytes` and the writer (new entry for the format version; `is_development_v1` and its tests removed).
- `tp-app`: `open_file` in `project_io.rs` becomes a background job with a small box (same pattern as the texture export job); `AppState` and the frame loop poll it.
- `tp-i18n`: one new message ("Opening <name>…") in English, French, Spanish and German.
- Tests: file round trip and invalid-file tests in `tp-file`; a headless UI test that the window answers while a project opens.
- No new dependency. The `ron` defect behind the slowness stays in the library (fixed upstream, not released yet); it no longer matters here because no whole-document read is left that ignores fields.
- Left out on purpose: reading each file once instead of twice (the recent-projects thumbnail reads it again), restoring a recovered project in the background, and a cancel button on the loading box.
