## Why

TruckPaint has no icon. Its window, its place in the taskbar or task switcher and its icon in the macOS Dock are the system's generic ones, so the app is hard to find among open windows and looks unfinished. Players also expect to recognize a program by its picture.

## What Changes

- TruckPaint gets an icon: a front view of a semi-truck cab, in off-white with a red livery stripe, on a dark rounded square, in the colors of the app.
- The app shows it as its window icon, in the taskbar or task switcher, and in the macOS Dock, whichever way the title bar is shown (native, drawn or system).
- The picture is kept in the repository as a 1024 px master (the rounded square fills about 80 % of the canvas, with a transparent margin) and as smaller sizes made from it: 512, 256, 128, 64, 32 and 16 px. A note next to the files records where the picture comes from and how the sizes were made.
- The icon is part of the program, so it needs no file next to the downloads. If the picture cannot be read, the app still starts, without an icon.
- Out of scope, planned as separate changes: a `TruckPaint.app` for macOS and the icon in the Windows `.exe` and a Linux desktop entry (which also give the icon to launchers, the Finder and the Explorer), and using the picture in place of the vehicle glyph that the interface draws in a few places (About, home screen, menu bar, new project, drawn title bar).

## Capabilities

### New Capabilities
- `app-icon`: the icon of the app, where it is shown, what the picture files are, and what happens when it cannot be read.

### Modified Capabilities

None. The `window-title-bar` capability is not changed: the icon is set on the window, not drawn in the bar.

## Impact

- New: `assets/icon/` (the master, the smaller sizes and a note on their origin).
- Code: the window options in `tp-app`'s `main.rs` get the icon; a small module in `tp-app` embeds the picture and decodes it, with unit tests.
- No change to projects, vehicle packages, the mod export, the downloads or the release workflows.
- The picture was made with an image generator, at the maintainer's request. Its copyright status is uncertain in some countries; the maintainer accepts it for this project (MIT).
