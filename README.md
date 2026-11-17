# Familiar

**[Website & docs](https://thoshiranchanilka.github.io/familiar/)** · A desk buddy for Claude Code. A small pixel mascot pops up in a corner of your Mac while you're using Claude Code, reacts to what Claude is doing, and goes away when you're done.

- **Moods:** idle, working, thinking longer, done, error and asleep, plus a little hello when a session starts.
- **Usage at a glance:** hover over the mascot to see the current model, your 5-hour and weekly limits with reset times, and how much of the context window is used.
- **Nine mascots:** Ember (spark blob), Mochi (sleepy cat), Bolt (tiny robot), Boo (tiny ghost), Inky (octopus), Sprout (seedling), Lotl (axolotl), Hop (frog) and Dot (dragon hatchling). Each has its own little touches, like Sprout blooming when a task is done.

macOS on Apple Silicon only.

## When it shows up

Familiar runs quietly in the background and opens at login. The mascot appears when you start a Claude Code session:

- **Where:** any terminal, and Claude Desktop's **Code** tab if your version runs Claude Code hooks there (otherwise terminal only).
- **Leaving:** when you exit the session it waves goodbye and hides.
- **Closed terminal:** if you close a terminal without exiting, it hides after 5 minutes without activity.
- **Long tasks:** while Claude is busy with a long task, it waits up to 30 minutes.

Claude Desktop **chat** isn't supported, because the app doesn't share what it's doing.

## Install

1. Download the latest `.dmg` from this repo's **Releases** page.
2. Open it and drag **Familiar** into **Applications**.
3. Open Familiar. macOS will block it the first time, because the app isn't notarized by Apple (that needs a paid developer account). To allow it:
   - Open **System Settings → Privacy & Security**.
   - Scroll down to the message about Familiar and click **Open Anyway**.
   - Confirm. From now on it opens normally.

   Alternatively, if you're comfortable with Terminal:

   ```sh
   xattr -dr com.apple.quarantine "/Applications/Familiar.app"
   ```

> On a work Mac managed by IT, policy may block apps that aren't notarized. Check with your IT team if **Open Anyway** doesn't appear.

## Connect Claude Code

On first launch the mascot says *"Click me to connect Claude Code."* Click it, or right-click it and choose **Connect Claude Code**. Then start a new Claude Code session, and the mascot pops up.

Connecting adds a few entries to `~/.claude/settings.json`: hooks that report events, and a status line that reports model and usage. Your existing hooks and status line are kept. A backup is saved first, and **Disconnect** removes only Familiar's entries.

## Using it

- **Hover** over the mascot to see model and usage.
- **Right-click** it for Settings, Hide, Connect or Disconnect, and Quit.
- **Drag** it anywhere on screen. It remembers the spot. For exact placement, open **Settings → Position → Advanced** and use the two sliders, or **Reset to corner**.
- **Settings** lets you pick a mascot, size, corner and animation level (Full, Gentle or Still), and switch **Open at login** on or off.
- **Want to see it now?** Open Familiar again from Applications or Spotlight. The mascot comes back and Settings opens, even if you'd hidden it.

## Privacy

Everything stays on your Mac. The app listens only on `127.0.0.1`, and requests must carry a random token stored in `~/.claude-mascot/token`.

- **Hook:** forwards only the event name, session id and model name. It never sends prompts, replies, file contents or tool output.
- **Status line:** forwards Claude Code's status data to the local app. That data includes the working folder, session details, cost and rate limits. The app uses only the model, context use and rate limits, and keeps nothing on disk.

## Power use

Familiar is designed to be near-zero:

- Between sessions the mascot is hidden and nothing is drawn.
- It redraws only when the picture changes.
- In the default **Gentle** level it animates only while Claude is busy. Otherwise it blinks every 12 s, and while asleep its Zs drift once every 4 s.
- It switches to still frames in Low Power Mode.

Measured on an M-series MacBook (Gentle animation, 30 s samples):

| State | CPU | macOS energy score |
|---|---|---|
| Hidden (no session) | usually 0.00% (0.00–0.36% across 15 runs) | 0.00–0.39 |
| On screen, idle | 0.07–0.28% | 0.17–0.35 |

That's about what a menu-bar clock uses. While Claude is working, the mascot animates at 8 frames per second, which is small next to the terminal streaming the reply.

### Measuring power

```sh
npm run tauri build -- --bundles app
POWER_CHECK_HOME=$(mktemp -d) scripts/power-check.sh            # mascot hidden
POWER_CHECK_HOME=$(mktemp -d) scripts/power-check.sh --visible  # mascot on screen
```

This launches the app with a throwaway home folder and samples it and its web view for 30 s. It fails if average CPU is above 0.3% or the macOS energy score is above 0.5.

## Uninstall

1. Right-click the mascot and choose **Disconnect Claude Code**. This restores your Claude Code settings.
2. Open **Settings**, switch off **Open at login**, then quit Familiar. If you've already deleted the app, delete `~/Library/LaunchAgents/Familiar.plist` instead.
3. Delete Familiar from Applications.
4. Delete `~/.claude-mascot`.

## Build from source

You'll need Node 20+, Rust (stable) and the Xcode command-line tools.

```sh
npm install
npm run tauri build -- --bundles app,dmg
```

The app lands in `src-tauri/target/release/bundle/`. Apps you build yourself open without the Gatekeeper warning.

Development:

```sh
npm run tauri dev   # run with live reload (never turns on Open at login)
npx vitest run      # UI, mood and mascot tests
cargo test --manifest-path src-tauri/Cargo.toml   # core tests
```

Adding a mascot means adding one file in `src/ui/mascot/defs/`, an entry in `MASCOTS`, and its name in `scripts/statusline.sh`. A test checks that last part.

### Releasing

Push a version tag, for example `git tag v0.3.0 && git push origin v0.3.0`. GitHub Actions builds the `.dmg` and creates a **draft** release to review and publish.

## License

[MIT](LICENSE).

---

*Familiar is an unofficial fan project and is not affiliated with or endorsed by Anthropic.*
