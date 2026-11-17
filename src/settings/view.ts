// Small text helpers for the settings window.
export type Corner = "br" | "bl" | "tr" | "tl";

const CORNERS: Record<Corner, string> = {
  br: "Bottom right",
  bl: "Bottom left",
  tr: "Top right",
  tl: "Top left",
};

export function cornerLabel(c: Corner): string {
  return CORNERS[c];
}

export function animationNote(lowPower: boolean): string {
  return lowPower ? "Low Power Mode is on, so the mascot shows still frames until it's off." : "";
}

export function statusText(s: { connected: boolean }): string {
  return s.connected ? "Connected" : "Not connected";
}

export function desktopText(running: boolean): string {
  return running ? "Detected" : "Not running";
}
