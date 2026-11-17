import { describe, expect, test } from "vitest";
import { frameAt, frameIntervalMs } from "./animation";

describe("frameIntervalMs", () => {
  test("gentle idle waits 4 s between blinks", () => expect(frameIntervalMs("gentle", "idle", false, 0)).toBe(4000));
  test("gentle idle holds a blink for 150 ms", () => expect(frameIntervalMs("gentle", "idle", false, 1)).toBe(150));
  test("gentle animates working at 8 fps", () => expect(frameIntervalMs("gentle", "working", false, 0)).toBe(125));
  test.each(["thinking", "done", "greeting", "error"] as const)("gentle animates %s at 8 fps", (m) =>
    expect(frameIntervalMs("gentle", m, false, 0)).toBe(125));
  test("gentle sleeping drifts at 2 fps", () => expect(frameIntervalMs("gentle", "sleeping", false, 0)).toBe(500));
  test("full animates idle at 8 fps", () => expect(frameIntervalMs("full", "idle", false, 0)).toBe(125));
  test("still never schedules frames", () => expect(frameIntervalMs("still", "working", false, 0)).toBeNull());
  test("Low Power Mode forces still", () => expect(frameIntervalMs("full", "working", true, 0)).toBeNull());
});

describe("frameAt", () => {
  test("still uses one fixed frame", () => {
    expect(new Set([0, 1, 7, 99].map((t) => frameAt("still", "working", false, t))).size).toBe(1);
  });
  test("gentle idle alternates open eyes and a blink", () => {
    expect(frameAt("gentle", "idle", false, 0)).toBe(0);
    expect(frameAt("gentle", "idle", false, 1)).toBe(23);
    expect(frameAt("gentle", "idle", false, 2)).toBe(0);
  });
  test("full advances one frame per tick", () => {
    expect(frameAt("full", "done", false, 5)).toBe(5);
  });
});
