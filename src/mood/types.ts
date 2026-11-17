export type Mood = "greeting" | "idle" | "working" | "thinking" | "done" | "error" | "sleeping";

export interface Usage {
  fiveHourPct?: number;
  fiveHourResetsAt?: number;
  sevenDayPct?: number;
  sevenDayResetsAt?: number;
  contextPct?: number;
}

export type MascotEventName =
  | "session_start" | "prompt" | "tool" | "stop" | "error" | "attention" | "session_end"
  | "app_launch" | "app_focus" | "app_quit" | "activity_start" | "activity_stop" | "status";

export interface MascotEvent {
  source: "code" | "desktop";
  sessionId?: string;
  event: MascotEventName;
  model?: string;
  usage?: Usage;
  /** epoch ms */
  ts: number;
}

export interface MascotState {
  mood: Mood;
  /** when the displayed mood started (epoch ms) */
  since: number;
  model?: string;
  usage: Usage;
  sessions: Record<string, { working: boolean; startedAt: number }>;
  desktopActive: boolean;
  desktopStartedAt: number;
  greetingUntil: number;
  doneUntil: number;
  errorUntil: number;
  /** last event that counts as activity (status refreshes don't) */
  lastActivityAt: number;
}
