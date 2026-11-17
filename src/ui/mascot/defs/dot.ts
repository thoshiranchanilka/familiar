import type { MascotDef } from "./types";

const SMOKE = "#b9bcc6";

export const dot: MascotDef = {
  id: "dot",
  name: "Dot",
  kind: "Dragon hatchling",
  blurb: "Just hatched and still wearing half the egg. Puffs smoke when busy.",
  pal: { o: "#5a3a12", d: "#f2c14e", n: "#ffe39a", e: "#fbf3e4", F: "#ff7a2f", y: "#ffd166", p: "#f7a07a" },
  eye: "#2e1d08",
  glow: false,
  eyes: { l: 5, r: 9, y: 6 },
  mouth: { x: 7, y: 8 },
  soft: "#ffe39a",
  map: [
    "................",
    "...o........o...",
    "....on....no....",
    "....oooooooo....",
    "...oddddddddo...",
    "o.oddddddddddo.o",
    "onoddddddddddono",
    ".noddddddddddon.",
    "..odpddddddpdo..",
    "..oddddddddddo..",
    ".oeoeeoeeoeeoeo.",
    ".oeeeeeeeeeeeeo.",
    ".oeeeeeeeeeeeeo.",
    "..oeeeeeeeeeeo..",
    "...oooooooooo...",
    "................",
  ],
  effects(mood, f, d) {
    if (mood === "working") {
      // little smoke puffs rising from the nostrils
      const rise = f % 4;
      d.px(d.bx + 13, d.by + 3 - rise, SMOKE, 2, 2);
      if (rise > 1) d.px(d.bx + 15, d.by + 1 - rise, SMOKE);
    }
    if (mood === "error") {
      // a tiny flame out of the mouth
      const long = f % 2 === 0;
      d.px(d.bx + 14, d.by + 8, this.pal.F, long ? 4 : 3, 2);
      d.px(d.bx + 15, d.by + 8, this.pal.y, long ? 2 : 1, 1);
    }
    if (mood === "sleeping") {
      // pulls the top of the shell over its head
      d.px(d.bx + 4, d.by + 1, this.pal.o, 8, 1);
      d.px(d.bx + 3, d.by + 2, this.pal.e, 10, 3);
      d.px(d.bx + 3, d.by + 5, this.pal.o, 2, 1);
      d.px(d.bx + 11, d.by + 5, this.pal.o, 2, 1);
    }
  },
};
