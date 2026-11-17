import type { MascotDef } from "./types";

export const hop: MascotDef = {
  id: "hop",
  name: "Hop",
  kind: "Frog",
  blurb: "Calm, a little sleepy-eyed, and springy when things go right.",
  pal: { o: "#24502a", h: "#7cc96a", l: "#d9f2b8", t: "#eefad8", p: "#f7a8a0" },
  eye: "#17301a",
  glow: false,
  eyes: { l: 4, r: 10, y: 3 },
  mouth: { x: 7, y: 7 },
  soft: "#d9f2b8",
  bob: { done: [0, -3, -5, -6, -5, -3, 0, 0] },
  map: [
    "................",
    "................",
    "...oooo..oooo...",
    "..ohhhhoohhhho..",
    "..ohhhhhhhhhho..",
    ".ohhhhhhhhhhhho.",
    "ohhhhhhhhhhhhhho",
    "ohhhhhhhhhhhhhho",
    "ohhphhhhhhhhphho",
    "ohhllllllllllhho",
    ".ohllllllllllho.",
    ".ohllllllllllho.",
    "..ohhllllllhho..",
    ".ohhoooooooohho.",
    ".oooo......oooo.",
    "................",
  ],
  effects(mood, f, d) {
    if (mood === "idle" && f % 24 !== 23) {
      // heavy lids: the top half of each eye is covered (except mid-blink)
      d.px(d.bx + this.eyes.l, d.by + this.eyes.y, this.pal.h, 2, 1);
      d.px(d.bx + this.eyes.r, d.by + this.eyes.y, this.pal.h, 2, 1);
    }
    if (mood === "thinking") {
      // throat sac puffs in and out
      const big = (f >> 2) % 2 === 1;
      d.px(d.bx + (big ? 5 : 6), d.by + 9, this.pal.t, big ? 6 : 4, big ? 3 : 2);
    }
  },
};
