// Familiar website: a retro desktop. Every mascot here is drawn live by the app's own renderer.
import type { Mood } from "../src/mood/types";
import { MASCOTS, getMascot, type MascotDef } from "../src/ui/mascot/defs";
import { createMascot, type MascotView } from "../src/ui/mascot/mascot";

const REPO = "https://github.com/thoshiranchanilka/familiar";
const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const $ = <T extends Element = HTMLElement>(sel: string, root: ParentNode = document) => root.querySelector(sel) as T;
const $$ = <T extends Element = HTMLElement>(sel: string, root: ParentNode = document) => [...root.querySelectorAll(sel)] as T[];

for (const a of $$<HTMLAnchorElement>("[data-download]")) a.href = `${REPO}/releases/latest`;
for (const a of $$<HTMLAnchorElement>("[data-github]")) a.href = REPO;
for (const a of $$<HTMLAnchorElement>("[data-releases]")) a.href = `${REPO}/releases`;

/** A mascot that animates only while it's on screen. */
function live(canvas: HTMLCanvasElement, def: MascotDef, mood: Mood = "idle", level: "full" | "gentle" = "full"): MascotView {
  const view = createMascot(canvas, def);
  view.setAnimation(level);
  view.setMood(mood);
  new IntersectionObserver(([e]) => view.setVisible(e.isIntersecting)).observe(canvas);
  return view;
}

// ---------- menu bar ----------
live($<HTMLCanvasElement>(".mb-mascot"), getMascot("ember"), "idle", "gentle");
const clock = $(".mb-clock");
const tickClock = () => (clock.textContent = new Date().toLocaleTimeString([], { hour: "numeric", minute: "2-digit" }));
tickClock();
setInterval(tickClock, 30_000);

// ---------- desktop icons (tiny pixel maps) ----------
const ICONS: Record<string, { pal: Record<string, string>; map: string[] }> = {
  doc: {
    pal: { o: "#2a1d17", w: "#fffaf0", l: "#c9b8ad", f: "#ef6a3a" },
    map: ["oooooooo....", "owwwwwwoo...", "owlllllwoo..", "owwwwwwwwo..", "owllllllwo..", "owwwwwwwwo..",
      "owlllllwwo..", "owwwwwwwwo..", "owllllllwo..", "owwwwwffwo..", "owwwwwffwo..", "oooooooooo.."],
  },
  folder: {
    pal: { o: "#2a1d17", y: "#ffd166", d: "#e9a93f" },
    map: ["............", ".oooo.......", "oyyyyo......", "oyyyyyoooooo", "oddddddddddo", "oyyyyyyyyyyo",
      "oyyyyyyyyyyo", "oyyyyyyyyyyo", "oyyyyyyyyyyo", "oyyyyyyyyyyo", "oooooooooooo", "............"],
  },
  book: {
    pal: { o: "#2a1d17", b: "#ef6a3a", w: "#fffaf0", s: "#c94d22" },
    map: ["............", ".oooooooooo.", ".obbbbbbbbo.", ".obwwwwwwbo.", ".obbbbbbbbo.", ".obbbbbbbbo.",
      ".obbbbbbbbo.", ".obbbbbbbbo.", ".osssssssso.", ".owwwwwwwwo.", ".oooooooooo.", "............"],
  },
  cat: {
    pal: { o: "#2a1d17", g: "#3a3f4b", w: "#fffaf0" },
    map: ["............", ".o........o.", ".oo......oo.", ".oggggggggo.", "oggggggggggo", "oggwggggwggo",
      "oggggggggggo", "oggggggggggo", ".oggggggggo.", "..oggggggo..", "...oooooo...", "............"],
  },
};
for (const c of $$<HTMLCanvasElement>("canvas[data-icon]")) {
  const icon = ICONS[c.dataset.icon!];
  c.width = c.height = 12;
  const ctx = c.getContext("2d")!;
  icon.map.forEach((row, y) => [...row].forEach((k, x) => {
    if (k === ".") return;
    ctx.fillStyle = icon.pal[k];
    ctx.fillRect(x, y, 1, 1);
  }));
}

// ---------- hero: a Claude Code session, and the mascot reacting in the corner ----------
type Step = { wait: number; line?: string; mood?: Mood; bubble?: string; show?: boolean };
const SCRIPT: Step[] = [
  { wait: 900, line: `<span class="p">$</span> claude`, show: true, mood: "greeting", bubble: "Hi!" },
  { wait: 1300, line: `<span class="m">✻ Welcome to Claude Code</span>`, mood: "idle", bubble: "" },
  { wait: 1500, line: `<span class="p">&gt;</span> add a dark mode to the settings page`, mood: "working", bubble: "Working…" },
  { wait: 1200, line: `<span class="m">⏺ Read(src/settings.css)</span>` },
  { wait: 1300, line: `<span class="m">⏺ Edit(src/settings.css)  +24 −3</span>` },
  { wait: 1500, line: `<span class="m">✻ Thinking…</span>`, mood: "thinking", bubble: "Thinking longer…" },
  { wait: 2600, line: `<span class="m">⏺ Bash(npm test)</span>` },
  { wait: 1100, line: `  <span class="ok">✓ 42 passed</span>` },
  { wait: 900, line: `<span class="ok">⏺ Dark mode is in, and all tests pass.</span>`, mood: "done", bubble: "Done!" },
  { wait: 2600, mood: "idle", bubble: "" },
  { wait: 1800, line: `<span class="p">&gt;</span> /exit`, show: false },
  { wait: 2200 },
];
const heroCanvas = $<HTMLCanvasElement>(".hero-mascot");
const hero = createMascot(heroCanvas, MASCOTS[0]);
hero.setAnimation("full");
const term = $(".term-body");
const bubble = $(".bubble");
let lines: string[] = [];
let step = 0;
let round = 0;
let demoTimer: ReturnType<typeof setTimeout> | undefined;
let demoOn = false;

function paintTerm() {
  const prompt = `<span class="p">${lines.length && step < SCRIPT.length - 1 ? "&gt;" : "$"}</span> <span class="caret"></span>`;
  term.innerHTML = [...lines, prompt].map((l) => `<div>${l}</div>`).join("");
}
function pop(show: boolean) {
  heroCanvas.classList.remove("in", "out");
  void heroCanvas.offsetWidth;
  heroCanvas.classList.add(show ? "in" : "out");
  if (!show) bubble.hidden = true;
}
function runStep() {
  const s = SCRIPT[step];
  if (s.line) lines.push(s.line);
  if (s.show !== undefined) pop(s.show);
  if (s.mood) hero.setMood(s.mood);
  if (s.bubble !== undefined) { bubble.textContent = s.bubble; bubble.hidden = !s.bubble; }
  paintTerm();
  step++;
  if (step >= SCRIPT.length) {
    step = 0;
    lines = [];
    round++;
    hero.setMascot(MASCOTS[round % MASCOTS.length]);
  }
  if (demoOn) demoTimer = setTimeout(runStep, SCRIPT[step].wait);
}
function setDemo(on: boolean) {
  if (on === demoOn) return;
  demoOn = on;
  hero.setVisible(on);
  clearTimeout(demoTimer);
  if (on) demoTimer = setTimeout(runStep, SCRIPT[step].wait);
}
const onScreen = (el: Element) => { const r = el.getBoundingClientRect(); return r.bottom > 0 && r.top < innerHeight; };
if (reduced) {
  lines = SCRIPT.slice(0, 9).flatMap((s) => (s.line ? [s.line] : []));
  hero.setMood("done");
  bubble.textContent = "Done!";
  bubble.hidden = false;
  paintTerm();
} else {
  heroCanvas.classList.add("out");
  paintTerm();
  const desk = $(".desktop");
  new IntersectionObserver(([e]) => setDemo(e.isIntersecting && !document.hidden)).observe(desk);
  document.addEventListener("visibilitychange", () => setDemo(!document.hidden && onScreen(desk)));
}

// ---------- session timeline + usage peek (follow the chosen mascot) ----------
const timeline = $$<HTMLCanvasElement>(".timeline canvas").map((c) => live(c, MASCOTS[0], c.dataset.mood as Mood));
const usageMascot = live($<HTMLCanvasElement>(".usage-mascot"), MASCOTS[0], "idle");

// ---------- character select ----------
const QUIRKS: Record<string, string> = {
  ember: "Bounces with a big hop when a task lands.",
  mochi: "Naps the moment you stop typing.",
  bolt: "Its antenna blinks while Claude works.",
  boo: "Fades to see-through when it naps, and says BOO on errors.",
  inky: "Taps the keys with its tentacles, and puffs ink on errors.",
  sprout: "Its leaf droops while thinking, and a flower opens when done.",
  lotl: "Gills flutter while it works, and it blushes when done.",
  hop: "Sleepy-eyed, puffs its throat while thinking, and hops extra high.",
  dot: "Puffs smoke while busy, and breathes a tiny flame on errors.",
};
let stageMood: Mood = "idle";
const stage = live($<HTMLCanvasElement>(".stage-mascot"), MASCOTS[0]);
const roster = $(".roster");
const rosterButtons = MASCOTS.map((m, i) => {
  const b = document.createElement("button");
  b.type = "button";
  b.setAttribute("role", "radio");
  b.setAttribute("aria-checked", String(i === 0));
  b.innerHTML = `<canvas aria-hidden="true"></canvas><span></span>`;
  $("span", b).textContent = m.name;
  roster.append(b);
  live($<HTMLCanvasElement>("canvas", b), m, "idle", "gentle");
  b.addEventListener("click", () => choose(m));
  return { m, b };
});
function choose(m: MascotDef) {
  for (const r of rosterButtons) r.b.setAttribute("aria-checked", String(r.m === m));
  stage.setMascot(m);
  stage.setMood(stageMood);
  $(".stage-title").textContent = `${m.name}.app`;
  $(".stage-name").textContent = m.name;
  $(".stage-kind").textContent = m.kind;
  $(".stage-blurb").textContent = m.blurb;
  $(".stage-quirk").textContent = QUIRKS[m.id] ?? "";
  for (const t of timeline) t.setMascot(m);
  usageMascot.setMascot(m);
}
for (const b of $$<HTMLButtonElement>(".moods button")) {
  b.addEventListener("click", () => {
    for (const o of $$(".moods button")) o.setAttribute("aria-checked", String(o === b));
    stageMood = b.dataset.mood as Mood;
    stage.setMood(stageMood);
  });
}
// arrow keys move through radio groups
for (const group of $$(".roster, .moods")) {
  group.addEventListener("keydown", (e) => {
    const items = $$<HTMLButtonElement>("button", group);
    const i = items.indexOf(document.activeElement as HTMLButtonElement);
    const d = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 0;
    if (i < 0 || !d) return;
    e.preventDefault();
    const next = items[(i + d + items.length) % items.length];
    next.focus();
    next.click();
  });
}
choose(MASCOTS[0]);

// ---------- install checklist (remembered on this device only) ----------
const boxes = $$<HTMLInputElement>(".checklist input");
let saved: boolean[] = [];
try { saved = JSON.parse(localStorage.getItem("familiar-install") ?? "[]"); } catch { /* storage unavailable */ }
boxes.forEach((box, i) => {
  box.checked = Boolean(saved[i]);
  box.addEventListener("change", () => {
    try { localStorage.setItem("familiar-install", JSON.stringify(boxes.map((b) => b.checked))); } catch { /* ignore */ }
  });
});
for (const b of $$<HTMLButtonElement>(".copy")) {
  b.addEventListener("click", async () => {
    const text = b.previousElementSibling?.textContent ?? "";
    try { await navigator.clipboard.writeText(text); b.textContent = "Copied!"; } catch { b.textContent = "Select & copy"; }
    setTimeout(() => (b.textContent = "Copy"), 1600);
  });
}

// ---------- docs tabs ----------
const tabs = $$<HTMLButtonElement>('[role="tab"]');
function selectTab(tab: HTMLButtonElement) {
  for (const t of tabs) {
    const on = t === tab;
    t.setAttribute("aria-selected", String(on));
    t.tabIndex = on ? 0 : -1;
    $(`#${t.getAttribute("aria-controls")}`).hidden = !on;
  }
}
tabs.forEach((t, i) => {
  t.addEventListener("click", () => selectTab(t));
  t.addEventListener("keydown", (e) => {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (!d) return;
    const next = tabs[(i + d + tabs.length) % tabs.length];
    next.focus();
    selectTab(next);
  });
});

// ---------- taskbar dock: the whole cast ----------
const dock = $(".dock");
for (const m of MASCOTS) {
  const c = document.createElement("canvas");
  dock.append(c);
  live(c, m, "idle", "gentle");
}
