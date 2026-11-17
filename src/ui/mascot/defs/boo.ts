import type { MascotDef } from "./types";

// 3×5 pixel letters for Boo's "BOO!" on error.
const B = ["##.", "#.#", "##.", "#.#", "##."];
const O = [".#.", "#.#", "#.#", "#.#", ".#."];
const TEXT = "#6b6f8a";

export const boo: MascotDef = {
  id: "boo",
  name: "Boo",
  kind: "Tiny ghost",
  blurb: "Shy, floaty and harmless. Fades to see-through when it naps.",
  pal: { o: "#5b6078", w: "#f4f6ff", s: "#d9def0", p: "#ffc2d1" },
  eye: "#2a2d3d",
  glow: false,
  eyes: { l: 5, r: 9, y: 6 },
  mouth: { x: 7, y: 9 },
  soft: "#d9def0",
  shadow: false,
  sleepAlpha: 0.45,
  map: [
    "................",
    "................",
    ".....oooooo.....",
    "....owwwwwwo....",
    "...owwwwwwwwo...",
    "..owwwwwwwwwwo..",
    "..owwwwwwwwwwo..",
    "..owwwwwwwwwwo..",
    "..owpwwwwwwpwo..",
    "..owwwwwwwwwwo..",
    "..owwwwwwwwwwo..",
    "..owwwwwwwwwso..",
    "..owwwwwwwwsso..",
    "..owsoowwoosso..",
    "..oso..oo..oso..",
    "...o........o...",
  ],
  effects(mood, f, d) {
    if (mood !== "error" || f % 4 === 3) return; // "BOO!" pops on and off
    [B, O, O].forEach((glyph, n) => glyph.forEach((row, j) => [...row].forEach((ch, i) => {
      if (ch === "#") d.px(d.ox + 10 + n * 4 + i, d.oy - 6 + j, TEXT);
    })));
  },
};
