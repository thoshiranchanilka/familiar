import { describe, expect, test } from "vitest";
import { EMBER, GRID, emberPixels } from "./ember";
import type { Mood } from "../../mood/types";

const MOODS: Mood[] = ["greeting", "idle", "working", "thinking", "done", "error", "sleeping"];
const at = (px: ReturnType<typeof emberPixels>, x: number, y: number) =>
  px.filter((p) => p.x === x && p.y === y).map((p) => p.color).pop();
const bodyTop = (px: ReturnType<typeof emberPixels>) =>
  Math.min(...px.filter((p) => p.color === EMBER.pal.y).map((p) => p.y));

describe("emberPixels", () => {
  test.each(MOODS)("%s stays inside the 28×28 grid for every frame", (mood) => {
    for (let f = 0; f < 48; f++) {
      for (const p of emberPixels(mood, f)) {
        expect(p.x).toBeGreaterThanOrEqual(0);
        expect(p.x).toBeLessThan(GRID);
        expect(p.y).toBeGreaterThanOrEqual(0);
        expect(p.y).toBeLessThan(GRID);
      }
    }
  });

  test("draws the body palette", () => {
    const colors = new Set(emberPixels("idle", 0).map((p) => p.color));
    for (const c of [EMBER.pal.o, EMBER.pal.r, EMBER.pal.y, EMBER.pal.h]) expect(colors).toContain(c);
  });

  test("done hops: frame 2 is 3 px higher than frame 0", () => {
    expect(bodyTop(emberPixels("done", 0)) - bodyTop(emberPixels("done", 2))).toBe(3);
  });

  test("sleeping floats Zs and closes the eyes", () => {
    const px = emberPixels("sleeping", 0);
    expect(px.some((p) => p.color === "#8fa3c7")).toBe(true);
    // open-eye highlight pixel (white) never appears while asleep
    expect(px.some((p) => p.color === "#ffffff")).toBe(false);
  });

  test("working shows the keyboard under the mascot", () => {
    expect(at(emberPixels("working", 0), 7, 23)).toBe("#3a3f4b");
  });

  test("thinking grows thought dots over time", () => {
    const dots = (f: number) => emberPixels("thinking", f).filter((p) => p.color === "#9fb0cc").length;
    expect(dots(0)).toBe(0);
    expect(dots(12)).toBeGreaterThan(dots(4));
  });

  test("error shakes side to side", () => {
    const left = (f: number) => Math.min(...emberPixels("error", f).filter((p) => p.color === EMBER.pal.o).map((p) => p.x));
    expect(left(0)).not.toBe(left(1));
  });

  test("idle blinks once every 24 frames", () => {
    const white = (f: number) => emberPixels("idle", f).some((p) => p.color === "#ffffff");
    expect(white(0)).toBe(true);
    expect(white(23)).toBe(false);
  });
});
