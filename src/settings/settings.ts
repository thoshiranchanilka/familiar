// Settings window: every change applies and saves immediately through `set_settings`.
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MASCOTS, getMascot, type Animation } from "../ui/mascot/defs";
import { createMascot } from "../ui/mascot/mascot";
import { animationNote, cornerLabel, offsetLabels, statusText, type Corner } from "./view";

interface Settings {
  corner: Corner; size: number; hidden: boolean; offsetX: number; offsetY: number;
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
  card.innerHTML = `<canvas></canvas><b>${m.name}</b>`;
  card.title = `${m.name}: ${m.kind}. ${m.blurb}`;
  card.setAttribute("aria-label", `${m.name}, ${m.kind}`);
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

// Advanced position: sliders for the distance from the corner's edges (dragging the mascot sets them too).
const advToggle = $("advanced-toggle");
advToggle.addEventListener("click", async () => {
  const open = $("advanced").hidden;
  $("advanced").hidden = !open;
  advToggle.setAttribute("aria-expanded", String(open));
  if (open) await refreshLimits();
});
async function refreshLimits() {
  const [mx, my] = await invoke<[number, number]>("position_limits").catch(() => [0, 0] as [number, number]);
  $<HTMLInputElement>("offset-x").max = String(mx);
  $<HTMLInputElement>("offset-y").max = String(my);
  render();
}
const offsetTimers: Partial<Record<"offsetX" | "offsetY", ReturnType<typeof setTimeout>>> = {};
for (const [id, key] of [["offset-x", "offsetX"], ["offset-y", "offsetY"]] as const) {
  const input = $<HTMLInputElement>(id);
  input.addEventListener("input", () => {
    $(`${id}-value`).textContent = `${input.value} px`;
    clearTimeout(offsetTimers[key]);
    offsetTimers[key] = setTimeout(() => save({ [key]: Number(input.value) }), SIZE_SEND_MS);
  });
}
$("offset-reset").addEventListener("click", () => save({ offsetX: 0, offsetY: 0 }));

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
  const labels = offsetLabels(settings.corner);
  $("offset-x-label").textContent = labels.x;
  $("offset-y-label").textContent = labels.y;
  for (const [id, v] of [["offset-x", settings.offsetX], ["offset-y", settings.offsetY]] as const) {
    const input = $<HTMLInputElement>(id);
    if (document.activeElement !== input) input.value = String(v ?? 0);
    $(`${id}-value`).textContent = `${input.value} px`;
  }
  pendingSize = Number(size.value);
  if (sizeTimer) return;
  sizeTimer = setTimeout(() => {
    sizeTimer = undefined;
    if (pendingSize !== undefined) save({ size: pendingSize });
    pendingSize = undefined;
  }, SIZE_SEND_MS);
});
size.addEventListener("change", () => save({ size: Number(size.value) }));


$<HTMLInputElement>("open-at-login").addEventListener("change", async (e) => {
  const box = e.target as HTMLInputElement;
  try {
    await invoke("set_open_at_login", { on: box.checked });
    $("error").textContent = "";
  } catch (err) {
    box.checked = !box.checked;
    $("error").textContent = `Couldn't change Open at login: ${err}`;
  }
});

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
  $("animation-note").textContent = animationNote(lowPower);
  for (const v of [hero, ...cardViews.map((c) => c.view)]) v.setLowPower(lowPower);
}

async function refreshStatus() {
  connected = await invoke<boolean>("mascot_status").catch(() => false);
  $("code-status").textContent = statusText({ connected });
  $("code-toggle").textContent = connected ? "Disconnect" : "Connect";
  $<HTMLInputElement>("open-at-login").checked = await invoke<boolean>("get_open_at_login").catch(() => false);
}

async function init() {
  [settings, lowPower] = await Promise.all([invoke<Settings>("get_settings"), invoke<boolean>("low_power")]);
  render();
  await refreshStatus();
  $("version").textContent = `Version ${await getVersion()}`;
  listen<Settings>("mascot-settings", (e) => { settings = e.payload; render(); });
  listen<boolean>("mascot-low-power", (e) => { lowPower = e.payload; render(); });
  // Statuses can change while the window is in the background (e.g. from the menu bar).
  getCurrentWindow().onFocusChanged(({ payload: focused }) => { if (focused) refreshStatus(); });
}

init();
