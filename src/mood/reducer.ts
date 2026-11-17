import {
  DONE_HOLD_MS, ERROR_HOLD_MS, GREETING_MS, MIN_MOOD_MS, SESSION_FORGET_MS, SESSION_STALE_MS, SLEEP_AFTER_MS,
  THINKING_AFTER_CODE_MS,
} from "./config";
import { presenceDeadline } from "./presence";
import type { MascotEvent, MascotState, Mood } from "./types";

export type ReducerInput = MascotEvent | { tick: number };

export function initialState(now: number): MascotState {
  return {
    mood: "idle",
    since: now - MIN_MOOD_MS, // free to change immediately
    usage: {},
    sessions: {},
    greetingUntil: 0,
    doneUntil: 0,
    errorUntil: 0,
    lastActivityAt: now,
  };
}

/** The mood the facts call for right now, before anti-flicker. Priority: error > working > transient > idle/sleep. */
function targetMood(s: MascotState, now: number): Mood {
  if (s.errorUntil > now) return "error";
  const working = Object.values(s.sessions).filter((x) => x.working && now - x.lastEventAt < SESSION_STALE_MS);
  if (working.length > 0) {
    const latest = Math.max(...working.map((x) => x.startedAt));
    return now - latest >= THINKING_AFTER_CODE_MS ? "thinking" : "working";
  }
  if (s.doneUntil > now) return "done";
  if (s.greetingUntil > now) return "greeting";
  if (now - s.lastActivityAt >= SLEEP_AFTER_MS) return "sleeping";
  return "idle";
}

function applyEvent(s: MascotState, e: MascotEvent): MascotState {
  const t = e.ts;
  const id = e.sessionId ?? "_";
  const sessions = { ...s.sessions };
  for (const [k, v] of Object.entries(sessions)) if (t - v.lastEventAt >= SESSION_FORGET_MS) delete sessions[k];
  let next: MascotState = { ...s, sessions };
  if (e.event !== "status") {
    next.lastActivityAt = t;
    // Any hook event creates the session (the app may start mid-session) and keeps it present.
    sessions[id] = sessions[id]
      ? { ...sessions[id], lastEventAt: t }
      : { working: false, startedAt: t, lastEventAt: t };
  }

  switch (e.event) {
    case "session_start":
      next.greetingUntil = t + GREETING_MS;
      break;
    case "prompt":
      sessions[id] = { working: true, startedAt: t, lastEventAt: t };
      next.errorUntil = 0; // the user moved on
      break;
    case "tool":
      if (!sessions[id].working) sessions[id] = { working: true, startedAt: t, lastEventAt: t };
      break;
    case "stop":
      sessions[id] = { ...sessions[id], working: false };
      next.doneUntil = t + DONE_HOLD_MS;
      break;
    case "error":
      sessions[id] = { ...sessions[id], working: false };
      next.errorUntil = t + ERROR_HOLD_MS;
      break;
    case "attention":
      sessions[id] = { ...sessions[id], working: false };
      next.greetingUntil = t + GREETING_MS;
      break;
    case "session_end":
      delete sessions[id];
      break;
    case "status":
      if (e.model) next.model = e.model;
      if (e.usage) next.usage = { ...s.usage, ...e.usage };
      break;
  }
  if (e.model && e.event !== "status") next.model = e.model;
  return next;
}

/**
 * The earliest future time the displayed mood could change without a new event,
 * so the UI can sleep until then instead of polling. `undefined` = nothing pending.
 */
export function nextDeadline(s: MascotState, now: number): number | undefined {
  const candidates: number[] = [s.greetingUntil, s.doneUntil, s.errorUntil];
  if (targetMood(s, now) !== s.mood) candidates.push(s.since + MIN_MOOD_MS);
  for (const x of Object.values(s.sessions)) {
    if (!x.working) continue;
    candidates.push(x.startedAt + THINKING_AFTER_CODE_MS, x.lastEventAt + SESSION_STALE_MS);
  }
  if (s.mood !== "sleeping") candidates.push(s.lastActivityAt + SLEEP_AFTER_MS);
  const leave = presenceDeadline(s, now);
  if (leave !== undefined) candidates.push(leave);
  const future = candidates.filter((t) => t > now);
  return future.length ? Math.min(...future) : undefined;
}

export function reduce(state: MascotState, input: ReducerInput): MascotState {
  const now = "tick" in input ? input.tick : input.ts;
  const s = "tick" in input ? state : applyEvent(state, input);
  const target = targetMood(s, now);
  if (target === s.mood || now - s.since < MIN_MOOD_MS) return s;
  return { ...s, mood: target, since: now };
}
