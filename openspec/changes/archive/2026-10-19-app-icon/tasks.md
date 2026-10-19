## 1. Icon pictures

- [x] 1.1 Measure the box of the rounded square in the generated picture (`~/Downloads/Symmetrical Semi-Truck App Icon.png`, 1254 px) from its alpha channel, with a Python script (`scripts/icon-box.py`) using only the standard library, and verify that it prints the box (left, top, right, bottom) and its width
- [x] 1.2 Make `assets/icon/truckpaint-1024.png`: scale the picture so that the box is 824 px wide, put it centered on a transparent 1024 px canvas by rendering an SVG that embeds it with `rsvg-convert`; verify with `sips -g pixelWidth -g pixelHeight -g hasAlpha` (1024 by 1024, with alpha) and by measuring the box again (824 px wide within 4 px, centered within 2 px, corner pixels fully transparent)
- [x] 1.3 Make `truckpaint-512.png`, `-256`, `-128`, `-64`, `-32` and `-16` from the master with `sips -z`, and verify that the seven files exist in `assets/icon/` with the right sizes
- [x] 1.4 Look at the master at 1024 px and the 32 px version, and verify that the picture has no text and no logo of a vehicle maker, a game or a studio
- [x] 1.5 Write `assets/icon/README.md` (origin: made with an image generator at the maintainer's request, the prompt that was given, the date, the rights note, and the commands that remake the files, in a script `scripts/icon.sh`), and verify by running the script in an empty scratch folder that it gives the same files

## 2. Code

- [x] 2.1 Add `crates/tp-app/src/icon.rs` (declared in `lib.rs`) with the 512 px picture built in by `include_bytes!`, a private `decode(&[u8]) -> Option<Arc<IconData>>` that writes a `tracing::warn!` on error, and `pub fn app_icon() -> Option<Arc<IconData>>`; verify with unit tests: the built-in picture decodes to 512 by 512 with 512 x 512 x 4 bytes, its four corners have an alpha of 0 and its center an alpha of at least 250, and `decode(b"not a png")` is `None`
- [x] 2.2 Add a test that reads the 1024 px master (built into the test with `include_bytes!`) and checks that it is 1024 by 1024 with transparent corners and that the box of its opaque pixels (alpha of at least 128) is between 78 % and 82 % of the canvas and centered, and verify it passes
- [x] 2.3 In `crates/tp-app/src/main.rs`, add `.with_icon(...)` to the base viewport builder, before `title_bar::initial_viewport`, only when `app_icon()` is `Some`; verify with `cargo build -p tp-app` and `mise run run` that the app starts and that the log has no icon warning
- [x] 2.4 Delegate `mise run ci` to the rust-ui-tester agent, ask for the exit code of `fmt:check`, `lint`, `test` and `build` read from the commands, and fix its failures (fmt:check, lint, test and build all exited 0; 1577 tests passed, 0 failed, 59 ignored for the GPU and timing ones; the three icon tests ran.)

## 3. Check by eye (by the maintainer)

- [x] 3.1 Run `mise run run` on the Mac and verify that the Dock shows the TruckPaint icon, not the generic one, with each title bar mode that Preferences offers on macOS (The maintainer confirmed that the Dock shows the icon; the title bar mode in use was the native one, and no other mode was reported.)

## 4. Documentation

- [x] 4.1 Delegate the docs-drift-checker agent and list in the apply summary any stale page (none is expected: no screen changed and the Installation pages do not show the icon) (Found and fixed: the Installation page, in the four languages, said the app had no icon; that sentence was removed, as it told players nothing useful. Left to the maintainer: a mention of the icon's origin in the README credits.)
