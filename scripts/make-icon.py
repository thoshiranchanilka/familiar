#!/usr/bin/env python3
"""Renders Ember's idle face as a 1024x1024 PNG app icon (no dependencies).

Usage: python3 scripts/make-icon.py out.png && npx tauri icon out.png
Keep MAP and PAL in sync with src/ui/mascot/ember.ts.
"""
import struct
import sys
import zlib

MAP = [
    "................",
    "........oo......",
    ".......oyo......",
    "......oyo.......",
    ".....oooooo.....",
    "...oorrrrrroo...",
    "..orrrrrrrrrro..",
    ".orrrrrrrrrrrro.",
    ".orrrrrrrrrrrro.",
    "orrrrrrrrrrrrrro",
    "orprrrrrrrrrrpro",
    "orrrrrrrrrrrrrro",
    "orrhhhhhhhhhhrro",
    ".orrhhhhhhhhrro.",
    "..oooooooooooo..",
    "................",
]
PAL = {"o": "4a1f12", "r": "ef6a3a", "y": "ffd166", "h": "f9a46b", "p": "ffb3a7", "e": "2b120a", "w": "ffffff"}
SIZE = 1024
GRID = 18  # 16 px sprite plus a 1 px transparent border each side
CELL = SIZE // GRID  # 56 px per sprite pixel


def sprite():
    rows = [list(r) for r in MAP]
    for x in (5, 9):  # open eyes with a highlight, matching the idle frame
        rows[8][x] = rows[9][x] = rows[9][x + 1] = "e"
        rows[8][x + 1] = "e"
        rows[8][x] = "w"
    rows[11][7] = rows[11][8] = "o"  # small smile
    return rows


def rgba_rows():
    rows = sprite()
    offset = (SIZE - CELL * 16) // 2
    for py in range(SIZE):
        line = bytearray([0])  # PNG filter type: none
        gy = (py - offset) // CELL
        for px in range(SIZE):
            gx = (px - offset) // CELL
            k = rows[gy][gx] if 0 <= gy < 16 and 0 <= gx < 16 and py >= offset and px >= offset else "."
            line += bytes.fromhex(PAL[k]) + b"\xff" if k != "." else b"\x00\x00\x00\x00"
        yield bytes(line)


def png(path):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    raw = b"".join(rgba_rows())
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(raw, 9)))
        f.write(chunk(b"IEND", b""))


if __name__ == "__main__":
    png(sys.argv[1] if len(sys.argv) > 1 else "ember-icon.png")
