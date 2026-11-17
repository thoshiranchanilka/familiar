// Pressing the mascot and moving the mouse this far starts dragging it to a new spot;
// anything less is a click.
export const DRAG_THRESHOLD_PX = 5;

export function isDrag(x0: number, y0: number, x1: number, y1: number): boolean {
  return Math.hypot(x1 - x0, y1 - y0) >= DRAG_THRESHOLD_PX;
}
