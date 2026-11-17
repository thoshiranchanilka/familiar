export type Animation = "full" | "gentle" | "still";

export interface MascotDef {
  id: "ember" | "mochi" | "bolt";
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
}
