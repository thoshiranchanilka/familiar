import { describe, expect, test } from "vitest";
import { isDrag } from "./drag";

describe("isDrag", () => {
  test("a small wobble during a click is still a click", () => expect(isDrag(0, 0, 3, -2)).toBe(false));
  test("moving 5 px or more starts a drag", () => {
    expect(isDrag(10, 10, 15, 10)).toBe(true);
    expect(isDrag(10, 10, 10, 4)).toBe(true);
  });
});
