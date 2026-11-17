import { describe, expect, test } from "vitest";
import { moodLabel } from "./label";

describe("moodLabel", () => {
  test("asks to connect whenever Claude Code isn't connected", () => {
    expect(moodLabel("idle", false, "Boo")).toBe("Hi, I'm Boo! Click me to connect Claude Code.");
    expect(moodLabel("sleeping", false, "Ember")).toBe("Hi, I'm Ember! Click me to connect Claude Code.");
  });
  test("shows the mood label once connected", () => {
    expect(moodLabel("working", true, "Boo")).toBe("Working…");
    expect(moodLabel("idle", true, "Boo")).toBe("");
  });
});
