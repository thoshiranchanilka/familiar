// Settings window: every change applies and saves immediately through `set_settings`.
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MASCOTS, getMascot, type Animation } from "../ui/mascot/defs";
import { createMascot } from "../ui/mascot/mascot";
import { animationNote, cornerLabel, desktopText, statusText, type Corner } from "./view";

interface Settings {
  corner: Corner; size: number; autoHide: boolean; hidden: boolean;
  mascot: string; animation: Animation;
}

const SIZE_SEND_MS = 60;
const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

let settings: Settings;
let connected = false;
let lowPower = false;

const hero = createMascot($("hero-mascot"));

// Mascot cards
const cardViews = MASCOTS.map((m) => {
  const card = document.createElement("button");
  card.type = "button";
  card.className = "card";
  card.setAttribute("role", "radio");
  card.innerHTML = `<canvas></canvas><b>${m.name}</b><span>${m.kind}</span>`;
  card.addEventListener("click", () => save({ mascot: m.id }));
  $("mascots").appendChild(card);
  const view = createMascot(card.querySelector("canvas")!, m);
  return { id: m.id, card, view };
});

// Corners
const CORNER_ORDER: Corner[] = ["tl", "tr", "bl", "br"];
const cornerButtons = CORNER_ORDER.map((c) => {
  const b = document.createElement("button");
  b.type = "button";
  b.setAttribute("role", "radio");
  b.textContent = cornerLabel(c);
  b.addEventListener("click", () => save({ corner: c }));
  $("corners").appendChild(b);
  return { c, b };
});

// Animation levels
const LEVELS: [Animation, string][] = [["full", "Full"], ["gentle", "Gentle"], ["still", "Still"]];
const levelButtons = LEVELS.map(([level, label]) => {
  const b = document.createElement("button");
  b.type = "button";
  b.setAttribute("role", "radio");
  b.textContent = label;
  b.addEventListener("click", () => save({ animation: level }));
  $("animation").appendChild(b);
  return { level, b };
});

// Size slider: live while dragging, at most one update per SIZE_SEND_MS, plus the final value.
const size = $<HTMLInputElement>("size");
let sizeTimer: ReturnType<typeof setTimeout> | undefined;
let pendingSize: number | undefined;
size.addEventListener("input", () => {
  $("size-value").textContent = `${size.value} px`;
  pendingSize = Number(size.value);
  if (sizeTimer) return;
  sizeTimer = setTimeout(() => {
    sizeTimer = undefined;
    if (pendingSize !== undefined) save({ size: pendingSize });
    pendingSize = undefined;
  }, SIZE_SEND_MS);
});
size.addEventListener("change", () => save({ size: Number(size.value) }));

$<HTMLInputElement>("auto-hide").addEventListener("change", (e) =>
  save({ autoHide: (e.target as HTMLInputElement).checked }));

$("code-toggle").addEventListener("click", async () => {
  $("error").textContent = "";
  try {
    await invoke(connected ? "mascot_uninstall" : "mascot_install");
  } catch (e) {
    $("error").textContent = String(e);
  }
  await refreshStatus();
});

$("quit").addEventListener("click", () => invoke("quit_app"));

async function save(patch: Partial<Settings>) {
  settings = await invoke<Settings>("set_settings", { patch });
  render();
}

function render() {
  const current = getMascot(settings.mascot);
  hero.setMascot(current);
  for (const { id, card } of cardViews) card.setAttribute("aria-checked", String(id === current.id));
  for (const { c, b } of cornerButtons) b.setAttribute("aria-checked", String(c === settings.corner));
  for (const { level, b } of levelButtons) b.setAttribute("aria-checked", String(level === settings.animation));
  if (document.activeElement !== size) size.value = String(settings.size);
  $("size-value").textContent = `${size.value} px`;
  $<HTMLInputElement>("auto-hide").checked = settings.autoHide;
  $("animation-note").textContent = animationNote(lowPower);
  for (const v of [hero, ...cardViews.map((c) => c.view)]) v.setLowPower(lowPower);
}

async function refreshStatus() {
  connected = await invoke<boolean>("mascot_status").catch(() => false);
  const desktop = await invoke<boolean>("desktop_status").catch(() => false);
  $("code-status").textContent = statusText({ connected });
  $("code-toggle").textContent = connected ? "Disconnect" : "Connect";
  $("desktop-status").textContent = desktopText(desktop);
}

async function init() {
  [settings, lowPower] = await Promise.all([invoke<Settings>("get_settings"), invoke<boolean>("low_power")]);
  render();
  await refreshStatus();
  $("version").textContent = `Version ${await getVersion()}`;
  listen<Settings>("mascot-settings", (e) => { settings = e.payload; render(); });
  listen<boolean>("mascot-low-power", (e) => { lowPower = e.payload; render(); });
  // Statuses can change while the window is in the background (menu bar, Claude Desktop launch).
  getCurrentWindow().onFocusChanged(({ payload: focused }) => { if (focused) refreshStatus(); });
}

init();
