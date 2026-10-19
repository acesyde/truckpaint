#!/usr/bin/env python3
"""Measures the opaque part of an icon PNG, with the standard library only.

Usage: icon-box.py FILE [MIN_ALPHA]

FILE is an 8-bit RGBA PNG without interlacing (what `sips` and `rsvg-convert`
write). A pixel counts as opaque when its alpha is at least MIN_ALPHA (default
128). Prints the canvas, the box of the opaque pixels, that box's size and
center, and the alpha of the four corners and of the center, as `key=value`
lines. See assets/icon/README.md.
"""

import struct
import sys
import zlib


def read_alpha(path):
    """Returns (width, height, rows) where rows[y][x] is the alpha of a pixel."""
    with open(path, "rb") as f:
        data = f.read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"{path}: not a PNG")
    pos, idat, header = 8, [], None
    while pos < len(data):
        length, kind = struct.unpack(">I4s", data[pos : pos + 8])
        body = data[pos + 8 : pos + 8 + length]
        pos += 12 + length
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", body)
        elif kind == b"IDAT":
            idat.append(body)
        elif kind == b"IEND":
            break
    width, height, depth, color, _, _, interlace = header
    if (depth, color, interlace) != (8, 6, 0):
        raise SystemExit(f"{path}: needs an 8-bit RGBA PNG without interlacing")
    raw = zlib.decompress(b"".join(idat))
    stride = width * 4
    rows, prev = [], bytearray(stride)
    for y in range(height):
        start = y * (stride + 1)
        kind = raw[start]
        line = bytearray(raw[start + 1 : start + 1 + stride])
        if kind == 1:
            for i in range(4, stride):
                line[i] = (line[i] + line[i - 4]) & 255
        elif kind == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 255
        elif kind == 3:
            for i in range(stride):
                left = line[i - 4] if i >= 4 else 0
                line[i] = (line[i] + ((left + prev[i]) >> 1)) & 255
        elif kind == 4:
            for i in range(stride):
                a = line[i - 4] if i >= 4 else 0
                b = prev[i]
                c = prev[i - 4] if i >= 4 else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pred = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                line[i] = (line[i] + pred) & 255
        elif kind != 0:
            raise SystemExit(f"{path}: unknown filter {kind}")
        rows.append(line[3::4])
        prev = line
    return width, height, rows


def main():
    if len(sys.argv) not in (2, 3):
        raise SystemExit(__doc__)
    minimum = int(sys.argv[2]) if len(sys.argv) == 3 else 128
    width, height, alpha = read_alpha(sys.argv[1])
    left, top, right, bottom = width, height, -1, -1
    for y in range(height):
        row = alpha[y]
        xs = [x for x in range(width) if row[x] >= minimum]
        if xs:
            top = min(top, y)
            bottom = max(bottom, y)
            left = min(left, xs[0])
            right = max(right, xs[-1])
    if right < 0:
        raise SystemExit("no opaque pixel")
    print(f"canvas={width}x{height}")
    print(f"left={left} top={top} right={right} bottom={bottom}")
    print(f"width={right - left + 1} height={bottom - top + 1}")
    print(f"center={(left + right + 1) / 2:.1f},{(top + bottom + 1) / 2:.1f}")
    print(f"corners={alpha[0][0]},{alpha[0][width - 1]},{alpha[height - 1][0]},{alpha[height - 1][width - 1]}")
    print(f"middle={alpha[height // 2][width // 2]}")


if __name__ == "__main__":
    main()
