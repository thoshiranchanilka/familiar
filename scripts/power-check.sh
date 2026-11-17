#!/bin/bash
# Measures Claude Puppet's idle CPU and energy use and fails above the targets.
#
# Usage: scripts/power-check.sh [path/to/Claude Puppet.app]
#   POWER_CHECK_HOME=/tmp/x scripts/power-check.sh   # launch with a throwaway home folder,
#                                                   # so your real Claude Code settings are untouched
#
# If Claude Puppet is already running, that copy is measured and left running.
# Otherwise the given (or freshly built) app is launched, measured, and quit.
# WebKit helper processes are not children of the app, so they are matched by
# starting within 15 s of the app process.
set -u

CPU_MAX=0.3
ENERGY_MAX=0.5
SETTLE_SECS=15
SAMPLES=11     # first sample is a baseline and is skipped
INTERVAL=3

APP=${1:-"$(cd "$(dirname "$0")/.." && pwd)/src-tauri/target/release/bundle/macos/Claude Puppet.app"}
launched=""

pid=$(pgrep -x claude-puppet | head -n 1)
if [ -z "$pid" ]; then
  [ -d "$APP" ] || { echo "No running Claude Puppet and no app at: $APP" >&2; exit 2; }
  if [ -n "${POWER_CHECK_HOME:-}" ]; then
    mkdir -p "$POWER_CHECK_HOME"
    HOME="$POWER_CHECK_HOME" "$APP/Contents/MacOS/claude-puppet" >/dev/null 2>&1 &
  else
    open -n "$APP"
  fi
  for _ in $(seq 1 30); do pid=$(pgrep -x claude-puppet | head -n 1); [ -n "$pid" ] && break; sleep 1; done
  [ -n "$pid" ] || { echo "Claude Puppet did not start" >&2; exit 2; }
  launched=$pid
  echo "Launched Claude Puppet (pid $pid); settling for ${SETTLE_SECS}s…"
  sleep "$SETTLE_SECS"
else
  echo "Measuring the running Claude Puppet (pid $pid)."
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
