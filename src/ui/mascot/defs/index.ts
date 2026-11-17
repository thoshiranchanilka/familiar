import { bolt } from "./bolt";
import { boo } from "./boo";
import { dot } from "./dot";
import { hop } from "./hop";
import { ember } from "./ember";
import { inky } from "./inky";
import { lotl } from "./lotl";
import { mochi } from "./mochi";
import { sprout } from "./sprout";
import type { MascotDef } from "./types";

export type { Animation, Draw, MascotDef } from "./types";

/** Every mascot we ship. Adding one = one new file in this folder + an entry here. */
export const MASCOTS: MascotDef[] = [ember, mochi, bolt, boo, inky, sprout, lotl, hop, dot];

export function getMascot(id: string): MascotDef {
  return MASCOTS.find((m) => m.id === id) ?? ember;
}
