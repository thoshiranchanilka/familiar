// Draws any mascot definition in any mood on a 28×28 grid (16×16 sprite plus room
// for effects). Frame `f` advances at the caller's chosen rate (8 fps when animating).
import type { Mood } from "../../mood/types";
import type { MascotDef } from "./defs";

export interface Pixel { x: number; y: number; color: string; }

export const GRID = 28;
const OX = 6;
const OY = 7;

const KEY_LIT = "#ffd166";
const WHITE = "#ffffff";
const SPARK = "#ffe08a";
const SHADOW = "rgba(110,110,130,.35)";
const KEYBOARD = "#3a3f4b";
const KEY = "#8b93a3";
const THOUGHT = "#9fb0cc";
const SWEAT_LIGHT = "#bfe7ff";
const SWEAT = "#6cc3ff";
const ZZZ = "#8fa3c7";
const RED = "#e2574c";
const Z = ["###", ".#.", "###"];

export function mascotPixels(def: MascotDef, mood: Mood, f: number): Pixel[] {
  const out: Pixel[] = [];
  const px = (x: number, y: number, color: string, w = 1, h = 1) => {
    for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) {
      const X = x + i, Y = y + j;
      if (X >= 0 && X < GRID && Y >= 0 && Y < GRID) out.push({ x: X, y: Y, color });
    }
  };
  const sparkle = (x: number, y: number, big: boolean) => {
    px(x, y, WHITE);
    if (big) { px(x - 1, y, SPARK); px(x + 1, y, SPARK); px(x, y - 1, SPARK); px(x, y + 1, SPARK); }
  };

  let dy = 0;
  let dx = 0;
  if (mood === "idle") dy = (f % 16) < 8 ? 0 : 1;
  if (mood === "working") dy = f % 2 ? 0 : -1;
  if (mood === "done") dy = [0, -2, -3, -2, 0, 0, 0, 0][f % 8];
  if (mood === "sleeping") dy = (f % 24) < 12 ? 1 : 0;
  if (mood === "greeting") dy = [0, -1, -2, -1, 0, 0][f % 6];
  if (mood === "error") dx = [0, 1, 0, -1][f % 4];
  const bx = OX + dx;
  const by = OY + dy;

  const sw = dy < -1 ? 8 : 10;
  px(OX + 8 - sw / 2, OY + 15, SHADOW, sw, 1);

  def.map.forEach((row, j) => [...row].forEach((k, i) => {
    if (k === ".") return;
    let color = def.pal[k];
    if (k === def.blinkLetter && mood === "working" && f % 2) color = "#ffe2c4";
    if (k === def.blinkLetter && mood === "sleeping") color = "#5a4a40";
    px(bx + i, by + j, color);
  }));

  // eyes
  const ey = by + def.eyes.y, lx = bx + def.eyes.l, rx = bx + def.eyes.r;
  const line = def.glow ? def.eye : def.pal.o;
  const openEye = (x: number) => { px(x, ey, def.eye, 2, 2); if (!def.glow) px(x, ey, WHITE); };
  const flat = (x: number, c: string) => px(x, ey + 1, c, 2, 1);
  const caret = (x: number) => { px(x, ey + 1, line); px(x + 1, ey, line); px(x + 2, ey + 1, line); };

  if (mood === "idle" || mood === "greeting") {
    const blinking = mood === "idle" && f % 24 === 23;
    [lx, rx].forEach((x) => (blinking ? flat(x, line) : openEye(x)));
  } else if (mood === "working") {
    [lx, rx].forEach((x) => px(x, ey + 1, def.eye, 2, 1));
    px(lx + 1, ey - 1, line); px(rx, ey - 1, line);
  } else if (mood === "thinking") {
    [lx, rx].forEach((x) => { px(x + 1, ey, def.eye); if (!def.glow) px(x, ey + 1, def.soft); });
  } else if (mood === "done") {
    caret(lx - 1); caret(rx);
  } else if (mood === "sleeping") {
    [lx, rx].forEach((x) => flat(x, line));
  } else if (mood === "error") {
    // squeezed ">" "<" eyes
    px(lx, ey, line); px(lx + 1, ey + 1, line); px(lx, ey + 2, line);
    px(rx + 1, ey, line); px(rx, ey + 1, line); px(rx + 1, ey + 2, line);
  }

  // mouth
  const mx = bx + def.mouth.x, my = by + def.mouth.y, M = def.glow ? def.eye : def.pal.o;
  if (mood === "idle" || mood === "working") px(mx, my, M, 2, 1);
  if (mood === "thinking" || mood === "sleeping") px(mx + 1, my, M);
  if (mood === "done" || mood === "greeting") { px(mx - 1, my, M, 4, 1); px(mx, my + 1, def.glow ? M : RED, 2, 1); }
  if (mood === "error") { px(mx - 1, my + 1, M); px(mx, my, M); px(mx + 1, my + 1, M); px(mx + 2, my, M); }

  // effects
  if (mood === "working") {
    px(OX + 1, OY + 16, KEYBOARD, 14, 3);
    for (let i = 0; i < 6; i++) px(OX + 2 + i * 2, OY + 17, (i + f) % 3 === 0 ? KEY_LIT : KEY);
    if (f % 4 < 2) { px(OX + 16, OY + 2, SPARK); px(OX + 18, OY + 4, SPARK); }
    else { px(OX + 17, OY + 1, SPARK); px(OX - 2, OY + 3, SPARK); }
  }
  if (mood === "thinking") {
    const n = Math.min(3, Math.floor((f % 16) / 4));
    if (n > 0) px(OX + 15, OY - 1, THOUGHT);
    if (n > 1) px(OX + 17, OY - 3, THOUGHT, 2, 2);
    if (n > 2) { px(OX + 19, OY - 6, THOUGHT, 3, 3); px(OX + 20, OY - 5, WHITE); }
    const d = OY + 4 + ((f >> 1) % 4);
    px(OX + 14, d, SWEAT_LIGHT); px(OX + 14, d + 1, SWEAT);
  }
  if (mood === "done") {
    ([[2, 5], [24, 3], [1, 17], [25, 15]] as const).forEach(([x, y], i) => {
      if ((f + i * 2) % 8 < 5) sparkle(x, y, (f + i) % 4 < 2);
    });
  }
  if (mood === "sleeping") {
    for (let k = 0; k < 3; k++) {
      const ph = (f + k * 8) % 24;
      if (ph >= 20) continue;
      const zx = OX + 13 + Math.floor(ph / 6), zy = OY + 1 - Math.floor(ph / 3);
      Z.forEach((row, j) => [...row].forEach((ch, i) => { if (ch === "#") px(zx + i, zy + j, ZZZ); }));
    }
  }
  if (mood === "greeting" && f % 6 < 3) { sparkle(3, 8, true); sparkle(24, 6, false); }
  if (mood === "error") { px(OX + 15, OY + 1, RED, 1, 3); px(OX + 15, OY + 5, RED); }
  return out;
}
