// Paints a mascot onto a canvas. Frames are scheduled one at a time by the
// animation level, identical frames are skipped, and nothing runs while hidden.
import type { Mood } from "../../mood/types";
import { frameAt, frameIntervalMs } from "./animation";
import { getMascot, type Animation, type MascotDef } from "./defs";
import { GRID, mascotPixels } from "./render";

export interface MascotView {
  setMascot(def: MascotDef): void;
  setMood(m: Mood): void;
  setAnimation(level: Animation): void;
  setLowPower(on: boolean): void;
  setVisible(on: boolean): void;
  popIn(): void;
  popOut(): void;
}

export function createMascot(canvas: HTMLCanvasElement, initial: MascotDef = getMascot("ember")): MascotView {
  canvas.width = GRID;
  canvas.height = GRID;
  canvas.style.imageRendering = "pixelated";
  const ctx = canvas.getContext("2d")!;
  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  let def = initial;
  let mood: Mood = "idle";
  let level: Animation = "gentle";
  let lowPower = false;
  let visible = true;
  let tick = 0;
  let lastKey = "";
  let timer: ReturnType<typeof setTimeout> | undefined;

  const effective = (): Animation => (reduced ? "still" : level);

  const paint = () => {
    const pixels = mascotPixels(def, mood, frameAt(effective(), mood, lowPower, tick));
    const key = pixels.map((p) => `${p.x},${p.y},${p.color}`).join(";");
    if (key === lastKey) return;
    lastKey = key;
    ctx.clearRect(0, 0, GRID, GRID);
    for (const p of pixels) {
      ctx.fillStyle = p.color;
      ctx.fillRect(p.x, p.y, 1, 1);
    }
  };

  const loop = () => {
    clearTimeout(timer);
    if (!visible) return;
    paint();
    const wait = frameIntervalMs(effective(), mood, lowPower, tick);
    if (wait !== null) timer = setTimeout(() => { tick++; loop(); }, wait);
  };

  const restart = () => { tick = 0; loop(); };
  restart();

  const replay = (cls: string) => {
    canvas.classList.remove("pop-in", "pop-out");
    void canvas.offsetWidth; // restart the CSS animation
    canvas.classList.add(cls);
  };

  return {
    setMascot(d) { if (d.id !== def.id) { def = d; restart(); } },
    setMood(m) { if (m !== mood) { mood = m; restart(); } },
    setAnimation(l) { if (l !== level) { level = l; restart(); } },
    setLowPower(on) { if (on !== lowPower) { lowPower = on; restart(); } },
    setVisible(on) { if (on !== visible) { visible = on; lastKey = ""; restart(); } },
    popIn() { replay("pop-in"); },
    popOut() { replay("pop-out"); },
  };
}
