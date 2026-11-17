import type { MascotDef } from "./types";

const INK = "#3b2a55";

export const inky: MascotDef = {
  id: "inky",
  name: "Inky",
  kind: "Octopus",
  blurb: "Four busy tentacles, one big heart. Taps away while Claude works.",
  pal: { o: "#3d2357", u: "#a678de", l: "#c9a8f0", p: "#ff9fc4" },
  eye: "#24123a",
  glow: false,
  eyes: { l: 5, r: 9, y: 6 },
  mouth: { x: 7, y: 9 },
  soft: "#c9a8f0",
  map: [
    "................",
    "................",
    ".....oooooo.....",
    "....oullllllo...",
    "...ouulluuuuo...",
    "..ouuuuuuuuuuo..",
    "..ouuuuuuuuuuo..",
    "..ouuuuuuuuuuo..",
    "..oupuuuuuupuo..",
    "..ouuuuuuuuuuo..",
    "..ouuuuuuuuuuo..",
    "..ouuuuuuuuuuo..",
    "..ouoouoouoouo..",
    "..ouoouoouoouo..",
    "...o..o..o..o...",
    "................",
  ],
  effects(mood, f, d) {
    if (mood === "working") {
      // two tentacles at a time reach down to tap the keys
      const legs = f % 2 ? [6, 12] : [3, 9];
      for (const x of legs) { d.px(d.bx + x, d.by + 14, this.pal.u); d.px(d.bx + x, d.by + 15, this.pal.o); }
    }
    if (mood === "error") {
      // a puff of ink that billows out to the side
      const r = 1 + (f % 4);
      d.px(d.ox - 2, d.oy + 10, INK, 2 + r, 2);
      d.px(d.ox - 1, d.oy + 9, INK, 1 + r, 1);
      d.px(d.ox - 1, d.oy + 12, INK, 1 + r, 1);
    }
  },
};
