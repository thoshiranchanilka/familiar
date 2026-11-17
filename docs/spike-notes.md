# Overlay spike notes (Task 1, 2026-11-17)

Verified by screenshot on macOS (Apple Silicon, Retina 3600x2338):
- Window is transparent, frameless and shadowless; only the mascot is visible.
- Always on top of other apps (drawn over VS Code and the Dock).
- Positioned at the screen's bottom-right corner with no Dock icon (Accessory activation policy).

Known limits / follow-ups:
- Positioning uses the full monitor size, so the mascot can overlap the Dock. Later: use the visible work area (NSScreen.visibleFrame) or let the user drag it.
- Click-through (`set_ignore_cursor_events(true)`), visibility on other Spaces and over full-screen apps still need a manual check by the user; they cannot be verified from a screenshot.
