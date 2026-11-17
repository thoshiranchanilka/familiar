# Claude Mascot: desktop companion for Claude Code and Claude Desktop

## Context
The user wants an animated desktop mascot that mirrors what Claude is doing (working, thinking longer, idle), shows the current model and usage, and pops out when Claude Code or Claude Desktop opens. Many intended users are non-technical and use only Claude Desktop, so install must be one-click and Desktop chat must be supported, even though it exposes far less data than Claude Code.

The project folder is empty and not a git repo. Platform: macOS (Apple Silicon). Windows is out of scope for v1.

## Findings (verified from docs and this machine)
- **Claude Code hooks:** `SessionStart`, `UserPromptSubmit`, `PreToolUse`/`PostToolUse`, `Stop`, `Notification`, `SubagentStart`/`SubagentStop`, `PreModelSwitch`/`PostModelSwitch`, `SessionEnd`. Hooks can be `type: "http"` and POST JSON to localhost. There is no "thinking" event, so we infer it from a timer.
- **Claude Code status line:** JSON on stdin with `model.display_name`, `context_window.used_percentage`, `cost.total_cost_usd`, and `rate_limits.five_hour` / `seven_day` (`used_percentage`, `resets_at`). Rate limits appear only for Pro/Max subscribers, only after the first reply, and each window may be absent. `refreshInterval` keeps it fresh when idle.
- **Claude Desktop (Electron):** chat runs inside a claude.ai web view. `~/Library/Logs/Claude/main.log` has only plugin, auth and updater lines, with no per-message or generating events. There is no hook API for chat. Desktop's Code and Cowork tabs run Claude Code (`~/Library/Application Support/Claude/claude-code-sessions` exists), so hooks probably apply there. This is not yet confirmed.

## Goals
- Overlay mascot with moods: idle, working, thinking-longer, done, error or needs-attention, sleeping, and a pop-in greeting on launch.
- Show current model and usage (5-hour and weekly limits, context %) in a small panel on hover or click.
- Pop out automatically when Claude Code starts a session or Claude Desktop launches or gains focus.
- One-click setup for non-technical users.

## Non-goals and anti-goals
- Not reading or storing conversation content. The mascot only consumes state events.
- No scraping of claude.ai cookies or private APIs.
- v1 does not use Accessibility scraping of the Desktop window (see Approaches).
- Must not block clicks or steal focus, and must not annoy (mute, hide and corner controls are required).

## Approaches considered (user picked #1)
1. **Two-tier (chosen).** Exact sync for Claude Code through hooks and the status line. Process and activity heuristic for Desktop chat. Model and usage appear only where reliably available. *Tradeoff:* Desktop chat is coarse.
2. Add Accessibility scraping of the Desktop window (model picker, Stop button). *Tradeoff:* richer, but needs a permission prompt and breaks when the UI changes. Kept as a possible optional "enhanced mode" later.
3. Claude Code only first. *Tradeoff:* most reliable, but does not serve Desktop-only users, so rejected.

## Architecture (my calls, correctable)
- **App shell:** Tauri 2 (Rust core plus a web UI), a transparent, frameless, always-on-top, click-through window. Fallback: Electron if Tauri overlay behavior is problematic.
- **State bus:** a localhost HTTP server in the app on a fixed port, bound to 127.0.0.1 only, with a random token the installer writes into the hook config. The server accepts events as `{source, event, model?, usage?, ts}`.
- **Claude Code adapter:** the installer merges hooks (`type: http`) and a `statusLine` command into `~/.claude/settings.json`, keeping a backup and not overwriting existing entries. The status-line script forwards model and usage to the app and still prints its own text.
- **Desktop adapter (heuristic):** the macOS Rust process monitor watches `Claude.app` (launch, focus, quit) with NSWorkspace notifications. While the Claude helper processes show sustained CPU and network activity the mood is *working*; after about 20 seconds it becomes *thinking-longer*; when activity stops it becomes *done* then *idle*. The thresholds are tunable and need real-world calibration.
- **Mood state machine:** one reducer that merges the sources by priority (Code events beat heuristics) with debounce and minimum display times so moods don't flicker. Thinking-longer comes from `UserPromptSubmit` without `Stop` after N seconds (default 15).
- **Usage panel:** a small card showing model, 5-hour % with reset time, weekly %, and context %. Shows "n/a" when the source can't supply it (Desktop chat, API-key users).

## Mascot and animation pipeline
- **Character (decided 2026-11-17): "Ember"**, an original pixel-art spark blob. It's orange and red with a yellow flame tuft, big eyes and pink cheeks, drawn on a 16×16 grid and scaled up crisply to about 96–140 px on screen. It nods to Claude's warm orange but is not Anthropic's mascot or logo. The concept page is https://claude.ai/artifact/EQncxPaYwqqyPzKyXAeqbi (palette: outline #4a1f12, body #ef6a3a, tuft #ffd166, belly #f9a46b, cheeks #ffb3a7, eyes #2b120a).
- **Authoring:** pixel art in Aseprite (or a similar tool), with the body and the face (eyes, mouth) as separate layers so moods reuse one body.
- **Animation:** one sprite sheet per mood, 4 to 8 frames each, played at about 8 fps (`image-rendering: pixelated`). The moods are idle (breathe, blink), working (fast bounce, focused eyes, keyboard, sparks), thinking (eyes up, sweat drop, growing thought dots), done (hop, ^ ^ eyes, sparkles), error (shake), sleep (closed eyes, floating Zs), and greeting or pop-in (hop with a sparkle). The concept page already prototypes these moods procedurally.
- **Audio:** optional and off by default.

## Downfalls and risks
- Desktop chat sync is heuristic and can mislabel moods. Model and usage are unavailable there.
- If Desktop's Code tab doesn't run the user's hooks, those sessions fall back to the heuristic. Needs verification.
- Hook and status-line fields may change across Claude Code versions, so the adapter must tolerate missing fields.
- Editing `settings.json` is sensitive: back up first, merge safely, and offer a one-click uninstall.
- macOS overlay quirks: Spaces and full-screen apps, multiple displays, and permission prompts (none needed for the process monitor).
- Code signing and notarization are needed for non-technical users to run it without Gatekeeper warnings (an Apple Developer account).
- Subscription rate-limit data is absent for API-key users.

## Milestones
1. **Spike (throwaway):** transparent Tauri window with a placeholder mascot reacting to a fake event feed. This proves the overlay and click-through.
2. Local state server and mood state machine, with a simulated event CLI for testing.
3. Claude Code adapter and installer (hooks and status line, backup and uninstall). Verify live with a real session.
4. Desktop adapter heuristic. Calibrate against real Desktop chat. Verify whether the Code tab runs hooks.
5. Real mascot art and Rive animations for all moods.
6. Usage panel, settings (corner, size, mute, auto-hide), and the launch pop-in.
7. Packaging, signing, notarization and the first-run "Connect" flow.

## Open questions for the user
- Where should it live on screen by default (a corner, near the Dock, or draggable)? Default until answered: bottom-right corner, draggable.

## Verification
- Simulated events drive each mood; screenshots and a screen recording confirm each animation and transition.
- Run a real Claude Code session and confirm the hooks fire the correct moods, the model name updates after `/model`, and the usage numbers match the status line.
- Run a real Claude Desktop chat and measure the heuristic against actual start and end of replies, then tune thresholds.
- Install and uninstall on a clean user profile and confirm `settings.json` is restored exactly.
- Confirm the overlay never intercepts clicks and behaves on multiple displays and full-screen apps.
