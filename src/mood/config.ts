// Tunable timings for the mood state machine (milliseconds).
export const THINKING_AFTER_CODE_MS = 15_000;
export const SLEEP_AFTER_MS = 300_000;
export const MIN_MOOD_MS = 800;
export const DONE_HOLD_MS = 2_000;
export const GREETING_MS = 2_000;
export const ERROR_HOLD_MS = 2_000;
/** A working session with no events for this long (e.g. interrupted with Esc, terminal closed) stops counting. */
export const SESSION_STALE_MS = 300_000;
/** Sessions with no events for this long are forgotten. */
export const SESSION_FORGET_MS = 3_600_000;
/** A session with no hook events for this long leaves (the mascot hides once none are left). */
export const PRESENT_IDLE_MS = 300_000;
/** …unless it is mid-task: a long tool run fires no events, so working sessions get longer. */
export const PRESENT_WORKING_MS = 1_800_000;
