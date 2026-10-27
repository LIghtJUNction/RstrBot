#!/usr/bin/env python3
"""Rebuild the original geometric R icon assets. Requires Pillow for developers.

The generated assets are committed; Pillow is not a build or runtime dependency.
The path coordinates match Brand in web/src/ui.tsx and the bundled launcher.
"""
from io import BytesIO
from pathlib import Path
import struct

from PIL import Image, ImageDraw

DESTINATION = Path(__file__).resolve().parent / "src-tauri" / "icons"
POINTS = [(7, 26), (7, 6), (21, 6), (26, 13), (15, 19), (26, 26),
          (18, 26), (7, 18), (21, 12), (13, 12), (13, 26)]
PATH = "M7 26V6h14l5 7-11 6 11 7h-8L7 18l14-6h-8v14"


def main():
    DESTINATION.mkdir(parents=True, exist_ok=True)
    scale = 64
    master = Image.new("RGBA", (32 * scale, 32 * scale), (0, 0, 0, 0))
    canvas = ImageDraw.Draw(master)
    canvas.rounded_rectangle((scale, scale, 31 * scale, 31 * scale),
                             radius=6 * scale, fill="#171b18")
    canvas.line([(x * scale, y * scale) for x, y in POINTS],
                fill="#dfcbaa", width=round(2.3 * scale), joint="curve")

    def resized(size):
        return master.resize((size, size), Image.Resampling.LANCZOS)

    def png(size):
        output = BytesIO()
        resized(size).save(output, format="PNG", optimize=True)
        return output.getvalue()

    for name, size in [("32x32.png", 32), ("128x128.png", 128),
                       ("128x128@2x.png", 256), ("icon.png", 512)]:
        (DESTINATION / name).write_bytes(png(size))

    # Keep the 32px entry first, as recommended by Tauri for Windows title bars.
    sizes = (32, 16, 24, 48, 64, 256)
    images = [png(size) for size in sizes]
    offset = 6 + 16 * len(sizes)
    entries = []
    for size, contents in zip(sizes, images):
        entries.append(struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0,
                                   1, 32, len(contents), offset))
        offset += len(contents)
    (DESTINATION / "icon.ico").write_bytes(
        struct.pack("<HHH", 0, 1, len(sizes)) + b"".join(entries) + b"".join(images))
    resized(1024).save(DESTINATION / "icon.icns", format="ICNS")
    (DESTINATION / "icon.svg").write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">\n'
        '  <rect x="1" y="1" width="30" height="30" rx="6" fill="#171b18"/>\n'
        f'  <path d="{PATH}" fill="none" stroke="#dfcbaa" '
        'stroke-width="2.3" stroke-linejoin="round"/>\n</svg>\n')
    print(f"Wrote original RstrBot icon assets to {DESTINATION}")


if __name__ == "__main__":
    main()
