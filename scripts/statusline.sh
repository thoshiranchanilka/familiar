#!/bin/sh
# Claude Puppet status line.
# Forwards Claude Code's status JSON to the mascot app (never blocks, never fails),
# then runs the user's previous status line, if any, so its output still shows.
input=$(cat)
dir="$HOME/.claude-mascot"
token=$(cat "$dir/token" 2>/dev/null)
if [ -n "$token" ]; then
  printf '%s' "$input" | curl -s -m 1 -o /dev/null -X POST \
    -H "Authorization: Bearer $token" -H "Content-Type: application/json" \
    --data-binary @- http://127.0.0.1:47653/status >/dev/null 2>&1 &
fi
if [ -s "$dir/prev-statusline" ]; then
  printf '%s' "$input" | sh -c "$(cat "$dir/prev-statusline")"
fi
exit 0
