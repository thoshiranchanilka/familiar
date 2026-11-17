#!/bin/sh
# Familiar hook: tells the Familiar app which Claude Code event just happened.
# It forwards only the event name, session id and model — never tool inputs or
# outputs — and never fails or prints anything, so Claude Code is unaffected when
# the app isn't running.
dir="$HOME/.claude-mascot"
port="${FAMILIAR_PORT:-47653}"

# Read only the first 4 KB of the event (the fields we need come first in it), then
# drain the rest so Claude Code never sees a broken pipe, however large the payload.
start=$(head -c 4096)
cat >/dev/null

token=
read -r token < "$dir/token" 2>/dev/null
[ -n "$token" ] || exit 0

# Sets $value to a top-level string field, limited to safe characters. Pure shell
# string trimming (no extra processes, so it is fast). Escaped quotes inside other
# values (\"name\") never match, and no JSON parser is involved.
field() {
  value=
  rest=${start#*\"$1\"}
  [ "$rest" = "$start" ] && return           # key not present
  rest=${rest#"${rest%%[! ]*}"}               # spaces before the colon
  case $rest in :*) rest=${rest#:} ;; *) return ;; esac
  rest=${rest#"${rest%%[! ]*}"}               # spaces after the colon
  case $rest in \"*) rest=${rest#\"} ;; *) return ;; esac   # strings only
  value=${rest%%\"*}
  case $value in
    *[!A-Za-z0-9._:\ \[\]-]*) value=$(printf '%s' "$value" | tr -cd 'A-Za-z0-9._:[] -') ;;
  esac
}

field hook_event_name; event=$value
[ -n "$event" ] || exit 0
field session_id; session=$value
field to_model; model=$value
[ -n "$model" ] || { field model; model=$value; }

body="{\"hook_event_name\":\"$event\",\"session_id\":\"$session\""
[ -n "$model" ] && body="$body,\"to_model\":\"$model\""
body="$body}"

# Sent in the foreground so events arrive in order. It is instant when the app
# isn't running (connection refused) and capped at 1 s otherwise.
/usr/bin/curl -s -m 1 -o /dev/null -X POST \
  -H "Authorization: Bearer $token" -H "Content-Type: application/json" \
  --data "$body" "http://127.0.0.1:$port/hook" >/dev/null 2>&1
exit 0
