## 1. The dialog seam

- [x] 1.1 Add `DialogRequest`, `DialogAnswer` and the free function that shows a request with `rfd` (the only place that builds an `rfd` dialog, titles and filters translated when the request is built); change `FileDialogs` to `start`, `poll` and `busy`; make `NativeDialogs` run `show` on a `file-dialog` thread on Linux and in place elsewhere, and `ScriptedDialogs` pop its queues at `start`; verify with unit tests that a scripted answer is ready at the first `poll`, and that `grep -rn "rfd::" crates/tp-app/src` finds only the `show` function
- [x] 1.2 Add a test double (`SlowDialogs`, in `tests/file_dialogs.rs`) that answers when the test releases it, as the Linux thread does; verify with a test that `busy` stays true until released, `poll` gives the answer once, and a repaint was requested

## 2. The request path in the application

- [x] 2.1 Add `dialog_wanted`, `dialog_pending` and the `Purpose` enum to `AppState`, and in `show()` poll, draw, start a wanted request and poll again; a request while one is pending is dropped; verify with tests using the double that a second request starts nothing and that an answer is applied once, to its purpose
- [x] 2.2 Make `dispatch` and `handle_keyboard` do nothing while a dialog is pending, and show the box "Waiting for the file dialog…" (modal with a spinner, as the opening box) with the message `file-dialog-waiting` in the four languages; verify with a test that Cmd/Ctrl+N and Cmd/Ctrl+W change nothing while pending and that the box exists then and not after; check the German text in a temporary rendering

## 3. The twelve call sites

- [x] 3.1 Open and Save As: set the requests, move the code after the answer into the `OpenProject` and `SaveProject { then, session }` handlers, and make `save()` and `save_then` use them; verify with the existing persistence tests, plus tests that cancelling Save As in Save before closing keeps the project open and that choosing a file saves then closes
- [x] 3.2 Place… and the mod pictures: set the requests from `place_request` and `picture_request` and apply the answers to the workspace; verify with the existing image and mod settings tests
- [x] 3.3 Export Texture, Export Mod's Change… and the package export: reach the open dialog through `self.modal` in the handlers; verify with the existing export tests, plus a test that an answer arriving after the dialog was closed is dropped without a message
- [x] 3.4 Package install in New Project, Vehicle Library and Add Vehicle, and the template images of Custom Vehicle: one `InstallPackages { into }` and one `PickTemplates { replace }` handler; verify with the existing vehicle and custom vehicle tests
- [x] 3.5 The two folder choices of Preferences: the Preferences code returns a request instead of taking the dialogs, and the handlers record the folder; verify with the existing preferences tests

## 4. Check

- [x] 4.1 Remove the old synchronous trait methods, verify that no code outside `show` calls a dialog (`grep`), and that `mise run ci` passes
- [x] 4.2 Check on Linux with the real application: open Save As, wait more than ten seconds, and verify that GNOME shows no "does not respond" message, that the box is shown and menus and shortcuts do nothing, and that choosing a file saves it; repeat for Open and Export Texture
