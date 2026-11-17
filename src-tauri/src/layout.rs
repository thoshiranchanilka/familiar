//! Geometry for placing the overlay window and finding the mascot's hover area.
//! All values are physical pixels.

use crate::settings::Corner;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x as f64 && px < (self.x + self.w) as f64 && py >= self.y as f64 && py < (self.y + self.h) as f64
    }
}

/// Top-left position for a `win_w`×`win_h` window in `corner` of `area`, `mx` from the side
/// edge and `my` from the top or bottom edge of that corner.
pub fn window_position(corner: Corner, area: Rect, win_w: i32, win_h: i32, mx: i32, my: i32) -> (i32, i32) {
    let left = area.x + mx;
    let right = area.x + area.w - win_w - mx;
    let top = area.y + my;
    let bottom = area.y + area.h - win_h - my;
    match corner {
        Corner::Br => (right, bottom),
        Corner::Bl => (left, bottom),
        Corner::Tr => (right, top),
        Corner::Tl => (left, top),
    }
}

/// The mascot's square (side `size`) inside the window, `inset` from the window's corner.
pub fn mascot_rect(corner: Corner, window: Rect, size: i32, inset: i32) -> Rect {
    let inner = Rect { x: window.x + inset, y: window.y + inset, w: window.w - 2 * inset, h: window.h - 2 * inset };
    let (x, y) = window_position(corner, inner, size, size, 0, 0);
    Rect { x, y, w: size, h: size }
}

/// The largest extra offsets (beyond `margin`) that keep the window on screen.
pub fn max_offsets(area: Rect, win_w: i32, win_h: i32, margin: i32) -> (i32, i32) {
    ((area.w - win_w - 2 * margin).max(0), (area.h - win_h - 2 * margin).max(0))
}

/// Where to anchor the window so the mascot stays exactly where the user dropped it:
/// the corner nearest the mascot, and the extra offsets from that corner (beyond `margin`),
/// kept within the screen.
pub fn placement_for_mascot(m: Rect, area: Rect, win_w: i32, win_h: i32, inset: i32, margin: i32) -> (Corner, i32, i32) {
    let right = m.x + m.w / 2 >= area.x + area.w / 2;
    let bottom = m.y + m.h / 2 >= area.y + area.h / 2;
    let corner = match (right, bottom) {
        (true, true) => Corner::Br,
        (false, true) => Corner::Bl,
        (true, false) => Corner::Tr,
        (false, false) => Corner::Tl,
    };
    let wx = if right { m.x + m.w + inset - win_w } else { m.x - inset };
    let wy = if bottom { m.y + m.h + inset - win_h } else { m.y - inset };
    let ox = if right { area.x + area.w - win_w - margin - wx } else { wx - area.x - margin };
    let oy = if bottom { area.y + area.h - win_h - margin - wy } else { wy - area.y - margin };
    let (max_x, max_y) = max_offsets(area, win_w, win_h, margin);
    (corner, ox.clamp(0, max_x), oy.clamp(0, max_y))
}

/// Distance in px from a point to the nearest edge of `r` (0 inside).
pub fn distance_to(r: &Rect, px: f64, py: f64) -> f64 {
    let dx = (r.x as f64 - px).max(px - (r.x + r.w) as f64).max(0.0);
    let dy = (r.y as f64 - py).max(py - (r.y + r.h) as f64).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

/// How long the hover watch sleeps: quick near the mascot, slow when the cursor is far away.
pub fn hover_delay(distance_px: f64, scale: f64) -> std::time::Duration {
    const NEAR_LOGICAL_PX: f64 = 200.0;
    if distance_px < NEAR_LOGICAL_PX * scale {
        std::time::Duration::from_millis(250)
    } else {
        std::time::Duration::from_millis(500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn distance_is_zero_inside_and_grows_outside() {
        let r = Rect { x: 100, y: 100, w: 50, h: 50 };
        assert_eq!(distance_to(&r, 120.0, 120.0), 0.0);
        assert_eq!(distance_to(&r, 160.0, 120.0), 10.0);
        assert_eq!(distance_to(&r, 50.0, 50.0), (50.0f64 * 50.0 * 2.0).sqrt());
    }

    #[test]
    fn hover_polls_fast_only_near_the_mascot() {
        // distances are physical px; scale 2 means 200 logical px = 400 physical px
        assert_eq!(hover_delay(0.0, 2.0), Duration::from_millis(250));
        assert_eq!(hover_delay(399.0, 2.0), Duration::from_millis(250));
        assert_eq!(hover_delay(401.0, 2.0), Duration::from_millis(500));
    }

    const AREA: Rect = Rect { x: 0, y: 50, w: 3000, h: 2000 };

    #[test]
    fn places_window_in_each_corner() {
        assert_eq!(window_position(Corner::Br, AREA, 600, 700, 30, 30), (2370, 1320));
        assert_eq!(window_position(Corner::Bl, AREA, 600, 700, 30, 30), (30, 1320));
        assert_eq!(window_position(Corner::Tr, AREA, 600, 700, 30, 30), (2370, 80));
        assert_eq!(window_position(Corner::Tl, AREA, 600, 700, 30, 30), (30, 80));
    }

    #[test]
    fn margins_push_inward_from_the_chosen_corner() {
        // 100 px further from the right edge, 40 px further from the bottom
        assert_eq!(window_position(Corner::Br, AREA, 600, 700, 130, 70), (2270, 1280));
        assert_eq!(window_position(Corner::Tl, AREA, 600, 700, 130, 70), (130, 120));
    }

    #[test]
    fn offsets_are_limited_to_the_screen() {
        assert_eq!(max_offsets(AREA, 600, 700, 30), (2340, 1240));
        assert_eq!(max_offsets(Rect { x: 0, y: 0, w: 500, h: 500 }, 600, 700, 30), (0, 0));
    }

    #[test]
    fn a_dragged_mascot_anchors_to_its_nearest_corner() {
        let size = 256;
        let inset = 24;
        // mascot dropped near the top-left quarter
        let m = Rect { x: 400, y: 300, w: size, h: size };
        let (c, ox, oy) = placement_for_mascot(m, AREA, 640, 720, inset, 32);
        assert_eq!(c, Corner::Tl);
        let (wx, wy) = window_position(c, AREA, 640, 720, 32 + ox, 32 + oy);
        assert_eq!(mascot_rect(c, Rect { x: wx, y: wy, w: 640, h: 720 }, size, inset), m);
        // mascot dropped near the bottom right
        let m = Rect { x: 2500, y: 1700, w: size, h: size };
        assert_eq!(placement_for_mascot(m, AREA, 640, 720, inset, 32).0, Corner::Br);
    }

    #[test]
    fn placement_round_trips_for_every_corner() {
        for c in [Corner::Br, Corner::Bl, Corner::Tr, Corner::Tl] {
            let (wx, wy) = window_position(c, AREA, 640, 720, 32 + 150, 32 + 90);
            let m = mascot_rect(c, Rect { x: wx, y: wy, w: 640, h: 720 }, 256, 24);
            let (c2, ox, oy) = placement_for_mascot(m, AREA, 640, 720, 24, 32);
            assert_eq!(window_position(c2, AREA, 640, 720, 32 + ox, 32 + oy), (wx, wy), "{c:?}");
        }
    }

    #[test]
    fn a_mascot_dragged_off_screen_is_kept_on_it() {
        let m = Rect { x: -900, y: 5000, w: 256, h: 256 };
        let (c, ox, oy) = placement_for_mascot(m, AREA, 640, 720, 24, 32);
        assert_eq!((c, ox, oy), (Corner::Bl, 0, 0));
    }

    #[test]
    fn respects_a_monitor_offset() {
        let second = Rect { x: -1920, y: 0, w: 1920, h: 1080 };
        assert_eq!(window_position(Corner::Tl, second, 100, 100, 10, 10), (-1910, 10));
    }

    #[test]
    fn mascot_sits_in_the_matching_window_corner() {
        let win = Rect { x: 1000, y: 500, w: 640, h: 720 };
        assert_eq!(mascot_rect(Corner::Br, win, 256, 24), Rect { x: 1360, y: 940, w: 256, h: 256 });
        assert_eq!(mascot_rect(Corner::Tl, win, 256, 24), Rect { x: 1024, y: 524, w: 256, h: 256 });
    }

    #[test]
    fn contains_is_half_open() {
        let r = Rect { x: 10, y: 10, w: 5, h: 5 };
        assert!(r.contains(10.0, 14.9));
        assert!(!r.contains(15.0, 12.0));
        assert!(!r.contains(9.9, 12.0));
    }
}
