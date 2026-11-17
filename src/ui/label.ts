// The line shown above the mascot when there's no notice: a connect prompt until Claude Code is
// connected (the mascot stays on screen then, so it must say why), otherwise the mood.
import type { Mood } from "../mood/types";

const LABELS: Record<Mood, string> = {
  greeting: "Hi!", idle: "", working: "Working…", thinking: "Thinking longer…",
  done: "Done!", error: "Something went wrong", sleeping: "",
};

export function moodLabel(mood: Mood, connected: boolean, name: string): string {
  return connected ? LABELS[mood] : `Hi, I'm ${name}! Click me to connect Claude Code.`;
}
