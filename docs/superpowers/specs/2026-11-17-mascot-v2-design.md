# Claude Puppet v2: power diet, settings window, more mascots, sharing

Builds on `2026-11-17-claude-mascot-design.md` (v1, shipped on branch `feat/mascot`).

## Intent
Claude Puppet is a hobby project the user wants to share with coworkers through a public Git repo. Three things block that today:
1. **Power.** v1 measured ~3.3% CPU and a macOS energy score of ~5 at idle (app 3.6 + web view 1.6). An always-on overlay that warms the Mac is a barrier at work. An earlier `nettop`-based network sampler overheated an M4 MacBook and has already been removed.
2. **Control.** The only controls live in the menu-bar icon, which the MacBook notch can hide. The user could not find a way to quit and had to use Activity Monitor.
3. **Choice and distribution.** Users should pick their mascot and size, and coworkers need an easy, free way to install it.

## Goals
- Near-zero power at idle, proven by a repeatable measurement.
- Every control is reachable without the menu-bar icon.
- A settings window where users pick a mascot (Ember, Mochi, Bolt) and adjust size, position and animation.
- Adding a mascot later means adding one definition file.
- Coworkers can install a free build from GitHub Releases with clear steps.

## Non-goals and anti-goals
- No user-made or imported mascots (mascots come only from the set we ship).
- No paid Apple Developer signing or notarization (the user has no budget for it).
- No sounds, no Windows or Linux.
- Never publish, push, or create the GitHub repo without the user's explicit go-ahead.
- Anti-goal: any change that raises idle power back above the targets below.

## Decisions

### 1. Power diet
Targets, measured over 30 s with Ember idle and no Claude activity (app process plus its web view helpers combined):
- average CPU at most **0.3%**
- macOS energy score (`top -stats power`) at most **0.5**

If a target proves unreachable in practice, the check reports the measured value and the user decides; the target is not quietly loosened.

Changes:
- **Claude Desktop watcher:** no whole-system process scan while Claude Desktop isn't running. Detect launch, quit and focus cheaply: event notifications, or a check through `NSRunningApplication` for the Claude bundle id at most once per 2 s. Sample CPU only for Claude's own processes, once per second, and only while Claude Desktop runs.
- **Hover detection:** cursor polling drops from 10×/s to 4×/s and stops while Ember is hidden by the user.
- **Animation levels** (new setting `animation`):
  - *Full:* 8 fps in every mood (v1 behavior).
  - *Gentle (default):* 8 fps only while working, thinking, done, greeting or error. Idle is a still frame plus a blink about every 4 s. Asleep is about 2 fps.
  - *Still:* one frame per mood, redrawn only when the mood changes.
- **Low Power Mode:** while macOS Low Power Mode is on, Ember behaves as *Still* whatever the setting.
- **Redraw only on change:** no canvas repaint when the frame is identical, and no drawing while Ember is hidden.
- **Mood timer:** the overlay's 250 ms polling tick is replaced by one timer set to the reducer's next deadline (thinking threshold, done/greeting/error expiry, session staleness, sleep).
- **Power check:** `scripts/power-check.sh` launches a built app, waits for it to settle, samples 30 s, prints the numbers and exits non-zero when a target is missed. It is run before every release.

### 2. Right-click menu on Ember
Right-clicking Ember opens a native menu with these items:
- Settings…
- Hide Ember
- Connect or Disconnect Claude Code
- Quit Claude Puppet

**Getting Ember back after hiding it:** opening Claude Puppet again (from Applications or Spotlight) while it is running shows Ember and opens Settings. This requires a single running instance.

### 3. Settings window
A normal macOS window opened from the right-click menu, the menu-bar icon, or by reopening the app. Every change applies and saves immediately; there is no Save button. Sections:
- **Mascot:** a card per mascot, each animating live in the *Gentle* level; click to choose.
- **Size:** a slider from 64 to 192 px; Ember resizes live as it moves.
- **Position:** four corner buttons and "Hide while asleep".
- **Animation:** Full / Gentle / Still, with a note that Low Power Mode forces Still.
- **Claude Code:** Connected or Not connected, with a Connect or Disconnect button.
- **Claude Desktop:** Detected / Not running.
- **Footer:** the version and a Quit button.

### 4. Mascot system
- Each mascot is one definition: id, name, kind, palette, 16×16 pixel map, eye and mouth positions, eye style (dark with a highlight, or glowing), and optional extras such as Bolt's antenna light.
- One shared renderer draws every mood for any definition. This generalizes v1's `emberPixels`.
- v2 ships Ember, Mochi and Bolt, using the maps and palettes from the concept page (https://claude.ai/artifact/EQncxPaYwqqyPzKyXAeqbi). Each gets all seven moods, including error and greeting.
- A `mascot` setting stores the chosen id, defaulting to Ember. An unknown id falls back to Ember.
- The app icon stays Ember.

### 5. Sharing setup
- Free ad-hoc signature for the whole app bundle (`signingIdentity: "-"`). This avoids the "damaged" error but not the normal Gatekeeper warning.
- A GitHub Actions workflow builds the Apple Silicon `.dmg` when a `v*` tag is pushed and attaches it to a draft GitHub Release.
- A README covering:
  - what it is
  - a privacy note: no conversation content is read, and everything stays on localhost
  - install with the **Open Anyway** steps and the `xattr` alternative
  - building from source
  - connecting and disconnecting Claude Code
  - uninstalling
  - power use
- Nothing is pushed or published by the builder.

## Constraints
- macOS on Apple Silicon only. Tauri 2 with the existing architecture: Rust core with a localhost server, and a TypeScript overlay with the pure mood reducer.
- v1's Global Constraints still apply: localhost-only token-checked server, no conversation content, safe settings.json edits, click-through overlay.
- Existing settings files from v1 must keep loading. New fields get defaults.

## Left to the builder
- Event notifications versus cheap polling for Claude Desktop launch, quit and focus, as long as the power targets are met.
- How Low Power Mode is detected (notification or occasional check).
- The settings window's visual design, within the v1 concept page's look (warm Ember palette, pixel display font).
- How the single-instance behavior is implemented (a plugin, or the existing local server).

## Open for the user (not blocking)
- **License:** a public repo needs one (MIT is common for hobby projects). The README links to `LICENSE` once the user picks.
- **Name:** "Claude Puppet" uses Anthropic's product name in a public repo. Consider a name that doesn't, such as "Ember Desk Pet", to avoid trademark trouble.
