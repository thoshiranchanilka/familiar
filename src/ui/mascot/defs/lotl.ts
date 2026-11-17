import type { MascotDef } from "./types";

type Pt = readonly [number, number];
// Frilly gills on the left of the head (mirrored on the right), in two wiggle poses.
const GILLS: Pt[][] = [
  [[1, 4], [0, 3], [1, 6], [0, 6], [1, 8], [0, 9]],
  [[1, 4], [0, 4], [1, 6], [0, 7], [1, 8], [0, 8]],
];

export const lotl: MascotDef = {
  id: "lotl",
  name: "Lotl",
  kind: "Axolotl",
  blurb: "Always smiling, gills always fluttering. Blushes when a task lands.",
  pal: { o: "#7a3048", k: "#ffb6c9", l: "#ffe0e8", g: "#e8577e", b: "#ff7f9e" },
  eye: "#3a1422",
  glow: false,
  eyes: { l: 4, r: 10, y: 6 },
  mouth: { x: 7, y: 8 },
  soft: "#ffe0e8",
  map: [
    "................",
    "................",
    "................",
    "....oooooooo....",
    "...okkkkkkkko...",
    "..okkkkkkkkkko..",
    "..okkkkkkkkkko..",
    "..okkkkkkkkkko..",
    "..okkkkkkkkkko..",
    "...okkkkkkkko...",
    "....okllllko....",
    "...okllllllko...",
    "...okllllllko...",
    "...okko..okko...",
    "....oo....oo....",
    "................",
  ],
  effects(mood, f, d) {
    const pose = GILLS[mood === "working" ? (f >> 1) % 2 : 0];
    for (const [x, y] of pose) {
      d.px(d.bx + x, d.by + y, this.pal.g);
      d.px(d.bx + 15 - x, d.by + y, this.pal.g);
    }
    if (mood === "done") {
      d.px(d.bx + 3, d.by + 8, this.pal.b, 2, 1);
      d.px(d.bx + 11, d.by + 8, this.pal.b, 2, 1);
    }
  },
};
