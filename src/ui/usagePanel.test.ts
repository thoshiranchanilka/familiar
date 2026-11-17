import { describe, expect, test } from "vitest";
import { formatDuration, formatUsage } from "./usagePanel";

const NOW = 1_800_000_000_000;
const H = 3_600_000;
const M = 60_000;

describe("formatUsage", () => {
  test("all fields present", () => {
    expect(formatUsage({
      fiveHourPct: 23.4, fiveHourResetsAt: NOW + 2 * H + 10 * M,
      sevenDayPct: 41.2, sevenDayResetsAt: NOW + 3 * 24 * H + 4 * H,
      contextPct: 8,
    }, NOW)).toEqual({
      fiveHour: "23% · resets in 2h 10m",
      sevenDay: "41% · resets in 3d 4h",
      context: "8%",
    });
  });

  test("only the 5-hour percentage", () => {
    expect(formatUsage({ fiveHourPct: 5 }, NOW)).toEqual({ fiveHour: "5%", sevenDay: "n/a", context: "n/a" });
  });

  test("reset time in the past says resetting", () => {
    expect(formatUsage({ fiveHourPct: 99, fiveHourResetsAt: NOW - M }, NOW).fiveHour).toBe("99% · resetting…");
  });

  test("over 100% shows 100%+", () => {
    expect(formatUsage({ sevenDayPct: 104 }, NOW).sevenDay).toBe("100%+");
  });

  test("empty usage is all n/a", () => {
    expect(formatUsage({}, NOW)).toEqual({ fiveHour: "n/a", sevenDay: "n/a", context: "n/a" });
  });
});

describe("formatDuration", () => {
  test.each([
    [45 * M, "45m"],
    [30_000, "1m"],
    [2 * H + 10 * M, "2h 10m"],
    [5 * H, "5h 0m"],
    [26 * H, "1d 2h"],
  ])("%i ms → %s", (ms, want) => expect(formatDuration(ms)).toBe(want));
});
