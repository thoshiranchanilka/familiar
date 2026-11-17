#!/bin/sh
# Familiar status line.
# Forwards Claude Code's status JSON to the Familiar app (never blocks, never fails),
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
else
  id=$(sed -n 's/.*"mascot"[[:space:]]*:[[:space:]]*"\([a-z]*\)".*/\1/p' "$dir/settings.json" 2>/dev/null | head -n 1)
  case $id in
    ember) name="Ember" ;;
    mochi) name="Mochi" ;;
    bolt) name="Bolt" ;;
    boo) name="Boo" ;;
    inky) name="Inky" ;;
    sprout) name="Sprout" ;;
    lotl) name="Lotl" ;;
    hop) name="Hop" ;;
    dot) name="Dot" ;;
    *) name="Familiar" ;;
  esac
  model=$(printf '%s' "$input" | sed -n 's/.*"display_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
  if [ -n "$model" ]; then echo "$name · $model"; else echo "$name"; fi
fi
exit 0
