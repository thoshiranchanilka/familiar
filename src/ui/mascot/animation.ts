// How often the mascot redraws, per animation level and mood. Fewer frames = less power.
import type { Mood } from "../../mood/types";
import type { Animation } from "./defs";

const FPS8_MS = 125;
const IDLE_BLINK_EVERY_MS = 4000;
const BLINK_HOLD_MS = 150;
const SLEEP_MS = 500;
/** frame used when nothing animates: open eyes, Zs visible while asleep */
const STILL_FRAME = 10;
/** the idle frame in which the renderer closes the eyes */
const BLINK_FRAME = 23;

const BUSY: Mood[] = ["working", "thinking", "done", "greeting", "error"];

/** Milliseconds until the next frame, or null when the picture only changes with the mood. */
export function frameIntervalMs(level: Animation, mood: Mood, lowPower: boolean, tick: number): number | null {
  if (lowPower || level === "still") return null;
  if (level === "full" || BUSY.includes(mood)) return FPS8_MS;
  if (mood === "sleeping") return SLEEP_MS;
  return tick % 2 === 1 ? BLINK_HOLD_MS : IDLE_BLINK_EVERY_MS; // gentle idle
}

/** The renderer frame to draw at this tick. */
export function frameAt(level: Animation, mood: Mood, lowPower: boolean, tick: number): number {
  if (lowPower || level === "still") return STILL_FRAME;
  if (level === "gentle" && mood === "idle") return tick % 2 === 1 ? BLINK_FRAME : 0;
  return tick;
}
