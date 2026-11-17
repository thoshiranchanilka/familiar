import type { Usage } from "../mood/types";

export interface UsageText { fiveHour: string; sevenDay: string; context: string }

/** Countdown text rounded up to the minute: "45m", "2h 10m", "1d 2h". */
export function formatDuration(ms: number): string {
  const totalMin = Math.max(1, Math.ceil(ms / 60_000));
  const d = Math.floor(totalMin / 1440);
  const h = Math.floor((totalMin % 1440) / 60);
  const m = totalMin % 60;
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

function pct(p: number | undefined): string | undefined {
  if (p === undefined || p === null || Number.isNaN(p)) return undefined;
  return p > 100 ? "100%+" : `${Math.round(p)}%`;
}

function limit(p: number | undefined, resetsAt: number | undefined, now: number): string {
  const base = pct(p);
  if (!base) return "n/a";
  if (resetsAt === undefined) return base;
  return resetsAt <= now ? `${base} · resetting…` : `${base} · resets in ${formatDuration(resetsAt - now)}`;
}

export function formatUsage(u: Usage, now: number): UsageText {
  return {
    fiveHour: limit(u.fiveHourPct, u.fiveHourResetsAt, now),
    sevenDay: limit(u.sevenDayPct, u.sevenDayResetsAt, now),
    context: pct(u.contextPct) ?? "n/a",
  };
}
