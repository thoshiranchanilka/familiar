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

/// Top-left position for a `win_w`×`win_h` window in `corner` of `area`, `margin` from the edges.
pub fn window_position(corner: Corner, area: Rect, win_w: i32, win_h: i32, margin: i32) -> (i32, i32) {
    let left = area.x + margin;
    let right = area.x + area.w - win_w - margin;
    let top = area.y + margin;
    let bottom = area.y + area.h - win_h - margin;
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
    let (x, y) = window_position(corner, inner, size, size, 0);
    Rect { x, y, w: size, h: size }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect { x: 0, y: 50, w: 3000, h: 2000 };

    #[test]
    fn places_window_in_each_corner() {
        assert_eq!(window_position(Corner::Br, AREA, 600, 700, 30), (2370, 1320));
        assert_eq!(window_position(Corner::Bl, AREA, 600, 700, 30), (30, 1320));
        assert_eq!(window_position(Corner::Tr, AREA, 600, 700, 30), (2370, 80));
        assert_eq!(window_position(Corner::Tl, AREA, 600, 700, 30), (30, 80));
    }

    #[test]
    fn respects_a_monitor_offset() {
        let second = Rect { x: -1920, y: 0, w: 1920, h: 1080 };
        assert_eq!(window_position(Corner::Tl, second, 100, 100, 10), (-1910, 10));
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
