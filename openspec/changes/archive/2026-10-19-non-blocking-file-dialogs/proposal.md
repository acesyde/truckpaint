## Why

On Linux, TruckPaint stops answering while a file dialog is open. Save As, Open, Export and the other dialogs wait for the answer on the thread that draws the window, so after a few seconds GNOME offers to force the application to quit ("truckpaint does not respond"). Taking a moment to choose a folder is normal, so this shows up often, and "Force quit" loses unsaved work.

## What Changes

- On Linux, a file dialog runs on its own thread. The window keeps drawing and answering the system while the dialog is open, so it is never reported as not responding. Windows and macOS keep waiting in place as today: their dialogs already keep the window alive.
- While a file dialog is open, a small box "Waiting for the file dialog…" shows over the window, and menus, shortcuts and a second dialog do nothing, so nothing else changes under the dialog. If the dialog is hidden behind the window, the box says what the application is waiting for.
- The answer of a dialog is used by what asked for it, once, and never by anything else: choosing a file in Save As continues the Save As (and the Quit or Open that came with "Save changes before closing?"), cancelling continues nothing.
- Every file dialog goes through one path (twelve places today, each calling the dialog directly): the answer comes back to a single place that knows what the dialog was for.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-files`: adds a requirement that file dialogs keep the window alive and that their answer goes only to what asked for it. Save As, Open, Export, Place, Install and the folder choices of Preferences all rely on it.

## Impact

- `tp-app`: the `FileDialogs` trait (`file_dialogs.rs`) changes from "ask and wait" to "start, then poll"; the twelve call sites move to one request path in `AppState`; a new box in `ui/` and one message in `tp-i18n` (four languages).
- Tests: `ScriptedDialogs` keeps answering at once, so existing tests keep their flow; a test double that answers later covers the waiting, the ignored commands and a stale answer.
- No new dependency. Windows and macOS behavior is not changed and is not tested here.
- Left out on purpose: attaching the dialog to the TruckPaint window as its parent (it would make the system block clicks on the window for us, but needs a window-handle adapter), a Cancel button on the waiting box, and Windows and macOS threads.
