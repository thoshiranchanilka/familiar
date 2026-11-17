import { describe, expect, test } from "vitest";
import { animationNote, cornerLabel, statusText } from "./view";

describe("settings view helpers", () => {
  test("names each corner", () => {
    expect(cornerLabel("br")).toBe("Bottom right");
    expect(cornerLabel("bl")).toBe("Bottom left");
    expect(cornerLabel("tr")).toBe("Top right");
    expect(cornerLabel("tl")).toBe("Top left");
  });
  test("explains Low Power Mode only when it is on", () => {
    expect(animationNote(true)).toMatch(/Low Power Mode/);
    expect(animationNote(false)).toBe("");
  });
  test("Claude Code status text", () => {
    expect(statusText({ connected: true })).toBe("Connected");
    expect(statusText({ connected: false })).toBe("Not connected");
  });
});
