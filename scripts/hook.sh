#!/bin/sh
# Claude Puppet hook: tells the mascot app which Claude Code event just happened.
# It forwards only the event name, session id and model — never tool inputs or
# outputs — and never fails or prints anything, so Claude Code is unaffected when
# the app isn't running.
dir="$HOME/.claude-mascot"
port="${CLAUDE_PUPPET_PORT:-47653}"
token=$(cat "$dir/token" 2>/dev/null) || exit 0
[ -n "$token" ] || exit 0
input=$(cat)

# One JSON field as plain text, limited to safe characters.
field() {
  printf '%s' "$input" | /usr/bin/plutil -extract "$1" raw -o - - 2>/dev/null | tr -cd 'A-Za-z0-9._:-'
}

event=$(field hook_event_name)
[ -n "$event" ] || exit 0
session=$(field session_id)
model=$(field to_model)
[ -n "$model" ] || model=$(field model)

body="{\"hook_event_name\":\"$event\",\"session_id\":\"$session\""
[ -n "$model" ] && body="$body,\"to_model\":\"$model\""
body="$body}"

( curl -s -m 1 -o /dev/null -X POST \
    -H "Authorization: Bearer $token" -H "Content-Type: application/json" \
    --data "$body" "http://127.0.0.1:$port/hook" >/dev/null 2>&1 & )
exit 0
