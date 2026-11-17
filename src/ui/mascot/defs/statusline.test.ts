import { describe, expect, test } from "vitest";
import { MASCOTS } from ".";
import script from "../../../../scripts/statusline.sh?raw";

// scripts/statusline.sh prints the selected mascot's name; its id→name table must cover every mascot we ship.
describe("statusline.sh", () => {
  test("statusline maps every shipped mascot", () => {
    for (const m of MASCOTS) expect(script).toContain(`    ${m.id}) name="${m.name}" ;;`);
  });
});
