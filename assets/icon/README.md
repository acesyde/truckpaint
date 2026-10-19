# TruckPaint icon

The icon of the app: a front view of a semi-truck cab, in off-white with a red
livery stripe, on a dark rounded square (the colors of the app's design tokens).

| File | Use |
|---|---|
| `truckpaint-1024.png` | The master. A 1024 px canvas with transparency; the rounded square is 824 by 824 px (about 80 % of the canvas, as macOS icons are), centered. |
| `truckpaint-512.png` | Built into the program (`crates/tp-app/src/icon.rs`) and given to the window. |
| `truckpaint-256.png`, `-128`, `-64`, `-32`, `-16` | Reductions of the master, for the icon formats that come later (macOS `.icns`, Windows `.ico`, Linux). |

The pictures in this folder are the source of truth. Nothing generates them at
build time.

## Where the picture comes from

It was made with an image generator (ChatGPT), at the maintainer's request, on
2026-10-19. The prompt that was given (written for the maintainer, who may have
adjusted it) asked for a flat vector-style icon on a dark rounded square, with
the palette `#34343a` to `#1c1c1f` for the background, `#ececec` for the cab,
`#a9a9b0` for the bumper and mirrors, `#3a3a40` for the wheels and `#f56b6b` for
one flowing red livery stripe, and said:

- a front view of a modern semi-truck cab (a big rig, not a bus, van or pickup);
- no text, letters or numbers;
- no real manufacturer logo or badge, no logo of any game or studio, and not an
  exact copy of an existing truck;
- simple shapes that stay recognizable at 32 and 16 px.

The generated picture is a 1254 px PNG. It is not kept in the repository: the
master was made from it with `scripts/icon.sh` (below).

**Rights.** The copyright status of a picture made by an image generator is
uncertain in some countries. The maintainer accepts it for this project, which
is under the MIT license. The picture may be replaced by a drawn one later; the
file names would not change.

## Remake the pictures

You only need this to change the picture. It needs `rsvg-convert` (librsvg),
`sips` (macOS) and `python3`, and a PNG with a transparent margin that holds the
rounded square of the icon (8-bit RGBA, no interlacing).

```sh
bash scripts/icon.sh path/to/generated.png          # writes assets/icon/truckpaint-*.png
python3 -I scripts/icon-box.py assets/icon/truckpaint-1024.png   # check the box
```

`scripts/icon.sh` measures the rounded square with `scripts/icon-box.py` (the
standard library only, a pixel counts when its alpha is at least 128), scales
it to 824 by 824 px, centers it on a transparent 1024 px canvas with
`rsvg-convert`, then reduces the master with `sips -z`. The check prints the
canvas, the box (`width=824 height=824`, `center=512.0,512.0`) and the alpha of
the corners (`0`) and of the middle (the picture is not fully opaque inside: the
middle is 253 out of 255).

Checks that the tests make: the master is 1024 by 1024, its corners are
transparent, and its box is about 80 % of the canvas and centered.
