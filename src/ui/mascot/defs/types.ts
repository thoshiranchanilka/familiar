import type { Mood } from "../../../mood/types";

export type Animation = "full" | "gentle" | "still";

/** What a mascot's own effects get to draw with (grid coordinates). */
export interface Draw {
  px(x: number, y: number, color: string, w?: number, h?: number): void;
  /** top-left of the sprite this frame (includes the mood's bob/shake) */
  bx: number;
  by: number;
  /** top-left of the sprite at rest */
  ox: number;
  oy: number;
}

export interface MascotDef {
  id: "ember" | "mochi" | "bolt" | "boo" | "inky" | "sprout" | "lotl" | "hop" | "dot";
  name: string;
  kind: string;
  blurb: string;
  /** map letter → color */
  pal: Record<string, string>;
  /** 16 rows of 16 chars; "." is transparent */
  map: string[];
  eye: string;
  /** glowing screen eyes: eyes and mouth drawn in `eye` color, no white highlight */
  glow: boolean;
  eyes: { l: number; r: number; y: number };
  mouth: { x: number; y: number };
  /** light color under the eyes while glancing up (thinking) */
  soft: string;
  /** map letter that blinks while working (Bolt's antenna) */
  blinkLetter?: string;
  /** ground shadow under the sprite (default true; Boo floats) */
  shadow?: boolean;
  /** per-mood vertical offsets by frame, replacing the shared bob/hop */
  bob?: Partial<Record<Mood, number[]>>;
  /** body opacity while asleep (Boo fades) */
  sleepAlpha?: number;
  /** signature details, drawn after the shared effects; uses only the frame counter, no timers */
  effects?(mood: Mood, f: number, d: Draw): void;
}
