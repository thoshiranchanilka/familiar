import { bolt } from "./bolt";
import { ember } from "./ember";
import { mochi } from "./mochi";
import type { MascotDef } from "./types";

export type { Animation, MascotDef } from "./types";

/** Every mascot we ship. Adding one = one new file in this folder + an entry here. */
export const MASCOTS: MascotDef[] = [ember, mochi, bolt];

export function getMascot(id: string): MascotDef {
  return MASCOTS.find((m) => m.id === id) ?? ember;
}
