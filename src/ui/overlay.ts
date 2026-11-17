// Overlay entry point: listens for mascot events, runs the mood reducer, renders the mascot,
// the hover usage panel, and short notices.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { initialState, nextDeadline, reduce } from "../mood/reducer";
import type { MascotEvent, MascotState, Mood } from "../mood/types";
import { formatUsage } from "./usagePanel";
import { createMascot } from "./mascot/mascot";
import { getMascot, type Animation } from "./mascot/defs";

interface Settings {
  corner: "br" | "bl" | "tr" | "tl"; size: number; autoHide: boolean; hidden: boolean;
  mascot: string; animation: Animation;
}

const NOTICE_MS = 5000;
const ONBOARDING_MS = 20000;

const style = document.createElement("style");
style.textContent = `
  :root { --panel-bg: rgba(28, 24, 22, .92); --panel-fg: #f7efe9; --panel-muted: #c9b8ad; --accent: #ef6a3a; --track: rgba(255,255,255,.14); }
  #stage { position: fixed; display: flex; flex-direction: column; gap: 6px; transition: opacity .3s;
           font: 12px -apple-system, system-ui, sans-serif; color: var(--panel-fg); }
  #stage.corner-br { right: 12px; bottom: 12px; align-items: flex-end; }
  #stage.corner-bl { left: 12px; bottom: 12px; align-items: flex-start; }
  #stage.corner-tr { right: 12px; top: 12px; align-items: flex-end; flex-direction: column-reverse; }
  #stage.corner-tl { left: 12px; top: 12px; align-items: flex-start; flex-direction: column-reverse; }
  #stage.gone { opacity: 0; }
  .bubble { background: var(--panel-bg); border-radius: 10px; padding: 6px 10px; max-width: 260px; line-height: 1.35; }
  .bubble:empty, .bubble[hidden] { display: none; }
  .panel { width: 230px; display: grid; gap: 3px; }
  .panel b { font-size: 13px; }
  .row { display: flex; justify-content: space-between; gap: 10px; color: var(--panel-muted); font-variant-numeric: tabular-nums; }
  .bar { height: 4px; background: var(--track); border-radius: 2px; overflow: hidden; margin-bottom: 3px; }
  .bar i { display: block; height: 100%; background: var(--accent); }
  .mascot { display: block; cursor: pointer; transform-origin: 50% 100%; }
  .mascot.pop-in { animation: pop-in .45s cubic-bezier(.2, 1.6, .4, 1) both; }
  .mascot.pop-out { animation: pop-out .3s ease-in both; }
  @keyframes pop-in { from { transform: scale(0); } to { transform: scale(1); } }
  @keyframes pop-out { from { transform: scale(1); opacity: 1; } to { transform: scale(0); opacity: 0; } }
  @media (prefers-reduced-motion: reduce) { .mascot.pop-in, .mascot.pop-out { animation: none; } }
`;
document.head.appendChild(style);

const stage = document.getElementById("stage")!;
const notice = document.createElement("div");
notice.className = "bubble";
const panel = document.createElement("div");
panel.className = "bubble panel";
panel.hidden = true;
const mascot = document.createElement("canvas");
mascot.className = "mascot";
mascot.setAttribute("aria-label", "Ember, the Claude Puppet mascot");
stage.append(notice, panel, mascot);
const ember = createMascot(mascot);

const LABELS: Record<Mood, string> = {
  greeting: "Hi!", idle: "", working: "Working…", thinking: "Thinking longer…",
  done: "Done!", error: "Something went wrong", sleeping: "",
};

let state: MascotState = initialState(Date.now());
let settings: Settings = { corner: "br", size: 128, autoHide: false, hidden: false, mascot: "ember", animation: "gentle" };
let serverError = "";
let noticeText = "";
let noticeUntil = 0;
let hovering = false;

function bar(p: number | undefined) {
  const w = Math.max(0, Math.min(100, p ?? 0));
  return `<div class="bar"><i style="width:${w}%"></i></div>`;
}

function renderPanel() {
  const u = formatUsage(state.usage, Date.now());
  const esc = (t: string) => t.replace(/[&<>]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" })[c]!);
  panel.innerHTML = `
    <b>${esc(state.model ?? "Model unknown")}</b>
    <div class="row"><span>5-hour limit</span><span>${esc(u.fiveHour)}</span></div>${bar(state.usage.fiveHourPct)}
    <div class="row"><span>Weekly limit</span><span>${esc(u.sevenDay)}</span></div>${bar(state.usage.sevenDayPct)}
    <div class="row"><span>Context used</span><span>${esc(u.context)}</span></div>${bar(state.usage.contextPct)}`;
}

function render() {
  const now = Date.now();
  stage.className = `corner-${settings.corner}`;
  const asleepHidden = settings.autoHide && state.mood === "sleeping" && !hovering;
  stage.classList.toggle("gone", settings.hidden || asleepHidden);
  mascot.style.width = mascot.style.height = `${settings.size}px`;
  ember.setMascot(getMascot(settings.mascot));
  ember.setAnimation(settings.animation);
  ember.setMood(state.mood);
  ember.setVisible(!(settings.hidden || asleepHidden));
  notice.textContent = serverError || (noticeUntil > now ? noticeText : "") || LABELS[state.mood];
  panel.hidden = !hovering;
  if (hovering) renderPanel();
}

function showNotice(text: string, ms = NOTICE_MS) {
  noticeText = text;
  noticeUntil = Date.now() + ms;
  render();
  setTimeout(render, ms + 5); // clear it when it expires
}

// One timer, set to the next moment the mood could change; nothing polls.
let timer: ReturnType<typeof setTimeout> | undefined;
function schedule() {
  clearTimeout(timer);
  const now = Date.now();
  const at = nextDeadline(state, now);
  if (at !== undefined) timer = setTimeout(() => apply({ tick: Date.now() }), at - now + 5);
}

function apply(input: MascotEvent | { tick: number }) {
  const before = state.mood;
  state = reduce(state, input);
  if (state.mood === "greeting" && before !== "greeting") ember.popIn();
  render();
  schedule();
}

// First run: until Claude Code is connected, clicking Ember connects it.
async function onMascotClick() {
  const connected = await invoke<boolean>("mascot_status").catch(() => false);
  if (connected) return;
  try {
    await invoke("mascot_install");
    showNotice("Connected! Start a new Claude Code session to see me react.");
  } catch (e) {
    showNotice(String(e), ONBOARDING_MS);
  }
}
mascot.addEventListener("click", onMascotClick);
mascot.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  invoke("show_context_menu");
});
invoke<boolean>("mascot_status").then((connected) => {
  if (!connected) showNotice("Hi, I'm Ember! Click me to connect Claude Code.", ONBOARDING_MS);
});
ember.popIn();

listen<MascotEvent>("mascot-event", (e) => apply(e.payload));
listen<string>("mascot-server-error", (e) => { serverError = e.payload; render(); });
listen<Settings>("mascot-settings", (e) => { settings = e.payload; render(); });
listen<boolean>("mascot-hover", (e) => { hovering = e.payload; render(); });
listen<string>("mascot-notice", (e) => showNotice(e.payload));
invoke<string | null>("bridge_error").then((m) => { if (m) { serverError = m; render(); } });
listen<boolean>("mascot-low-power", (e) => ember.setLowPower(e.payload));
invoke<boolean>("low_power").then((on) => ember.setLowPower(on));
invoke<Settings>("get_settings").then((s) => { settings = s; render(); });
schedule();
render();
