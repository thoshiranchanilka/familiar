// When the mascot should be on screen: while at least one Claude Code session is present.
// A session is present from its first hook event until SessionEnd, or until it has been
// quiet for PRESENT_IDLE_MS (PRESENT_WORKING_MS while it is mid-task). Status-line
// refreshes never count, so an idle terminal left open doesn't keep the mascot around.
import { PRESENT_IDLE_MS, PRESENT_WORKING_MS } from "./config";
import type { MascotState } from "./types";

function leavesAt(x: MascotState["sessions"][string]): number {
  return x.lastEventAt + (x.working ? PRESENT_WORKING_MS : PRESENT_IDLE_MS);
}

export function isPresent(s: MascotState, now: number): boolean {
  return Object.values(s.sessions).some((x) => now < leavesAt(x));
}

/** The next time `isPresent` could turn false without a new event; `undefined` if nobody is present. */
export function presenceDeadline(s: MascotState, now: number): number | undefined {
  const future = Object.values(s.sessions).map(leavesAt).filter((t) => t > now);
  return future.length ? Math.min(...future) : undefined;
}
