import type { Mood } from "../../../mood/types";
import type { MascotDef } from "./types";

type Pt = readonly [number, number];
// Leaf shapes on top of the head, in sprite coordinates.
const STEM: Pt[] = [[7, 3], [7, 2]];
const LEAVES: Record<"up" | "droop" | "folded", Pt[]> = {
  up: [[6, 1], [5, 1], [5, 0], [4, 0], [8, 1], [9, 1], [9, 0], [10, 0]],
  droop: [[6, 2], [5, 2], [4, 3], [3, 3], [8, 2], [9, 2], [10, 3], [11, 3]],
  folded: [[7, 1], [7, 0], [8, 0], [6, 1]],
};
const PETALS: Pt[] = [[6, 0], [8, 0], [7, -1], [7, 1]];

function leafFor(mood: Mood): keyof typeof LEAVES | "flower" {
  if (mood === "thinking") return "droop";
  if (mood === "sleeping") return "folded";
  if (mood === "done") return "flower";
  return "up";
}

export const sprout: MascotDef = {
  id: "sprout",
  name: "Sprout",
  kind: "Seedling",
  blurb: "A little bean with a leaf on top. Blooms when a task is done.",
  pal: { o: "#2f5a2a", g: "#8fd16a", l: "#c8eda0", p: "#ffb3a7", s: "#3f9d3a", L: "#4cae3f", f: "#ff8fb1", c: "#ffd166" },
  eye: "#1f3a1b",
  glow: false,
  eyes: { l: 5, r: 9, y: 7 },
  mouth: { x: 7, y: 10 },
  soft: "#c8eda0",
  map: [
    "................",
    "................",
    "................",
    "................",
    ".....oooooo.....",
    "....oggggggo....",
    "...oggggggggo...",
    "..oggggggggggo..",
    "..oggggggggggo..",
    "..ogpggggggpgo..",
    "..oggggggggggo..",
    "..ogglllllllgo..",
    "..ogllllllllgo..",
    "...ollllllllo...",
    "....oooooooo....",
    "................",
  ],
  effects(mood, _f, d) {
    const at = ([x, y]: Pt, c: string) => d.px(d.bx + x, d.by + y, c);
    STEM.forEach((p) => at(p, this.pal.s));
    const leaf = leafFor(mood);
    if (leaf === "flower") {
      LEAVES.up.forEach((p) => at(p, this.pal.L));
      PETALS.forEach((p) => at(p, this.pal.f));
      at([7, 0], this.pal.c);
    } else {
      LEAVES[leaf].forEach((p) => at(p, this.pal.L));
    }
  },
};
