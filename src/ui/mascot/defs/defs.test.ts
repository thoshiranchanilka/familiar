import { describe, expect, test } from "vitest";
import { MASCOTS, getMascot } from ".";
import { mascotPixels } from "../render";
import type { Mood } from "../../../mood/types";

const MOODS: Mood[] = ["greeting", "idle", "working", "thinking", "done", "error", "sleeping"];

describe("every shipped mascot", () => {
  test.each(MASCOTS.map((m) => [m.id, m] as const))("%s map is 16×16 and every letter has a colour", (_id, m) => {
    expect(m.map).toHaveLength(16);
    for (const row of m.map) {
      expect(row).toHaveLength(16);
      for (const ch of row) if (ch !== ".") expect(m.pal[ch], `letter ${ch}`).toBeDefined();
    }
    expect(m.pal.o, "outline colour 'o' (used for eye lines)").toBeDefined();
  });

  test.each(MASCOTS.map((m) => [m.id, m] as const))("%s draws every mood for f = 0..23", (_id, m) => {
    for (const mood of MOODS) for (let f = 0; f < 24; f++) expect(mascotPixels(m, mood, f).length).toBeGreaterThan(40);
  });

  test("ids are unique and getMascot finds each", () => {
    expect(new Set(MASCOTS.map((m) => m.id)).size).toBe(MASCOTS.length);
    for (const m of MASCOTS) expect(getMascot(m.id)).toBe(m);
  });

  test("each mascot looks different from the others", () => {
    const maps = MASCOTS.map((m) => m.map.join("\n"));
    expect(new Set(maps).size).toBe(maps.length);
  });
});
