// Familiar website. Every mascot on the page is drawn live by the app's own renderer.
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
const onScreen = (el: Element) => { const r = el.getBoundingClientRect(); return r.bottom > 0 && r.top < innerHeight; };

live($<HTMLCanvasElement>(".brand-mascot"), getMascot("ember"), "idle", "gentle");

// ---------- hero sparks: little pixel embers drifting up ----------
{
  const canvas = $<HTMLCanvasElement>(".sparks");
  const ctx = canvas.getContext("2d")!;
  type Spark = { x: number; y: number; v: number; s: number; life: number; hue: number };
  let sparks: Spark[] = [];
  let w = 0, h = 0, timer: ReturnType<typeof setInterval> | undefined;
  const COLORS = ["239,106,58", "255,138,92", "255,209,102"];
  const spawn = (anywhere: boolean): Spark => ({
    x: Math.random() * w, y: anywhere ? Math.random() * h : h + 10, v: 0.25 + Math.random() * 0.6,
    s: Math.random() < 0.2 ? 4 : Math.random() < 0.6 ? 3 : 2, life: 0.35 + Math.random() * 0.6, hue: Math.floor(Math.random() * 3),
  });
  const size = () => {
    const r = canvas.getBoundingClientRect();
    const dpr = Math.min(devicePixelRatio || 1, 2);
    w = r.width; h = r.height;
    canvas.width = Math.round(w * dpr); canvas.height = Math.round(h * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    sparks = Array.from({ length: Math.round(w / 22) }, () => spawn(true));
  };
  const draw = () => {
    ctx.clearRect(0, 0, w, h);
    for (const p of sparks) {
      p.y -= p.v * 3;
      p.x += Math.sin(p.y / 40) * 0.3;
      if (p.y < -10) Object.assign(p, spawn(false));
      const fade = Math.min(1, p.y / (h * 0.9));
      ctx.fillStyle = `rgba(${COLORS[p.hue]},${(p.life * fade).toFixed(2)})`;
      ctx.fillRect(Math.round(p.x), Math.round(p.y), p.s, p.s);
    }
  };
  size();
  draw();
  addEventListener("resize", () => { size(); draw(); });
  if (!reduced) {
    new IntersectionObserver(([e]) => {
      clearInterval(timer);
      if (e.isIntersecting) timer = setInterval(() => { if (!document.hidden) draw(); }, 66);
    }).observe(canvas);
  }
}

// ---------- hero demo: a Claude Code session, and the mascot reacting ----------
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
{
  const canvas = $<HTMLCanvasElement>(".hero-mascot");
  const hero = createMascot(canvas, MASCOTS[0]);
  hero.setAnimation("full");
  const term = $(".term-body");
  const bubble = $(".bubble");
  let lines: string[] = [];
  let step = 0, round = 0, on = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const paint = () => {
    const prompt = `<span class="p">${lines.length && step < SCRIPT.length - 1 ? "&gt;" : "$"}</span> <span class="caret"></span>`;
    term.innerHTML = [...lines, prompt].map((l) => `<div>${l}</div>`).join("");
  };
  const pop = (show: boolean) => {
    canvas.classList.remove("in", "out");
    void canvas.offsetWidth;
    canvas.classList.add(show ? "in" : "out");
    if (!show) bubble.hidden = true;
  };
  const run = () => {
    const s = SCRIPT[step];
    if (s.line) lines.push(s.line);
    if (s.show !== undefined) pop(s.show);
    if (s.mood) hero.setMood(s.mood);
    if (s.bubble !== undefined) { bubble.textContent = s.bubble; bubble.hidden = !s.bubble; }
    paint();
    if (++step >= SCRIPT.length) { step = 0; lines = []; hero.setMascot(MASCOTS[++round % MASCOTS.length]); }
    if (on) timer = setTimeout(run, SCRIPT[step].wait);
  };
  const set = (next: boolean) => {
    if (next === on) return;
    on = next;
    hero.setVisible(on);
    clearTimeout(timer);
    if (on) timer = setTimeout(run, SCRIPT[step].wait);
  };
  if (reduced) {
    lines = SCRIPT.slice(0, 9).flatMap((s) => (s.line ? [s.line] : []));
    hero.setMood("done");
    bubble.textContent = "Done!";
    bubble.hidden = false;
    paint();
  } else {
    canvas.classList.add("out");
    paint();
    const scene = $(".scene");
    new IntersectionObserver(([e]) => set(e.isIntersecting && !document.hidden)).observe(scene);
    document.addEventListener("visibilitychange", () => set(!document.hidden && onScreen(scene)));
  }
}

// ---------- parade: the whole cast, each in a different mood ----------
{
  const track = $(".parade-track");
  const moods: Mood[] = ["idle", "working", "done", "thinking", "idle", "done", "working", "sleeping", "idle"];
  for (const copy of [0, 1]) {
    MASCOTS.forEach((m, i) => {
      const c = document.createElement("canvas");
      track.append(c);
      if (copy === 0) live(c, m, moods[i]);
      else live(c, m, moods[i]);
    });
  }
}

// ---------- moods grid + hover demo (follow the chosen mascot) ----------
const MOOD_INFO: [Mood, string, string][] = [
  ["greeting", "Hello", "A session just started."],
  ["working", "Working", "Claude is editing and running things."],
  ["thinking", "Thinking longer", "Still going after 15 seconds."],
  ["done", "Done", "Claude finished the task."],
  ["error", "Uh-oh", "Something failed. Worth a look."],
  ["sleeping", "Asleep", "Nothing's happened for a while."],
];
const grid = $(".moods-grid");
const moodViews = MOOD_INFO.map(([mood, title, text]) => {
  const card = document.createElement("div");
  card.className = "mood";
  card.innerHTML = `<canvas aria-hidden="true"></canvas><b></b><span></span>`;
  $("b", card).textContent = title;
  $("span", card).textContent = text;
  grid.append(card);
  return live($<HTMLCanvasElement>("canvas", card), MASCOTS[0], mood);
});

const hoverMascot = live($<HTMLCanvasElement>(".hover-mascot"), MASCOTS[0], "idle");
{
  const target = $<HTMLButtonElement>(".hover-target");
  const usage = $(".usage");
  const show = (on: boolean) => { usage.hidden = !on; target.setAttribute("aria-expanded", String(on)); };
  target.addEventListener("mouseenter", () => show(true));
  target.addEventListener("mouseleave", () => show(false));
  target.addEventListener("focus", () => show(true));
  target.addEventListener("blur", () => show(false));
  target.addEventListener("click", () => show(usage.hasAttribute("hidden")));
}

// ---------- the cast ----------
const QUIRKS: Record<string, string> = {
  ember: "Bounces with a big hop when a task lands.",
  mochi: "Naps the moment you stop typing.",
  bolt: "Its antenna blinks while Claude works.",
  boo: "Fades to see-through when it naps, and says BOO on errors.",
  inky: "Taps the keys with its tentacles, and puffs ink on errors.",
  sprout: "Its leaf droops while thinking, and a flower opens when it's done.",
  lotl: "Gills flutter while it works, and it blushes when it's done.",
  hop: "Sleepy-eyed. Puffs its throat while thinking and hops extra high.",
  dot: "Puffs smoke while busy, and breathes a tiny flame on errors.",
};
let current = 0;
let stageMood: Mood = "idle";
const stage = live($<HTMLCanvasElement>(".stage-mascot"), MASCOTS[0]);
const thumbs = MASCOTS.map((m, i) => {
  const b = document.createElement("button");
  b.type = "button";
  b.setAttribute("role", "radio");
  b.setAttribute("aria-label", m.name);
  b.innerHTML = `<canvas aria-hidden="true"></canvas>`;
  $(".thumbs").append(b);
  live($<HTMLCanvasElement>("canvas", b), m, "idle", "gentle");
  b.addEventListener("click", () => choose(i));
  return b;
});
function choose(i: number) {
  current = (i + MASCOTS.length) % MASCOTS.length;
  const m = MASCOTS[current];
  thumbs.forEach((b, j) => { b.setAttribute("aria-checked", String(j === current)); b.tabIndex = j === current ? 0 : -1; });
  stage.setMascot(m);
  stage.setMood(stageMood);
  $(".stage-count").textContent = `${String(current + 1).padStart(2, "0")} / ${String(MASCOTS.length).padStart(2, "0")}`;
  $(".stage-name").textContent = m.name;
  $(".stage-kind").textContent = m.kind;
  $(".stage-blurb").textContent = m.blurb;
  $(".stage-quirk").textContent = QUIRKS[m.id] ?? "";
  for (const v of moodViews) v.setMascot(m);
  hoverMascot.setMascot(m);
}
for (const a of $$<HTMLButtonElement>(".arrow")) a.addEventListener("click", () => choose(current + Number(a.dataset.step)));
$(".thumbs").addEventListener("keydown", (e) => {
  const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
  if (!d) return;
  e.preventDefault();
  choose(current + d);
  thumbs[current].focus();
});
const chips = $$<HTMLButtonElement>(".chips button");
chips.forEach((b, i) => {
  b.tabIndex = i === 0 ? 0 : -1;
  b.addEventListener("click", () => {
    chips.forEach((o) => { o.setAttribute("aria-checked", String(o === b)); o.tabIndex = o === b ? 0 : -1; });
    stageMood = b.dataset.mood as Mood;
    stage.setMood(stageMood);
  });
  b.addEventListener("keydown", (e) => {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const next = chips[(i + d + chips.length) % chips.length];
    next.focus();
    next.click();
  });
});
choose(0);

// ---------- finale: a different mascot every few seconds ----------
{
  const finale = live($<HTMLCanvasElement>(".finale-mascot"), MASCOTS[0], "done");
  let i = 0;
  if (!reduced) setInterval(() => { if (!document.hidden) finale.setMascot(MASCOTS[(i = (i + 1) % MASCOTS.length)]); }, 2600);
}

// ---------- copy ----------
for (const b of $$<HTMLButtonElement>(".copy")) {
  b.addEventListener("click", async () => {
    const text = b.previousElementSibling?.textContent ?? "";
    try { await navigator.clipboard.writeText(text); b.textContent = "Copied"; } catch { b.textContent = "Select & copy"; }
    setTimeout(() => (b.textContent = "Copy"), 1600);
  });
}

// ---------- docs: highlight the section in view ----------
{
  const links = $$<HTMLAnchorElement>(".docs-nav a");
  const byId = new Map(links.map((a) => [a.hash.slice(1), a] as const));
  const io = new IntersectionObserver((entries) => {
    for (const e of entries) {
      if (!e.isIntersecting) continue;
      for (const a of links) a.classList.toggle("active", a === byId.get(e.target.id));
    }
  }, { rootMargin: "-30% 0px -60% 0px" });
  for (const art of $$(".docs-body article")) io.observe(art);
}
