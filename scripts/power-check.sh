#!/bin/bash
# Measures Familiar's idle CPU and energy use and fails above the targets.
#
# Usage: scripts/power-check.sh [--visible] [path/to/Familiar.app]
#   POWER_CHECK_HOME=/tmp/x scripts/power-check.sh   # launch with a throwaway home folder,
#                                                   # so your real Claude Code settings are untouched
#   --visible   (with POWER_CHECK_HOME) start a pretend Claude Code session first, so the
#               mascot is on screen; without it the mascot is hidden, as between sessions.
#               The throwaway home is set up as "connected" so the connect prompt doesn't show.
#
# If Familiar is already running, that copy is measured and left running.
# Otherwise the given (or freshly built) app is launched, measured, and quit.
# WebKit helper processes are not children of the app, so they are matched by
# starting within 15 s of the app process.
set -u

CPU_MAX=0.3
ENERGY_MAX=0.5
SETTLE_SECS=15
SAMPLES=11     # first sample is a baseline and is skipped
INTERVAL=3

visible=""
if [ "${1:-}" = "--visible" ]; then visible=1; shift; fi
APP=${1:-"$(cd "$(dirname "$0")/.." && pwd)/src-tauri/target/release/bundle/macos/Familiar.app"}
launched=""

pid=$(pgrep -x familiar | head -n 1)
if [ -z "$pid" ]; then
  [ -d "$APP" ] || { echo "No running Familiar and no app at: $APP" >&2; exit 2; }
  if [ -n "${POWER_CHECK_HOME:-}" ]; then
    mkdir -p "$POWER_CHECK_HOME/.claude"
    # Looks connected (one of our hook entries), so the mascot only shows during a session.
    [ -f "$POWER_CHECK_HOME/.claude/settings.json" ] || printf '%s' \
      '{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"'"$POWER_CHECK_HOME"'/.claude-mascot/hook.sh"}]}]}}' \
      > "$POWER_CHECK_HOME/.claude/settings.json"
    HOME="$POWER_CHECK_HOME" FAMILIAR_NO_LOGIN=1 "$APP/Contents/MacOS/familiar" >/dev/null 2>&1 &
  else
    open -n "$APP"
  fi
  for _ in $(seq 1 30); do pid=$(pgrep -x familiar | head -n 1); [ -n "$pid" ] && break; sleep 1; done
  [ -n "$pid" ] || { echo "Familiar did not start" >&2; exit 2; }
  launched=$pid
  if [ -n "$visible" ]; then
    [ -n "${POWER_CHECK_HOME:-}" ] || { echo "--visible needs POWER_CHECK_HOME" >&2; kill "$pid"; exit 2; }
    sleep 3
    token=$(cat "$POWER_CHECK_HOME/.claude-mascot/token" 2>/dev/null)
    curl -s -m 2 -o /dev/null -X POST -H "Authorization: Bearer $token" -H "Content-Type: application/json" \
      --data '{"hook_event_name":"SessionStart","session_id":"power-check"}' http://127.0.0.1:47653/hook \
      || echo "warning: could not start the pretend session" >&2
  fi
  echo "Launched Familiar (pid $pid)${visible:+ with the mascot on screen}; settling for ${SETTLE_SECS}s…"
  sleep "$SETTLE_SECS"
else
  echo "Measuring the running Familiar (pid $pid)."
fi

start_epoch() { date -j -f "%a %b %d %T %Y" "$(ps -o lstart= -p "$1" | sed 's/  */ /g')" +%s 2>/dev/null; }
app_start=$(start_epoch "$pid")
pids=("$pid")
for w in $(pgrep -f "com.apple.WebKit.(WebContent|GPU|Networking)"); do
  ws=$(start_epoch "$w")
  if [ -n "$ws" ] && [ -n "$app_start" ] && [ $((ws - app_start)) -ge 0 ] && [ $((ws - app_start)) -le 15 ]; then
    pids+=("$w")
  fi
done
echo "Sampling pids: ${pids[*]} (${SAMPLES} × ${INTERVAL}s)"

args=()
for p in "${pids[@]}"; do args+=(-pid "$p"); done
out=$(top -l "$SAMPLES" -s "$INTERVAL" -stats pid,cpu,power "${args[@]}" 2>/dev/null)

read -r cpu_avg energy_avg <<<"$(printf '%s\n' "$out" | awk '
  /^PID/ { block++; next }
  block >= 2 && $1 ~ /^[0-9]+$/ { cpu[block] += $2; pow[block] += $3 }
  END {
    n = 0; c = 0; e = 0
    for (b = 2; b <= block; b++) { c += cpu[b]; e += pow[b]; n++ }
    if (n == 0) { print "nan nan"; exit }
    printf "%.2f %.2f\n", c / n, e / n
  }')"

if [ -n "$launched" ]; then kill "$launched" 2>/dev/null; fi

echo "cpu_avg=${cpu_avg}% energy_avg=${energy_avg} (targets: cpu ≤ ${CPU_MAX}%, energy ≤ ${ENERGY_MAX})"
if awk -v c="$cpu_avg" -v e="$energy_avg" -v cm="$CPU_MAX" -v em="$ENERGY_MAX" \
     'BEGIN { exit !(c != "nan" && c + 0 <= cm && e + 0 <= em) }'; then
  echo "PASS"
else
  echo "FAIL"
  exit 1
fi
