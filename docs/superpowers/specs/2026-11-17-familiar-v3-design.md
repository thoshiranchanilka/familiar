# Familiar v3: Claude Code only, pops in per session, nine mascots, new name

Builds on `2026-11-17-mascot-v2-design.md` (v2, on branch `feat/mascot`, commit `8407383`).

## Intent
The user wants the mascot to be a quiet companion for **Claude Code only**: invisible while you're not using Claude Code, popping in when a session starts and leaving when it ends. Detecting Claude Desktop chat replies from CPU use was never reliable, so that feature is dropped. The app is renamed so it can be shared publicly without looking like an official Anthropic product. The lineup grows from three to nine mascots so coworkers can pick one they like.

## Goals
- Ember, or the chosen mascot, appears only while a Claude Code session is active. That means sessions in any terminal and in Claude Desktop's **Code** tab, which runs Claude Code and its hooks.
- The app runs in the background and starts at login by default, so nobody has to remember to open it.
- Six new mascots: Boo, Inky, Sprout, Lotl, Hop and Dot. Each has one signature mood detail.
- The app is renamed **Familiar** everywhere a user can see it.
- Idle power stays within the v2 targets (CPU ≤ 0.3%, energy ≤ 0.5), and should drop now that the Desktop watcher is gone.

## Non-goals and anti-goals
- No Claude Desktop **chat** detection of any kind. That rules out CPU heuristics, Accessibility scraping and log reading.
- No user-imported mascots, no sounds, and no Windows or Linux support.
- No push, publish or GitHub repo creation without the user's explicit go-ahead.
- **Anti-goal:** the mascot showing up when the user isn't using Claude Code, or staying on screen all day because a terminal was left open.
- **Anti-goal:** breaking existing installs. The user's current connection, settings and token must keep working after the rename.

## Approaches (for visibility; the user picked A)
- **A. Visible while a session is active (chosen).** The mascot pops in on session start or on any Claude Code event. It waves goodbye and hides when the last session ends, or when no session has had activity for 5 minutes. Rationale: it's on screen only during real use, and a left-open terminal doesn't keep it there.
- B. Stay while any session is open; sleep after 5 min, then hide; pop back on the next prompt. Rejected: it shows up more often than the user wants.
- (Earlier) Two-tier with a Desktop heuristic. Dropped: the user judged it unreliable.

## Decisions

### 1. Remove Claude Desktop detection
Delete the following:
- The Desktop monitor: `desktop_monitor.rs`, its startup, and the `desktop_status` command.
- Desktop events and state:
  - The events `app_launch`, `app_focus`, `app_quit`, `activity_start` and `activity_stop`.
  - The fields `desktopActive` and `desktopStartedAt`, and `THINKING_AFTER_DESKTOP_MS`.
- The Desktop row in Settings, and Desktop text in the README.
- `docs/desktop-calibration.md`.
- Dependencies used only by the Desktop monitor: `sysinfo`, and the `NSWorkspace` / `NSRunningApplication` features if unused. Low Power Mode detection (`power.rs`) and window activation still use objc2.

`MascotEvent.source` can become `"code"` only, or be removed.

### 2. Presence (when the mascot is on screen)
Each session is **present** from its first event (`SessionStart` or any other hook) until one of:
- `SessionEnd` for that session;
- 5 minutes with no hook events while the session is not working;
- 30 minutes with no hook events while the session is working. A long-running tool, such as a 10-minute build, fires no events in between, and the mascot must not vanish mid-task.

Status-line updates **do not** count as activity: `refreshInterval` would otherwise keep an idle open terminal present forever.

The mascot is shown when at least one session is present, the user hasn't hidden it, and Claude Code is connected.
- **Popping in** plays the existing pop-in.
- **Leaving:** when the last session stops being present, it plays a short goodbye (a wave or the existing `popOut`) and the window goes invisible. While invisible, nothing draws and the hover watch runs at its hidden rate.
- **Not connected:** the mascot stays visible with "Click me to connect Claude Code", as today. Otherwise a new user would never see it.
- **Right after connecting:** the mascot says "I'll pop up when you start Claude Code" for a few seconds, then hides until a session starts.
- **The user's Hide** (right-click → Hide, or the tray) keeps the mascot hidden even during sessions, until the user shows it again. This is existing `hidden` behavior.
- **Reopening the app** from Applications or Spotlight shows the mascot and opens Settings, even with no session, so the user can always find it. Without a session it hides again when Settings closes.

Remove the "Hide while asleep" (`autoHide`) setting. It's superseded, and old settings files that still contain it must load without error.

The presence rules are pure logic in `src/mood/`, next to the reducer, and fully unit-tested. Session timing deadlines go into `nextDeadline` so the UI keeps a single timer and doesn't poll.

### 3. Open at login
- Launch at login, using a login item or LaunchAgent (for example `tauri-plugin-autostart`). It is **on by default**: enabled on the first launch of v3.
- Settings gets an **Open at login** switch.
- The README's uninstall steps cover removing it.

### 4. Rename to Familiar
- **What changes:**
  - Product name: **Familiar**.
  - Bundle identifier: `com.familiar.desktop`.
  - The window title, tray tooltip and menu, Settings, README, release workflow, `package.json` / `Cargo.toml` names, and the CLAUDE_PUPPET_PORT override, which becomes `FAMILIAR_PORT`.
- **Unchanged, so existing installs keep working:**
  - The hidden `~/.claude-mascot` folder: token, settings, scripts and backups.
  - The port, `47653`.
  - The installer's recognition of existing hook entries, which match by the `/.claude-mascot/hook.sh` path.
- **Script refresh:** when an install's `hook.sh` or `statusline.sh` differs from the shipped version, startup rewrites the scripts. It does not touch `settings.json` unless entries need migrating.
- **Status-line text:** shows the selected mascot's name and the model, for example `Boo · Opus 5.5`. With no mascot info available it shows `Familiar · <model>`.
- **Disclaimer:** the README keeps the "unofficial fan project, not affiliated with Anthropic" line, and describes the app as "a desk buddy for Claude Code".
- **Out of scope:** the repo folder name on disk (`Claude Puppet`) stays as it is; the user can rename it.

### 5. Six new mascots
Each new mascot is one file in `src/ui/mascot/defs/` plus an entry in `MASCOTS`, with the same 16×16 map, palette, eyes and mouth as today. All seven moods must read clearly at 64 px. The `MascotDef.id` union grows to nine.

| id | Name | Kind | Look | Signature detail |
|---|---|---|---|---|
| `boo` | Boo | Tiny ghost | White, round top, wavy bottom | Floats (no ground shadow). Semi-transparent while asleep. Says "boo" with a pop on error. |
| `inky` | Inky | Octopus | Purple, 4 short tentacles | Tentacles tap while working. Little ink cloud on error. |
| `sprout` | Sprout | Seedling | Green bean body, leaf on head | Leaf droops while thinking. A flower opens when done. Leaf folds while asleep. |
| `lotl` | Lotl | Axolotl | Pink with frilly side gills | Gills wiggle while working. Blush when done. |
| `hop` | Hop | Frog | Green, big eyes on top | Throat puffs while thinking. Extra-high hop when done. Half-closed eyes while idle. |
| `dot` | Dot | Dragon hatchling | Small dragon in half an eggshell | Smoke puffs while working. Tiny flame on error. Curls into the shell to sleep. |

- **Signature details:** these need a small per-mascot hook in the renderer, such as an optional per-mood effects function or overlay on `MascotDef`. Shared effects like the keyboard and Zs stay shared, though a mascot may opt out of one. For example, Boo floats instead of casting a shadow.
- **Frame rate:** signature effects must use the frame counter the renderer already gets, with no new timers, so Gentle and Still behave and measure as before.
- **Originality:** characters are original designs; they don't copy Tamagotchi characters or Anthropic's "Clawd".
- **Settings layout:** the mascot picker shows nine cards, for example in a 3×3 grid, in a window that still fits on a 13″ MacBook screen.

## Verification
- **Unit tests:**
  - The presence rules.
  - Every new mascot renders every mood within the grid.
  - Settings load an old file that has `autoHide` and Desktop fields.
  - The installer rewrites outdated scripts.
- **Manual checks (the user):**
  - Start Claude Code in a terminal: the mascot pops in.
  - `/exit`: the mascot says goodbye and hides.
  - Close a terminal window mid-session: the mascot hides within 5 minutes.
  - Start a session in Claude Desktop's **Code** tab: confirm whether the mascot pops in there too. If not, the README says terminal only.
- **Power:** `scripts/power-check.sh` passes, both visible-idle and hidden. Update the README's numbers.
- **Login:** open at login works after a logout and login.

## Calls made after the builder check (my calls, correctable)
- **Old "Claude Puppet.app":** no one but the user has installed it, and it never had a login item, so there's no migration code. The user deletes their old build. If both apps run at once, the second one gets the existing "port in use" notice.
- **Open at login for upgraders:** v3 turns it on at its first launch for everyone, upgraders included, using a marker in `settings.json` so it happens only once. After that, the switch in Settings decides.
- **Status-line name:** `statusline.sh` reads the `mascot` id from `~/.claude-mascot/settings.json` with plain shell (no extra processes beyond the existing ones), maps it to the display name, and falls back to `Familiar`. If the user had their own status line (`prev-statusline`), theirs is still shown instead of ours.
- **"Working" for presence:** this uses the reducer's existing per-session `working` flag, set on a prompt or tool event and cleared on `Stop`, `StopFailure` or `Notification`. If `Stop` never arrives (a crash, or a killed terminal), the 30-minute rule ends presence.
- **Reopen versus Hide:** reopening the app counts as "show it again", so it clears `hidden`, as today. When Settings closes, the mascot hides only if no session is present.
- **`CLAUDE_PUPPET_PORT`:** no fallback. It's a test-only override that no user relies on.

## Left to the builder
- Exact pixel maps and palettes.
- How the per-mascot effect hook is shaped.
- Exact goodbye animation.
- Copy for notices, within the tone above.
- Whether `source` is removed or narrowed.
- Settings layout details.
