import { describe, expect, test } from "vitest";
import { initialState, nextDeadline, reduce } from "./reducer";
import type { MascotEvent, MascotState } from "./types";

const code = (event: MascotEvent["event"], ts: number, sessionId = "A", extra: Partial<MascotEvent> = {}): MascotEvent =>
  ({ source: "code", event, ts, sessionId, ...extra });
const desk = (event: MascotEvent["event"], ts: number): MascotEvent => ({ source: "desktop", event, ts });
const tick = (t: number) => ({ tick: t });
const run = (s: MascotState, ...inputs: Parameters<typeof reduce>[1][]) => inputs.reduce(reduce, s);

describe("mood reducer", () => {
  test("session_start greets, then settles to idle after 2 s", () => {
    let s = run(initialState(0), code("session_start", 1000));
    expect(s.mood).toBe("greeting");
    s = reduce(s, tick(3000));
    expect(s.mood).toBe("idle");
  });

  test("prompt works, turns to thinking after 15 s, stop shows done then idle", () => {
    let s = run(initialState(0), code("prompt", 0));
    expect(s.mood).toBe("working");
    s = reduce(s, tick(14_999));
    expect(s.mood).toBe("working");
    s = reduce(s, tick(15_000));
    expect(s.mood).toBe("thinking");
    s = reduce(s, code("stop", 16_000));
    expect(s.mood).toBe("done");
    s = reduce(s, tick(18_000));
    expect(s.mood).toBe("idle");
  });

  test("one session stopping does not idle while another still works", () => {
    const s = run(initialState(0), code("prompt", 0, "A"), code("prompt", 1000, "B"), code("stop", 5000, "A"));
    expect(s.mood).toBe("working");
  });

  test("code timing beats the desktop heuristic", () => {
    const s = run(initialState(0), desk("activity_start", 0), code("prompt", 10_000), tick(20_000));
    expect(s.mood).toBe("working");
  });

  test("desktop activity works, thinks after 20 s, then done and idle", () => {
    let s = run(initialState(0), desk("activity_start", 0));
    expect(s.mood).toBe("working");
    s = reduce(s, tick(20_000));
    expect(s.mood).toBe("thinking");
    s = reduce(s, desk("activity_stop", 21_000));
    expect(s.mood).toBe("done");
    s = reduce(s, tick(23_000));
    expect(s.mood).toBe("idle");
  });

  test("error is held at least MIN_MOOD_MS even if a prompt follows", () => {
    let s = run(initialState(0), code("error", 1000));
    expect(s.mood).toBe("error");
    s = reduce(s, code("prompt", 1300));
    expect(s.mood).toBe("error");
    s = reduce(s, tick(1800));
    expect(s.mood).toBe("working");
  });

  test("falls asleep after 5 idle minutes and app_launch wakes with a greeting", () => {
    let s = reduce(initialState(0), tick(300_000));
    expect(s.mood).toBe("sleeping");
    s = reduce(s, desk("app_launch", 301_000));
    expect(s.mood).toBe("greeting");
  });

  test("a prompt wakes a sleeping mascot straight into working", () => {
    const s = run(initialState(0), tick(300_000), code("prompt", 301_000));
    expect(s.mood).toBe("working");
  });

  test("status updates model and merges usage without changing or waking the mood", () => {
    let s = run(initialState(0), tick(300_000));
    s = reduce(s, code("status", 301_000, "A", { model: "Opus 5.5", usage: { fiveHourPct: 23, contextPct: 8 } }));
    s = reduce(s, code("status", 302_000, "A", { usage: { fiveHourPct: 30 } }));
    expect(s.mood).toBe("sleeping");
    expect(s.model).toBe("Opus 5.5");
    expect(s.usage).toEqual({ fiveHourPct: 30, contextPct: 8 });
  });

  test("brief activity gaps do not flicker through done", () => {
    let s = run(initialState(0), desk("activity_start", 1000));
    s = reduce(s, desk("activity_stop", 1300));
    expect(s.mood).toBe("working");
    s = run(s, desk("activity_start", 1500), tick(2000));
    expect(s.mood).toBe("working");
  });

  test("attention pauses the session and waves for the user", () => {
    const s = run(initialState(0), code("prompt", 0), code("attention", 5000));
    expect(s.mood).toBe("greeting");
  });

  test("an interrupted session (no Stop) stops counting as working after 5 quiet minutes", () => {
    const s = run(initialState(0), code("prompt", 0), tick(300_001));
    expect(s.mood).not.toBe("thinking");
    expect(s.mood).not.toBe("working");
  });

  test("tool events keep a long agentic run alive", () => {
    const s = run(initialState(0), code("prompt", 0), code("tool", 240_000), tick(360_000));
    expect(s.mood).toBe("thinking");
  });

  test("session_end forgets the session", () => {
    const s = run(initialState(0), code("prompt", 0), code("session_end", 5000), tick(10_000));
    expect(s.sessions).toEqual({});
    expect(s.mood).toBe("idle");
  });
});

describe("nextDeadline", () => {
  test("a fresh prompt next changes when thinking starts", () => {
    const s = run(initialState(0), code("prompt", 0));
    expect(nextDeadline(s, 1)).toBe(15_000);
  });

  test("after stop, the next change is when done expires", () => {
    const s = run(initialState(0), code("prompt", 0), code("stop", 16_000));
    expect(nextDeadline(s, 16_001)).toBe(18_000);
  });

  test("idle with nothing pending waits for sleep", () => {
    expect(nextDeadline(initialState(0), 1)).toBe(300_000);
  });

  test("sleeping with nothing pending has no deadline", () => {
    const s = reduce(initialState(0), tick(300_000));
    expect(nextDeadline(s, 300_001)).toBeUndefined();
  });

  test("a deferred mood change wakes when the anti-flicker hold ends", () => {
    const s = run(initialState(0), code("error", 1000), code("prompt", 1300));
    expect(nextDeadline(s, 1301)).toBe(1800);
  });

  test("desktop activity next changes when thinking starts", () => {
    const s = run(initialState(0), desk("activity_start", 0));
    expect(nextDeadline(s, 1)).toBe(20_000);
  });
});
