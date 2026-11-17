# Claude Mascot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A macOS overlay mascot that shows Claude's mood, current model and usage, synced exactly with Claude Code and heuristically with Claude Desktop chat, installable in one click.

**Architecture:** A Tauri 2 app owns a transparent click-through overlay window (TypeScript UI) and a Rust core. The core runs a localhost-only HTTP server that receives Claude Code hook and status-line POSTs, plus a Claude Desktop process monitor. Both feed normalized `MascotEvent`s to a pure TypeScript mood reducer that drives the animation.

**Tech Stack:** Tauri 2, Rust (axum or tiny_http, sysinfo, objc2 NSWorkspace), TypeScript + Vite + Vitest, pixel sprite sheets for the "Ember" character (canvas, 8 fps).

**Spec:** `docs/superpowers/specs/2026-11-17-claude-mascot-design.md`

## Global Constraints

- Platform: macOS on Apple Silicon only for v1. No Windows or Linux.
- Server binds `127.0.0.1` only, fixed port `47653`, requires header `Authorization: Bearer <token>`. Token is random, stored in `~/.claude-mascot/token`.
- Never read or store conversation content. Events carry only `source`, `event`, `model`, `usage`, `ts`.
- Original character, no Anthropic logo or branding.
- Editing `~/.claude/settings.json`: back up first to `~/.claude-mascot/settings.backup.json`, merge without overwriting existing entries, and uninstall must restore the exact previous bytes.
- Hooks must never block or slow Claude Code: mascot being down must not break a session or the status line.
- Thinking-longer threshold default 15 s (Code) and 20 s (Desktop heuristic); sleep after 5 min idle; all tunable constants live in `src/mood/config.ts`.
- Overlay must be click-through and must not steal focus.

## Review Focus

- `settings.json` missing, empty, or invalid JSON: installer must not crash or destroy it (report error, leave file untouched).
- Existing user hooks and an existing `statusLine`: kept and chained, not replaced.
- Status-line JSON with `rate_limits` absent, one window absent, or `used_percentage` null: panel shows "n/a", no crash.
- Mascot app not running or port 47653 in use: hooks fail silently; status-line script still prints its text; app shows a clear error if the port is taken.
- Several concurrent Claude Code sessions: mood follows the most recently active, and one session's `Stop` doesn't set idle while another still works.
- Desktop activity flicker (brief traffic gaps mid-reply): no done/idle flicker; minimum display times apply.

---

## File Structure

- `src-tauri/src/main.rs`: app setup, window config, wiring.
- `src-tauri/src/server.rs`: HTTP server, auth, emits events.
- `src-tauri/src/normalize.rs`: raw hook/status JSON to `MascotEvent`.
- `src-tauri/src/installer.rs`: settings merge, backup, uninstall, token, status-line script.
- `src-tauri/src/desktop_monitor.rs`: Claude.app launch/focus/quit and activity sampling.
- `src/mood/types.ts`, `src/mood/config.ts`, `src/mood/reducer.ts`: mood state machine.
- `src/ui/overlay.ts`, `src/ui/usagePanel.ts`, `src/ui/settings.ts`: rendering.
- `src/ui/mascot/` : placeholder mascot now, Rive file later.
- `scripts/statusline.sh`: forwarded copy installed to `~/.claude-mascot/`.

## Shared Interfaces (used by several tasks)

```ts
// src/mood/types.ts
export type Mood = 'greeting'|'idle'|'working'|'thinking'|'done'|'error'|'sleeping';
export interface Usage { fiveHourPct?: number; fiveHourResetsAt?: number;
  sevenDayPct?: number; sevenDayResetsAt?: number; contextPct?: number; }
export interface MascotEvent {
  source: 'code'|'desktop'; sessionId?: string;
  event: 'session_start'|'prompt'|'tool'|'stop'|'error'|'attention'|'session_end'
       |'app_launch'|'app_focus'|'app_quit'|'activity_start'|'activity_stop'|'status';
  model?: string; usage?: Usage; ts: number; // epoch ms
}
export interface MascotState { mood: Mood; since: number; model?: string; usage: Usage;
  sessions: Record<string, { working: boolean; startedAt: number }>; desktopActive: boolean; }
```

---

### Task 1: Overlay spike (throwaway proof of the window)

**Files:**
- Create: Tauri 2 + Vite TS scaffold in the project root; `src/ui/overlay.ts`, `index.html`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: a running app whose window is transparent, frameless, always-on-top, shown on all Spaces, and ignores cursor events; renders a placeholder emoji mascot bottom-right.

- [ ] **Step 1:** Scaffold with `npm create tauri-app@latest` (TypeScript, vanilla, npm); init git (`git init`).
- [ ] **Step 2:** Configure window: `transparent: true`, `decorations: false`, `alwaysOnTop: true`, `visibleOnAllWorkspaces: true`, `skipTaskbar: true`, `shadow: false`; call `setIgnoreCursorEvents(true)` at startup; enable `macOSPrivateApi` for transparency.
- [ ] **Step 3:** Render a 128px emoji at bottom-right in `overlay.ts` with a CSS bobbing animation.
- [ ] **Step 4:** Run `npm run tauri dev`. Expected: mascot floats over other apps, clicks pass through to windows beneath, no Dock focus steal, visible on another Space and over a full-screen app (note any failure in `docs/spike-notes.md`; if full-screen fails, record it as a known limit).
- [ ] **Step 5:** Commit `git commit -m "chore: tauri overlay spike"`.

### Task 2: Mood reducer (pure TypeScript, TDD)

**Files:**
- Create: `src/mood/types.ts`, `src/mood/config.ts`, `src/mood/reducer.ts`
- Test: `src/mood/reducer.test.ts`

**Interfaces:**
- Consumes: `MascotEvent`, `MascotState` (Shared Interfaces).
- Produces: `initialState(now: number): MascotState`; `reduce(state: MascotState, input: MascotEvent | {tick: number}): MascotState`; config constants `THINKING_AFTER_CODE_MS=15000`, `THINKING_AFTER_DESKTOP_MS=20000`, `SLEEP_AFTER_MS=300000`, `MIN_MOOD_MS=800`, `DONE_HOLD_MS=2000`.

- [ ] **Step 1:** Write failing tests (Vitest), one per assertion:
  - `session_start` → mood `greeting`; after 2000 ms tick → `idle`.
  - `prompt` (code, session A) → `working`; tick at +15000 → `thinking`; `stop` → `done`; tick at +2000 → `idle`.
  - Two sessions A,B: A `prompt`, B `prompt`, A `stop` → still `working`.
  - Code activity beats desktop: desktop `activity_start` while a code session is working leaves mood driven by code.
  - Desktop `activity_start` → `working`; tick +20000 → `thinking`; `activity_stop` → `done` → `idle`.
  - `error` → `error` held `MIN_MOOD_MS` even if `prompt` arrives sooner; then `working`.
  - `idle` for `SLEEP_AFTER_MS` → `sleeping`; any event wakes to `greeting` if `app_launch` else back to normal.
  - `status` event updates `model` and merges `usage` without changing mood; partial usage (only `fiveHourPct`) keeps other fields.
  - Mood changes inside `MIN_MOOD_MS` of `since` are deferred (anti-flicker): `activity_stop` then `activity_start` within 300 ms stays `working`.
- [ ] **Step 2:** Run `npx vitest run src/mood` → FAIL (module missing).
- [ ] **Step 3:** Implement `reduce` and `initialState`; it must be pure (time comes only from `ts`/`tick`). Priority: code sessions > desktop > idle.
- [ ] **Step 4:** Run `npx vitest run src/mood` → all PASS.
- [ ] **Step 5:** Commit `feat: mood reducer`.

### Task 3: Event normalization (Rust, TDD)

**Files:**
- Create: `src-tauri/src/normalize.rs`
- Test: inline `#[cfg(test)]` module in `normalize.rs`

**Interfaces:**
- Produces: `pub fn from_hook(raw: &serde_json::Value, now_ms: i64) -> Option<MascotEvent>` and `pub fn from_status(raw: &serde_json::Value, now_ms: i64) -> Option<MascotEvent>`; `MascotEvent` is a serde struct matching the TS shape (camelCase JSON).

- [ ] **Step 1:** Write failing tests with fixtures from the docs: `SessionStart` → `session_start`; `UserPromptSubmit` → `prompt`; `PreToolUse`/`PostToolUse` → `tool`; `Stop` → `stop`; `StopFailure` → `error`; `Notification` → `attention`; `SessionEnd` → `session_end`; `PostModelSwitch` with `to_model` → `status` with `model`; unknown event → `None`; missing `session_id` → still returns event with `sessionId: None`.
- [ ] **Step 2:** `status` tests: full JSON with `model.display_name`, `context_window.used_percentage`, `rate_limits.five_hour/seven_day` → populated `Usage`; `rate_limits` absent → usage has only `contextPct`; `used_percentage: null` → field omitted; extra unknown fields ignored.
- [ ] **Step 3:** Run `cargo test -p <app> normalize` → FAIL.
- [ ] **Step 4:** Implement both functions using `serde_json::Value` lookups (no strict structs, so unknown versions don't break).
- [ ] **Step 5:** Run tests → PASS. Commit `feat: normalize hook and status payloads`.

### Task 4: Event server (Rust, TDD)

**Files:**
- Create: `src-tauri/src/server.rs`
- Modify: `src-tauri/src/main.rs` (start server, `app.emit("mascot-event", event)`), `src/ui/overlay.ts` (listen and call `reduce`)
- Test: `src-tauri/tests/server.rs`

**Interfaces:**
- Consumes: `normalize::from_hook`, `normalize::from_status`.
- Produces: `pub fn start(token: String, port: u16, sink: Sender<MascotEvent>) -> Result<ServerHandle, ServerError>`; routes `POST /hook` (hook JSON) and `POST /status` (status-line JSON); `ServerError::PortInUse`.

- [ ] **Step 1:** Write failing integration tests (random free port): valid token + hook body → 200 and event on channel; missing/wrong token → 401 and nothing on channel; malformed JSON → 400; non-localhost bind not possible (assert `local_addr` is 127.0.0.1); second `start` on same port → `Err(PortInUse)`.
- [ ] **Step 2:** Run `cargo test --test server` → FAIL.
- [ ] **Step 3:** Implement with a small HTTP library, constant-time token compare, body limit 64 KB.
- [ ] **Step 4:** Wire in `main.rs`; on `PortInUse` show a native dialog "Mascot already running or port 47653 is busy".
- [ ] **Step 5:** Manual: `curl -H "Authorization: Bearer $(cat ~/.claude-mascot/token)" -d '{"hook_event_name":"UserPromptSubmit","session_id":"s1"}' localhost:47653/hook` → mascot switches to working. Commit `feat: local event server`.

### Task 5: Claude Code installer (Rust, TDD)

**Files:**
- Create: `src-tauri/src/installer.rs`, `scripts/statusline.sh`
- Test: inline tests using temp dirs

**Interfaces:**
- Produces: `pub fn install(home: &Path, token: &str, port: u16) -> Result<InstallReport, InstallError>`; `pub fn uninstall(home: &Path) -> Result<(), InstallError>`; `pub fn is_installed(home: &Path) -> bool`; Tauri commands `mascot_install`, `mascot_uninstall`, `mascot_status`.

- [ ] **Step 1:** Write failing tests against a temp `home`:
  - No `settings.json` → creates one with `hooks` for `SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, Stop, StopFailure, Notification, SessionEnd, PostModelSwitch` as `type: http` to `http://127.0.0.1:47653/hook` with Bearer header, plus `statusLine`.
  - Existing hooks for the same event are preserved and ours appended; installing twice doesn't duplicate.
  - Existing `statusLine.command` is saved to `~/.claude-mascot/prev-statusline` and chained by `statusline.sh`.
  - Invalid JSON → `Err(InstallError::InvalidSettings)` and file bytes unchanged.
  - `uninstall` after `install` restores byte-identical original (and removes file if none existed).
  - Backup file exists after install.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Implement atomic write (temp file + rename), preserve key order with `serde_json` `preserve_order`, mark our entries by URL for removal. `statusline.sh`: read stdin, `curl --max-time 1 -s -o /dev/null` POST to `/status` in the background, then run the previous status-line command with the same stdin (or print nothing); always exit 0.
- [ ] **Step 4:** Run tests → PASS. Add test that `statusline.sh` exits 0 and still prints previous output when the server is down (`bats` or shell test).
- [ ] **Step 5:** Manual: install, start a real `claude` session, confirm moods follow prompts, tools and stop; run `/model` and confirm the model label updates. Commit `feat: claude code installer`.

### Task 6: Claude Desktop monitor (Rust, TDD on the classifier)

**Files:**
- Create: `src-tauri/src/desktop_monitor.rs`
- Test: inline tests for the classifier

**Interfaces:**
- Produces: `pub struct ActivitySample { cpu_pct: f32, net_bytes_per_s: u64 }`; `pub struct Classifier` with `pub fn push(&mut self, s: ActivitySample, now_ms: i64) -> Option<ActivityEdge>` where `ActivityEdge::{Start, Stop}`; `pub fn spawn(sink: Sender<MascotEvent>)` emitting `app_launch`, `app_focus`, `app_quit`, `activity_start`, `activity_stop` with `source: "desktop"`.

- [ ] **Step 1:** Write failing classifier tests: sustained samples above thresholds for 1.5 s → `Start`; a single spike → nothing; gap below threshold shorter than 2 s mid-activity → no `Stop`; below threshold for 2 s → `Stop`.
- [ ] **Step 2:** Run → FAIL. **Step 3:** Implement classifier with constants in one place; sample `Claude Helper*` processes (sum of CPU and network via `sysinfo`/`nettop`) every 500 ms.
- [ ] **Step 4:** Watch `Claude.app` with NSWorkspace notifications (launch, activate, terminate), bundle id from `/Applications/Claude.app`.
- [ ] **Step 5:** Manual calibration: send a long and a short reply in Desktop, log samples to `docs/desktop-calibration.md`, tune thresholds. Also check whether the Code tab fires installed hooks and record the answer. Commit `feat: desktop monitor`.

### Task 7: Usage panel and settings (UI)

**Files:**
- Create: `src/ui/usagePanel.ts`, `src/ui/settings.ts`
- Test: `src/ui/usagePanel.test.ts`

**Interfaces:**
- Consumes: `MascotState` from the reducer.
- Produces: `formatUsage(u: Usage, now: number): { fiveHour: string; sevenDay: string; context: string }` (e.g. `"23% · resets in 2h 10m"`, `"n/a"` when missing); settings `{ corner: 'br'|'bl'|'tr'|'tl'; size: number; muted: boolean; autoHide: boolean }` persisted via the Tauri store; a tray menu with Connect, Disconnect, Hide, Quit.

- [ ] **Step 1:** Failing tests for `formatUsage`: all fields present; only `fiveHourPct`; `resetsAt` in the past → `"resetting…"`; `pct > 100` clamps display to `100%+`; empty → all `"n/a"`.
- [ ] **Step 2:** Run → FAIL. **Step 3:** Implement formatter, panel (hover reveals it by temporarily disabling click-through only over the mascot hit-area), and settings.
- [ ] **Step 4:** Run tests → PASS. Manual: hover shows model and usage, clicking elsewhere still passes through. Commit `feat: usage panel and settings`.

### Task 8: Mascot art integration and packaging

**Files:**
- Create: `src/ui/mascot/mascot.ts` (adapter: `setMood(mood: Mood)`), `src/ui/mascot/ember.ts` (procedural Ember from the concept page, used until the final sheets exist), `public/sprites/ember-<mood>.png` (final sheets)
- Modify: `src-tauri/tauri.conf.json` (bundle, signing)

**Interfaces:**
- Consumes: `Mood`. Produces: `createMascot(canvas: HTMLCanvasElement): { setMood(m: Mood): void; popIn(): void; popOut(): void }`, which plays at 8 fps on a 28×28 logical canvas scaled with `image-rendering: pixelated`.

- [ ] **Step 1:** Port the Ember `draw(ctx, ch, mood, f)` routine from the concept page (https://claude.ai/artifact/EQncxPaYwqqyPzKyXAeqbi) into `ember.ts`, adding the `error` (shake) mood. Replace the Task 1 emoji with it.
- [ ] **Step 2:** Final art: redraw Ember in Aseprite with a sheet of 4 to 8 frames per mood using the spec's palette. `mascot.ts` prefers the PNG sheets when present and falls back to procedural drawing otherwise.
- [ ] **Step 3:** First-run "Connect" screen calls `mascot_install`; "Disconnect" calls `mascot_uninstall`.
- [ ] **Step 4:** Build signed and notarized `.dmg` (`npm run tauri build`); verify launching on a clean user profile has no Gatekeeper warning.
- [ ] **Step 5:** Run the spec's Verification section end to end. Commit `feat: art integration and packaging`.

---

## Self-Review Notes
- Spec coverage: moods and state machine (T2), Code sync (T3–T5), Desktop sync (T6), usage panel and settings (T7), pop-in and pop-out (T2 greeting, T8 triggers), one-click install (T5, T8), signing (T8). Accessibility scraping is intentionally excluded (non-goal).
- Character decided: Ember (pixel spark blob). Task 8 step 1 can start from the concept page's code; only the polished sprite sheets (step 2) need art time.
