// Familiar website: live mascots drawn with the app's own renderer, a scripted Claude Code
// demo in the hero, a pixel field background, and small reveal/copy niceties.
import type { Mood } from "../src/mood/types";
import { MASCOTS, getMascot, type MascotDef } from "../src/ui/mascot/defs";
import { createMascot, type MascotView } from "../src/ui/mascot/mascot";

const REPO = "https://github.com/thoshiranchanilka/familiar";
const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const $ = <T extends Element = HTMLElement>(sel: string, root: ParentNode = document) => root.querySelector(sel) as T;
const $$ = <T extends Element = HTMLElement>(sel: string, root: ParentNode = document) => [...root.querySelectorAll(sel)] as T[];

// ---------- links ----------
for (const a of $$<HTMLAnchorElement>("[data-download]")) a.href = `${REPO}/releases/latest`;
for (const a of $$<HTMLAnchorElement>("[data-github]")) a.href = REPO;
for (const a of $$<HTMLAnchorElement>("[data-releases]")) a.href = `${REPO}/releases`;

// ---------- mascots that pause while off screen ----------
function live(canvas: HTMLCanvasElement, def: MascotDef, mood: Mood = "idle"): MascotView {
  const view = createMascot(canvas, def);
  view.setAnimation("full");
  view.setMood(mood);
  new IntersectionObserver(([e]) => view.setVisible(e.isIntersecting)).observe(canvas);
  return view;
}

live($<HTMLCanvasElement>(".brand-mascot"), getMascot("ember"));

// ---------- gallery ----------
const gallery = $(".gallery");
const views = MASCOTS.map((m) => {
  const card = document.createElement("article");
  card.className = "mascot-card";
  card.innerHTML = `<canvas aria-hidden="true"></canvas><div><b></b><span></span><p></p></div>`;
  $("b", card).textContent = m.name;
  $("span", card).textContent = m.kind;
  $("p", card).textContent = m.blurb;
  gallery.append(card);
  return live($<HTMLCanvasElement>("canvas", card), m);
});
for (const b of $$<HTMLButtonElement>(".moods button")) {
  b.addEventListener("click", () => {
    for (const o of $$(".moods button")) o.setAttribute("aria-checked", String(o === b));
    for (const v of views) v.setMood(b.dataset.mood as Mood);
  });
}

// ---------- CTA: a different mascot every few seconds ----------
const cta = live($<HTMLCanvasElement>(".cta-mascot"), getMascot("ember"), "done");
let ctaIndex = 0;
if (!reduced) setInterval(() => { ctaIndex = (ctaIndex + 1) % MASCOTS.length; cta.setMascot(MASCOTS[ctaIndex]); }, 2600);

// ---------- hero demo: a short Claude Code session the mascot reacts to ----------
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

function setShown(show: boolean) {
  heroCanvas.classList.remove("in", "out");
  void heroCanvas.offsetWidth;
  heroCanvas.classList.add(show ? "in" : "out");
  if (!show) bubble.hidden = true;
}

function runStep() {
  const s = SCRIPT[step];
  if (s.line) lines.push(s.line);
  if (s.show !== undefined) setShown(s.show);
  if (s.mood) hero.setMood(s.mood);
  if (s.bubble !== undefined) { bubble.textContent = s.bubble; bubble.hidden = !s.bubble; }
  paintTerm();
  step++;
  if (step >= SCRIPT.length) {
    // next round: fresh terminal, next mascot
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

if (reduced) {
  // a single still frame of the story instead of the animation
  lines = SCRIPT.slice(0, 9).flatMap((s) => (s.line ? [s.line] : []));
  hero.setMood("done");
  bubble.textContent = "Done!";
  bubble.hidden = false;
  paintTerm();
} else {
  heroCanvas.classList.add("out");
  paintTerm();
  new IntersectionObserver(([e]) => setDemo(e.isIntersecting && !document.hidden)).observe($(".demo"));
  document.addEventListener("visibilitychange", () => setDemo(!document.hidden && isOnScreen($(".demo"))));
}

function isOnScreen(el: Element) {
  const r = el.getBoundingClientRect();
  return r.bottom > 0 && r.top < innerHeight;
}

// ---------- pixel field behind the hero ----------
const field = $<HTMLCanvasElement>(".field");
const fctx = field.getContext("2d")!;
const CELL = 11;
let cols = 0, rows = 0, cells: Float32Array = new Float32Array(0), fieldTimer: ReturnType<typeof setInterval> | undefined;

function hash(i: number) { const x = Math.sin(i * 127.1) * 43758.5453; return x - Math.floor(x); }

function sizeField() {
  const r = field.getBoundingClientRect();
  const dpr = Math.min(devicePixelRatio || 1, 2);
  field.width = Math.round(r.width * dpr);
  field.height = Math.round(r.height * dpr);
  fctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  cols = Math.ceil(r.width / CELL);
  rows = Math.ceil(r.height / CELL);
  // density: a soft diagonal band rising from bottom-left to top-right, like a wave of pixels
  cells = new Float32Array(cols * rows);
  for (let y = 0; y < rows; y++) for (let x = 0; x < cols; x++) {
    const bandY = rows * (0.92 - 0.62 * (x / cols)) + Math.sin(x / 9) * 3;
    const d = (y - bandY) / (rows * 0.12);
    cells[y * cols + x] = Math.exp(-d * d) * (0.35 + 0.65 * hash(x * 7 + y * 131));
  }
}

function drawField(t: number) {
  const w = cols * CELL, h = rows * CELL;
  fctx.clearRect(0, 0, w, h);
  for (let y = 0; y < rows; y++) for (let x = 0; x < cols; x++) {
    const i = y * cols + x;
    const base = cells[i];
    if (base < 0.08) continue;
    const flicker = 0.55 + 0.45 * Math.sin(t * 0.0016 + hash(i) * 40);
    const a = base * flicker;
    if (a < 0.1) continue;
    const special = hash(i + 999);
    fctx.fillStyle = special > 0.975 ? `rgba(239,106,58,${a})` : special > 0.965 ? `rgba(255,209,102,${a * 0.8})` : `rgba(235,235,228,${a * 0.85})`;
    const s = a > 0.6 ? 5 : a > 0.35 ? 4 : 2;
    fctx.fillRect(x * CELL + (CELL - s) / 2, y * CELL + (CELL - s) / 2, s, s);
  }
}

sizeField();
drawField(0);
addEventListener("resize", () => { sizeField(); drawField(performance.now()); });
if (!reduced) {
  new IntersectionObserver(([e]) => {
    clearInterval(fieldTimer);
    if (e.isIntersecting) fieldTimer = setInterval(() => { if (!document.hidden) drawField(performance.now()); }, 120);
  }).observe(field);
}

// ---------- copy buttons ----------
for (const b of $$<HTMLButtonElement>(".copy")) {
  b.addEventListener("click", async () => {
    const text = b.previousElementSibling?.textContent ?? "";
    try { await navigator.clipboard.writeText(text); b.textContent = "Copied"; } catch { b.textContent = "Select & copy"; }
    setTimeout(() => (b.textContent = "Copy"), 1600);
  });
}

// ---------- reveal on scroll ----------
if (!reduced) {
  const io = new IntersectionObserver((entries) => {
    for (const e of entries) if (e.isIntersecting) { e.target.classList.add("shown"); io.unobserve(e.target); }
  }, { rootMargin: "0px 0px -8% 0px" });
  for (const el of $$(".section > *, .card, .mascot-card, .steps li, .doc-grid article, .faq details, .cta")) {
    el.classList.add("reveal");
    io.observe(el);
  }
}
