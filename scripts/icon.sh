#!/usr/bin/env bash
# Makes the icon pictures of TruckPaint from a generated source picture.
#
# Usage: icon.sh SOURCE.png [OUT_DIR]
#   SOURCE.png: an 8-bit RGBA PNG with a transparent margin, with the rounded
#               square of the icon in it (not necessarily centered or sized).
#   OUT_DIR:    where the files are written (default: assets/icon).
#
# Measures the rounded square (scripts/icon-box.py), scales it so that it is
# 824 by 824 px (about 80 % of the canvas, as macOS icons are; a source that is
# a little wider than tall is stretched by the difference) and centers it on a
# transparent 1024 px canvas, then reduces that master to 512, 256, 128, 64,
# 32 and 16 px. Writes OUT_DIR/truckpaint-<size>.png for the seven sizes.
#
# Needs `rsvg-convert` (librsvg), `sips` (macOS) and `python3`. The committed
# pictures are the source of truth: this only remakes them. See
# assets/icon/README.md.

set -euo pipefail

source_png="${1:?usage: icon.sh SOURCE.png [OUT_DIR]}"
out_dir="${2:-assets/icon}"
here="$(cd "$(dirname "$0")" && pwd)"
body=824
canvas=1024

for tool in rsvg-convert sips python3; do
    command -v "$tool" > /dev/null || { echo "icon: $tool is needed" >&2; exit 1; }
done

mkdir -p "$out_dir"
out_dir="$(cd "$out_dir" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp "$source_png" "$work/source.png"

# left=.. top=.. right=.. bottom=.. width=.. height=.. canvas=WxH
box="$(python3 -I "$here/icon-box.py" "$work/source.png")"
field() { tr ' ' '\n' <<< "$box" | sed -n "s/^$1=//p" | head -n 1; }
width="$(field width)"
height="$(field height)"
[ -n "$width" ] && [ -n "$height" ] || { echo "icon: could not measure $source_png" >&2; exit 1; }
source_size="$(sed -n 's/^canvas=\([0-9]*\)x\([0-9]*\)$/\1 \2/p' <<< "$box")"
source_w="${source_size% *}"
source_h="${source_size#* }"
center="$(sed -n 's/^center=\(.*\)$/\1/p' <<< "$box")"
cx="${center%,*}"
cy="${center#*,}"

# The source is scaled so that its rounded square is $body px by $body px, then
# placed so that the middle of that square is the middle of the canvas.
read -r new_w new_h x y < <(awk -v body="$body" -v w="$width" -v h="$height" -v sw="$source_w" \
    -v sh="$source_h" -v cx="$cx" -v cy="$cy" -v c="$canvas" 'BEGIN {
        sx = body / w
        sy = body / h
        printf "%.4f %.4f %.4f %.4f\n", sw * sx, sh * sy, c / 2 - cx * sx, c / 2 - cy * sy
    }')

cat > "$work/wrap.svg" << SVG
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="$canvas" height="$canvas" viewBox="0 0 $canvas $canvas">
  <image x="$x" y="$y" width="$new_w" height="$new_h" preserveAspectRatio="none" xlink:href="source.png"/>
</svg>
SVG

(cd "$work" && rsvg-convert wrap.svg -o "$out_dir/truckpaint-$canvas.png")
for size in 512 256 128 64 32 16; do
    sips -z "$size" "$size" "$out_dir/truckpaint-$canvas.png" --out "$out_dir/truckpaint-$size.png" > /dev/null
done

echo "source box=${width}x${height} scaled image=${new_w}x${new_h} offset=$x,$y"
python3 -I "$here/icon-box.py" "$out_dir/truckpaint-$canvas.png"
