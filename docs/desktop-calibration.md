# Claude Desktop activity calibration

The Desktop monitor (`src-tauri/src/desktop_monitor.rs`) treats Claude Desktop as
"generating" when the summed CPU of all `Claude.app` processes is at least
`CPU_ACTIVE_PCT`. Network sampling with `nettop` was removed on 2026-11-17: each
run cost over 100% CPU and one ran almost continuously, overheating the Mac.
Activity must last `START_AFTER_MS` (1.5 s) to start and quiet must last
`STOP_AFTER_MS` (2 s) to stop.

## Idle baseline (2026-11-17, MacBook, Apple Silicon)

| Signal | Typical | Max seen (40 samples) |
|---|---|---|
| CPU, all Claude processes | 0.7–1.1 % | 4.0 % |
| Network in+out | 0 B/s | 358 B/s |

Starting threshold: CPU 6 %.

## How to calibrate against real replies

1. Run `PUPPET_CALIBRATE=1 npm run tauri dev`. Each 0.5 s sample prints `[calibrate] cpu=…`.
2. In Claude Desktop, send one short question and one long one (for example, "write a 600-word story").
3. Note the sample values during streaming and right after, and adjust the constants so that:
   - streaming stays above the threshold the whole time (no gaps longer than 2 s), and
   - idle and typing in the input box stay below it.

## Open questions

- Not yet measured: CPU while a reply streams.
- Not yet checked: whether Claude Desktop's Code tab runs hooks from `~/.claude/settings.json`.
