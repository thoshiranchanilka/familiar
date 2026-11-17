import {
  DONE_HOLD_MS, ERROR_HOLD_MS, GREETING_MS, MIN_MOOD_MS, SLEEP_AFTER_MS,
  THINKING_AFTER_CODE_MS, THINKING_AFTER_DESKTOP_MS,
} from "./config";
import type { MascotEvent, MascotState, Mood } from "./types";

export type ReducerInput = MascotEvent | { tick: number };

export function initialState(now: number): MascotState {
  return {
    mood: "idle",
    since: now - MIN_MOOD_MS, // free to change immediately
    usage: {},
    sessions: {},
    desktopActive: false,
    desktopStartedAt: 0,
    greetingUntil: 0,
    doneUntil: 0,
    errorUntil: 0,
    lastActivityAt: now,
  };
}

/** The mood the facts call for right now, before anti-flicker. Priority: error > code > desktop > transient > idle/sleep. */
function targetMood(s: MascotState, now: number): Mood {
  if (s.errorUntil > now) return "error";
  const working = Object.values(s.sessions).filter((x) => x.working);
  if (working.length > 0) {
    const latest = Math.max(...working.map((x) => x.startedAt));
    return now - latest >= THINKING_AFTER_CODE_MS ? "thinking" : "working";
  }
  if (s.desktopActive) {
    return now - s.desktopStartedAt >= THINKING_AFTER_DESKTOP_MS ? "thinking" : "working";
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
  let next: MascotState = { ...s, sessions };
  if (e.event !== "status") next.lastActivityAt = t;

  switch (e.event) {
    case "session_start":
    case "app_launch":
      next.greetingUntil = t + GREETING_MS;
      break;
    case "prompt":
      sessions[id] = { working: true, startedAt: t };
      next.errorUntil = 0; // the user moved on
      break;
    case "tool":
      if (!sessions[id]?.working) sessions[id] = { working: true, startedAt: t };
      break;
    case "stop":
      if (sessions[id]) sessions[id] = { ...sessions[id], working: false };
      next.doneUntil = t + DONE_HOLD_MS;
      break;
    case "error":
      if (sessions[id]) sessions[id] = { ...sessions[id], working: false };
      next.errorUntil = t + ERROR_HOLD_MS;
      break;
    case "attention":
      if (sessions[id]) sessions[id] = { ...sessions[id], working: false };
      next.greetingUntil = t + GREETING_MS;
      break;
    case "session_end":
      delete sessions[id];
      break;
    case "activity_start":
      if (!next.desktopActive) next.desktopStartedAt = t;
      next.desktopActive = true;
      break;
    case "activity_stop":
      if (next.desktopActive) next.doneUntil = t + DONE_HOLD_MS;
      next.desktopActive = false;
      break;
    case "app_quit":
      next.desktopActive = false;
      break;
    case "status":
      if (e.model) next.model = e.model;
      if (e.usage) next.usage = { ...s.usage, ...e.usage };
      break;
    case "app_focus":
      break;
  }
  if (e.model && e.event !== "status") next.model = e.model;
  return next;
}

export function reduce(state: MascotState, input: ReducerInput): MascotState {
  const now = "tick" in input ? input.tick : input.ts;
  const s = "tick" in input ? state : applyEvent(state, input);
  const target = targetMood(s, now);
  if (target === s.mood || now - s.since < MIN_MOOD_MS) return s;
  return { ...s, mood: target, since: now };
}
