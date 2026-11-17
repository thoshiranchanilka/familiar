# Claude Puppet

A small pixel mascot that sits in a corner of your Mac and reacts to Claude while you work.

- **Moods:** idle, working, thinking longer, done, error, asleep, plus a little hello when you start a session.
- **Usage at a glance:** hover over the mascot to see the current model, your 5-hour and weekly limits with reset times, and how much of the context window is used.
- **Three mascots:** Ember (spark blob), Mochi (sleepy cat) and Bolt (tiny robot). More can be added later.
- **Works with:** Claude Code in any terminal. Claude Desktop is detected automatically, but it only shows working or idle, since the app doesn't expose the model or usage.

macOS on Apple Silicon only.

## Install

1. Download the latest `.dmg` from this repo's **Releases** page.
2. Open it and drag **Claude Puppet** into **Applications**.
3. Open Claude Puppet. macOS will block it the first time, because the app isn't notarized by Apple (that needs a paid developer account). To allow it:
   - Open **System Settings → Privacy & Security**.
   - Scroll down to the message about Claude Puppet and click **Open Anyway**.
   - Confirm. From now on it opens normally.

   Alternatively, if you're comfortable with Terminal:

   ```sh
   xattr -dr com.apple.quarantine "/Applications/Claude Puppet.app"
   ```

> On a work Mac managed by IT, policy may block apps that aren't notarized. Check with your IT team if **Open Anyway** doesn't appear.

## Connect Claude Code

On first launch the mascot says *"Click me to connect Claude Code."* Click it, or right-click it and choose **Connect Claude Code**. Then start a new Claude Code session.

Connecting adds a few entries to `~/.claude/settings.json`: hooks that report events, and a status line that reports model and usage. Your existing hooks and status line are kept. A backup is saved first, and **Disconnect** removes only Claude Puppet's entries.

## Using it

- **Hover** over the mascot to see model and usage.
- **Right-click** it for Settings, Hide, Connect or Disconnect, and Quit.
- **Settings** lets you pick a mascot, size, corner and animation level (Full, Gentle or Still).
- **Lost the mascot?** If you hid it, open Claude Puppet again from Applications or Spotlight. The mascot comes back and Settings opens.

## Privacy

Everything stays on your Mac. The app listens only on `127.0.0.1`, and requests must carry a random token stored in `~/.claude-mascot/token`.

- **Hook:** forwards only the event name, session id and model name. It never sends prompts, replies, file contents or tool output.
- **Status line:** forwards Claude Code's status data to the local app. That data includes the working folder, session details, cost and rate limits; the app uses only the model, context use and rate limits, and keeps nothing on disk.

## Power use

The mascot is designed to be near-zero when idle:

- It redraws only when the picture changes.
- In the default **Gentle** level it animates only while Claude is busy. Otherwise it blinks every 12 s, and while asleep its Zs drift once every 4 s.
- It switches to still frames in Low Power Mode.
- While hidden it doesn't draw or track the cursor.

Measured idle use (MacBook with M-series chip, Gentle animation, Claude Desktop closed, 30 s samples, three runs):
**0.07–0.17% CPU** and a macOS energy score of **0.07–0.17**, about what a menu-bar clock uses.
While Claude is working, the mascot animates at 8 frames per second. That's small next to the terminal or app streaming the reply.

### Measuring power

```sh
npm run tauri build -- --bundles app
scripts/power-check.sh
```

This samples the app and its web view for 30 s. It fails if average CPU is above 0.3% or the macOS energy score is above 0.5.

## Uninstall

1. Right-click the mascot and choose **Disconnect Claude Code**. This restores your Claude Code settings.
2. Quit Claude Puppet, then delete it from Applications.
3. Delete `~/.claude-mascot`.

## Build from source

You'll need Node 20+, Rust (stable) and the Xcode command-line tools.

```sh
npm install
npm run tauri build -- --bundles app,dmg
```

The app lands in `src-tauri/target/release/bundle/`. Apps you build yourself open without the Gatekeeper warning.

Development:

```sh
npm run tauri dev   # run with live reload
npx vitest run      # UI and mood logic tests
cargo test --manifest-path src-tauri/Cargo.toml   # core tests
```

### Releasing

Push a version tag, for example `git tag v0.2.0 && git push origin v0.2.0`. GitHub Actions builds the `.dmg` and creates a **draft** release to review and publish.

## License

TBD. Choose one before publishing (MIT is a common choice for hobby projects).

---

*Claude Puppet is an unofficial fan project and is not affiliated with or endorsed by Anthropic.*
