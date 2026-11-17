import { describe, expect, test } from "vitest";
import { shouldShow } from "./visibility";

const base = { hidden: false, connected: true, present: false, settingsOpen: false, noticeActive: false, serverError: false };

describe("shouldShow", () => {
  test("hidden by the user wins over everything", () => {
    expect(shouldShow({ hidden: true, connected: false, present: true, settingsOpen: true, noticeActive: true, serverError: true })).toBe(false);
  });
  test("shows while not connected", () => expect(shouldShow({ ...base, connected: false })).toBe(true));
  test("shows a server error", () => expect(shouldShow({ ...base, serverError: true })).toBe(true));
  test("shows while a session is present", () => expect(shouldShow({ ...base, present: true })).toBe(true));
  test("shows while Settings is open", () => expect(shouldShow({ ...base, settingsOpen: true })).toBe(true));
  test("shows while a notice is up", () => expect(shouldShow({ ...base, noticeActive: true })).toBe(true));
  test("connected, no session, no notice → hidden", () => expect(shouldShow(base)).toBe(false));
});
