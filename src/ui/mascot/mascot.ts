// Paints Ember onto a canvas at 8 fps and handles pop-in / pop-out.
import type { Mood } from "../../mood/types";
import { GRID, emberPixels } from "./ember";

export interface MascotView { setMood(m: Mood): void; popIn(): void; popOut(): void; }

const FRAME_MS = 125;
const REDUCED_MOTION_FRAME = 10;

export function createMascot(canvas: HTMLCanvasElement): MascotView {
  canvas.width = GRID;
  canvas.height = GRID;
  canvas.style.imageRendering = "pixelated";
  const ctx = canvas.getContext("2d")!;
  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  let mood: Mood = "idle";
  let f = 0;

  const paint = () => {
    ctx.clearRect(0, 0, GRID, GRID);
    for (const p of emberPixels(mood, reduced ? REDUCED_MOTION_FRAME : f)) {
      ctx.fillStyle = p.color;
      ctx.fillRect(p.x, p.y, 1, 1);
    }
    f++;
  };
  paint();
  setInterval(paint, reduced ? 1000 : FRAME_MS);

  const replay = (cls: string) => {
    canvas.classList.remove("pop-in", "pop-out");
    void canvas.offsetWidth; // restart the CSS animation
    canvas.classList.add(cls);
  };

  return {
    setMood(m) { if (m !== mood) { mood = m; f = 0; } },
    popIn() { replay("pop-in"); },
    popOut() { replay("pop-out"); },
  };
}
