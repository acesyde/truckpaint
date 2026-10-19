## Context

All file dialogs go through the `FileDialogs` trait (`tp-app/src/file_dialogs.rs`), one method per dialog, each returning the answer: `NativeDialogs` calls `rfd::FileDialog`, `ScriptedDialogs` pops queues in tests. `AppState.dialogs` holds it and twelve places call it on the UI thread: Open and Save As (`project_io.rs`), Export Texture, Export Mod's Change…, Place… and the mod pictures (`state.rs`), vehicle package install (three dialogs) and export, template images of a custom vehicle, and the two folder choices of Preferences. On Linux, `rfd`'s synchronous dialog runs `pollster::block_on` on the calling thread, which stops the event loop; see proposal.md for the effect, and the spec for the behavior.

Patterns already in the application: a request left in a field during the frame and handled after it (`vehicle_request`, `palette_request`, `ws.place_request`, `ws.picture_request`), and a worker thread with a channel and a repaint call (`OpenJob`, `ExportJob`, `PackJob`).

## Goals / Non-Goals

**Goals:**
- One path for every file dialog: start, then an answer that comes back to one place that knows what the dialog was for.
- The Linux dialog runs off the UI thread; other systems behave exactly as today.
- Existing tests keep their flow: scripted dialogs answer at once.

**Non-Goals:**
- Threads on Windows and macOS, which cannot be tested here and do not show the problem.
- Attaching the dialog to the TruckPaint window as its parent, a Cancel button on the waiting box, and dialog state kept across an application restart.

## Decisions

**A dialog is data, and its answer is data.** `DialogRequest` is an enum with a variant per dialog (`OpenProject`, `SaveProject { suggested }`, `PickImages`, `SaveExport { suggested }`, `PickPackages`, `PickTemplates`, `SavePackage { suggested }`, `SaveMod { suggested, folder }`, `PickModImage`, `PickFolder { title }`); `DialogAnswer` is `Cancelled` or `Paths(Vec<PathBuf>)`. The trait becomes `start(request, wake)`, `poll() -> Option<DialogAnswer>` and `busy()`, with `show_folder`, `suggested` and `shown_folders` unchanged. A free function `show(&DialogRequest) -> DialogAnswer` holds today's `rfd` calls, so there is one place that builds a dialog. The titles and filters it needs are translated when the request is built, on the UI thread, since the language is a property of the UI.

**The thread is a `cfg(target_os = "linux")` detail of `NativeDialogs::start`.** On Linux it spawns a thread named `file-dialog` that calls `show`, sends the answer and calls `wake` (a repaint). On other systems `start` calls `show` in place and keeps the answer for the next `poll`, which is how they work today. `ScriptedDialogs::start` pops its queue the same way, so the answer is ready for the first `poll`. A thread that cannot start answers `Cancelled` and logs.

**One request path in `AppState`.** Sites no longer call the dialogs; they set `dialog_wanted = Some((request, purpose))`, like `vehicle_request`. `Purpose` says what to do with the answer:

| Purpose | When the answer comes |
|---|---|
| `OpenProject` | `open_file` (starts the background read) |
| `SaveProject { then, session }` | `start_save`, with `after_save = then` |
| `PlaceImages { at }`, `ModPicture(which)` | the workspace's `place_files`, `use_picture_file` |
| `ExportTexture` | the Export Texture dialog starts its job |
| `ExportMod` | Export Mod's destination is changed |
| `ExportPackage { from }` | the package is copied to the chosen file |
| `InstallPackages { into }` | the packages are installed; the result messages go to the New Project, Vehicle Library or Add Vehicle dialog named by `into` |
| `PickTemplates { replace }` | the Custom Vehicle dialog adds or replaces templates |
| `GameFolder(game)`, `ModFolder(code)` | Preferences records the folder |

Each frame, `show()` (1) polls the dialogs and, when an answer is ready, applies it to its purpose; (2) draws the UI; (3) if a request was set and none is pending, starts it, then polls once more so that an answer that is already there (scripted, or a system that waits in place) is applied in the same frame. Sites that sit in a modal reach the dialog's state through `self.modal` (`if let Some(Modal::Export(d)) = &mut self.modal`); when the modal is gone or is another one, the answer is dropped, as the spec says.

```
 frame N                                frame N+k (Linux)
 click Save As
   |  dialog_wanted = (SaveProject, Purpose::SaveProject{then})
   v
 start(request) ----> [file-dialog thread: rfd shows the dialog ...]
   |                                          |
 draw frames, box "Waiting...",               | (answer)
 commands ignored  <---- wake ---------------+
                                    poll() -> answer
                                    Purpose::SaveProject -> start_save(path)
```

**Saving keeps its two exits.** `save(ctx, save_as)` used to return whether a save started, which `save_then` used to set `after_save`. It now returns nothing: when the path is known it starts the save at once; otherwise it asks for the dialog with `Purpose::SaveProject { then }`, and the continuation (`after_save = then`, `start_save`) runs when a file is chosen. A cancelled dialog continues nothing. `save_then` passes the pending action as `then`.

**While a dialog is open, the application is closed to commands.** `dispatch` and `handle_keyboard` return at once, a request made while another is pending is dropped, and a box "Waiting for the file dialog…" (an `egui::Modal` like the opening box, with a spinner, no delay) covers the window. The box has two uses: it keeps clicks from reaching the window on systems where the dialog is not modal, and it tells what the application is waiting for when the dialog is hidden behind the window. `SaveProject` carries the project's session number: an answer for another session is dropped.

**Alternatives considered:** `AsyncFileDialog` with an executor (a second runtime for the same effect); a request path per site that leaves its code in place and takes the answer a frame later (does not fit the commands that have no place to come back to, such as Open, and does not carry `after_save`); setting the TruckPaint window as parent (the system would block the clicks, but it needs an adapter around the raw window handle, and does not fix the not-responding message by itself).

## Risks / Trade-offs

- [The real dialog cannot run in the tests] → A test double answers from another thread after a delay and covers the waiting, the ignored commands, a second request, a cancelled Save As, a stale answer, and the continuation of Save before closing. The Linux thread itself is checked by hand: wait more than ten seconds in Save As and see that GNOME shows no message.
- [A dialog hidden behind the window with the window closed to commands looks like a freeze] → The box says what the application is waiting for; closing the window still works (the close request is not a command).
- [`save()` no longer returns whether a save started] → The two callers outside `save_then` ignore it, and tests read the status bar.
- [Windows and macOS are not changed and not tested] → The behavior there is the one of today by construction: the dialog runs in place.
- [Twelve sites change] → Each change is mechanical (set a request, move the code after the answer into the purpose's handler), and `ScriptedDialogs` keeps the existing tests' flow.

## Migration Plan

None. Order of work: the trait and the request path, then the sites in the order of the table (the project dialogs first, since Save As carries the continuation), then the box, then the test double and the manual check on Linux.

## Open Questions

- Attach the dialog to the TruckPaint window as its parent. Deferred: it changes nothing in the spec and can be added later behind `start`.
