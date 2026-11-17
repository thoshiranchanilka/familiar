import { describe, expect, test } from "vitest";
import { initialState, nextDeadline, reduce } from "./reducer";
import { isPresent, presenceDeadline } from "./presence";
import type { MascotEvent, MascotState } from "./types";

const ev = (event: MascotEvent["event"], ts: number, sessionId = "A"): MascotEvent => ({ source: "code", event, ts, sessionId });
const run = (s: MascotState, ...inputs: Parameters<typeof reduce>[1][]) => inputs.reduce(reduce, s);

describe("presence", () => {
  test("no sessions → not present", () => {
    expect(isPresent(initialState(0), 1)).toBe(false);
  });

  test("session_start makes it present", () => {
    expect(isPresent(run(initialState(0), ev("session_start", 1000)), 1001)).toBe(true);
  });

  test("any event creates a session", () => {
    expect(isPresent(run(initialState(0), ev("stop", 1000, "unseen")), 1001)).toBe(true);
  });

  test("session_end of the last session → not present", () => {
    const s = run(initialState(0), ev("session_start", 0), ev("prompt", 10), ev("session_end", 20));
    expect(isPresent(s, 21)).toBe(false);
  });

  test("idle session leaves after 5 min", () => {
    const s = run(initialState(0), ev("prompt", 0), ev("stop", 1000));
    expect(isPresent(s, 1000 + 299_999)).toBe(true);
    expect(isPresent(s, 1000 + 300_000)).toBe(false);
  });

  test("working session stays present up to 30 min", () => {
    const s = run(initialState(0), ev("tool", 1000));
    expect(isPresent(s, 1000 + 1_799_999)).toBe(true);
    expect(isPresent(s, 1000 + 1_800_000)).toBe(false);
  });

  test("status updates do not keep a session present", () => {
    let s = run(initialState(0), ev("stop", 1000));
    for (let t = 11_000; t < 301_000; t += 10_000) s = reduce(s, ev("status", t));
    expect(isPresent(s, 301_000)).toBe(false);
  });

  test("two sessions: present while either is", () => {
    const s = run(initialState(0), ev("stop", 0, "A"), ev("stop", 200_000, "B"));
    expect(isPresent(s, 350_000)).toBe(true); // A gone, B still here
    expect(isPresent(s, 500_000)).toBe(false);
  });

  test("presenceDeadline returns the earliest leave time", () => {
    const s = run(initialState(0), ev("stop", 0, "A"), ev("tool", 100_000, "B"));
    expect(presenceDeadline(s, 1)).toBe(300_000);
    expect(presenceDeadline(s, 300_001)).toBe(1_900_000);
    expect(presenceDeadline(initialState(0), 1)).toBeUndefined();
  });

  test("nextDeadline includes the presence deadline", () => {
    // asleep (no mood deadlines left) but a working session is still present
    const s = run(initialState(0), ev("tool", 0), { tick: 600_000 });
    expect(nextDeadline(s, 600_001)).toBe(1_800_000);
  });
});
