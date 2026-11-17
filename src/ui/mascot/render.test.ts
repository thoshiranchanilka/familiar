import { describe, expect, test } from "vitest";
import { GRID, mascotPixels } from "./render";
import { MASCOTS, getMascot } from "./defs";
import type { Mood } from "../../mood/types";

const MOODS: Mood[] = ["greeting", "idle", "working", "thinking", "done", "error", "sleeping"];
const EMBER = getMascot("ember");
const BOLT = getMascot("bolt");
const ember = (mood: Mood, f: number) => mascotPixels(EMBER, mood, f);
const at = (px: ReturnType<typeof mascotPixels>, x: number, y: number) =>
  px.filter((p) => p.x === x && p.y === y).map((p) => p.color).pop();
const bodyTop = (px: ReturnType<typeof mascotPixels>) =>
  Math.min(...px.filter((p) => p.color === EMBER.pal.y).map((p) => p.y));

describe("mascot definitions", () => {
  test("ships Ember, Mochi and Bolt in that order", () => {
    expect(MASCOTS.map((m) => m.id)).toEqual(["ember", "mochi", "bolt"]);
  });

  test.each(MASCOTS.map((m) => [m.id, m] as const))("%s map is 16×16 and uses only palette letters", (_id, m) => {
    expect(m.map).toHaveLength(16);
    for (const row of m.map) {
      expect(row).toHaveLength(16);
      for (const ch of row) if (ch !== ".") expect(m.pal[ch], `letter ${ch}`).toBeDefined();
    }
  });

  test("unknown ids fall back to Ember", () => {
    expect(getMascot("nope").id).toBe("ember");
  });
});

describe("mascotPixels", () => {
  test.each(MASCOTS.flatMap((m) => MOODS.map((mood) => [m.id, mood] as const)))(
    "%s %s stays inside the 28×28 grid for every frame", (id, mood) => {
      for (let f = 0; f < 48; f++) {
        for (const p of mascotPixels(getMascot(id), mood, f)) {
          expect(p.x).toBeGreaterThanOrEqual(0);
          expect(p.x).toBeLessThan(GRID);
          expect(p.y).toBeGreaterThanOrEqual(0);
          expect(p.y).toBeLessThan(GRID);
        }
      }
    });

  test("draws Ember's body palette", () => {
    const colors = new Set(ember("idle", 0).map((p) => p.color));
    for (const c of [EMBER.pal.o, EMBER.pal.r, EMBER.pal.y, EMBER.pal.h]) expect(colors).toContain(c);
  });

  test("done hops: frame 2 is 3 px higher than frame 0", () => {
    expect(bodyTop(ember("done", 0)) - bodyTop(ember("done", 2))).toBe(3);
  });

  test("sleeping floats Zs and closes the eyes", () => {
    const px = ember("sleeping", 0);
    expect(px.some((p) => p.color === "#8fa3c7")).toBe(true);
    expect(px.some((p) => p.color === "#ffffff")).toBe(false);
  });

  test("working shows the keyboard under the mascot", () => {
    expect(at(ember("working", 0), 7, 23)).toBe("#3a3f4b");
  });

  test("thinking grows thought dots over time", () => {
    const dots = (f: number) => ember("thinking", f).filter((p) => p.color === "#9fb0cc").length;
    expect(dots(0)).toBe(0);
    expect(dots(12)).toBeGreaterThan(dots(4));
  });

  test("error shakes side to side", () => {
    const left = (f: number) => Math.min(...ember("error", f).filter((p) => p.color === EMBER.pal.o).map((p) => p.x));
    expect(left(0)).not.toBe(left(1));
  });

  test("idle blinks once every 24 frames", () => {
    const white = (f: number) => ember("idle", f).some((p) => p.color === "#ffffff");
    expect(white(0)).toBe(true);
    expect(white(23)).toBe(false);
  });

  test("Bolt's antenna light blinks while working", () => {
    // antenna "a" pixels are on map row 0, columns 7–8 → grid (13, 7 + dy)
    const antenna = (f: number) => mascotPixels(BOLT, "working", f).filter((p) => p.x === 13 && p.y <= 7).map((p) => p.color).pop();
    expect(antenna(0)).not.toBe(antenna(1));
  });

  test("Bolt's glowing eyes have no white highlight", () => {
    expect(mascotPixels(BOLT, "idle", 0).some((p) => p.color === "#ffffff")).toBe(false);
  });
});
