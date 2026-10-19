## Context

See proposal.md for why, and the `app-icon` spec for the behavior. Today:

- There is no icon anywhere: `assets/` holds fonts only, the site has no favicon, `tp-app` has no `build.rs`, no `Info.plist` and no call to `with_icon`.
- The window is made in `crates/tp-app/src/main.rs` from `egui::ViewportBuilder` (title, app id `truckpaint`, size), which `title_bar::initial_viewport` then adapts to the title bar mode (macOS native, drawn without decorations, or the system's). The icon does not depend on that mode.
- `eframe` 0.36.2 sets the icon of the viewport at run time (`native/app_icon.rs`): on Windows it sends the window messages for the small and big icons, on macOS it sets the image of the application (the Dock icon) from raw pixels, deliberately not from a PNG, because of a macOS bug that can crash the app. On Linux the viewport icon goes through `winit`. `eframe::icon_data::from_png_bytes` turns PNG bytes into the `IconData` that `with_icon` takes. `image` is already a dependency of `tp-app`, with its PNG feature.
- The interface draws a vehicle glyph from the Phosphor icon font as its brand mark in a few places (About, home, menu bar, new project, drawn title bar). That is a different thing from the app icon.
- The picture comes from an image generator (ChatGPT), asked by the maintainer with a written prompt; the result is a 1254 px PNG with transparency, whose rounded square fills about 87 % of the canvas.

## Goals / Non-Goals

**Goals:**
- The window, the taskbar or task switcher and the Dock show the TruckPaint icon, with no file next to the program.
- The picture and its sizes are in the repository, reproducible from a written recipe.

**Non-Goals:**
- The macOS bundle, the Windows `.exe` resource, the Linux desktop entry and the changes to the interface (listed in the proposal).
- A hand-drawn vector source or a simplified 16 px version.
- Code signing.

## Decisions

### The master is a recropped 1024 px PNG
macOS draws a third-party icon as given, so the rounded square must carry its own margin: about 80 % of the canvas, as other macOS icons do. The generated picture fills 87 %, so it would look larger than its neighbors in the Dock. The recipe, as the script `scripts/icon.sh` described in `assets/icon/README.md`:

1. measure the box of the rounded square in the source from its alpha channel, with `scripts/icon-box.py` (the Python standard library only, a pixel counts as opaque when its alpha is at least 128; fainter pixels are a soft shadow that is not part of the box);
2. scale it so that this box is 824 by 824 px (80.5 % of 1024) and put it centered on a transparent 1024 px canvas, by rendering a small SVG that embeds the source picture with `rsvg-convert`: the generated square is a little wider than tall (1096 by 1078 px), so it is stretched by the difference in height, 1.7 %, which the drawing does not show;
3. save it as `assets/icon/truckpaint-1024.png`.

`scripts/icon.sh` does the three steps and the reductions, so the recipe cannot drift from a README. The picture is not fully opaque inside (alpha 253 out of 255 at the middle), so "opaque" in the tests means an alpha of at least 250 (center) or 128 (box).

The other sizes (512, 256, 128, 64, 32, 16) are reductions of the master made with `sips -z` (macOS), and they have the same picture. Both tools are on the maintainer's Mac and are not pinned in `mise.toml`: the committed files are the source of truth, and a contributor only needs the recipe to remake them. The original 1254 px image is not committed; the README keeps the prompt and the date instead.

Alternatives: keep the generated picture as it is (rejected: wrong margin on macOS); redraw it as an SVG (rejected for now: the shapes are flat and could be redrawn, but a 1024 px raster is enough for every output below, and an SVG can come later without changing the files' names); `sips` for the recrop (rejected: its padding color is not guaranteed to be transparent).

### The icon is built into the program
`crates/tp-app/src/icon.rs` holds the 512 px PNG with `include_bytes!` and one function, `app_icon() -> Option<Arc<IconData>>`, which decodes it with `eframe::icon_data::from_png_bytes`. A private `decode(&[u8])` does the work so that a test can give it bad bytes. A decoding error writes a `tracing::warn!` and returns `None`, and `main.rs` adds `.with_icon(...)` to the viewport builder only when there is an icon, so a broken picture never stops the app. 512 px is enough for the Dock at its largest and for the scaling Windows does for the title bar and the taskbar, and it costs about 1 MB of pixels in memory.

Alternatives: load the file from next to the program (rejected: a download of one program must show the icon); embed the 1024 px master (rejected: four times the memory for no visible gain).

### What each platform does with it
No new platform code: `eframe` already does it (see Context). `with_icon` is set on the base builder before `title_bar::initial_viewport`, so the three title bar modes get it. On Wayland the icon of a window usually comes from the desktop entry that matches the app id, not from the program, so a Linux session on Wayland may keep a generic icon until the desktop entry change; this was not checked here.

### Origin and rights
`assets/icon/README.md` states that the picture was made with an image generator at the maintainer's request, gives the prompt and the date, and says that the copyright status of generated pictures is uncertain in some countries and that the maintainer accepts it for this MIT project. The icon contains no text and no logo; this is checked by looking at the master.

## Risks / Trade-offs

- [At 16 px the picture is a gray block with a bit of red] → accepted: 16 px is rarely shown (non-Retina lists, a Windows title bar at 100 % scale), and a simplified version can be added later.
- [The 80 % margin makes the icon look smaller on Windows and Linux than a full-bleed one] → accepted: the macOS shape is the reference, and the other systems show it on their own background.
- [The generated picture's copyright status] → recorded in the README and accepted by the maintainer; a redraw as SVG remains possible.
- [A moved or renamed icon file breaks `include_bytes!`] → the build fails at once, so it cannot reach a download.
- [A short flash of the generic icon before the first frame] → `eframe` sets the icon on every frame until the window exists; nothing to do.
- [Wayland and the Dock icon were not run here] → the manual check at the end of the tasks looks at macOS; Windows and Linux are looked at on the first pre-release that holds the icon.

## Migration Plan

Nothing to migrate. Rollback is removing the `with_icon` call: the app goes back to the generic icon. The icon is only seen by players once a release is published with it; the downloads are made by the existing release workflows, unchanged.

## Open Questions

- None that change the specs or the tasks. How Wayland shows the icon is checked later on a real session.
